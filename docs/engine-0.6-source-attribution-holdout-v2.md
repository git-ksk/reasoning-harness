# Engine 0.6 source-attribution holdout v2

Status: successor runner preparation only. No v2 holdout corpus exists and no v2 provider observation has occurred.

## Why v2 exists

Independent holdout v1 is an immutable operational FAIL at run `37300354564` attempt 1. Mistral and Google completed 18/18 and passed every acceptance gate. Groq completed 7/18 and then failed on a provider-side best-effort structured-JSON generation error surfaced as typed `UnsupportedCapability`; every observed semantic hard gate remained zero.

The v1 workflow set `REASON_GROQ_STRUCTURED_OUTPUT_RETRIES=0`. That setting came from the evidence-relevance evaluation line, where zero adapter retries intentionally surfaces a structured-generation failure after one provider attempt so a separate Harness-owned strict-Text fallback can execute inside the case budget. The source-attribution runner has no such transport fallback: typed `UnsupportedCapability` is an immediate operational provider failure. Therefore the v1 retry override removed the Groq adapter's existing generic bounded recovery without providing the fallback that justified that override.

This is an evaluation-wiring defect, not a source-attribution semantic defect. The immutable v1 result, corpus, tag, and score remain unchanged.

## Frozen v2 transport contract

The successor keeps production source-attribution semantics, prompts, JSON schemas, parsing, materialization, hard gates, provider models, and thresholds unchanged from the v6 semantics freeze and v1 acceptance contract.

For Groq only, the future v2 canonical workflow will explicitly set `REASON_GROQ_STRUCTURED_OUTPUT_RETRIES=2`, matching the adapter's existing bounded policy. Retry is permitted only for the recognized provider-side best-effort structured-generation HTTP 400 family (`failed to validate JSON`, `failed to generate JSON`, or generated JSON not matching the requested schema). The exact same Harness request/schema remains authoritative; no extraction, repair, relabeling, semantic retry, provider-specific truth logic, or fallback answer is introduced. Provider attempts remain separately counted from Harness model calls.

Quota and transient rate-limit handling remain distinct. Daily/credit/balance exhaustion is typed `Quota`; ordinary 429 throttling is typed `RateLimit` and uses the existing bounded rate-limit policy. A structured-generation retry cannot convert quota into success or hide an exhausted provider window.

## Evaluation sequence

1. Freeze this dedicated v2 runner before any v2 corpus exists.
2. After exact-head CI is green and the runner freeze tag is pushed, author a fresh independent 18-case v2 corpus from scratch.
3. Enforce zero reuse of prior observed holdout/development identities/tasks/source text and exact 8-token windows.
4. Freeze the v2 corpus/workflow only after freshness, checksum, validate-only, offline tests, Clippy, fmt, and exact-head CI pass.
5. Run Mistral `ministral-8b-2512`, Google `gemini-3.5-flash-lite`, and Groq `openai/gpt-oss-120b` exactly once as the canonical v2 acceptance observation.

The v1 canonical is never rerun, rescored, relabelled, or retagged. v2 uses a new corpus, new seed, new runner identity, and new freeze coordinates.
