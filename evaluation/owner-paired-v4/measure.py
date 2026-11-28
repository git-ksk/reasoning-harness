"""Owner development comparison: per-observation immutable claims and sealed evidence.

No access to Gold. Never rerun a claimed observation. Only an explicitly
classified and fully accounted local failure may leave the remaining slots open.
"""
import argparse
import base64
import contextlib
import datetime
import gzip
import hashlib
import hmac
import importlib.util
import io
import json
import os
import platform
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
from build import CLI, ARMS, CHANGED, COMMON_PATCH_SHA

encode, sha, save, load = legacy.encode, legacy.sha, legacy.save, legacy.load
class HoldError(ValueError):
    pass
def need(condition, reason):
    if not condition:
        raise HoldError(reason)
MODEL = "gemini-3.5-flash-lite"
PROVIDER = "google"
REPO = "git-ksk/reasoning-harness"
PREFIX = "eval-owner-paired-gemini-ci-focus-"
FROZEN_PAYLOAD_SHA = "6a7d305f035c57e61fb0283f63f3b06015f5f4a9d49bd0ea47c122521dbac7d5"
CERT = HERE.parent/"engine073-531-provisional24-20261114/reviewer-cert.pem"
CERT_SHA = "d917aeb4c55699b1ea19117c776e5f515aa40193488197a12c00082517695b43"
CAPS = {"max_model_calls": 12, "max_output_tokens": 8192, "max_total_tokens": 32192}
SLOT_TIMEOUT = 2700
ACCOUNTING = "reported_plus_conservative_reservations_v1"
POLICY = {
    "id": "owner-paired-gemini-ci-focus-20261128", **CAPS,
    "common_patch_sha256": COMMON_PATCH_SHA,
    "runner_os": "Linux", "runner_arch": "x86_64", "build_strategy": "one_serial_job",
    "provider": PROVIDER, "max_generation_output": 1024,
    "max_retries_per_http_logical_request": 3, "retry_status": 503,
    "backoff_seconds": [30,60,120], "max_retry_wait_seconds": 210,
    "minimum_request_interval_ms": 30000,
    "usage_ceiling": "each attempt reserves max(logical JSON UTF8 bytes,wire JSON UTF8 bytes)+512 input and requested output <=1024; unknown 503 debited full reservation; other unknown usage halts",
    "free_tier": "owner project billing disabled at pinned admission; exact key digest match; no billing changes",
    "token_accounting": ACCOUNTING,
    "slot_timeout_seconds": SLOT_TIMEOUT, "inter_slot_seconds": 60,
    "transient_failure_cooldown_seconds": 120,
    "predecessor_runs": [45504130723, 45628519549, 45637587367, 45639690396, 45640828200, 45643273890, 46148253145, 46148450513],
    "selected_questions": [3, 6, 7, 8, 12, 13, 14],
    "classification": "post_outcome_selected_development_diagnostic_not_independent_holdout",
}
TERMINAL = {"complete", "budget_exhausted", "budget_reserve_exhausted", "local_decode_failed"}
SHARDS = tuple(sorted({i//3 for i in POLICY["selected_questions"]}))
SLOTS = {f"{i:02}-{arm}" for i in POLICY["selected_questions"] for arm in ARMS}
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
    if os.environ.get("GITHUB_ACTIONS") == "true":
        need(b.get("runner_os") == POLICY["runner_os"] == platform.system()
             and b.get("runner_arch") == POLICY["runner_arch"] == platform.machine(), "runtime_platform_pin")
    need(b["built"] is True and b["source_identical_except_declared_changes"] is True and set(b["changed_files"]) == CHANGED
         and b["cli_commit"] == CLI and b.get("common_patch_sha256")==COMMON_PATCH_SHA and b["provider_generation_authorized"] is False, "build_pin")
    for arm, pin in ARMS.items():
        a = b["arms"][arm]
        need(a["engine_commit"] == pin and sha(Path(a["binary"]).read_bytes()) == a["binary_sha256"], "binary_pin")
    return b


def argv(root, b, row, arm, index, plan=None):
    a = legacy.argv(root, b, row, arm, index, plan)
    a[a.index("--provider")+1] = PROVIDER
    a[a.index("--model")+1] = MODEL
    return a+["--google-accounted-retry"]


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
            need(d["question"] == row["question"] and d["model"] == MODEL and d["provider"] == PROVIDER, "request_identity")
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
    return (f"rhg2-cohort-{c}", f"rhg2-slot-{c}-",
            f"rhg2-s-{c}-", f"rhg2-halt-{c}")


def artifact_name(c, slot, run):
    return f"owner-paired-gemini-r2-{c[:16]}-{run}-{slot}"


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


def verify_unstarted_macos_predecessor():
    prior = api("actions/runs/45639690396")
    need(prior["status"] == "completed" and prior["conclusion"] == "cancelled"
         and prior["head_sha"] == "b42673fd530c04b21f0391f301e4fe0dfae3d849"
         and prior["run_attempt"] == 1, "macos_predecessor_not_cancelled")
    old = "f72a53629dc39f4298f63d03a1cf7142bf2a17827240dccad6ed54583d8facf8"
    need(all(not matching("rhg2-"+kind+"-"+old) for kind in ("slot","s","halt"))
         and api("actions/runs/45639690396/artifacts")["total_count"] == 0,
         "macos_predecessor_observations_exist")


def verify_failed_ubuntu_predecessor():
    prior=api("actions/runs/45640828200")
    need(prior["status"]=="completed" and prior["conclusion"]=="failure"
         and prior["head_sha"]=="87395e1e32d92f8c851eed16aa32a802f6a1da1c"
         and prior["run_attempt"]==1,"ubuntu_predecessor_not_terminal")
    old="53c3403a7a527fc62c8a82195150d2cc101337bae1b4d1be6f4708516d74e6f1"
    need(matching("rhg2-slot-"+old)=={"rhg2-slot-"+old+"-03-candidate":prior["head_sha"]}
         and not matching("rhg2-s-"+old)
         and matching("rhg2-halt-"+old)=={"rhg2-halt-"+old:prior["head_sha"]},
         "ubuntu_predecessor_inventory")
    artifact=api("actions/artifacts/15035198762")
    need(artifact["id"]==15035198762 and artifact["workflow_run"]["id"]==45640828200
         and artifact.get("digest")=="sha256:419c739392cd9fcf19902e7670cf328889ea56294da5983947a8d8181bbb23d2"
         and artifact["expired"] is False,"ubuntu_predecessor_artifact")


def verify_ci_pilot_predecessors():
    prior = api("actions/runs/45643273890")
    head = "1b4027c727992cac8713db3347eee174caafb1d1"
    old = "e186e248a996b227bf00ad390323ac1f5433f8b28ad8461000ece99dfbce3b5d"
    need(prior["status"] == "completed" and prior["conclusion"] == "failure"
         and prior["head_sha"] == head and prior["run_attempt"] == 1, "prior_accounted_run")
    need(matching("rhg2-slot-"+old) == {"rhg2-slot-"+old+"-03-candidate":head}
         and not matching("rhg2-s-"+old)
         and matching("rhg2-halt-"+old) == {"rhg2-halt-"+old:head}, "prior_accounted_inventory")
    artifact = api("actions/artifacts/15037681156")
    need(artifact["workflow_run"]["id"] == 45643273890 and artifact.get("digest") ==
         "sha256:db46502d152f97812b06e25662f9dc427d63fb6b77430f4530ae35d5f7a62707"
         and not artifact["expired"], "prior_accounted_artifact")
    smoke = api("actions/runs/46148253145")
    need(smoke["status"] == "completed" and smoke["conclusion"] == "success"
         and smoke["head_sha"] == "2395298a0d343b40160dd1d1fa07979652cb9ce2"
         and smoke["run_attempt"] == 1, "ci_smoke_not_verified")


def verify_completed_pilot():
    c = "d621641381b3fc4d1510ad83e7ac8b48c9746f50ef9068e1a633bd846e7d9c25"
    head = "f9e202c7f36b7c01db0b245d41a60244bda755f2"
    run = api("actions/runs/46148450513")
    need(run["status"] == "completed" and run["conclusion"] == "success"
         and run["head_sha"] == head and run["run_attempt"] == 1, "pilot_run_not_complete")
    cohort_ref, cp, sp, halt = names(c)
    slots = {"04-baseline","04-candidate","05-baseline","05-candidate"}
    need(exact_ref(cohort_ref) == head and exact_ref(halt) is None, "pilot_cohort_pin")
    need(matching(cp) == {cp+slot:head for slot in slots}, "pilot_claim_inventory")
    seals = matching(sp)
    need(len(seals) == len(slots), "pilot_seal_inventory")
    for slot in sorted(slots):
        verify_seal(c,slot,head,seals)


def admit(root, ref):
    head, run = context(ref)
    p = legacy.validate(load(root/"payload.json"))
    c, previous = validate_ref(p,ref)
    cohort_ref, _, _, halt = names(c)
    need(exact_ref(halt) is None, "cohort_halted")
    if previous is None:
        verify_unstarted_macos_predecessor()
        verify_failed_ubuntu_predecessor()
        verify_ci_pilot_predecessors()
        verify_completed_pilot()
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


def accounting(raw, usage):
    events=[json.loads(line) for line in raw.splitlines() if line.strip()]
    reported=[0,0];reserved=[0,0];debit=[0,0]
    closed=set();attempt_count=0;retries=0;unknown=0;wait_total=0
    terminal_503=None;pos=0
    def take(call,attempt,event,schema):
        nonlocal pos
        need(pos<len(events),"attempt_event_missing")
        e=events[pos];pos+=1
        need(e.get("schema_version")==schema and e.get("model")==MODEL
             and e.get("call_id")==call and type(e.get("attempt")) is int
             and e["attempt"]==attempt and e.get("event")==event,"attempt_event_identity")
        return e
    ordinary="reason-google-attempt-telemetry-v1";accounted="reason-google-accounted-v1"
    while pos<len(events):
        call=events[pos].get("call_id")
        need(isinstance(call,str) and call not in closed and terminal_503 is None,"duplicate_or_post_terminal_call")
        closed.add(call)
        retry_sum=0;first_reserve=None
        for a in range(1,5):
            take(call,a,"attempt_start",ordinary)
            h=take(call,a,"http_headers",ordinary)
            r=take(call,a,"http_response",ordinary)
            status=r.get("status_code")
            need(status in (200,503) and h.get("status_code")==status
                 and not h.get("error_class")
                 and (not r.get("error_class") if status==200 else r.get("error_class")=="provider_unavailable"),
                 "nonretryable_or_unaccounted_http")
            d=take(call,a,"accounted_attempt",accounted)
            vals=[d.get(k) for k in ("charged_input","charged_output","charged_total","reserved_input","reserved_output")]
            need(all(type(v) is int and v>=0 for v in vals),"invalid_debit")
            i,o,t,ri,ro=vals
            need(d.get("status_code")==status and i+o==t and i<=ri and o<=ro and ri>=512 and 1<=ro<=1024,"reservation_violation")
            need(first_reserve is None or first_reserve==(ri,ro),"retry_reservation_drift")
            first_reserve=(ri,ro)
            need(debit[1]+ro<=CAPS["max_output_tokens"] and sum(debit)+ri+ro<=CAPS["max_total_tokens"],"pre_attempt_reservation_ceiling")
            state=d.get("usage")
            if state=="reserved":
                need(status==503 and (i,o)==(ri,ro),"invalid_unknown_reservation")
                reserved[0]+=i;reserved[1]+=o;unknown+=1
            else:
                need(state=="reported","unknown_debit")
                reported[0]+=i;reported[1]+=o
            debit[0]+=i;debit[1]+=o;attempt_count+=1
            if status==200:break
            decision=take(call,a,"retry_decision",accounted)
            reason=decision.get("reason");delay=decision.get("wait_ms")
            need(type(delay) is int and decision.get("elapsed_ms")==0,"invalid_retry_decision")
            if reason!="retry":
                need(reason in ("attempt_limit","token_reservation_limit","retry_wait_limit","retry_hint_invalid","execution_wait_budget")
                     and delay==0,"invalid_terminal_503")
                if reason=="attempt_limit":need(a==4,"premature_attempt_limit")
                if reason=="token_reservation_limit":
                    need(debit[1]+ro>CAPS["max_output_tokens"] or sum(debit)+ri+ro>CAPS["max_total_tokens"],"false_reservation_limit")
                terminal_503=reason;break
            need(a<=3 and (30000,60000,120000)[a-1]<=delay<=120000
                 and retry_sum+delay<=210000
                 and debit[1]+ro<=CAPS["max_output_tokens"] and sum(debit)+ri+ro<=CAPS["max_total_tokens"],"retry_bound")
            w=take(call,a,"retry_wait",accounted)
            elapsed=w.get("elapsed_ms")
            need(w.get("reason")=="retry" and w.get("wait_ms")==delay and type(elapsed) is int and elapsed>=delay,"retry_wait_missing")
            retries+=1;wait_total+=elapsed;retry_sum+=delay
        else:
            raise HoldError("attempt_limit")
    need(attempt_count==usage["provider_attempts"] and debit==[usage["input_tokens"],usage["output_tokens"]]
         and sum(debit)==usage["total_tokens"],"attempt_accounting")
    need(attempt_count<=4*len(closed) and len(closed)<=2*CAPS["max_model_calls"],"attempt_count_ceiling")
    return {"http_attempts":attempt_count,"reported_tokens":{"input":reported[0],"output":reported[1],"total":sum(reported)},
            "actual_total_tokens":None if unknown else sum(reported),"unknown_usage_attempts":unknown,
            "reserved_unknown_tokens":{"input":reserved[0],"output":reserved[1],"total":sum(reserved)},
            "retries":retries,"retry_wait_ms":wait_total,"terminal_503":terminal_503,
            "pacing_interval_ms":POLICY["minimum_request_interval_ms"]}


def terminal_observation(stdout, telemetry_raw, rc, obs):
    data = json.loads(stdout)
    need(data["schema_version"]=="reason-cli-output-v1" and data["command"]=="ask", "cli_envelope")
    result = data["result"]
    report = result.get("usage") or (result.get("partial_diagnostics") or {}).get("usage")
    need(type(report) is dict and report.get("token_accounting") == ACCOUNTING, "unexpected_accounting_policy")
    need(all(report["budget"].get(k)==v for k,v in CAPS.items()), "cli_budget_pin")
    used = report["run"]
    need(used["tracked_from_session_start"] is True
         and all(type(used.get(k)) is int and used[k]>=0 for k in USAGE_FIELDS), "unknown_usage")
    usage = {k:used[k] for k in USAGE_FIELDS}
    need(usage["model_calls"]<=CAPS["max_model_calls"]
         and usage["output_tokens"]<=CAPS["max_output_tokens"]
         and usage["total_tokens"]<=CAPS["max_total_tokens"]
         and usage["input_tokens"]+usage["output_tokens"]==usage["total_tokens"], "usage_ceiling")
    telemetry = accounting(telemetry_raw,usage)
    obs.update(usage=usage,telemetry=telemetry,answer_status=None)
    final = result.get("finalization")
    if rc==0 and result.get("status")!="failed":
        need(telemetry["terminal_503"] is None,"concealed_terminal_503")
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
    if telemetry["terminal_503"] is not None:
        raise HoldError("terminal_503_"+telemetry["terminal_503"])
    if cls=="usage_budget_exceeded" and "google_attempt_reservation_exhausted" in message:
        status="budget_reserve_exhausted"
    elif cls=="usage_budget_exceeded" and expected.get(message,False):
        status="budget_exhausted"
    elif cls=="protocol" and (f.get("model_output_diagnostic") or {}).get("origin")=="local_model_output_decode":
        status="local_decode_failed"
    else:
        raise HoldError("global_stop_failure")
    obs.update(status=status,failure_class=cls)


from ci_probe import CHECKED as BILLING_CHECKED_UTC, EXPIRES as BILLING_EXPIRES_UTC

def billing(key):
    expected = os.environ.pop("ENGINE620_GEMINI_KEY_SHA256","")
    need(re.fullmatch(HEX,expected) is not None and
         hmac.compare_digest(hashlib.sha256(key.encode()).hexdigest(),expected),
         "billing_project_key_mismatch")
    need(datetime.datetime.fromisoformat(BILLING_CHECKED_UTC) <= datetime.datetime.now(datetime.timezone.utc)
         < datetime.datetime.fromisoformat(BILLING_EXPIRES_UTC), "billing_attestation_expired")
    url = "https://generativelanguage.googleapis.com/v1beta/models/"+MODEL
    opener = urllib.request.build_opener(legacy.NoRedirect())
    request = urllib.request.Request(url,headers={"x-goog-api-key":key,"Accept":"application/json"})
    try:
        with opener.open(request,timeout=20) as response:
            need(response.status==200 and response.geturl()==url,"metadata_origin")
            raw=response.read(1000001)
            need(len(raw)<=1000000,"metadata_size")
            need(json.loads(raw).get("name")=="models/"+MODEL,"model_metadata_mismatch")
    except urllib.error.HTTPError as exc:
        raise HoldError("gemini_metadata_http_"+str(exc.code)) from None
    return {"billing_gate":"PASS","project_billing_disabled_at_attestation":True,
             "credential_bound_to_verified_project":True,"fixed_model_available":True,
             "billing_checked_utc":BILLING_CHECKED_UTC,"billing_expires_utc":BILLING_EXPIRES_UTC,
             "measurement_policy":POLICY}


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
                "GEMINI_API_KEY":key,"NO_COLOR":"1","NO_PROXY":"127.0.0.1",
                "REASON_GOOGLE_MIN_REQUEST_INTERVAL_MS":str(POLICY["minimum_request_interval_ms"]),
                "REASON_GOOGLE_ATTEMPT_TELEMETRY_PATH":str((path/"google-attempts.jsonl").resolve())})
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
    key=os.environ.pop("GEMINI_API_KEY","")
    need(bool(key),"credential")
    obs={**existing,"status":"incomplete","exit_code":None,"usage":None}
    began=time.monotonic()
    try:
        save(path/"billing.json",billing(key))
        print("Billing-disabled project key matched; current model metadata PASS",flush=True)
        save(path/"started.json",{"state":"launch_committed","time_utc":utc_now(),"run_id":run,"slot":slot})
        (path/"google-attempts.jsonl").touch(exist_ok=False)
        rc=run_cli(argv(root,b,p["questions"][i],arm,i),path,key)
        obs["exit_code"]=rc
        stdout=(path/"stdout.json").read_bytes()
        stderr=(path/"stderr.log").read_bytes()
        terminal_observation(stdout,(path/"google-attempts.jsonl").read_bytes(),rc,obs)
    finally:
        for f in ("stdout.json","stderr.log","google-attempts.jsonl"):
            if (path/f).is_file():
                obs[{"stdout.json":"stdout_sha256","stderr.log":"stderr_sha256","google-attempts.jsonl":"google_attempts_sha256"}[f]]=sha((path/f).read_bytes())
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
    files=[f for f in ("reservation.json","billing.json","started.json","stdout.json","stderr.log","google-attempts.jsonl","observation.json") if (path/f).is_file()]
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
        for f,key in (("stdout.json","stdout_sha256"),("stderr.log","stderr_sha256"),("google-attempts.jsonl","google_attempts_sha256")):
            need(sha((path/f).read_bytes())==obs[key],"evidence_drift")
        # Revalidate all accounting and terminal classification independently of the saved status.
        checked={}
        terminal_observation((path/"stdout.json").read_bytes(),(path/"google-attempts.jsonl").read_bytes(),obs["exit_code"],checked)
        need(all(obs.get(k)==v for k,v in checked.items()),"terminal_drift")
        gate=load(path/"billing.json")
        need(gate.get("project_billing_disabled_at_attestation") is True
             and gate.get("credential_bound_to_verified_project") is True and gate.get("fixed_model_available") is True
             and gate.get("billing_checked_utc")==BILLING_CHECKED_UTC
             and gate.get("billing_expires_utc")==BILLING_EXPIRES_UTC
             and gate.get("measurement_policy")==POLICY and (path/"started.json").is_file(), "billing_evidence")
        zip_digest=artifact_digest.removeprefix("sha256:")
        need(re.fullmatch(r"[1-9][0-9]*",artifact) is not None and re.fullmatch(HEX,zip_digest) is not None,"artifact_id")
        check_artifact(c,slot,run,int(artifact),zip_digest)
        encrypted=root/(slot+".cms")
        need(encrypted.is_file() and encrypted.stat().st_size>0,"encrypted_evidence_missing")
        digest=sha(encrypted.read_bytes())
        ready=load(root/(slot+".encrypted.json"))
        files=("reservation.json","billing.json","started.json","stdout.json","stderr.log","google-attempts.jsonl","observation.json")
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
    time.sleep(60)


def main():
    os.umask(0o077)
    signal.signal(signal.SIGTERM,interrupt)
    signal.signal(signal.SIGINT,interrupt)
    parser=argparse.ArgumentParser()
    sub=parser.add_subparsers(dest="cmd",required=True)
    t=sub.add_parser("pack");t.add_argument("prepared",type=Path);t.add_argument("output",type=Path)
    sub.add_parser("entitlement")
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
    if cmd=="entitlement":
        key=os.environ.pop("GEMINI_API_KEY","")
        need(bool(key),"credential")
        billing(key)
        print("Verified project key and model metadata; no generation")
    else:
        globals()[cmd](**kwargs)


if __name__=="__main__":
    try:
        main()
    except BaseException as e:
        # Only a fixed diagnostic class, never provider contents or credentials.
        print("HOLD ("+(str(e) if isinstance(e,HoldError) else type(e).__name__)+")",file=sys.stderr)
        sys.exit(2)
