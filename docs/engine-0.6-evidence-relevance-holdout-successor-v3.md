# Engine 0.6 evidence relevance holdout successor v3

Status: design/implementation candidate after immutable holdout v2 FAIL.

## Evidence boundary

Independent holdout v2 remains immutable FAIL:

- freeze: `engine-0.6-evidence-relevance-holdout-v2-freeze`
- commit: `c38b5f7dbfcf8c01dcaa6f6cd7341e43d3b87325`
- canonical run: `36533340582`, attempt 1 only
- all three required providers completed 26/26 operationally
- wrong-target Relevant retention remained 0 for Mistral, Groq, and Google
- Groq passed authority/materialization 26/26
- Mistral and Google each missed only `v2h13_negative_unrelated_launch`, conservatively materializing Ambiguous instead of Irrelevant

This successor may replay frozen observations for postmortem/regression. It does not rerun, rescore, relabel, or mutate holdout v2.

## Generic gap

v4/v17 correctly blocks positive relevance when a strict Harness target anchor is absent. However, when both model outputs over-bind or under-resolve identity, the Harness currently cannot distinguish:

1. a genuinely ambiguous identity with URL-only, mapping, ownership, truncation, or omitted-referent uncertainty; and
2. a locally substantive sibling-subject candidate where the visible subject is repeatedly named across independent local signals and owns the requested relation.

Mere target-anchor absence is not sufficient negative identity evidence and must remain Ambiguous.

## Successor rule: repeated sibling subject

A candidate may provide Harness-owned negative identity corroboration only when all of the following are true:

1. identity policy is `require_harness_anchor`;
2. no canonical or authorized alias target anchor occurs in an identity-capable signal;
3. no target-only canonical URL signal is present;
4. deterministic local scope risk is `none`;
5. the requested relation is locally present;
6. a `SourceTitle`, `Heading`, or `StructuredMetadata` signal contains a normalized 2-4 token phrase that:
   - shares at least one token with the Harness target canonical/alias identity,
   - contains at least one additional non-target, non-generic/relation token, and
   - is repeated verbatim at token boundaries in a separate `Excerpt`, `Fact`, or `StructuredMetadata` signal;
7. both advisory model stages completed, so the deterministic rule cannot mask an operationally missing proposal/verifier result.

The repeated phrase is evidence of a locally stable sibling subject, not a generic relation phrase. The rule never creates positive identity or relevance authority.

## Explicit non-goals / abstention controls

The rule must not fire for:

- `allow_semantic_equivalent` policy;
- URL-only target identity;
- single-signal near-name mentions;
- partial target-token fragments split across different signals;
- explicit alias/rename/successor uncertainty;
- omitted/shared ownership;
- clipped/truncated context;
- pronoun-only or unnamed-subject relation passages;
- generic landing/catalog absence already handled by the existing target-absence path.

These remain on existing v17 semantics and are Ambiguous where previously Ambiguous.

## Versioning

- effective qualification: v5
- materialization: v18
- v1 holdout runner remains v3/v16
- v2 holdout runner remains v4/v17
- successor replay uses v5/v18 only in separately versioned tests/fixtures

v18 may materialize Irrelevant from the repeated-sibling rule only when the v5 effective identity is `distinct_target`, relation is `requested_relation`, deterministic/effective risk are both none, and both model stages are present. Otherwise it delegates to immutable v17 behavior.

## Regression contract before semantic freeze

Before a successor-v3 semantic freeze:

- generic property/unit controls for both positive trigger and all abstention boundaries must pass;
- immutable calibration v23 replay must preserve 48/48 authority/materialization for all three providers;
- immutable holdout v1 replay must preserve zero wrong-target Relevant and not reclassify the historical result itself;
- immutable holdout v2 replay must preserve Groq 26/26 and reduce Mistral/Google terminal v2h13 utility misses without creating any wrong-target Relevant or false rejection;
- full core/providers/CLI tests, workspace all-target Clippy, fmt, diff, YAML, public-safety and production special-case scans must pass;
- no provider name, fixture ID, synthetic entity name, or exact holdout text may appear in production branching.

Only after successor semantics are frozen may a fresh independent holdout v3 surface be authored.

## Pre-freeze validation

Local validation:

- repeated-sibling property / abstention controls: 8/8 PASS
- successor v3 replay: v23 48/48 x 3, holdout v1 26/26 x 3, holdout v2 26/26 x 3
- historical Google holdout-v1 h14 remains recorded Relevant and changes only under successor semantics to Irrelevant
- historical Mistral/Google holdout-v2 v2h13 remain recorded Ambiguous and change only under successor semantics to Irrelevant
- core full suite: PASS
- providers: 153 passed / 1 ignored / 0 failed
- CLI: 205 passed / 3 ignored / 0 failed plus integration suites green
- workspace all-target Clippy `-D warnings`: PASS
- fmt / diff / workflow YAML: PASS
- public-safety / production special-case scan: clean

Intended semantics freeze tag: `engine-0.6-evidence-relevance-successor-v3-semantics-freeze`. A fresh holdout v3 may be authored only after that tag exists.
