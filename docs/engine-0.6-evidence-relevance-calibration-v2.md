# Engine 0.6 candidate: evidence-target relevance calibration v2

Status: fresh unobserved successor authored after frozen v1. v1 remains immutable and is not rerun or rescored.

## Why v2 exists

Frozen v1 run `35991268202` was operationally complete on Mistral and Google but failed semantic gates. Two general findings drive v2:

1. the model guidance did not sharply distinguish affirmative irrelevance from unresolved applicability/binding, causing avoidable `irrelevant` decisions on ambiguity cases;
2. v1 case `24_conflicting_sections` represented factual contradiction about the same exact target/relation, which belongs downstream of relevance and therefore was not a valid wrong-target relevance failure.

## Successor changes

v2 changes only the calibration-facing semantic boundary needed by those findings.

The advisory decision rule now states:

- `relevant`: material is sufficiently about the exact target and requested relation to remain eligible downstream;
- `irrelevant`: material affirmatively concerns a different target or different requested relation;
- `ambiguous`: identity, relation binding, or applicability cannot be established, including partial/truncated passages, uncertain rename/alias relationships, mixed-product material with unresolved binding, or omitted local support;
- missing/insufficient local information is not itself evidence of irrelevance;
- factual contradiction about the same target/relation does not make material irrelevant; contradiction/truth remain downstream concerns.

No deterministic identity floor, authority boundary, source/provenance rule, freshness rule, verification rule, or finalization rule changes in v2.

## Fresh v2 corpus

- suite: `evidence-relevance-calibration-v2`
- issue: #462
- cases: 26
- status: `fresh_unobserved_calibration`
- production motivating incident: excluded from tuning

The v2 corpus is copied from the frozen v1 semantic coverage except for case 24, which is replaced with a genuine relation-binding ambiguity: a shared Cedar Vault / Cedar Vault Classic regional table contains a West row, but the supplied local excerpt does not establish which product the row applies to.

The v1 truth-conflict fixture remains preserved only in the v1 freeze/tag and v1 result documentation.

## Acceptance

Canonical Mistral and Google arms must both be operationally complete with:

- wrong-target / unresolved-binding materialized as `relevant`: **0**;
- utility misses: **0**;
- provider failures: **0**.

Proposal exact accuracy remains diagnostic. Harness-owned deterministic safety overrides are allowed and must remain observable.

The simple lexical baseline remains diagnostic only. It is expected to underperform on semantic/cross-lingual cases and to over-retain same-name wrong-feature material.

## Freeze discipline

Before first live observation:

1. deterministic v2 materialization must pass all 26 cases;
2. runner/core clippy and tests must pass;
3. the exact core/runner/fixture surface must be checksummed;
4. the live workflow must be committed;
5. a new `engine-0.6-evidence-relevance-calibration-v2-freeze` tag must bind the first/only canonical v2 observation.

No independent holdout is authored until v2 calibration acceptance passes and #462 semantics are then frozen.
