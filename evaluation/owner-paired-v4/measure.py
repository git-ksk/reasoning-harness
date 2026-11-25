"""Owner development comparison: per-observation immutable claims and sealed evidence.

No access to Gold. Never rerun a claimed observation. Only an explicitly
classified and fully accounted local failure may leave the remaining slots open.
"""
import argparse
import base64
import contextlib
import datetime
import gzip
import importlib.util
import io
import json
import os
from pathlib import Path
import re
import signal
import subprocess
import sys
import time
import urllib.request

HERE = Path(__file__).resolve().parent
REPO_ROOT = HERE.parents[1]
SPEC = importlib.util.spec_from_file_location("frozen_v3", HERE.parent/"owner-paired-v3/measure.py")
legacy = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(legacy)
from build import CLI, ARMS, CHANGED

encode, sha, save, load = legacy.encode, legacy.sha, legacy.save, legacy.load
class HoldError(ValueError):
    pass
def need(condition, reason):
    if not condition:
        raise HoldError(reason)
MODEL = legacy.MODEL
REPO = "git-ksk/reasoning-harness"
PREFIX = "eval-owner-paired-v4-quality-r2-"
FROZEN_PAYLOAD_SHA = "6a7d305f035c57e61fb0283f63f3b06015f5f4a9d49bd0ea47c122521dbac7d5"
CERT = HERE.parent/"engine073-531-provisional24-20261114/reviewer-cert.pem"
CERT_SHA = "d917aeb4c55699b1ea19117c776e5f515aa40193488197a12c00082517695b43"
CAPS = {"max_model_calls": 12, "max_output_tokens": 8192, "max_total_tokens": 32192}
SLOT_TIMEOUT = 2700
ACCOUNTING = "reported_plus_conservative_reservations_v1"
POLICY = {
    "id": "owner-paired-quality-r2-diagnostic", **CAPS,
    "max_generation_output": 1024, "max_retries_per_http_logical_request": 3,
    "backoff_seconds": [30, 60, 120], "max_retry_wait_seconds": 210,
    "reservation": "max(logical_json_utf8_bytes,wire_json_utf8_bytes)+512; requested_max_output",
    "token_accounting": ACCOUNTING,
    "slot_timeout_seconds": SLOT_TIMEOUT, "inter_slot_seconds": 60,
    "transient_failure_cooldown_seconds": 120,
    "predecessor_runs": [45433270563, 45504130723],
    "selected_shards": [1, 2, 4],
    "classification": "post_outcome_selected_development_diagnostic_not_independent_holdout",
}
TERMINAL = {"complete", "budget_exhausted", "budget_reserve_exhausted", "transient_failed", "local_decode_failed"}
SHARDS = tuple(POLICY["selected_shards"])
SLOTS = {f"{i:02}-{arm}" for shard in SHARDS for i in range(shard*3,shard*3+3) for arm in ARMS}
HEX = r"[a-f0-9]{64}"
REF_RE = re.compile(PREFIX + f"({HEX})-({HEX})(?:-resume-([1-9][0-9]*))?")
USAGE_FIELDS = ("model_calls", "provider_attempts", "input_tokens", "output_tokens", "total_tokens")


def utc_now():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


def cohort(p):
    return sha(encode({"questions": sorted((x["question"], p["sources"][x["source_id"]]) for x in p["questions"]),
                       "core": ARMS, "cli": CLI, "model": MODEL, "measurement_policy": POLICY}))


def tag(p):
    return PREFIX + cohort(p) + "-" + sha(encode(p))


def ref_info(ref):
    match = REF_RE.fullmatch(ref)
    need(match is not None, "launch_ref")
    c, digest, previous = match.groups()
    return c, digest, int(previous) if previous else None


def decode_payload(secret):
    need(type(secret) is str and 0 < len(secret) <= legacy.SECRET_LIMIT, "secret_size")
    with gzip.GzipFile(fileobj=io.BytesIO(base64.b64decode(secret, validate=True))) as f:
        raw = f.read(legacy.MAX_PAYLOAD + 1)
    need(len(raw) <= legacy.MAX_PAYLOAD, "payload_size")
    p = legacy.validate(json.loads(raw))
    need(raw == encode(p), "payload_canonical")
    return p


