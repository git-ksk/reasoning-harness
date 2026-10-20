#!/usr/bin/env python3
"""Read-only diagnostics over preserved grades; never executes an Engine or scorer."""

import argparse
from collections import Counter, defaultdict
import hashlib
import json
from pathlib import Path, PurePosixPath
import re


ARCHIVE = "engine073-launch-independent-v19"
FINAL_ANCHOR = "4f8fd8921fdf754de2088e3afe9ab23a86dbfbc25e89c3e3335c4ffcd02c930a"


class ArchiveError(ValueError):
    pass


def digest(path):
    if path.is_symlink():
        raise ArchiveError(f"symlink in frozen archive: {path.name}")
    return hashlib.sha256(path.read_bytes()).hexdigest()


def verify_archive(repo):
    evaluation = repo / "evaluation"
    archive = evaluation / ARCHIVE
    manifest_path = archive / "final-artifact-manifest.json"
    if digest(manifest_path) != FINAL_ANCHOR:
        raise ArchiveError("final manifest does not match the pinned v19 anchor")
    manifest = json.loads(manifest_path.read_text())
    expected = {f"{ARCHIVE}/final-artifact-manifest.json"}
    actual = set()
    for version, files in manifest["versions"].items():
        if not re.fullmatch(r"v[1-9][0-9]*", version):
            raise ArchiveError("invalid archive version")
        folder = evaluation / f"engine073-launch-independent-{version}"
        for name, sha in files.items():
            relative = PurePosixPath(name)
            if relative.is_absolute() or ".." in relative.parts:
                raise ArchiveError("unsafe frozen path")
            path = folder / name
            if digest(path) != sha:
                raise ArchiveError(f"frozen artifact changed: {version}/{name}")
            expected.add(path.relative_to(evaluation).as_posix())
        actual.update(
            path.relative_to(evaluation).as_posix()
            for path in folder.rglob("*")
            if path.is_file() or path.is_symlink()
        )
    if actual != expected:
        raise ArchiveError("frozen inventory differs from the pinned manifest")
    return archive, len(expected)


def analyze(repo):
    archive, verified_files = verify_archive(repo)
    summary = json.loads((archive / "first-summary.json").read_text())
    result = json.loads((archive / "first-results.json").read_text())
    if result["summary"] != summary:
        raise ArchiveError("saved first summaries disagree")
    rows = result["cases"]
    if len(rows) != summary["observed_cases"]:
        raise ArchiveError("saved observed-case count disagrees")
    if len({row["id"] for row in rows}) != len(rows):
        raise ArchiveError("duplicate case identity")
    pairs = defaultdict(list)
    for row in rows:
        if row["status"] not in ("pass", "fail"):
            raise ArchiveError("incomplete/unknown grade needs a separate diagnostic identity")
        pairs[row["pair_id"]].append(row)
    passed = sum(row["status"] == "pass" for row in rows)
    if passed != summary["passed"] or len(rows) - passed != summary["failed"]:
        raise ArchiveError("saved first grade totals disagree")

    pair_rows = []
    for pair_id, members in sorted(pairs.items()):
        if len(members) != 2 or {row["polarity"] for row in members} != {"positive", "negative"}:
            raise ArchiveError("contrast pair must contain one saved positive and negative")
        by_polarity = {row["polarity"]: row for row in members}
        if len({row["axis"] for row in members}) != 1:
            raise ArchiveError("contrast pair axes disagree")
        positive = by_polarity["positive"]
        negative = by_polarity["negative"]
        pattern = (
            "both_checks_pass" if positive["status"] == negative["status"] == "pass"
            else "positive_only_fail" if negative["status"] == "pass"
            else "negative_only_fail" if positive["status"] == "pass"
            else "both_fail"
        )
        pair_rows.append({
            "pair_id": pair_id,
            "axis": positive["axis"],
            "pattern_from_saved_grades": pattern,
            "positive_id": positive["id"],
            "negative_id": negative["id"],
            "positive_saved_disposition": positive["v30_disposition"],
            "negative_saved_disposition": negative["v30_disposition"],
        })

    failures = [row for row in rows if row["status"] == "fail"]
    return {
        "identity": "engine073-launch-research-diagnostics-v1",
        "kind": "post-observation archive analysis; not a score or acceptance run",
        "candidate_source_sha": summary["candidate_source_sha"],
        "original_first_outcome": summary["outcome"],
        "first_summary_sha256": digest(archive / "first-summary.json"),
        "first_results_sha256": digest(archive / "first-results.json"),
        "final_manifest_sha256": FINAL_ANCHOR,
        "verified_archive_files": verified_files,
        "saved_counts": {key: summary[key] for key in (
            "planned_cases", "observed_cases", "passed", "failed", "incomplete",
            "positive_passed", "negative_passed", "positive_v17_withheld",
            "positive_v30_withheld", "negative_v17_requested_relation_violations",
            "negative_v30_relevant_violations",
        )},
        "pair_patterns_from_saved_grades": dict(sorted(Counter(
            pair["pattern_from_saved_grades"] for pair in pair_rows
        ).items())),
        "failed_positive_ids": [row["id"] for row in failures if row["polarity"] == "positive"],
        "failure_reason_codes": dict(sorted(Counter(
            reason for row in failures for reason in row["v30_reasons"]
        ).items())),
        "contrast_pairs": pair_rows,
        "trace_limits": {
            "available": ["saved v17 relation scope", "saved v30 disposition/reason codes"],
            "deterministic_rule_path_available": False,
            "source_span_and_reference_edges_available": False,
            "implementation_root_causes_established": False,
        },
        "new_execution": {"engine": 0, "scorer": 0, "provider": 0},
        "acceptance_effect": "none; original FAIL and #531 real24/CLI gates remain unchanged",
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, default=Path(__file__).resolve().parents[2])
    args = parser.parse_args()
    try:
        report = analyze(args.repo)
    except (ArchiveError, OSError, KeyError, TypeError, json.JSONDecodeError) as error:
        parser.exit(2, f"archive analysis refused: {error}\n")
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
