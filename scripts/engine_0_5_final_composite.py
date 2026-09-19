#!/usr/bin/env python3
import argparse
import json
from pathlib import Path

FINAL_MANIFEST = Path("fixtures/engine-0.5-final-v1-finalization/manifest.json")
REPORT_SCHEMA = "reason-engine-0.5-final-v1-composite"


def load_optional(path):
    p = Path(path)
    if not p.is_file():
        return None
    try:
        value = json.loads(p.read_text())
    except json.JSONDecodeError:
        return None
    return value if isinstance(value, dict) else None


def classify(role, finalization, finalization_rc, materialization, materialization_rc):
    final_ok = (
        finalization_rc == 0
        and isinstance(finalization, dict)
        and finalization.get("acceptance_passed") is True
    )
    material_ok = (
        materialization_rc == 0
        and isinstance(materialization, dict)
        and materialization.get("acceptance_passed") is True
    )
    if final_ok and material_ok:
        return "PASS"
    if finalization is None or materialization is None:
        return "INCOMPLETE"
    return "FAIL"


def build(target, finalization_path, finalization_rc, materialization_path, materialization_rc):
    manifest = json.loads(FINAL_MANIFEST.read_text())
    targets = manifest["provider_targets"]
    if target not in targets:
        raise SystemExit(f"unknown target: {target}")
    policy = targets[target]
    finalization = load_optional(finalization_path)
    materialization = load_optional(materialization_path)
    classification = classify(
        policy["role"],
        finalization,
        finalization_rc,
        materialization,
        materialization_rc,
    )
    release_required = policy["role"] == "validated_required"
    release_gate_passed = (classification == "PASS") if release_required else None
    return {
        "schema_version": REPORT_SCHEMA,
        "target": target,
        "provider": policy["provider"],
        "requested_model": policy["model"],
        "role": policy["role"],
        "release_required": release_required,
        "classification": classification,
        "release_gate_passed": release_gate_passed,
        "components": {
            "finalization": {
                "exit_code": finalization_rc,
                "report_present": finalization is not None,
                "acceptance_passed": (
                    finalization.get("acceptance_passed")
                    if isinstance(finalization, dict)
                    else None
                ),
                "aggregate": (
                    finalization.get("aggregate")
                    if isinstance(finalization, dict)
                    else None
                ),
            },
            "materialization": {
                "exit_code": materialization_rc,
                "report_present": materialization is not None,
                "acceptance_passed": (
                    materialization.get("acceptance_passed")
                    if isinstance(materialization, dict)
                    else None
                ),
                "checks": (
                    materialization.get("checks")
                    if isinstance(materialization, dict)
                    else None
                ),
                "summary": (
                    materialization.get("summary")
                    if isinstance(materialization, dict)
                    else None
                ),
                "observed_model_identities": (
                    materialization.get("observed_model_identities")
                    if isinstance(materialization, dict)
                    else None
                ),
            },
        },
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--target", required=True)
    parser.add_argument("--finalization", required=True)
    parser.add_argument("--finalization-rc", required=True, type=int)
    parser.add_argument("--materialization", required=True)
    parser.add_argument("--materialization-rc", required=True, type=int)
    parser.add_argument("--output", required=True)
    args = parser.parse_args()
    report = build(
        args.target,
        args.finalization,
        args.finalization_rc,
        args.materialization,
        args.materialization_rc,
    )
    encoded = json.dumps(report, indent=2, sort_keys=True) + "\n"
    Path(args.output).write_text(encoded)
    print(encoded, end="")


if __name__ == "__main__":
    main()
