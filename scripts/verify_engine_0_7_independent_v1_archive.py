#!/usr/bin/env python3
"""Check immutable first provider artifacts using the PRE-FROZEN scorecard.

This post-observation integrity wrapper does not change or override any of the
pre-frozen evaluator, provider semantics, oracle labels or acceptance thresholds.
"""
import hashlib
import json
import runpy
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]
ARCHIVE=ROOT/"evaluation/engine-0.7-independent-v1-first-observation"
CORPUS=ROOT/"fixtures/engine-0.7-independent-holdout-v1/manifest.json"
FROZEN_SCORE=ROOT/"scripts/engine_0_7_independent_v1_score.py"

def verify():
    manifest=json.loads((ARCHIVE/"archive_manifest.json").read_text())
    assert manifest["schema"]=="engine-0.7-independent-v1-first-observation-archive"
    assert manifest["source_run_id"]==38013006323
    assert manifest["runner_freeze_commit"]=="1124e5a7c34e42c088d81908c3cdc75aae2db309"
    assert manifest["corpus_freeze_commit"]=="78f99559bf044013e66025bde1a612895f2d41e6"
    assert manifest["candidate_commit"]=="4cae9326bcc3e210c3250f05772d84f00e41b645"
    assert manifest["corpus_sha256"]=="d3ce38ee2ef06310afb6a247d3762fc408b93cb49bd602a52d8ce1b8bcf7fa0b"
    assert hashlib.sha256(CORPUS.read_bytes()).hexdigest()==manifest["corpus_sha256"]
    assert manifest["first_run_result"]=="PASS_INDEPENDENT_SYNTHETIC_PROVIDER_GATE"
    items=manifest["artifacts"]
    assert len(items)==13
    assert len({item["name"] for item in items})==13
    expected={
        "engine-070-independent-v1-cross-provider-summary.json",
        *(f"engine-070-independent-{provider}-{suffix}"
          for provider in ("mistral","google","groq")
          for suffix in ("checkpoint.jsonl","result.json","summary.json","validate.json"))
    }
    assert {item["name"] for item in items}==expected
    for item in items:
        path=ARCHIVE/item["name"]
        assert path.is_file() and not path.is_symlink()
        assert path.stat().st_size==item["size_bytes"]
        assert hashlib.sha256(path.read_bytes()).hexdigest()==item["sha256"],path.name

    # Re-score using the original evaluator that was tagged before *any*
    # corpus was created, without editing/replacing the first results.
    score=runpy.run_path(str(FROZEN_SCORE))["summarize"](CORPUS,ARCHIVE)
    assert score==json.loads((ARCHIVE/"engine-070-independent-v1-cross-provider-summary.json").read_text())
    assert score["release_gate_passed"] is True
    assert {name:v["net_useful_answer_gain"] for name,v in score["providers"].items()}=={
        "mistral":3,"google":6,"groq":6
    }
    for v in score["providers"].values():
        assert v["operational_failures"]==0
        assert v["hard_gate_violations"]==0
        assert v["completed_cases"]==12
        assert v["targets"]==15
    return {
        "archive_verified":True,
        "run_id":manifest["source_run_id"],
        "frozen_score_recomputed":True,
        "files_with_exact_sha256":len(items),
        "providers":{name:score["providers"][name]["net_useful_answer_gain"]
                     for name in sorted(score["providers"])},
        "independent_synthetic_gate_pass":True
    }

if __name__=="__main__":
    print(json.dumps(verify(),sort_keys=True,indent=2))
