# Engine 0.7.1 rereview — 2026-10-13

This is **root-authored, offline synthetic development/audit evidence**, not a fresh independent holdout, live-provider study, real-user error rate or device acceptance. It records the original failing observation before any corrective implementation.

Reviewed main: `09000ed89d390709d5c368f672212b87d08254be`. Its Engine implementation is byte-identical to published `engine-v0.7.1` at `344e708150eb158b55144a0228f154ef28f63d6d`; `evidence_relevance.rs` SHA-256: `e2b78736033a236778581ccdaa3d2d5137b599f9b76d4a4cb54b37914c58a76b`. The comparison is against **released 0.7.0 v30**, tag commit `5c7682d988c18a5a7e7a1954bd2a369cad2e9070`, not the historical v29 column.

| Audit result | Original observation |
| --- | --- |
| Launch/preparation public-API matrix | 56 cases: 7 positives retained; 49 negatives, of which 40 falsely Relevant and 9 withheld |
| [#520 governing fictional/false/retracted context](https://github.com/git-ksk/reasoning-harness/issues/520) | 28 false Relevant in 0.7.1; the same 28 are Ambiguous in released 0.7.0 v30 |
| [#521 raw strikethrough/negation markers](https://github.com/git-ksk/reasoning-harness/issues/521) | 12 false Relevant in 0.7.1; the same 12 are Ambiguous in released 0.7.0 v30 |
| Additional source-reconciliation authority/replay controls | 22/22 pass; original artifact/Conflict retained, no hard truth claim or receipt created |
| Locked workspace / strict all-target Clippy / format | 1,468 pass, 0 fail, 6 existing ignored / PASS / PASS |
| Standard fixtures / original 0.7 archive | 20/20, zero unsupported accepted claims / 13 exact hashes retained |

The failures concern relevance/acquisition qualification. They do not demonstrate downstream wrong answers or external-fact `Known`/`Supported` promotion. Existing checks passing does not disprove these new failures. Passing reconciliation probes cover their enumerated cases, not every possible bug.

`manifest.json` gives portable file hashes and the exact 28/12 case groups. `cases.json`, the two relevance outputs, Rust reproducers and reconciliation output are retained byte-for-byte. `validation.json` is the **original audit-time record**: absolute `/private/tmp` paths refer to the original run, and `modified_tracked_files=false` describes that audit before this evidence/roadmap commit. `workspace-tests.log` and `clippy.log` retain the corresponding logs; their original names/hashes appear in that record. `fixtures.json` retains the fixture result. No compiled binaries or copied Engine implementation are committed.

## Reproduce the public-API comparison

From the repository root, with Rust 1.88.0, Python 3, Git, tar and ripgrep available:

```sh
audit="$PWD/evaluation/engine-0.7.1-rereview-20261013"
scratch="$(mktemp -d)"
for version in 0.7.0 0.7.1; do
  source_dir="$scratch/source-$version"
  target_dir="$scratch/target-$version"
  mkdir -p "$source_dir"
  git archive "engine-v$version" | tar -x -C "$source_dir"
  cargo build --locked --manifest-path "$source_dir/Cargo.toml" \
    -p reasoning-harness-core --target-dir "$target_dir"
  core_rlib="$(rg --files "$target_dir/debug/deps" | rg '/libreasoning_harness_core-[^.]+\.rlib$')"
  json_rlib="$(rg --files "$target_dir/debug/deps" | rg '/libserde_json-[^.]+\.rlib$')"
  rustc --edition=2021 "$audit/probe.rs" \
    -L "dependency=$target_dir/debug/deps" \
    --extern "reasoning_harness_core=$core_rlib" --extern "serde_json=$json_rlib" \
    -o "$scratch/probe-$version"
  "$scratch/probe-$version" "$audit/cases.json" > "$scratch/results-$version.jsonl"
done
cmp "$audit/results-070.jsonl" "$scratch/results-0.7.0.jsonl"
cmp "$audit/results.jsonl" "$scratch/results-0.7.1.jsonl"
```

The original 0.7.0 observation instead compiled the unchanged tagged core source directly with `rustc` against the unchanged normal dependency lock records (`schemars`, `serde`, `serde_json`, `thiserror`). The clean tagged Cargo build above is the portable reproduction route; it does not change either tag or historical semantics. Cached dependencies permit adding `--offline` to `cargo build`.

This portable route was also executed before saving the bundle: both 56-row outputs and the 22-row 0.7.1 reconciliation output matched the original records byte-for-byte. `portable-validation.json` and `portable-reproduction.log` record that check. Generated source/build scratch was removed afterwards. It remains the same authored audit corpus, not fresh independent acceptance.

The runner invokes `derive_effective_evidence_local_qualification_v17` and `materialize_evidence_relevance_v30`, with exact advisory binding and raw qualification. Positive advisory labels cannot override the source-local factuality floor. The additional `materialize_evidence_relevance_v29` result is preserved for compatibility observation only.

For the 22 reconciliation controls, compile `reconciliation.rs` using the same 0.7.1 library/dependency arguments, execute without arguments and compare stdout with `reconciliation-results.jsonl`. These controls do not involve OS-keyring/human approval or provider calls.

## Verify the saved observation

```sh
python3 - <<'PY'
import hashlib, json
from pathlib import Path
p = Path('evaluation/engine-0.7.1-rereview-20261013')
m = json.loads((p / 'manifest.json').read_text())
for name, expected in m['files_sha256'].items():
    assert hashlib.sha256((p / name).read_bytes()).hexdigest() == expected, name
current = [json.loads(s) for s in (p / 'results.jsonl').read_text().splitlines()]
old = {r['id']: r for r in map(json.loads, (p / 'results-070.jsonl').read_text().splitlines())}
groups = m['groups']
assert len(groups['520_governing_context']) == 28
assert len(groups['521_raw_markers']) == 12
failures = {r['id'] for r in current if r['mismatch']}
assert failures == set(groups['520_governing_context'] + groups['521_raw_markers'])
assert len(current) == 56 and sum(r['expected_relevant'] for r in current) == 7
assert all(r['v30'] == 'relevant' for r in current if r['expected_relevant'])
assert all(old[i]['v30'] == 'ambiguous' for i in failures)
assert all(r['engine_version'] == '0.7.1' for r in current)
assert all(r['engine_version'] == '0.7.0' for r in old.values())
controls = list(map(json.loads, (p / 'reconciliation-results.jsonl').read_text().splitlines()))
assert len(controls) == 22 and all(r['pass'] for r in controls)
print('Hashes and original observation verified; 40 failures remain recorded.')
PY
```

Corrective implementation and fresh acceptance belong to the [planned 0.7.2 roadmap](../../docs/engine-0.7.2-roadmap.md). Preserve this original failing observation after later fixes.
