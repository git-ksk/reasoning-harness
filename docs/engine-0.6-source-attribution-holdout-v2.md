# Engine 0.6 source-attribution holdout v2

Status: frozen and observed. Canonical run `37326666360` attempt 1 is immutable PASS; see `engine-0.6-source-attribution-holdout-v2-result.md`.

## Why v2 exists

Independent holdout v1 is an immutable operational FAIL at run `37300354564` attempt 1. Mistral and Google completed 18/18 and passed every acceptance gate. Groq completed 7/18 and then failed on a provider-side best-effort structured-JSON generation error surfaced as typed `UnsupportedCapability`; every observed semantic hard gate remained zero.

The v1 workflow set `REASON_GROQ_STRUCTURED_OUTPUT_RETRIES=0`. That setting came from the evidence-relevance evaluation line, where zero adapter retries intentionally surfaces a structured-generation failure after one provider attempt so a separate Harness-owned strict-Text fallback can execute inside the case budget. The source-attribution runner has no such transport fallback: typed `UnsupportedCapability` is an immediate operational provider failure. Therefore the v1 retry override removed the Groq adapter's existing generic bounded recovery without providing the fallback that justified that override.

This is an evaluation-wiring defect, not a source-attribution semantic defect. The immutable v1 result, corpus, tag, and score remain unchanged.

## Frozen v2 transport contract

The successor keeps production source-attribution semantics, prompts, JSON schemas, parsing, materialization, hard gates, provider models, and thresholds unchanged from the v6 semantics freeze and v1 acceptance contract.

For Groq only, the future v2 canonical workflow will explicitly set `REASON_GROQ_STRUCTURED_OUTPUT_RETRIES=2`, matching the adapter's existing bounded policy. Retry is permitted only for the recognized provider-side best-effort structured-generation HTTP 400 family (`failed to validate JSON`, `failed to generate JSON`, or generated JSON not matching the requested schema). The exact same Harness request/schema remains authoritative; no extraction, repair, relabeling, semantic retry, provider-specific truth logic, or fallback answer is introduced. Provider attempts remain separately counted from Harness model calls.

Quota and transient rate-limit handling remain distinct. Daily/credit/balance exhaustion is typed `Quota`; ordinary 429 throttling is typed `RateLimit` and uses the existing bounded rate-limit policy. A structured-generation retry cannot convert quota into success or hide an exhausted provider window.

## Evaluation sequence

The dedicated v2 runner is frozen at `ed0e46e99049841482dd5028aff9bc4292da387e` / `engine-0.6-source-attribution-holdout-v2-runner-freeze`; its exact-head PR workflows were all green before the tag was created.

The fresh v2 corpus was authored only after that runner freeze. It contains 18 cases: 6 positive, 9 safety, and 3 mixed. Six cases are utility-eligible. Freshness is checked against development v1-v6 plus the observed holdout-v1 corpus for case IDs, canonical entities, exact tasks, source identifiers, exact source text, and exact 8-token windows. Current checks report zero overlap.

## Groq pre-freeze capacity check

No live readiness request is used before freeze. The latest tracked same-model canonical usage is evidence-relevance holdout v12: 63,336 tokens, Groq arm completed at `2026-10-04T19:11:00Z`. Source-attribution holdout v1 then consumed 5,312 Groq tokens and completed at `2026-10-05T11:08:22Z`. At v2 pre-freeze review these known uses total 68,648 tokens inside the preceding 24 hours, leaving 131,352 tokens against the repository's documented 200K TPD planning limit before considering natural window refill. This is not treated as proof against unknown organization-level usage; any typed runtime `Quota` result still fails the one-shot arm closed and is never converted to a semantic score.

Holdout seed: `463602`. Intended corpus freeze tag: `engine-0.6-source-attribution-holdout-v2-freeze`. Before that tag can be created, the full surface checksum, validate-only, deterministic/replay tests, Groq retry regression tests, Clippy, fmt, YAML parsing, runner-freeze diff guard, production-semantics diff guard, and exact-head CI must all pass. Only the first push of the corpus freeze tag may start the canonical Mistral/Google/Groq observation.

The v1 canonical is never rerun, rescored, relabelled, or retagged. v2 uses a new corpus, new seed, new runner identity, and new freeze coordinates.