def validate_ref(p, ref):
    c, digest, previous = ref_info(ref)
    need(c == cohort(p) and digest == sha(encode(p)) == FROZEN_PAYLOAD_SHA, "frozen_payload_pin")
    return c, previous


def pack(prepared, output):
    # Reuse the source-byte/quote/pin checks of the original frozen packer.
    with contextlib.redirect_stdout(io.StringIO()):
        legacy.pack(prepared, output)
    p = decode_payload(output.read_text())
    validate_ref(p, tag(p))
    print(tag(p))


def stage(root, ref):
    p = decode_payload(os.environ.pop("OWNER_EVAL_INPUT", ""))
    validate_ref(p, ref)
    root.mkdir(mode=0o700)
    save(root/"payload.json", p)
    cfg = root/"configs"
    cfg.mkdir(mode=0o700)
    for k, v in p["configs"].items():
        save(cfg/(k+".json"), v)
    print("Frozen input verified: 24 questions, no Gold")


def build_check(path):
    b = load(path)
    need(b["built"] is True and b["source_identical_except_declared_changes"] is True and set(b["changed_files"]) == CHANGED
         and b["cli_commit"] == CLI and b["provider_generation_authorized"] is False, "build_pin")
    for arm, pin in ARMS.items():
        a = b["arms"][arm]
        need(a["engine_commit"] == pin and sha(Path(a["binary"]).read_bytes()) == a["binary_sha256"], "binary_pin")
    return b


def argv(root, b, row, arm, index, plan=None):
    return legacy.argv(root, b, row, arm, index, plan) + ["--mistral-accounted-retry"]


def preflight(root, build, ref):
    p = legacy.validate(load(root/"payload.json"))
    validate_ref(p, ref)
    b = build_check(build)
    for k, v in p["configs"].items():
        need((root/"configs"/(k+".json")).read_bytes() == encode(v)+b"\n", "config_drift")
    out = root/"plans"
    out.mkdir(mode=0o700)
    hashes = []
    for i, row in enumerate(p["questions"]):
        pair = []
        for arm in ARMS:
            dest = out/f"{i:02}-{arm}.json"
            cp = subprocess.run(argv(root,b,row,arm,i,dest), cwd=out, stdin=subprocess.DEVNULL,
                                env={"PATH":os.environ["PATH"],"HOME":str(out),"NO_COLOR":"1"},
                                capture_output=True, timeout=35)
            need(cp.returncode == 0 and not cp.stdout and not cp.stderr, "request_plan_failure")
            d = load(dest)
            need(d["question"] == row["question"] and d["model"] == MODEL and d["provider"] == "mistral", "request_identity")
            pair.append(d)
        need(pair[0]["request"] == pair[1]["request"] and pair[0]["request_sha256"] == pair[1]["request_sha256"], "pair_mismatch")
        hashes.append(pair[0]["request_sha256"])
    need(len(set(hashes)) == 24, "duplicate_request")
    save(root/"preflight.json", {"tag":tag(p),"hashes":hashes,
         "config_hashes":{k:sha((root/"configs"/(k+".json")).read_bytes()) for k in p["configs"]},
         "pair_count":24,"measurement_policy":POLICY})
    print("48 provider-free request plans; all 24 pairs identical")


def context(ref):
    need(os.environ.get("GITHUB_ACTIONS") == "true"
         and os.environ.get("GITHUB_RUN_ATTEMPT") == "1"
         and os.environ.get("GITHUB_EVENT_NAME") == "push"
         and os.environ.get("GITHUB_REPOSITORY") == REPO
         and os.environ.get("GITHUB_REF") == "refs/tags/"+ref
         and re.fullmatch(r"[0-9a-f]{40}", os.environ.get("GITHUB_SHA",""))
         and os.environ.get("GITHUB_RUN_ID","").isdigit(), "one_shot_context")
    head = subprocess.check_output(["git","rev-parse","HEAD"],cwd=REPO_ROOT).decode().strip()
    need(head == os.environ["GITHUB_SHA"], "checkout_pin")
    return head, int(os.environ["GITHUB_RUN_ID"])


