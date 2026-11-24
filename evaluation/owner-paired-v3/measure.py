"""Reusable owner-managed Engine 0.7.3 paired measurement, not formal independence."""
import argparse
import base64
import gzip
import hashlib
import io
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time
import urllib.request

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
sys.path.insert(0, str(REPO / "evaluation/dev-lite"))
from issue531_real24_api_billing_gate import check_account, get_json, NoRedirect
from run import usage_from_cli

ARMS = {"baseline": "4abee90f0501f0d8fc09b453a0b45c942fbed69e",
        "candidate": "770ac6b59b74b0889a9bbe2cc97b1675dc928819"}
CLI = "b85a4a0b2ab03397b361a8620d7a97dd26b2eb36"
MODEL = "ministral-8b-2512"
PREFIX = "eval-owner-paired-v3-"
HEX = re.compile(r"[0-9a-f]{64}")
ID = re.compile(r"[A-Za-z0-9][A-Za-z0-9_-]{0,63}")
MAX_PAYLOAD = 2 * 1024 * 1024
SECRET_LIMIT = 48 * 1024
# Explicit successor experiment, not a retry of the v2 one-shot claim.
# Owner approved this one budget change after run 45101280387 exhausted 4096 output tokens.
MAX_CALLS = 12
MAX_OUTPUT = 8192
MAX_TOTAL = 32192
SLOT_TIMEOUT = 1200
POLICY = {"id": "owner-budget-expanded-once-v3", "predecessor_run": 45101280387,
          "max_model_calls": MAX_CALLS, "max_output_tokens": MAX_OUTPUT,
          "max_total_tokens": MAX_TOTAL, "slot_timeout_seconds": SLOT_TIMEOUT}


def need(condition, reason):
    if not condition:
        raise ValueError(reason)

def encode(value):
    return json.dumps(value, sort_keys=True, ensure_ascii=False, separators=(",", ":")).encode()

def sha(raw):
    return hashlib.sha256(raw).hexdigest()

def load(path):
    return json.loads(path.read_bytes())

