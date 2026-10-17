#!/usr/bin/env python3
"""Derive binary metrics from frozen first observations; never run Engine/scorer."""
import argparse
import hashlib
import json
import math
from pathlib import Path

AXES = ('v17_requested_relation', 'v30_relevant')

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def divide(a, b):
    return a / b if b else None

def binary_metrics(pairs):
    counts = dict(tp=0, fp=0, tn=0, fn=0)
    for expected, observed in pairs:
        if type(expected) is not bool or type(observed) is not bool:
            raise ValueError('Missing/non-boolean observation cannot become a negative')
        counts['tp' if expected and observed else 'fn' if expected else 'fp' if observed else 'tn'] += 1
    tp, fp, tn, fn = (counts[k] for k in ('tp', 'fp', 'tn', 'fn'))
    n = tp + fp + tn + fn
    recall, specificity = divide(tp, tp+fn), divide(tn, tn+fp)
    return {
        'confusion': counts, 'n': n,
        'positive_support': tp+fn, 'negative_support': tn+fp,
        'precision': divide(tp, tp+fp), 'recall': recall,
        'f1_positive': divide(2*tp, 2*tp+fp+fn),
        'specificity': specificity, 'accuracy': divide(tp+tn, n),
        'balanced_accuracy': (recall+specificity)/2 if recall is not None and specificity is not None else None,
        'predicted_positive_rate': divide(tp+fp, n),
        'false_accept_rate': divide(fp, fp+tn),
        'false_withhold_rate': divide(fn, tp+fn),
        'mcc': divide(tp*tn-fp*fn, math.sqrt((tp+fp)*(tp+fn)*(tn+fp)*(tn+fn)))
    }

def verified_versions(repo, version, anchor, archive_manifest=None):
    current = repo/'evaluation'/f'engine073-launch-independent-{version}'
    final = archive_manifest or current/'final-artifact-manifest.json'
    if sha(final) != anchor:
        raise ValueError('Final manifest anchor mismatch')
    manifest = json.loads(final.read_text())
    total = 0
    for v, files in manifest['versions'].items():
        directory = repo/'evaluation'/f'engine073-launch-independent-{v}'
        actual = {str(p.relative_to(directory)) for p in directory.rglob('*') if p.is_file()}
        expected = set(files) | ({'final-artifact-manifest.json'} if v == version and final.parent == current else set())
        if actual != expected:
            raise ValueError(f'Inventory mismatch: {v}')
        for name, digest in files.items():
            relative = Path(name)
            if relative.is_absolute() or '..' in relative.parts or sha(directory/relative) != digest:
                raise ValueError(f'Artifact mismatch: {v}/{name}')
        total += len(files)
    return manifest['versions'], total+(1 if final.parent == current else 0)

def corpus_metrics(directory):
    results = json.loads((directory/'first-results.json').read_text())
    summary = json.loads((directory/'first-summary.json').read_text())
    if results['summary'] != summary:
        raise ValueError('First-summary and embedded summary mismatch')
    if (summary.get('incomplete', 0) or summary.get('operational_errors') or
        summary['observed_cases'] != summary['planned_cases']):
        return {'status': 'NOT_MEASURED', 'reason': 'incomplete/operational first run', 'first_summary': summary}
    cases = json.loads((directory/'cases.json').read_text())['cases']
    rows = results['cases']
    if len(cases) != summary['planned_cases'] or len(rows) != len(cases):
        raise ValueError('Missing first observation')
    expected_by_id = {c['id']: c for c in cases}
    observed_by_id = {c['id']: c for c in rows}
    if len(expected_by_id) != len(cases) or len(observed_by_id) != len(rows) or expected_by_id.keys() != observed_by_id.keys():
        raise ValueError('Duplicate/mismatched case identity')
    axis_pairs = {axis: [] for axis in AXES}
    per_axis = {}
    for case_id, c in expected_by_id.items():
        row = observed_by_id[case_id]
        if row['expected'] != c['expected'] or row['polarity'] != c['polarity']:
            raise ValueError('Frozen label/polarity mismatch')
        for axis in AXES:
            e, o = c['expected'][axis], row['observed'][axis]
            if row['checks'][axis] != (e == o):
                raise ValueError('Stored first check mismatch')
            axis_pairs[axis].append((e, o))
    for axis, pairs in axis_pairs.items():
        per_axis[axis] = binary_metrics(pairs)
    joint_pass = sum(all(row['checks'][axis] for axis in AXES) for row in rows)
    if joint_pass != summary['passed'] or len(rows)-joint_pass != summary['failed']:
        raise ValueError('Stored first totals mismatch')
    return {
        'status': 'MEASURED', 'source_sha': summary['candidate_source_sha'],
        'joint_first_passed': joint_pass, 'joint_first_failed': len(rows)-joint_pass,
        'first_summary_sha256': sha(directory/'first-summary.json'),
        'first_results_sha256': sha(directory/'first-results.json'),
        'cases_sha256': sha(directory/'cases.json'), 'axes': per_axis
    }

def build_report(repo, version, anchor, archive_manifest=None):
    versions, count = verified_versions(repo, version, anchor, archive_manifest)
    reports = {}
    for v in versions:
        directory = repo/'evaluation'/f'engine073-launch-independent-{v}'
        if v in ('v1', 'v11', 'v17'):
            reports[v] = {'status': 'NOT_MEASURED', 'reason': 'operational/no semantic first observation; not a successful negative'}
        else:
            reports[v] = corpus_metrics(directory)
    return {
        'identity': 'engine073-launch-first-metrics-v1', 'current_version': version,
        'final_archive_anchor': anchor, 'verified_archive_files': count,
        'archive_manifest_path': str(archive_manifest) if archive_manifest else f'evaluation/engine073-launch-independent-{version}/final-artifact-manifest.json',
        'current_independent_final_review_complete': archive_manifest is None,
        'positive_definition': 'frozen label requires requested_relation for v17 / Relevant for v30',
        'undefined_rule': 'zero denominator => null; never substitute precision=1 for no positive predictions',
        'interpretation': 'Per-corpus synthetic API metrics only. Different corpus/source pairs are not a measured improvement trend. Predicted-positive rate is API coverage, not ordinary CLI answer rate. Seen replays are development regressions, not independent generalization. Real24=0/24; final-answer utility/precision/recall/F1 unmeasured.',
        'mutation_engine_scorer_provider_calls': 0, 'versions': reports
    }

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('repo', type=Path)
    parser.add_argument('version')
    parser.add_argument('anchor')
    parser.add_argument('--archive-manifest', type=Path, help='Explicit parent closure manifest when independent final save was interrupted; does not complete independent review')
    args = parser.parse_args()
    print(json.dumps(build_report(args.repo, args.version, args.anchor, args.archive_manifest), indent=2))

if __name__ == '__main__':
    main()