def api(path, *args):
    cp = subprocess.run(["gh", "api", f"repos/{REPO}/{path}", *args],
                        stdin=subprocess.DEVNULL, capture_output=True, timeout=35)
    need(cp.returncode == 0, "github_api_refused")
    return json.loads(cp.stdout)


def matching(prefix):
    rows = api("git/matching-refs/tags/"+prefix)
    need(type(rows) is list, "ref_inventory")
    result = {}
    for row in rows:
        name = row["ref"].removeprefix("refs/tags/")
        need(name.startswith(prefix) and name not in result and row["object"]["type"] == "commit", "ref_inventory")
        result[name] = row["object"]["sha"]
    return result


def exact_ref(name):
    rows = matching(name)
    return rows.get(name)


def create_ref(name, head):
    need(len(("refs/tags/"+name).encode("utf-8")) <= 255, "github_ref_length")
    row = api("git/refs", "--method", "POST", "-f", "ref=refs/tags/"+name, "-f", "sha="+head)
    need(row["ref"] == "refs/tags/"+name and row["object"]["sha"] == head, "ref_create_pin")


def names(c):
    return (f"rhq2-cohort-{c}", f"rhq2-slot-{c}-",
            f"rhq2-s-{c}-", f"rhq2-halt-{c}")


def artifact_name(c, slot, run):
    return f"owner-paired-v4-{c[:16]}-{run}-{slot}"


def check_artifact(c, slot, run, artifact, zip_digest):
    # Upload metadata can become visible shortly after a successful upload.
    for i in range(3):
        try:
            row = api(f"actions/artifacts/{artifact}")
            break
        except (ValueError, subprocess.SubprocessError):
            if i == 2:
                raise
            time.sleep(2)
    need(row["id"] == artifact and row["name"] == artifact_name(c,slot,run)
         and row["expired"] is False and row["size_in_bytes"] > 0
         and row["workflow_run"]["id"] == run
         and row.get("digest") == "sha256:"+zip_digest, "artifact_pin")


def verify_seal(c, slot, head, seals=None):
    _, _, prefix, _ = names(c)
    slot_prefix = prefix + slot + "-"
    rows = matching(slot_prefix) if seals is None else {k:v for k,v in seals.items() if k.startswith(slot_prefix)}
    need(len(rows) == 1, "claimed_slot_not_uniquely_sealed")
    name, ref_head = next(iter(rows.items()))
    match = re.fullmatch(re.escape(slot_prefix)+rf"([1-9][0-9]*)-([1-9][0-9]*)-({HEX})-({HEX})", name)
    need(match is not None and ref_head == head, "seal_pin")
    run, artifact, digest, zip_digest = match.groups()
    check_artifact(c,slot,int(run),int(artifact),zip_digest)
    return digest


def verify_existing(c, head):
    _, slot_prefix, seal_prefix, halt = names(c)
    need(exact_ref(halt) is None, "cohort_halted")
    claims, seals = matching(slot_prefix), matching(seal_prefix)
    slots = {name.removeprefix(slot_prefix) for name in claims}
    need(slots <= SLOTS and all(v == head for v in claims.values()), "claim_inventory")
    for slot in slots:
        verify_seal(c,slot,head,seals)
    need(len(seals) == len(slots), "orphan_seal")
    return slots


