# Engine 0.6 evidence relevance holdout v6 immutable result

Canonical run `36750505629`, attempt 1, is immutable **FAIL**.

The failure occurred before any provider observation. The frozen holdout-v6 tag and surface remain unchanged.

- tag: `engine-0.6-evidence-relevance-holdout-v6-freeze`
- freeze commit: `a8782eefd2609385ed7e598552aced05ddee0660`
- semantics baseline: effective qualification v8 + materialization v21
- Mistral / Google / Groq: runner RC 1, completed cases 0
- shared failure: `unexpected checkpoint suite id "evidence-relevance-holdout-v6"`

Root cause: the v6 runner added `HoldoutProfile::V6`, while the separate `checkpoint_profile()` mapping still ended at holdout v5. Validate-only did not write checkpoints, so pre-observation validation missed the wiring defect.

This is operational harness evidence, not semantic evidence. No v6 case outcome was observed and no provider output may shape successor semantics.

Successor rule: never rerun/rescore/relabel v6; keep v8/v21 frozen; fix checkpoint/profile completeness generically; add regression coverage; then author a fresh holdout with zero overlap against holdout v1-v6 and successor-v5/v6 development corpora.
