# Engine 0.6 evidence relevance independent holdout v4

Status: **pre-freeze / unobserved**.

Successor v4 semantics were frozen first at commit 663ade43de96dbda511a11f351e8ce1593d85ed1 with tag engine-0.6-evidence-relevance-successor-v4-semantics-freeze. The fresh holdout v4 corpus was authored only after that freeze.

## Frozen semantic baseline

- effective qualification: v6
- materialization: v19
- successor-v4 semantic checksum: fixtures/evidence-relevance-holdout-successor-v4/semantics-v4.sha256
- historical v23, holdout v1, holdout v2, and holdout v3a observations remain immutable
- runner routing stays v1=v3/v16, v2=v4/v17, v3=v5/v18, v4=v6/v19

## Fresh surface

- suite: evidence-relevance-holdout-v4
- cases: 26 = Relevant 8 / Irrelevant 10 / Ambiguous 8
- annotation protocol: evidence-relevance-effective-qualification-v6
- fixed core: evidence-relevance-fixed-core-v4
- corpus status: fresh_unobserved_holdout

The v4 surface uses new case IDs, canonical entities, tasks, and candidate text.

Independence against immutable holdout v1/v2/v3 requires zero case-ID overlap, zero canonical-entity overlap, zero task overlap, zero exact candidate-signal overlap, and zero exact 8-token candidate-signal n-gram overlap. The v3a terminal-case IDs/entities and failed v1/v2 motivating entities are explicitly excluded.

The corpus is not a relabel or paraphrase set of the v3a misses. It includes fresh positive, negative, and abstention controls across exact identity, authorized alias, semantic-equivalent policy, distributed support, relation mismatch, repeated sibling ownership, navigation-only mentions, explicit absence, mapping uncertainty, shared ownership, truncation, URL-only identity, and single-signal near siblings.

## Groq TPD admission

The next canonical Groq arm is conservatively re-anchored from immutable v3a actuals:

- v3a configured start headroom: 55,000
- v3a pacing: 300 seconds
- minimum request interval: 10 seconds
- v3a observed tokens: 65,015
- conservative completion anchor: 2026-09-29T17:12:55Z
- modeled post-v3a headroom: 7,947.96
- required v4 start headroom: 55,000
- v4 self-budget: 70,000
- pre-case reserve: 4,000
- v4 pacing: 300 seconds
- fail-closed earliest floor: 2026-09-29T22:51:42Z / 2026-09-30 07:51:42 JST

The workflow recomputes the floor from frozen constants. Any known material intervening organization-level Groq usage invalidates this model and requires re-anchoring before the freeze tag is pushed. Tiny readiness remains transport/credential/TPM/RPD evidence only, not TPD-headroom proof.

## Pre-freeze validation contract

Before creating engine-0.6-evidence-relevance-holdout-v4-freeze:

- v4 distribution and frozen expected semantics PASS
- v1/v2/v3 independence audit PASS
- validate-only reports 26 planned / 0 observed
- successor-v4 immutable replay remains fully exact
- full core/providers/CLI PASS
- workspace all-target Clippy -D warnings PASS
- rustfmt / diff check PASS
- workflow YAML parse PASS
- surface-v4.sha256 exact validation PASS
- production special-case scan clean

The first/only v4 freeze tag is canonical and must never be moved, recreated, or rerun as a replacement observation.

## Local pre-freeze validation

- v4 frozen expected semantics: 26/26 PASS
- v1/v2/v3 independence audit: PASS
- exact 8-token candidate-signal overlap: 0 against v1/v2/v3
- v3a terminal-case/entity exclusion: PASS
- validate-only: 26 planned / 0 observed
- successor-v4 replay: v23 48/48 x3, holdout v1 26/26 x3, holdout v2 26/26 x3, holdout v3a 26/26 x3
- focused v4 manifest tests: 2/2 PASS
- holdout runner tests: 4/4 PASS
- core: 587 passed / 0 failed
- providers: 153 passed / 1 ignored / 0 failed
- CLI: 368 passed / 4 ignored / 0 failed
- workspace all-target Clippy -D warnings: PASS
- rustfmt / diff check: PASS
- workflow YAML: 93/93
- production special-case scan: clean