def admit(root, ref):
    head, run = context(ref)
    p = legacy.validate(load(root/"payload.json"))
    c, previous = validate_ref(p,ref)
    cohort_ref, _, _, halt = names(c)
    need(exact_ref(halt) is None, "cohort_halted")
    if previous is None:
        need(exact_ref(cohort_ref) is None and not verify_existing(c,head), "existing_cohort")
        create_ref(cohort_ref,head)
        print(f"New diagnostic cohort claimed; all {len(SLOTS)} selected slots unclaimed")
    else:
        need(previous != run and exact_ref(cohort_ref) == head, "resume_cohort")
        current, prior = api(f"actions/runs/{run}"), api(f"actions/runs/{previous}")
        prior_ref = prior["head_branch"]
        need(prior["id"] == previous and prior["status"] == "completed" and prior["run_attempt"] == 1
             and prior["event"] == "push" and prior["head_sha"] == head
             and current["workflow_id"] == prior["workflow_id"]
             and (prior_ref == tag(p) or re.fullmatch(re.escape(tag(p))+r"-resume-[1-9][0-9]*", prior_ref)),
             "prior_run_not_proven_terminal")
        slots = verify_existing(c,head)
        need(len(slots) < len(SLOTS), "all_slots_already_sealed")
        print(f"Resume admitted; {len(slots)} sealed, {len(SLOTS)-len(slots)} unclaimed; no replay")


def position(shard, step):
    need(type(shard) is int and 0 <= shard < 8 and type(step) is int and 0 <= step < 6, "slot_index")
    i = shard*3 + step//2
    order = tuple(ARMS) if i%2 == 0 else tuple(reversed(ARMS))
    arm = order[step%2]
    return i, arm, f"{i:02}-{arm}"


def pins(root, build_path, ref, shard, step):
    p = legacy.validate(load(root/"payload.json"))
    c, previous = validate_ref(p,ref)
    b, pre = build_check(build_path), load(root/"preflight.json")
    need(pre["tag"] == tag(p) and pre["pair_count"] == 24 and len(pre["hashes"]) == 24
         and pre["measurement_policy"] == POLICY, "preflight_pin")
    i, arm, slot = position(shard,step)
    need(slot in SLOTS, "diagnostic_slot_scope")
    row, a = p["questions"][i], b["arms"][arm]
    config = root/"configs"/(row["source_id"]+".json")
    need(sha(config.read_bytes()) == pre["config_hashes"][row["source_id"]], "config_drift")
    return p,b,c,previous,i,arm,slot,{"case_id":row["case_id"],"arm":arm,"slot":slot,
        "request_sha256":pre["hashes"][i],"binary_sha256":a["binary_sha256"],
        "config_sha256":pre["config_hashes"][row["source_id"]],"measurement_policy":POLICY}


def output(name,value):
    path = Path(os.environ["GITHUB_OUTPUT"])
    with path.open("a") as f:
        f.write(f"{name}={value}\n")


def claim(root, build, ref, shard, step):
    head, run = context(ref)
    _,_,c,previous,_,_,slot,reservation = pins(root,build,ref,shard,step)
    cohort_ref, prefix, _, halt = names(c)
    need(exact_ref(cohort_ref) == head and exact_ref(halt) is None, "cohort_admission")
    name = prefix+slot
    existing = exact_ref(name)
    if existing is not None:
        need(previous is not None and existing == head, "duplicate_slot")
        verify_seal(c,slot,head)
        output("run","false")
        print("Already sealed; skipping immutable slot "+slot)
        return
    create_ref(name,head)
    path = root/"raw"/slot
    path.mkdir(mode=0o700,parents=True)
    save(path/"reservation.json", {**reservation,"cohort":c,"tag":ref,"run_id":run,
        "head":head,"claim":name,"attempt":1,"shard":shard,"step":step})
    output("run","true")
    output("slot",slot)
    output("artifact",artifact_name(c,slot,run))
    print("Claimed previously unexecuted slot "+slot)


ATTEMPT_RE = re.compile(r"\[mistral-accounted-attempt\] status=(200|429) attempt=([1-4]) usage=(reported|reserved) timestamp_ms=([0-9]+) charged_input=([0-9]+) charged_output=([0-9]+) charged_total=([0-9]+)")
DECISION_RE = re.compile(r"\[mistral-rate-limit\] status=429 timestamp_ms=[0-9]+ attempt=([1-4]) retry_index=([0-3]) class=([a-z_]+) retry_allowed=(true|false) reason=([a-z_]+) wait_ms=([0-9]+) usage=(reported|reserved|unknown|invalid) headers=.*")
WAIT_RE = re.compile(r"\[mistral-retry-wait\] attempt=([1-3]) requested_ms=([0-9]+) elapsed_ms=([0-9]+)")
PACING_RE = re.compile(r"\[mistral-pacing-wait\] requested_ms=([0-9]+) elapsed_ms=([0-9]+)")


