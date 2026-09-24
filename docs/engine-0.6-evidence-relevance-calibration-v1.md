# Engine 0.6 candidate: evidence-target relevance calibration v1

Status: fresh unobserved calibration authored. Core relevance contract and deterministic materialization tests pass. No live model-backed observation has been performed yet.

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

## Next sequence

1. implement the bounded live calibration runner using GitHub repository secrets;
2. record proposal separately from Harness-materialized assessment;
3. compare semantic path against simple lexical baseline;
4. tune only this fresh calibration if needed;
5. freeze #462 semantics/thresholds;
6. author an independent holdout only after freeze;
7. freeze the holdout before first observation;
8. promote #462 only after correctness and utility gates pass.
