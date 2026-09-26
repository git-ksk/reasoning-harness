# Engine 0.6 candidate: evidence-target relevance calibration v1

Status: fresh unobserved calibration authored. Core relevance contract, bounded live calibration runner, lexical baseline, and deterministic materialization validation are implemented. No live model-backed observation has been performed yet.

## Identity

- issue: #462
- suite: `evidence-relevance-calibration-v1`
- status: `fresh_unobserved_calibration`
- corpus: `fixtures/evidence-relevance-calibration-v1/manifest.json`
- cases: 26
- production motivating incident: excluded from tuning

## Contract under test

The model proposes only `relevant`, `irrelevant`, or `ambiguous`. Harness-owned policy fixes target identity, aliases, relation kind, strict-vs-semantic identity requirements, and assessment budgets. Materialization can block unsafe model relevance but cannot create evidence authority.

Strict entity identity requires a canonical/alias anchor in content-bearing material. Canonical URL and navigation/footer matches are observable but cannot self-authorize relevance. When policy explicitly permits semantic equivalence, non-lexical paraphrase or cross-lingual relevance may be accepted by the advisory semantic path.

Missing model output deterministically falls back to ambiguous, never relevant.

## Calibration families

Positive cases include exact product names, acronym/expanded aliases, semantic paraphrase, identity in title with relation in body, identity/relation split across sections, URLs that omit target tokens, Japanese target with English source, English target with Japanese alias, structured metadata plus body support, stale-but-relevant material, and multi-section support.

Negative cases include same service but wrong feature, sibling product overlap, navigation/footer-only target mention, broad landing pages without local support, unrelated announcements, comparison-only mentions, same entity with wrong relation, and prompt injection attempting to self-declare relevance/trust.

Ambiguous cases include unknown rename, partial identity, mixed multi-product material, conflicting sections, insufficient local passage, and URL-only identity.

## Acceptance metrics

Report separately:

- wrong-target relevance retention — hard gate 0;
- false rejection of expected-relevant material;
- expected-relevant material left ambiguous;
- ambiguous disposition rate;
- model proposal exact accuracy;
- materialized disposition exact accuracy;
- deterministic safety overrides;
- model calls / provider attempts / tokens / latency;
- provider/model operational failures separately from semantic failures;
- comparison against a simple lexical-overlap baseline for positive semantic/cross-lingual cases.

An always-relevant policy fails correctness. An always-irrelevant/ambiguous policy cannot pass utility.

## Current deterministic validation

- core relevance unit tests: 14 PASS;
- calibration manifest materialization: 26/26 exact expected dispositions;
- production motivating product excluded from corpus;
- negative-family expected cases never materialize relevant;
- core clippy with `-D warnings`: PASS;
- `git diff --check`: PASS.

## Live runner readiness

`reason-evidence-relevance-study` now validates the exact 26-case calibration surface and records the raw model proposal separately from the Harness-materialized assessment. It also computes a deliberately simple lexical baseline, deterministic safety overrides, correctness/utility metrics, provider/model operational failures, model-call count, provider attempts, token usage, and latency.

The Harness-owned v1 assessment budget is 2 model calls / 192 output tokens / 15,000 ms. The second call is available only as the bounded JSON-object fallback after structured-output incompatibility or parse failure; elapsed-time and model-call budgets apply across both calls.

Canonical live observation remains GitHub Actions + repository secrets only. No live result is claimed yet.

## Live calibration runner

The implemented `reason-evidence-relevance-study` runner is bound to the exact 26-case calibration directory and rejects other suite/status/issue identities. `--validate-only` performs deterministic corpus/materialization validation with zero provider calls. A canonical observation omits `--fixture` and evaluates all 26 cases exactly once.

For each case the runner records the advisory model proposal separately from the Harness-materialized relevance assessment. It also records a deliberately simple lexical baseline, deterministic safety overrides, model-call count, provider attempt count, token usage, latency, fallback usage, and typed operational failure class. Raw model responses and credentials are not persisted.

The Harness-owned v1 assessment budget is 2 model calls, 192 max output tokens per call, and 15,000 ms absolute assessment time per case. One primary JSON-Schema call is permitted plus at most one bounded JSON-object fallback. The fallback cannot exceed the model-call budget, and the primary plus fallback share one absolute elapsed deadline. Adapter-internal HTTP retries remain separately observable as `provider_attempts`.

The first canonical live observation runs in GitHub Actions using repository secrets, not local credentials. The frozen arms are Mistral `ministral-8b-latest` and Google `gemini-3.5-flash-lite`; the workflow is `.github/workflows/engine-0.6-evidence-relevance-calibration-v1-live.yml`. The first-observation surface is checksum-bound and must be tag-frozen before credentials are read. Workflow reruns are rejected for that freeze identity.

## Next sequence

1. freeze/checksum the calibration surface and live workflow before first observation;
2. run Mistral and Google canonical live calibration arms using repository secrets;
3. record proposal separately from Harness-materialized assessment and lexical baseline;
4. tune only this fresh calibration if needed;
5. freeze #462 semantics/thresholds;
6. author an independent holdout only after freeze;
7. freeze the holdout before first observation;
8. promote #462 only after correctness and utility gates pass.