def save(path, data):
    if not isinstance(data, bytes):
        data = encode(data) + b"\n"
    fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
    with os.fdopen(fd, "wb") as f:
        f.write(data)
        f.flush()
        os.fsync(f.fileno())
    fd = os.open(path.parent, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(fd)
    finally:
        os.close(fd)

def validate(payload):
    need(type(payload) is dict and set(payload) == {"schema", "questions", "sources", "configs"}, "payload_schema")
    need(payload["schema"] == "owner-paired-v3", "schema_version")
    q, s, c = (payload[k] for k in ("questions", "sources", "configs"))
    need(type(q) is list and len(q) == 24 and type(s) is dict and len(s) == 12 and type(c) is dict and set(c) == set(s), "inventory")
    for k, v in s.items():
        need(ID.fullmatch(k) and type(v) is str and HEX.fullmatch(v), "source_pin")
    for x in q:
        need(type(x) is dict and set(x) == {"case_id", "question", "source_id"}, "question_schema")
        need(type(x["case_id"]) is str and ID.fullmatch(x["case_id"]) and type(x["question"]) is str and 20 <= len(x["question"]) <= 4096 and x["source_id"] in s, "question")
    need(len({x["case_id"] for x in q}) == 24, "duplicate_case")
    need(all(sum(x["source_id"] == k for x in q) == 2 for k in s), "source_distribution")
    for k, conf in c.items():
        need(type(conf) is dict and set(conf) == {"schema_version", "resolution", "answer_context"} and conf["schema_version"] == "reason-config-v1", "config_schema")
        need(set(conf["resolution"]) == {"investigation"}, "resolution_schema")
        inv = conf["resolution"]["investigation"]
        need(set(inv) == {"capabilities", "max_actions", "max_no_progress_rounds", "max_rounds", "max_targets", "planner_max_tokens"}, "investigation_schema")
        need([inv[x] for x in ("max_actions", "max_no_progress_rounds", "max_rounds", "max_targets", "planner_max_tokens")] == [4, 2, 3, 3, 512], "investigation_caps")
        ac = conf["answer_context"]
        need(set(ac) == {"schema_version", "sources", "unresolved_inputs"} and ac["schema_version"] == "reason-answer-context-config-v1" and ac["unresolved_inputs"] == [], "answer_context")
        caps, refs = inv["capabilities"], ac["sources"]
        need(type(caps) is list and 1 <= len(caps) <= 32 and type(refs) is list and len(refs) == len(caps), "capability_inventory")
        ids = set()
        for cap, ref in zip(caps, refs):
            need(set(cap) == {"id", "kind", "read_only", "supported_fact_keys", "evidence", "admission"} and cap["kind"] == "captured_evidence" and cap["read_only"] is True and len(cap["evidence"]) == 1, "capture_only")
            ev = cap["evidence"][0]
            need(set(ev) == {"id", "source", "observation", "facts", "acquisition_metadata"}, "evidence_schema")
            ob = json.loads(ev["observation"])
            need(set(ob) == {"context_summary", "document_id", "locator", "origin_group", "quote", "scope", "source_version", "synthetic_mutation", "url"}, "observation_schema")
            need(ob["document_id"] == k and ob["source_version"] == "sha256:" + s[k] and ob["synthetic_mutation"] is False and re.fullmatch(r"https://raw\.githubusercontent\.com/[^/]+/[^/]+/[0-9a-f]{40}/.+", ob["url"]) and type(ob["quote"]) is str and bool(ob["quote"]), "source_binding")
            need(ev["id"] not in ids, "duplicate_evidence")
            ids.add(ev["id"])
            need(len(ev["facts"]) == 1 and list(ev["facts"]) == cap["supported_fact_keys"], "fact_binding")
            key, value = next(iter(ev["facts"].items()))
            need(ref["evidence_id"] == ev["id"] and ref["source_id"] == ev["source"] and ref["proposition"] == {"key": key, "value": value} and ref["url"] == ob["url"] and ref["source_version"] == ob["source_version"] and HEX.fullmatch(ref["evidence_sha256"]), "citation_binding")
    return payload

def cohort(payload):
    # Stable across ordering, config revisions and run identifiers. Do not
    # treat renamed case/source IDs as a new dataset.
    return sha(encode({"questions": sorted((x["question"], payload["sources"][x["source_id"]]) for x in payload["questions"]),
                       "core": ARMS, "cli": CLI, "model": MODEL, "measurement_policy": POLICY}))

def tag(payload):
    return PREFIX + cohort(payload) + "-" + sha(encode(payload))

def parse_secret(secret, expected_tag):
    need(type(secret) is str and 0 < len(secret) <= SECRET_LIMIT, "secret_size")
    with gzip.GzipFile(fileobj=io.BytesIO(base64.b64decode(secret, validate=True))) as f:
        raw = f.read(MAX_PAYLOAD + 1)
    need(len(raw) <= MAX_PAYLOAD, "payload_size")
    p = validate(json.loads(raw))
    need(raw == encode(p) and tag(p) == expected_tag, "tag_pin")
    return p

def pack(prepared, output):
    source = prepared / "runner-input"
    questions = load(source / "questions.json")
    manifest = load(source / "manifest.json")
    configs, sources = {}, {}
    for m in manifest:
        k = m["id"]
        need(ID.fullmatch(k), "source_id")
        raw = (source / "sources" / k).read_bytes()
        need(sha(raw) == m["sha256"], "source_drift")
        lines = raw.decode().splitlines()
        cfg = load(prepared / "source-configs" / (k + ".json"))
        for cap in cfg["resolution"]["investigation"]["capabilities"]:
            ob = json.loads(cap["evidence"][0]["observation"])
            locator = re.fullmatch(re.escape(m["path"]) + r":([0-9]+)-([0-9]+)", ob["locator"])
            need(locator is not None, "locator")
            a, b = map(int, locator.groups())
            need(1 <= a <= b <= len(lines) and "\n".join(lines[a-1:b]) == ob["quote"] and ob["url"] == m["raw_url"], "exact_quote")
        configs[k], sources[k] = cfg, m["sha256"]
    p = validate({"schema":"owner-paired-v3", "questions":questions, "sources":sources, "configs":configs})
    raw = encode(p)
    secret = base64.b64encode(gzip.compress(raw, mtime=0))
    need(len(secret) <= SECRET_LIMIT, "secret_limit")
    save(output, secret)
    print(tag(p)) # Public-safe hashes only.

def stage(root, ref):
    p = parse_secret(os.environ.pop("OWNER_EVAL_INPUT", ""), ref)
    root.mkdir(mode=0o700)
    save(root / "payload.json", p)
    cfg = root / "configs"; cfg.mkdir(mode=0o700)
    for k, v in p["configs"].items(): save(cfg / (k+".json"), v)
    print("frozen private input verified: 24 cases, 12 sources; no Gold provided")
    return p

def build_check(path):
    b = load(path)
    need(b["built"] is True and b["source_identical_except_core_and_lock"] is True and b["cli_commit"] == CLI and b["provider_generation_authorized"] is False, "build_pin")
    for arm, commit in ARMS.items():
        a = b["arms"][arm]
        need(a["engine_commit"] == commit and sha(Path(a["binary"]).read_bytes()) == a["binary_sha256"], "binary_pin")
    return b

def argv(root, b, row, arm, index, plan=None):
    cfg = root / "configs" / (row["source_id"]+".json")
    a = [b["arms"][arm]["binary"], "--plain", "--provider", "mistral", "--model", MODEL,
         "--seed", str(49200+index), "--max-tokens", "1024", "--safety-profile", "current",
         "--max-model-calls", str(MAX_CALLS), "--max-output-tokens", str(MAX_OUTPUT), "--max-total-tokens", str(MAX_TOTAL),
         "--config", str(cfg), "--format", "json", "--evaluation-isolated-config", row["question"]]
    if plan is not None: a += ["--evaluation-request-plan", str(plan)]
    return a

def preflight(root, build_path):
    p = validate(load(root/"payload.json")); b = build_check(build_path)
    for k, v in p["configs"].items():
        need((root/"configs"/(k+".json")).read_bytes() == encode(v)+b"\n", "staged_config_drift")
    out = root/"plans"; out.mkdir(mode=0o700)
    hashes = []
    for i, row in enumerate(p["questions"]):
        pair = []
        for arm in ARMS:
            dest = out / f"{i:02}-{arm}.json"
            cp = subprocess.run(argv(root,b,row,arm,i,dest), cwd=out, stdin=subprocess.DEVNULL,
                                env={"PATH":os.environ["PATH"],"HOME":str(out),"NO_COLOR":"1"},
                                capture_output=True, timeout=35)
            need(cp.returncode == 0 and cp.stdout == b"" and cp.stderr == b"", "request_plan_failure")
            d = load(dest)
            need(d["question"] == row["question"] and d["model"] == MODEL and d["provider"] == "mistral", "request_identity")
            pair.append(d)
        need(pair[0]["request"] == pair[1]["request"] and pair[0]["request_sha256"] == pair[1]["request_sha256"], "pair_mismatch")
        hashes.append(pair[0]["request_sha256"])
    need(len(set(hashes)) == 24, "duplicate_request")
    save(root/"preflight.json", {"tag":tag(p),"hashes":hashes,
        "config_hashes":{k:sha((root/"configs"/(k+".json")).read_bytes()) for k in p["configs"]},"pair_count":24,"measurement_policy":POLICY})
    print("24/24 pairs identical; 48 plans; model calls 0")

def terminal_observation(stdout, rc, obs):
    """Only accounted, exact ceiling exhaustion is a bounded final observation.
    It is NOT a successful answer. Any unexplained failure or exceeded ceiling
    remains fail-closed. No replacement or retry is undertaken."""
    usage, answer_status, status = usage_from_cli(stdout)
    obs["usage"] = usage
    need(usage["model_calls"] <= MAX_CALLS and
         usage["output_tokens"] <= MAX_OUTPUT and
         usage["total_tokens"] <= MAX_TOTAL, "usage_ceiling_violation")
    result = json.loads(stdout)["result"]
    if rc == 1 and status == "failed":
        f = result.get("failure") or {}
        expected = {
            f"output-token budget exhausted ({MAX_OUTPUT}/{MAX_OUTPUT})":
                usage["output_tokens"] == MAX_OUTPUT,
            f"model call budget exhausted ({MAX_CALLS}/{MAX_CALLS})":
                usage["model_calls"] == MAX_CALLS,
            f"total-token budget exhausted ({MAX_TOTAL}/{MAX_TOTAL})":
                usage["total_tokens"] == MAX_TOTAL,
        }
        if (f.get("failure_class") == "usage_budget_exceeded" and
            expected.get(f.get("message"), False) and answer_status is None and
            result.get("finalization") is None):
            obs.update(status="budget_exhausted", answer_status=None,
                       failure_class="usage_budget_exceeded")
            return
    need(rc == 0 and status != "failed", "slot_failure")
    final = result["finalization"]
    need(type(final["text"]) is str and type(final["status"]) is str, "display_missing")
    obs.update(status="complete", final_sha256=sha(final["text"].encode()),
               answer_status=answer_status)


def billing_for_policy(subscription, budget, models):
    # Preserve the actual no-overage/account/headroom/model checks; the old
    # helper's 28096 metadata was specific to its previous protocol.
    attestation = check_account(subscription, budget, models)
    attestation.update(total_token_upper_bound=48*MAX_TOTAL,
                       max_model_generations_after_admission=48*MAX_CALLS,
                       output_token_upper_bound=48*MAX_OUTPUT,
                       measurement_policy=POLICY)
    return attestation


def measure(root, build_path, shard, ref):
    need(os.environ.get("GITHUB_ACTIONS") == "true" and os.environ.get("GITHUB_RUN_ATTEMPT") == "1" and os.environ.get("GITHUB_REF") == "refs/tags/"+ref, "one_shot_context")
    execute_admitted_shard(root, build_path, shard, ref)


def execute_admitted_shard(root, build_path, shard, ref, *, supplement=None):
    # Entry points must verify the original one-shot or the dedicated per-shard
    # supplement claim before invoking this shared execution loop.
    p = validate(load(root/"payload.json")); b = build_check(build_path); pre = load(root/"preflight.json")
    need(tag(p) == pre["tag"] == ref and len(pre["hashes"]) == 24 and pre["pair_count"] == 24 and pre.get("measurement_policy") == POLICY, "preflight_pin")
    need(type(shard) is int and 0 <= shard < 8, "shard")
    key = os.environ.pop("MISTRAL_API_KEY", "")
    need(bool(key), "credential")
    opener = urllib.request.build_opener(NoRedirect())
    billing = billing_for_policy(*(get_json(opener,key,path) for path in ("/v1/billing/subscription","/v1/billing/budget","/v1/models")))
    result = root/"raw"; result.mkdir(mode=0o700)
    save(result/"billing.json", billing)
    save(result/"shard.json", {"tag":ref,"shard":shard,"run_id":os.environ["GITHUB_RUN_ID"],"attempt":1, **({"supplement":supplement} if supplement is not None else {})})
    completed = exhausted = 0
    for i in range(shard*3, shard*3+3):
        row = p["questions"][i]
        arms = tuple(ARMS) if i%2 == 0 else tuple(reversed(ARMS))
        config = root/"configs"/(row["source_id"]+".json")
        need(sha(config.read_bytes()) == pre["config_hashes"][row["source_id"]], "config_drift")
        for arm in arms:
            a = b["arms"][arm]
            need(sha(Path(a["binary"]).read_bytes()) == a["binary_sha256"], "binary_drift")
            slot = result/f"{i:02}-{arm}"; slot.mkdir(mode=0o700)
            reservation = {"case_id":row["case_id"], "arm":arm, "attempt":1,
                           "request_sha256":pre["hashes"][i],"binary_sha256":a["binary_sha256"],"config_sha256":pre["config_hashes"][row["source_id"]],"run_id":os.environ["GITHUB_RUN_ID"],"shard":shard,"measurement_policy":POLICY}
            if supplement is not None:
                reservation["supplement"] = supplement
            save(slot/"reservation.json", reservation)
            home = slot/"home"; home.mkdir(mode=0o700)
            try:
                cp = subprocess.run(argv(root,b,row,arm,i),cwd=home,stdin=subprocess.DEVNULL,capture_output=True,timeout=SLOT_TIMEOUT,
                    env={"HOME":str(home),"TMPDIR":str(home),"PATH":os.environ["PATH"],"MISTRAL_API_KEY":key,"NO_COLOR":"1","NO_PROXY":"127.0.0.1","REASON_MISTRAL_RATE_LIMIT_TELEMETRY":"1"})
                stdout, stderr, rc = cp.stdout, cp.stderr, cp.returncode
            except subprocess.TimeoutExpired as e:
                stdout, stderr, rc = e.stdout or b"", e.stderr or b"", None
            save(slot/"stdout.json",stdout); save(slot/"stderr.log",stderr)
            obs = {**reservation,"status":"incomplete","exit_code":rc,"stdout_sha256":sha(stdout),"stderr_sha256":sha(stderr),"usage":None}
            try:
                terminal_observation(stdout, rc, obs)
            finally:
                save(slot/"observation.json",obs)
            need(obs["status"] in ("complete", "budget_exhausted"), "incomplete_no_retry")
            completed += obs["status"] == "complete"
            exhausted += obs["status"] == "budget_exhausted"
    need(completed + exhausted == 6, "closure_count")
    save(result/"closure.json",{"status":"complete" if exhausted == 0 else "complete_with_budget_exhaustions",
                                "slots":6,"complete":completed,"budget_exhausted":exhausted,
                                "shard":shard,"tag":ref,"measurement_policy":POLICY})
    print(f"6/6 terminal first observations; {completed} complete answers; {exhausted} budget-exhausted (not scored)")

def main():
    os.umask(0o077)
    a=argparse.ArgumentParser()
    sub=a.add_subparsers(dest="cmd",required=True)
    t=sub.add_parser("pack"); t.add_argument("prepared",type=Path); t.add_argument("output",type=Path)
    t=sub.add_parser("stage"); t.add_argument("root",type=Path); t.add_argument("ref")
    t=sub.add_parser("preflight"); t.add_argument("root",type=Path); t.add_argument("build",type=Path)
    t=sub.add_parser("measure"); t.add_argument("root",type=Path); t.add_argument("build",type=Path); t.add_argument("shard",type=int); t.add_argument("ref")
    x=a.parse_args()
    if x.cmd=="pack":pack(x.prepared,x.output)
    elif x.cmd=="stage":stage(x.root,x.ref)
    elif x.cmd=="preflight":preflight(x.root,x.build)
    else:measure(x.root,x.build,x.shard,x.ref)
if __name__=="__main__":
    try:main()
    except BaseException as exc:
        # Never emit data-bearing exceptions, subprocess text or secret values.
        print("HOLD ("+type(exc).__name__+")",file=sys.stderr)
        sys.exit(2)
