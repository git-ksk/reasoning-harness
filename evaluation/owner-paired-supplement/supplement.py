"""Exact unstarted slots of 45198846318 only. No rerun of the original claim."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys

sys.path.insert(0,str(Path(__file__).resolve().parents[1]/"owner-paired-v3"))
import measure as m

RUN=45198846318
HEAD="8dd305bc443c19e493332b4daae985b290070c72"
ORIGINAL="eval-owner-paired-v3-f98e8263349706f5855935e4496409197f0df017c782105248192eaa134b959e-6a7d305f035c57e61fb0283f63f3b06015f5f4a9d49bd0ea47c122521dbac7d5"
COHORT=ORIGINAL.removeprefix(m.PREFIX).split("-")[0]
TAG="eval-owner-supplement-45198846318-v1"
REPO="git-ksk/reasoning-harness"
MISSING={0:137781506968,4:137781506789}
COMPLETE={1,2,3,5,6,7}
ORIGINAL_COMPLETED_PAIR_PINS="a9fa633476dadd53d182149c33f86c95e472a7ab4a893499d8c1ef5a5a995616"
ARTIFACTS={f"owner-paired-v3-{RUN}-shard-{s}" for s in COMPLETE}
need=m.need

def api(path,*args):
    cp=subprocess.run(["gh","api",f"repos/{REPO}/{path}",*args],
                      stdin=subprocess.DEVNULL,capture_output=True,timeout=30)
    need(cp.returncode==0,"github_api_refused")
    return json.loads(cp.stdout)

def context():
    need(os.getenv("GITHUB_ACTIONS")=="true" and os.getenv("GITHUB_RUN_ATTEMPT")=="1"
         and os.getenv("GITHUB_EVENT_NAME")=="push" and os.getenv("GITHUB_REPOSITORY")==REPO
         and os.getenv("GITHUB_REF")=="refs/tags/"+TAG,"supplement_context")
    need(os.getenv("GITHUB_SHA","") and os.getenv("GITHUB_RUN_ID","").isdigit(),"supplement_identity")

def validate_original(r,jobs,artifacts):
    need(r["id"]==RUN and r["head_sha"]==HEAD and r["head_branch"]==ORIGINAL
         and r["event"]=="push" and r["run_attempt"]==1 and r["status"]=="completed"
         and r["conclusion"]=="failure","original_run_pin")
    by={j["name"]:j for j in jobs}
    need(len(jobs)==len(by)==11 and set(by)=={"offline-controls","preflight","claim",*(f"live ({s})" for s in range(8))},"job_inventory")
    for name in ("offline-controls","preflight","claim"):
        need(by[name]["conclusion"]=="success","original_precondition")
    for s in COMPLETE:
        j=by[f"live ({s})"]
        need(j["status"]=="completed" and j["conclusion"]=="success" and j["runner_id"]>0
             and j["runner_name"] and j["steps"],"original_success")
    for s,jid in MISSING.items():
        j=by[f"live ({s})"]
        need(j["id"]==jid and j["status"]=="completed" and j["conclusion"]=="cancelled"
             and j["runner_id"]==0 and j["runner_name"]=="" and j["steps"]==[],"not_proven_unstarted")
    need(len(artifacts)==6 and {x["name"] for x in artifacts}==ARTIFACTS
         and all(x["expired"] is False for x in artifacts),"original_artifacts")

def verify_original():
    validate_original(api(f"actions/runs/{RUN}"),
        api(f"actions/runs/{RUN}/jobs?per_page=100")["jobs"],
        api(f"actions/runs/{RUN}/artifacts?per_page=100")["artifacts"])
    ref=api(f"git/ref/tags/eval-owner-claimed-v1-{COHORT}")
    need(ref["object"]["sha"]==HEAD,"original_claim_pin")
    print("Original terminal run: exactly two unstarted shards, six completed artifacts; claim retained")

def claim_name(shard):
    need(type(shard) is int and shard in MISSING,"shard_not_missing")
    return f"eval-owner-supplement-claimed-{COHORT}-shard-{shard}"

def lineage(shard):
    return {"original_run":RUN,"original_job":MISSING[shard],"original_head":HEAD,
            "supplement_tag":TAG,"claim":claim_name(shard),"supplement_head":os.environ["GITHUB_SHA"],
            "supplement_run":os.environ["GITHUB_RUN_ID"]}

def verify_preflight_pins(p,pre):
    need(m.tag(p)==pre["tag"]==ORIGINAL and pre.get("measurement_policy")==m.POLICY
         and len(pre["hashes"])==24 and pre["pair_count"]==24,"input_pin")
    rows=[]
    for s in sorted(COMPLETE):
        for i in range(3*s,3*s+3):
            sid=p["questions"][i]["source_id"]
            rows.append([i,sid,pre["hashes"][i],pre["config_hashes"][sid]])
    need(m.sha(m.encode(rows))==ORIGINAL_COMPLETED_PAIR_PINS,"original_completed_request_config_drift")


def claim(root,build,shard):
    context()
    name=claim_name(shard)
    verify_original()
    p=m.validate(m.load(root/"payload.json"));pre=m.load(root/"preflight.json")
    verify_preflight_pins(p,pre)
    m.build_check(build)
    # Atomic create-only tag: no retry once a claim exists, including if execution fails.
    ref=api("git/refs","--method","POST","-f",f"ref=refs/tags/{name}","-f",f"sha={os.environ['GITHUB_SHA']}")
    need(ref["ref"]=="refs/tags/"+name and ref["object"]["sha"]==os.environ["GITHUB_SHA"],"claim_identity")
    m.save(root/"supplement-claim.json",lineage(shard))
    print(f"Claimed only proven-unstarted shard {shard}")

def execute(root,build,shard):
    context()
    name=claim_name(shard)
    receipt=m.load(root/"supplement-claim.json")
    need(receipt==lineage(shard),"claim_receipt")
    need("GH_TOKEN" not in os.environ,"separate_write_and_model_credentials")
    verify_preflight_pins(m.validate(m.load(root/"payload.json")),m.load(root/"preflight.json"))
    # Claim receipt was exclusively created by the preceding successful API step,
    # in this job's private root. A retry cannot pass context or create the tag.
    m.execute_admitted_shard(root,build,shard,ORIGINAL,supplement=receipt)

def main():
    os.umask(0o077)
    p=argparse.ArgumentParser();p.add_argument("action",choices=("verify","claim","execute"))
    p.add_argument("root",type=Path,nargs="?");p.add_argument("build",type=Path,nargs="?");p.add_argument("shard",type=int,nargs="?")
    a=p.parse_args()
    if a.action=="verify":context();verify_original()
    elif a.action=="claim":claim(a.root,a.build,a.shard)
    else:execute(a.root,a.build,a.shard)

if __name__=="__main__":
    try:main()
    except Exception as e:
        print("HOLD ("+type(e).__name__+")",file=sys.stderr);sys.exit(2)
