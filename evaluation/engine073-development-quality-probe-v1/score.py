#!/usr/bin/env python3
"""Read-only, self-authored Core source-relevance development probe scorer; never adoption."""
from __future__ import annotations
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path

AXES = ("v17_requested_relation", "v30_relevant")
ARMS = ("baseline", "candidate")
ROOT = Path(__file__).resolve().parent

def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()

def read_json(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))

def read_jsonl(path):
    return [json.loads(l) for l in Path(path).read_text(encoding="utf-8").splitlines() if l.strip()]

def checked_cases(rows):
    if not isinstance(rows, list) or len(rows) != 40:
        raise ValueError("expected_exactly_40_predefined_cases")
    ids = {}
    for row in rows:
        if not isinstance(row, dict) or set(row) != {"id", "entity", "signals", "expected_relevant"}:
            raise ValueError("invalid_case_shape")
        id_ = row["id"]
        if not isinstance(id_, str) or not id_ or id_ in ids:
            raise ValueError("invalid_or_duplicate_case")
        if row["entity"] != "Quartz Lantern" or type(row["expected_relevant"]) is not bool:
            raise ValueError("invalid_expected_label_or_entity")
        if not isinstance(row["signals"], list) or not row["signals"]:
            raise ValueError("missing_source_signals")
        if any(not isinstance(s, dict) or set(s) != {"kind", "text"} or
               s["kind"] not in ("heading", "excerpt") or
               not isinstance(s["text"], str) or not s["text"] for s in row["signals"]):
            raise ValueError("invalid_source_signal")
        ids[id_] = row
    counts = Counter(row["expected_relevant"] for row in rows)
    if counts[True] != 20 or counts[False] != 20:
        raise ValueError("positive_negative_20_each_required")
    return ids

def checked_outputs(rows, cases):
    if not isinstance(rows, list) or len(rows) != len(cases):
        raise ValueError("missing_or_extra_observations")
    observed = {}
    for row in rows:
        if not isinstance(row, dict) or set(row) != {"id", *AXES}:
            raise ValueError("invalid_output_shape")
        id_ = row["id"]
        if not isinstance(id_, str) or id_ not in cases or id_ in observed:
            raise ValueError("unexpected_or_duplicate_observation")
        if any(type(row[key]) is not bool for key in AXES):
            raise ValueError("non_boolean_observation")
        observed[id_] = row
    return observed

def summarize(cases_rows, arms):
    cases = checked_cases(cases_rows)
    if not isinstance(arms, dict) or set(arms) != set(ARMS):
        raise ValueError("two_arms_required")
    outcomes = {arm: checked_outputs(arms[arm], cases) for arm in ARMS}
    results = {}
    for arm in ARMS:
        result = {}
        for axis in AXES:
            tp = fp = tn = fn = 0
            wrong = []
            for id_, case in cases.items():
                expected, actual = case["expected_relevant"], outcomes[arm][id_][axis]
                tp += int(expected and actual)
                fp += int(not expected and actual)
                tn += int(not expected and not actual)
                fn += int(expected and not actual)
                if expected != actual:
                    wrong.append(id_)
            precision = tp / (tp + fp) if tp + fp else None
            recall = tp / (tp + fn) if tp + fn else None
            f1 = (2 * tp / (2 * tp + fp + fn)) if 2 * tp + fp + fn else None
            result[axis] = {
                "tp":tp,"fp":fp,"tn":tn,"fn":fn,
                "precision_diagnostic":precision,"recall_diagnostic":recall,
                "f1_diagnostic":f1,"mismatched_case_ids":wrong
            }
        results[arm] = result
    changed = [
        cid for cid in cases
        if any(outcomes["baseline"][cid][axis] != outcomes["candidate"][cid][axis] for axis in AXES)
    ]
    return {
        "contract":"reason-engine073-self-authored-synthetic-core-probe-v1",
        "classification":"DEVELOPMENT_ONLY_NO_MODEL_NO_INDEPENDENT_HOLDOUT",
        "candidate_target_version":"0.7.3-unpublished",
        "source_core_commits":{
            "baseline":"4abee90f0501f0d8fc09b453a0b45c942fbed69e",
            "candidate":"ab3c6322921c8c738ec35bd018345917b5afa538",
            "common_cli":"988b69f086ac4a209fe93eb10cf1fa6786977567"
        },
        "case_count":len(cases),"positive_cases":20,"negative_cases":20,
        "model_api_calls":0,"independent_24_case_final_answer_cohort":False,
        "formal_f1":None,"quality_go":False,"adoption_go":False,
        "arms":results,"cross_arm_changed_case_ids":changed,
        "observation_note":"Model-free v17/v30 source relevance only; not actual final-answer correctness, refusal, or utility."
    }

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cases",type=Path,default=ROOT/"cases.json")
    parser.add_argument("--baseline",type=Path,default=ROOT/"baseline-output.jsonl")
    parser.add_argument("--candidate",type=Path,default=ROOT/"candidate-output.jsonl")
    parser.add_argument("--report",type=Path,help="Optional new report path. No overwrite")
    args = parser.parse_args()
    report = summarize(read_json(args.cases),{
        "baseline":read_jsonl(args.baseline),"candidate":read_jsonl(args.candidate)
    })
    report["file_sha256"] = {
        "cases":digest(args.cases),"baseline":digest(args.baseline),
        "candidate":digest(args.candidate)
    }
    output = json.dumps(report,ensure_ascii=False,indent=2) + "\n"
    if args.report:
        args.report.open("x",encoding="utf-8").write(output)
    else:
        print(output,end="")

if __name__=="__main__":
    main()
