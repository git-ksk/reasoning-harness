# Engine 0.6 evidence-target relevance calibration v22 — operational TPD successor

Status: pre-freeze implementation candidate. v22 preserves v21 semantics and changes canonical admission only. The fresh v22 suite/configuration, operational-equivalence tests, tag-triggered workflow, and TPD recovery-floor guard are implemented. No v22 freeze tag or live observation exists.

v22 follows immutable v21 run `36295631123`. It is not a rerun of v21 and must not reinterpret v21 evidence.

## Objective

Remove the operational failure mode demonstrated by v21 without tuning semantic labels, prompts, provider roles, or materialization behavior to the quota outcome.

v21 established that a three-request tiny Groq readiness check can pass immediately before canonical while the organization has insufficient Tokens Per Day (TPD) headroom for the full required arm. v22 therefore treats tiny readiness as transport/credential/TPM/RPD evidence only and adds a conservative TPD recovery floor before any canonical tag may be consumed.

## Frozen semantic inheritance

Keep unchanged from v21:
- fixed core `evidence-relevance-fixed-core-v1`, exactly 48 cases;
- all case IDs, expected proposals, expected local qualifications, and expected dispositions;
- primary binding proposal v5;
- raw verifier v8;
- effective qualification v2 (`reason-evidence-relevance-effective-qualification-v2`);
- materialization v16 (`target-evidence-relevance-binding-materialization-v16`);
- authority-qualified gate semantics;
- deterministic local-risk classifier;
- target-negative terminal rules and strict Harness-owned identity floors;
- all retry, timeout, provider-latch, telemetry, and public-safe diagnostic behavior;
- required providers Mistral + Groq;
- Google full non-gating replication;
- one-shot immutable canonical policy;
- independent holdout prohibition until canonical PASS.

Google v21 case 76 remains a non-gating relation-axis diagnostic only. Its final Irrelevant disposition was correct through the target-negative terminal, so v22 does not alter semantic authority rules for that observation.

## Measured TPD behavior

The v21 Groq quota response preserved:
- TPD limit: 200,000 tokens;
- used: 199,298;
- requested: 1,295;
- remaining headroom before the rejected request: 702;
- provider retry interval: 4m16.176s (256.176s);
- TPM remaining: 8,000/8,000;
- RPD remaining: 974/1,000.

The deficit for the rejected request was 593 tokens. `593 / (200000 / 86400) = 256.176s`, exactly matching the provider retry interval. v22 therefore treats the observed TPD as a continuously replenishing 24-hour token bucket for admission planning.

Historical full-arm demand is materially larger than a tiny readiness probe:
- v20: 97,259 Groq tokens across 39 successful cases;
- v21: 31,648 Groq tokens across 13 successful cases;
- straight-line estimates are approximately 116.9K–119.7K tokens for 48 cases under the current runner shape.

For admission planning, v22 reserves a conservative 150K-token Groq headroom target. This is an operational planning bound, not a semantic threshold.

## TPD recovery floor

The v21 Groq job completed at `2026-09-27T05:02:01Z` while the bucket had just been observed near exhaustion. A full 24-hour cooldown from that conservative anchor ends at:

- UTC: `2026-09-28T05:02:01Z`
- JST: `2026-09-28 14:02:01 +09:00`

No v22 freeze tag may be created before that time.

The 24-hour floor is intentionally stricter than the estimated 150K-headroom recovery time (~17.9 hours from the near-full observation). It avoids pretending that the tiny readiness can directly measure TPD headroom.

This floor is a lower bound, not a guarantee. Any material Groq organization usage after the v21 observation consumes the same organization-level allowance and therefore invalidates the assumption of a recovered bucket. If intervening usage is known or suspected, v22 freeze must be delayed again rather than weakening the gate.

## Readiness semantics

Immediately before freeze, run the existing synthetic Groq readiness workflow against the exact v22 candidate branch.

A PASS means only:
- credential works;
- model endpoint accepts requests;
- no current tiny-request quota block;
- TPM/RPD headers are sane across the bounded probes.

It does **not** prove full-run TPD headroom and must never be cited as such.

A quota/rate-limit failure must preserve the new public-safe `provider_diagnostic` evidence. No raw unredacted provider body may be emitted to public Actions logs or committed artifacts.

## Fresh observation identity

v22 is a fresh versioned observation surface with:
- suite: `evidence-relevance-calibration-v22`;
- configuration: `evidence-relevance-live-calibration-v22`;
- semantic annotation protocol remains `evidence-relevance-effective-qualification-v21` because semantics are unchanged;
- materialization remains v16;
- the same calibration seed as v21 is intentionally retained to isolate the operational admission change rather than introduce a new stochastic factor.

The new suite metadata is not permission to change any frozen case or label.

## Required pre-freeze proof

Before the v22 tag can be created:
- v22 case array is semantically identical to v21 (48/48, no relabel/growth/reordering);
- v18/v19/v20/v21 semantic regressions remain green;
- v22 operational-equivalence tests prove the same v16 materialization over all 48 expected cases;
- full core/providers/CLI package tests are green;
- all-target Clippy `-D warnings`, fmt, validate-only, and exact surface checksum are green;
- standard PR CI is green;
- public-safety scan is green;
- current time is at or after `2026-09-28T05:02:01Z`;
- a fresh synthetic Groq readiness run succeeds on the exact candidate branch;
- no known material Groq organization usage has occurred after the recovery anchor without extending the cooldown.

## Pre-freeze implementation evidence

The current candidate is green on the deterministic/operational pre-freeze proof:
- v22 `cases` are byte-semantic JSON-equivalent to frozen v21: 48/48, no growth, relabel, or reorder;
- annotation protocol remains `evidence-relevance-effective-qualification-v21` and materialization remains v16; no core semantic production rule changed for v22;
- v22 operational-equivalence suite: 3/3 PASS;
- regressions: v18 17/17, v19 12/12, v20 16/16, v21 20/20 PASS;
- calibration runner focused suite: 28/28 PASS;
- validate-only: configuration `evidence-relevance-live-calibration-v22`, 48 planned / 0 observed, `validate_only_non_scorable`;
- full core package main suite: 246/246 PASS plus all integration blocks;
- providers: 153 passed / 1 ignored / 0 failed;
- CLI main suite: 205 passed / 3 ignored / 0 failed plus all integration blocks;
- all-target Clippy `-D warnings` for core/providers/CLI: PASS;
- `cargo fmt --all -- --check` and `git diff --check`: PASS;
- v22 frozen-surface checksum covers 40 explicit files and revalidates cleanly;
- live workflow contains a fail-closed `2026-09-28T05:02:01Z` recovery-floor guard before checkout/provider work.

Standard PR CI and a fresh pre-freeze Groq readiness run still must be green on the exact pushed candidate. The recovery-floor time has not yet elapsed, so creating the v22 freeze tag remains prohibited.

## Canonical policy

The v22 live workflow remains first/only and tag-triggered. It must reject workflow reruns and fail closed if launched before the TPD recovery floor.

If required Groq still reaches TPD/RPD/TPM quota, v22 is immutable FAIL. Do not rerun the same tag. The preserved `provider_diagnostic` is successor evidence only.

If v22 passes required Mistral + Groq gates, Google remains replication-only for this calibration. Only then may independent holdout work resume.
