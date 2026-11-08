#!/usr/bin/env python3
"""One-shot V4 repository-history gate, executed ONCE per full 24-shard run.

Proof carries no credentials, source text, model output or private reviewer
data. A same-run downstream shard must byte-verify this immutable artifact.
"""
import argparse
import json
import os
from pathlib import Path
import re
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import tag_admission

from shard_contract import canonical,sha

TAG="cli-normal-real24-development-v4"
WORKFLOW=".github/workflows/cli-normal-real24-development-v4.yml"
CONTRACT="reason-cli-real24-v4-one-shot-global-tag-proof-v1"


def make_proof(environment,history):
    source=environment.get("GITHUB_SHA","")
    rid=environment.get("GITHUB_RUN_ID","")
    if (environment.get("GITHUB_REPOSITORY")!="git-ksk/reasoning-harness"
            or environment.get("GITHUB_EVENT_NAME")!="push"
            or environment.get("GITHUB_REF")!="refs/tags/"+TAG
            or environment.get("GITHUB_RUN_ATTEMPT")!="1"
            or not re.fullmatch("[0-9a-f]{40}",source)
            or not rid.isdigit() or int(rid)<=0):
        raise ValueError("v4_global_tag_gate_wrong_actual_push")
    verified=tag_admission.single_first_push(history,TAG,source,rid,
                                              expected_tag=TAG,workflow_path=WORKFLOW)
    return {"contract":CONTRACT,"repository":environment["GITHUB_REPOSITORY"],
            "tag":TAG,"workflow":WORKFLOW,"run_id":int(rid),"source_commit":source,
            "admission":verified}


def verify_proof(value,environment):
    if not isinstance(value,dict) or set(value)!={"contract","repository","tag","workflow",
                                                 "run_id","source_commit","admission"}:
        raise ValueError("v4_invalid_tag_proof")
    for name,expected in (("contract",CONTRACT),("repository","git-ksk/reasoning-harness"),
                          ("tag",TAG),("workflow",WORKFLOW),
                          ("run_id",int(environment["GITHUB_RUN_ID"])),
                          ("source_commit",environment["GITHUB_SHA"])):
        if value.get(name)!=expected:raise ValueError("v4_tag_proof_identity_changed")
    admitted=value["admission"]
    if (not isinstance(admitted,dict) or admitted.get("run_id")!=value["run_id"]
            or not isinstance(admitted.get("tag_runs"),list)
            or not any(r.get("id")==value["run_id"] and
                       r.get("head_sha")==value["source_commit"] and
                       r.get("path")==WORKFLOW for r in admitted["tag_runs"])):
        raise ValueError("v4_tag_proof_missing_original_first_run")
    return value


def main():
    p=argparse.ArgumentParser()
    p.add_argument("--output",type=Path,required=True)
    args=p.parse_args()
    proof=make_proof(os.environ,tag_admission.collect_history("git-ksk/reasoning-harness"))
    verify_proof(proof,os.environ)
    args.output.parent.mkdir(parents=True,exist_ok=True)
    fd=os.open(args.output,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
    with os.fdopen(fd,"wb") as stream:
        stream.write(canonical(proof));stream.flush();os.fsync(stream.fileno())
    print(sha(canonical(proof)))


if __name__=="__main__":
    main()