def accounting(stderr, usage):
    attempts, decisions, waits, pacing = [], [], [], []
    for raw in stderr.decode("utf-8",errors="strict").splitlines():
        for prefix, regex, target in [
            ("[mistral-accounted-attempt]",ATTEMPT_RE,attempts),
            ("[mistral-rate-limit] status=429",DECISION_RE,decisions),
            ("[mistral-retry-wait]",WAIT_RE,waits),
            ("[mistral-pacing-wait]",PACING_RE,pacing),
        ]:
            if raw.startswith(prefix):
                match = regex.fullmatch(raw)
                need(match is not None, "malformed_attempt_telemetry")
                target.append(match.groups())
                break
    need(len(attempts)==usage["provider_attempts"], "http_attempt_inventory")
    reported = [0,0,0]
    reserved = [0,0,0]
    di = wi = 0
    last_retry = False
    previous_attempt = 0
    terminal = None
    for status, attempt, state, _, inp, out, total in attempts:
        need(terminal is None or terminal["retry_allowed"], "request_after_terminal_429")
        attempt, inp, out, total = map(int,(attempt,inp,out,total))
        need(inp+out==total and out<=1024 and attempt==(previous_attempt+1 if last_retry else 1), "attempt_accounting")
        need(state=="reported" or status=="429", "reserved_success")
        target = reported if state=="reported" else reserved
        for j,v in enumerate((inp,out,total)):
            target[j] += v
        previous_attempt = attempt
        last_retry = False
        terminal = None
        if status=="429":
            need(di<len(decisions), "missing_429_decision")
            a, retry_index, cls, allowed, reason, wait, us = decisions[di]
            di += 1
            need(int(a)==attempt and int(retry_index)==attempt-1 and us==state
                 and cls in ("temporary_capacity","temporary_rate_limit"), "429_decision_pin")
            terminal = {"class":cls,"reason":reason,"retry_allowed":allowed=="true"}
            if allowed=="true":
                need(reason=="retry" and wi<len(waits), "missing_retry_wait")
                wa, requested, elapsed = map(int,waits[wi])
                wi += 1
                need(wa==attempt and requested==int(wait) and 0<requested<=120000 and elapsed+1>=requested, "retry_wait_pin")
                last_retry=True
            else:
                need(int(wait)==0, "terminal_wait_pin")
    need(not last_retry and di==len(decisions) and wi==len(waits), "unclosed_attempt")
    need([reported[j]+reserved[j] for j in range(3)] == [usage[k] for k in ("input_tokens","output_tokens","total_tokens")], "cli_accounting_mismatch")
    for requested,elapsed in pacing:
        need(0<int(requested)<=90000 and int(elapsed)+1>=int(requested), "pacing_wait_pin")
    return {"reported_tokens":dict(zip(("input","output","total"),reported)),
            "reserved_unknown_tokens":dict(zip(("input","output","total"),reserved)),
            "actual_total_tokens":reported[2] if reserved[2]==0 else None,
            "unknown_usage_attempts":sum(a[2]=="reserved" for a in attempts),
            "http_attempts":len(attempts),"retries":len(waits),
            "retry_wait_ms":sum(int(w[2]) for w in waits),
            "retry_requested_wait_ms":sum(int(w[1]) for w in waits),
            "pacing_wait_ms":sum(int(w[1]) for w in pacing),
            "terminal_429":terminal}


