# Engine 0.6 evidence-target relevance calibration v22 — operational TPD successor

Status: pre-freeze implementation candidate. v22 preserves v21 semantics and changes canonical admission only. The fresh v22 suite/configuration, operational-equivalence tests, tag-triggered workflow, and modeled TPD headroom/pacing guard are implemented. No v22 freeze tag or live observation exists.

v22 follows immutable v21 run `36295631123`. It is not a rerun of v21 and must not reinterpret v21 evidence.

## Objective

Remove the operational failure mode demonstrated by v21 without tuning semantic labels, prompts, provider roles, or materialization behavior to the quota outcome.

v21 established that a three-request tiny Groq readiness check can pass immediately before canonical while the organization has insufficient Tokens Per Day (TPD) headroom for the full required arm. v22 therefore treats tiny readiness as transport/credential/TPM/RPD evidence only and adds a modeled minimum starting headroom plus slow in-run pacing before any canonical tag may be consumed.

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

For admission planning, v22 does not wait for a fully recovered bucket. It requires a modeled 100K-token starting headroom, then lets the continuously replenishing bucket recover during a deliberately slow Groq arm. This is an operational planning bound, not a semantic threshold.

## Canonical self-budget guard

Modeled-headroom admission is necessary but not sufficient because provider output usage can vary. v22 therefore adds a second, in-run conservation guard for the required Groq arm:
- maximum observed provider-token budget: 140,000 tokens;
- reserve required before starting each next case: 4,000 tokens;
- historical maximum observed total usage for one completed Groq case across v20/v21: 2,792 tokens;
- historical projected full-arm demand remains approximately 117K-120K.

Before every new Groq case, the runner sums provider-reported token usage from prior observations. If `consumed + 4,000 > 140,000`, it latches the provider arm before issuing another external request and suppresses the remaining cases. If a prior model call has missing token-usage telemetry while the guard is active, it also latches fail-closed rather than assuming zero usage.

The 140K bound is deliberately below the modeled ~143.5K supply available from a 100K start plus mandatory pacing refill, while remaining above the historical ~117K-120K full-arm demand. With no concurrent organization use, the guard stops before the modeled bucket is drained even if observed demand expands. A guard-triggered canonical is still an immutable FAIL; conservation never weakens acceptance.

## Modeled TPD headroom and paced execution

The v21 quota observation anchored the bucket near exhaustion at `2026-09-27T05:02:01Z`, with only 702 tokens of headroom. At `200000 / 86400 = 2.314814...` tokens/second, recovering from 702 to 100,000 tokens of modeled headroom takes 42,896.736 seconds. The conservative rounded-up admission floor is therefore:

- UTC: `2026-09-27T16:56:58Z`;
- JST: `2026-09-28 01:56:58 +09:00`.

No v22 freeze tag may be created before that time. This is **not** a full-bucket reset assumption. It is the earliest modeled point at which the known near-empty bucket has replenished 100K of headroom, assuming no material organization-level Groq usage after the anchor. Any known or suspected intervening usage invalidates the estimate and requires delaying or re-anchoring admission.

The required Groq arm is then intentionally slow:
- inter-case delay: 390,000 ms (6.5 minutes);
- provider minimum request interval: 10,000 ms;
- two model calls per completed case under the frozen runner shape;
- Groq job timeout: 360 minutes.

Across 48 cases, the 47 inter-case waits alone contribute 18,330 seconds. The intra-case 10-second spacing contributes another 480 seconds, for at least 18,810 seconds (5h13m30s) of deliberate pacing before counting model execution time. At the observed TPD replenishment rate that restores about 43.5K tokens during the arm. Thus a modeled 100K start plus paced replenishment supplies about 143.5K tokens over the run, above the separate 140K self-budget cap. The margin is conservative relative to the historical ~117K-120K full-arm demand and the 2,792-token historical maximum completed case.

This model is still conditional on organization-level usage. The public API does not expose TPD remaining in normal success headers, so v22 does not pretend to measure exact headroom. If an unexpected TPD 429 occurs, the preserved provider diagnostic is authoritative and the immutable canonical fails.

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
- current time is at or after the modeled 100K-headroom floor `2026-09-27T16:56:58Z`;
- a fresh synthetic Groq readiness run succeeds on the exact candidate branch;
- no known material Groq organization usage has occurred after the anchor without delaying or re-anchoring the modeled headroom floor.

## Pre-freeze implementation evidence

The current candidate is green on the deterministic/operational pre-freeze proof:
- v22 `cases` are byte-semantic JSON-equivalent to frozen v21: 48/48, no growth, relabel, or reorder;
- annotation protocol remains `evidence-relevance-effective-qualification-v21` and materialization remains v16; no core semantic production rule changed for v22;
- v22 operational-equivalence suite: 3/3 PASS;
- regressions: v18 17/17, v19 12/12, v20 16/16, v21 20/20 PASS;
- calibration runner focused suite: 30/30 PASS;
- validate-only: configuration `evidence-relevance-live-calibration-v22`, 48 planned / 0 observed, `validate_only_non_scorable`;
- full core package main suite: 246/246 PASS plus all integration blocks;
- providers: 153 passed / 1 ignored / 0 failed;
- CLI main suite: 205 passed / 3 ignored / 0 failed plus all integration blocks;
- all-target Clippy `-D warnings` for core/providers/CLI: PASS;
- `cargo fmt --all -- --check` and `git diff --check`: PASS;
- v22 frozen-surface checksum covers 40 explicit files and revalidates cleanly;
- live workflow contains a fail-closed `2026-09-27T16:56:58Z` modeled 100K-headroom guard before checkout/provider work;
- required Groq execution additionally uses 390-second inter-case pacing, a 360-minute job bound, a 140K observed-token cap with a 4K pre-case reserve, and fail-closed handling for missing usage telemetry.

Standard PR CI and a fresh pre-freeze Groq readiness run still must be green on the exact pushed candidate. The modeled 100K-headroom floor has not yet elapsed, so creating the v22 freeze tag remains prohibited.

## Canonical policy

The v22 live workflow remains first/only and tag-triggered. It must reject workflow reruns and fail closed if launched before the modeled 100K-headroom floor.

If required Groq still reaches TPD/RPD/TPM quota, v22 is immutable FAIL. Do not rerun the same tag. The preserved `provider_diagnostic` is successor evidence only.

If v22 passes required Mistral + Groq gates, Google remains replication-only for this calibration. Only then may independent holdout work resume.