def terminal_observation(stdout, stderr, rc, obs):
    data = json.loads(stdout)
    need(data["schema_version"]=="reason-cli-output-v1" and data["command"]=="ask", "cli_envelope")
    result = data["result"]
    report = result.get("usage") or (result.get("partial_diagnostics") or {}).get("usage")
    need(type(report) is dict and report["token_accounting"]==ACCOUNTING, "accounting_policy_missing")
    need(all(report["budget"].get(k)==v for k,v in CAPS.items()), "cli_budget_pin")
    used = report["run"]
    need(used["tracked_from_session_start"] is True
         and all(type(used.get(k)) is int and used[k]>=0 for k in USAGE_FIELDS), "unknown_usage")
    usage = {k:used[k] for k in USAGE_FIELDS}
    need(usage["model_calls"]<=CAPS["max_model_calls"]
         and usage["output_tokens"]<=CAPS["max_output_tokens"]
         and usage["total_tokens"]<=CAPS["max_total_tokens"]
         and usage["input_tokens"]+usage["output_tokens"]==usage["total_tokens"], "usage_ceiling")
    telemetry = accounting(stderr,usage)
    obs.update(usage=usage,telemetry=telemetry,answer_status=None)
    final = result.get("finalization")
    if rc==0 and result.get("status")!="failed":
        need(type(final) is dict and type(final.get("text")) is str and type(final.get("status")) is str
             and usage["model_calls"]>0 and usage["provider_attempts"]>0, "display_missing")
        obs.update(status="complete",final_sha256=sha(final["text"].encode()),answer_status=final["status"])
        return
    need(rc==1 and result.get("status")=="failed" and final is None, "nonterminal_failure")
    f = result.get("failure") or {}
    cls, message = f.get("failure_class"), f.get("message","")
    expected = {
        f"output-token budget exhausted ({CAPS['max_output_tokens']}/{CAPS['max_output_tokens']})":usage["output_tokens"]==CAPS["max_output_tokens"],
        f"total-token budget exhausted ({CAPS['max_total_tokens']}/{CAPS['max_total_tokens']})":usage["total_tokens"]==CAPS["max_total_tokens"],
        f"model call budget exhausted ({CAPS['max_model_calls']}/{CAPS['max_model_calls']})":usage["model_calls"]==CAPS["max_model_calls"],
    }
    if cls=="usage_budget_exceeded" and expected.get(message,False):
        status="budget_exhausted"
    elif cls=="usage_budget_exceeded" and "mistral_attempt_reservation_exhausted" in message:
        status="budget_reserve_exhausted"
    elif cls=="rate_limit" and telemetry["terminal_429"] is not None:
        t=telemetry["terminal_429"]
        need(not t["retry_allowed"] and t["reason"] in ("attempt_limit","token_reservation_limit")
             and f"class={t['class']}" in message and f"retry_decision={t['reason']}" in message, "unresolved_rate_limit")
        status="transient_failed"
    elif cls=="protocol" and (f.get("model_output_diagnostic") or {}).get("origin")=="local_model_output_decode":
        status="local_decode_failed"
    else:
        raise ValueError("global_stop_failure")
    obs.update(status=status,failure_class=cls)


def billing(key):
    opener=urllib.request.build_opener(legacy.NoRedirect())
    result=legacy.billing_for_policy(*(legacy.get_json(opener,key,path) for path in ("/v1/billing/subscription","/v1/billing/budget","/v1/models")))
    result.update(measurement_policy=POLICY)
    return result


def interrupt(_sig,_frame):
    raise InterruptedError("runner_cancelled")


def run_cli(args, path, key):
    home=path/"home"
    home.mkdir(mode=0o700)
    fd1=os.open(path/"stdout.json",os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
    fd2=os.open(path/"stderr.log",os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
    rc=None
    with os.fdopen(fd1,"wb") as stdout, os.fdopen(fd2,"wb") as stderr:
        child=None
        try:
            child=subprocess.Popen(args,cwd=home,stdin=subprocess.DEVNULL,stdout=stdout,stderr=stderr,
                start_new_session=True,env={"HOME":str(home),"TMPDIR":str(home),"PATH":os.environ["PATH"],
                "MISTRAL_API_KEY":key,"NO_COLOR":"1","NO_PROXY":"127.0.0.1","REASON_MISTRAL_RATE_LIMIT_TELEMETRY":"1"})
            rc=child.wait(timeout=SLOT_TIMEOUT)
        finally:
            if child is not None and child.poll() is None:
                os.killpg(child.pid,signal.SIGKILL)
                child.wait()
            for f in (stdout,stderr):
                f.flush()
                os.fsync(f.fileno())
    return rc


def execute(root,build,ref,shard,step):
    head,run=context(ref)
    p,b,c,_,i,arm,slot,reservation=pins(root,build,ref,shard,step)
    path=root/"raw"/slot
    existing=load(path/"reservation.json")
    need(existing=={**reservation,"cohort":c,"tag":ref,"run_id":run,
        "head":head,"claim":names(c)[1]+slot,"attempt":1,"shard":shard,"step":step}, "claim_receipt")
    need(not (path/"started.json").exists() and not (path/"observation.json").exists(), "duplicate_execution")
    need("GH_TOKEN" not in os.environ and "GITHUB_TOKEN" not in os.environ, "write_token_in_live_step")
    key=os.environ.pop("MISTRAL_API_KEY","")
    need(bool(key),"credential")
    obs={**existing,"status":"incomplete","exit_code":None,"usage":None}
    began=time.monotonic()
    try:
        save(path/"billing.json",billing(key))
        print("Fresh same-key billing gate PASS; API PAYG disabled; fixed model available",flush=True)
        save(path/"started.json",{"state":"launch_committed","time_utc":utc_now(),"run_id":run,"slot":slot})
        rc=run_cli(argv(root,b,p["questions"][i],arm,i),path,key)
        obs["exit_code"]=rc
        stdout=(path/"stdout.json").read_bytes()
        stderr=(path/"stderr.log").read_bytes()
        terminal_observation(stdout,stderr,rc,obs)
    finally:
        for f in ("stdout.json","stderr.log"):
            if (path/f).is_file():
                obs[f.replace(".json","").replace(".log","")+"_sha256"]=sha((path/f).read_bytes())
        obs["elapsed_ms"]=round((time.monotonic()-began)*1000)
        save(path/"observation.json",obs)
    need(obs["status"] in TERMINAL,"slot_unsealed")
    print(f"Slot {slot}: {obs['status']} (no observation replay)")


def verify_cert():
    der=subprocess.check_output(["openssl","x509","-in",str(CERT),"-outform","DER"])
    need(sha(der)==CERT_SHA,"recipient_pin")
    subprocess.run(["openssl","x509","-in",str(CERT),"-checkend","21600","-noout"],check=True,capture_output=True)


def encrypt(root,shard,step):
    _,_,slot=position(shard,step)
    path=root/"raw"/slot
    verify_cert()
    files=[f for f in ("reservation.json","billing.json","started.json","stdout.json","stderr.log","observation.json") if (path/f).is_file()]
    need("reservation.json" in files,"missing_reservation")
    target=root/(slot+".cms")
    fd=os.open(target,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
    with os.fdopen(fd,"wb") as out:
        tar=subprocess.Popen(["tar","-czf","-","-C",str(path),*files],stdout=subprocess.PIPE,stderr=subprocess.DEVNULL)
        try:
            cp=subprocess.run(["openssl","cms","-encrypt","-binary","-aes256","-outform","DER",str(CERT)],
                              stdin=tar.stdout,stdout=out,stderr=subprocess.DEVNULL,timeout=60)
            tar.stdout.close()
            need(cp.returncode==0 and tar.wait(timeout=30)==0,"encryption_failed")
            out.flush()
            os.fsync(out.fileno())
        finally:
            if tar.poll() is None:
                tar.kill()
                tar.wait()
    need(target.stat().st_size>0,"empty_encrypted_evidence")
    # A truncated CMS must never acquire a terminal evidence seal.
    subprocess.run(["openssl","cms","-cmsout","-inform","DER","-in",str(target),"-noout"],
                   check=True,capture_output=True,timeout=30)
    save(root/(slot+".encrypted.json"), {"sha256":sha(target.read_bytes()),
         "file_hashes":{f:sha((path/f).read_bytes()) for f in files}})


def halt_cohort(c,head):
    halt=names(c)[3]
    existing=exact_ref(halt)
    if existing is None:
        create_ref(halt,head)
    else:
        need(existing==head,"halt_pin")


def seal(root,ref,shard,step,artifact,artifact_digest):
    head,run=context(ref)
    c,_,_=ref_info(ref)
    _,_,slot=position(shard,step)
    path=root/"raw"/slot
    try:
        reservation=load(path/"reservation.json")
        obs=load(path/"observation.json")
        need(all(obs.get(k)==v for k,v in reservation.items()) and reservation["run_id"]==run
             and reservation["head"]==head and reservation["slot"]==slot and obs["status"] in TERMINAL,
             "unsealed_terminal")
        need(exact_ref(names(c)[1]+slot)==head,"claim_pin")
        for f,key in (("stdout.json","stdout_sha256"),("stderr.log","stderr_sha256")):
            need(sha((path/f).read_bytes())==obs[key],"evidence_drift")
        # Revalidate all accounting and terminal classification independently of the saved status.
        checked={}
        terminal_observation((path/"stdout.json").read_bytes(),(path/"stderr.log").read_bytes(),obs["exit_code"],checked)
        need(all(obs.get(k)==v for k,v in checked.items()),"terminal_drift")
        gate=load(path/"billing.json")
        need(gate.get("api_payg_disabled") is True and gate.get("fresh_authenticated_account_status") is True
             and gate.get("minimum_headroom_met") is True and gate.get("fixed_model_available") is True
             and gate.get("measurement_policy")==POLICY and (path/"started.json").is_file(), "billing_evidence")
        zip_digest=artifact_digest.removeprefix("sha256:")
        need(re.fullmatch(r"[1-9][0-9]*",artifact) is not None and re.fullmatch(HEX,zip_digest) is not None,"artifact_id")
        check_artifact(c,slot,run,int(artifact),zip_digest)
        encrypted=root/(slot+".cms")
        need(encrypted.is_file() and encrypted.stat().st_size>0,"encrypted_evidence_missing")
        digest=sha(encrypted.read_bytes())
        ready=load(root/(slot+".encrypted.json"))
        files=("reservation.json","billing.json","started.json","stdout.json","stderr.log","observation.json")
        need(ready=={"sha256":digest,"file_hashes":{f:sha((path/f).read_bytes()) for f in files}}, "encryption_proof")
        prefix=names(c)[2]+slot+"-"
        need(not matching(prefix),"duplicate_seal")
        create_ref(prefix+f"{run}-{artifact}-{digest}-{zip_digest}",head)
        print("Immutable evidence seal created for "+slot)
    except BaseException:
        halt_cohort(c,head)
        raise


def cooldown(root,shard,step):
    _,_,slot=position(shard,step)
    obs=load(root/"raw"/slot/"observation.json")
    need(obs["status"] in TERMINAL,"cooldown_terminal")
    time.sleep(120 if obs["status"]=="transient_failed" else 60)


def main():
    os.umask(0o077)
    signal.signal(signal.SIGTERM,interrupt)
    signal.signal(signal.SIGINT,interrupt)
    parser=argparse.ArgumentParser()
    sub=parser.add_subparsers(dest="cmd",required=True)
    t=sub.add_parser("pack");t.add_argument("prepared",type=Path);t.add_argument("output",type=Path)
    for action in ("stage","admit","preflight","claim","execute","seal","encrypt","cooldown"):
        t=sub.add_parser(action);t.add_argument("root",type=Path)
        if action in ("preflight","claim","execute"):t.add_argument("build",type=Path)
        if action not in ("encrypt","cooldown"):t.add_argument("ref")
        if action in ("claim","execute","seal","encrypt","cooldown"):
            t.add_argument("shard",type=int);t.add_argument("step",type=int)
        if action=="seal":
            t.add_argument("artifact");t.add_argument("artifact_digest")
    a=parser.parse_args()
    kwargs=vars(a);cmd=kwargs.pop("cmd")
    globals()[cmd](**kwargs)


if __name__=="__main__":
    try:
        main()
    except BaseException as e:
        # Only a fixed diagnostic class, never provider contents or credentials.
        print("HOLD ("+(str(e) if isinstance(e,HoldError) else type(e).__name__)+")",file=sys.stderr)
        sys.exit(2)
