# Engine 0.6 evidence-target relevance calibration v21 — immutable result

Status: FAIL. The first/only frozen v21 canonical is immutable. Do not rerun, rescore, relabel, or retag v21.

Freeze:
- commit: `061aa323a2d5037511546a2eba4117292fa9cf4f`
- tag: `engine-0.6-evidence-relevance-calibration-v21-freeze`
- annotated tag object: `759655b1db1e093552c95dc6badce04a784ee36f`
- canonical run: `36295631123`
- run attempt: `1`
- preflight: success
- final gate: failure
- fixed core: `evidence-relevance-fixed-core-v1` (48 cases)
- annotation protocol: `evidence-relevance-effective-qualification-v21`

No frozen case, label, expected disposition, or v21 semantic rule is changed because of this result.

## Pre-canonical Groq readiness

Synthetic readiness run `36295565985` completed 3/3 tiny transport probes with HTTP 200 immediately before v21 freeze:
- probe 1: TPM remaining 7,840/8,000, RPD remaining 999/1,000
- probe 2: TPM remaining 7,883/8,000, RPD remaining 998/1,000
- probe 3: TPM remaining 7,883/8,000, RPD remaining 997/1,000

This proved only that small requests were presently admissible. It did not establish Tokens Per Day (TPD) headroom for a complete 48-case Groq arm. v21 therefore demonstrates that the old readiness definition is insufficient for canonical admission.

## Required Mistral arm

Model: `ministral-8b-latest`.

Operational:
- completed 48/48
- successful provider cases: 48
- failed provider cases: 0
- provider attempts: 96/96 completed
- model calls: 96
- input/output/total tokens: 81,544 / 2,686 / 84,230
- active execution: 52,855 ms
- provider/retry wait: 0 / 0 ms
- provider-arm latch: none

Semantic/materialization:
- proposal exact: 37/48 (77.08%)
- raw verifier exact: 25/48 (52.08%)
- raw scope-risk misses / spurious: 11 / 0
- raw identity-scope misses: 14
- raw relation-scope misses: 15
- effective qualification: 48/48 exact
- authority-qualified effective qualification: 48/48 exact
- effective risk / authority identity / authority relation misses: 0 / 0 / 0
- materialized exact: 48/48
- wrong-target Relevant retention: 0
- false relevance rejections: 0
- Relevant -> Ambiguous: 0
- utility misses: 0

Mistral is a complete semantic PASS for the frozen v21 surface.

## Required Groq arm

Model: `openai/gpt-oss-120b`.

Operational:
- successful provider cases: 13
- failed/suppressed provider cases: 35
- typed quota occurred at `74_v13_exact_target_same_relation_no_cue`
- provider arm latched immediately after the confirmed quota failure
- remaining 34 cases were suppressed
- provider attempts: 27/27 completed
- model calls: 27
- input/output/total tokens before latch: 28,319 / 3,329 / 31,648
- active execution: 15,420 ms
- provider/pacing wait: 219,931 ms
- retry wait: 0 ms

The new public-safe `provider_diagnostic` preserved the decisive Groq response while redacting account identity:

`Rate limit reached ... on tokens per day (TPD): Limit 200000, Used 199298, Requested 1295. Please try again in 4m16.176s.`

The same observation also preserved:
- `retry-after=257`
- RPD: 974/1,000 requests remaining
- TPM: 8,000/8,000 tokens remaining
- token reset: 1 ms

This proves the terminal 429 was TPD, not TPM or RPD. The 256.176-second provider retry interval also exactly matches the deficit of 593 tokens replenished at `200000 / 86400` tokens per second, consistent with a continuously replenishing daily token bucket rather than a midnight-only reset.

Successful-observation semantics before latch:
- proposal exact: 13/13
- raw local qualification exact: 13/13
- effective qualification: 13/13 exact
- authority-qualified effective qualification: 13/13 exact
- materialized exact: 13/13
- wrong-target Relevant retention: 0
- false relevance rejections: 0
- Relevant -> Ambiguous: 0
- utility misses: 0

No semantic miss was observed on the 13 completed Groq cases. The required arm is nevertheless non-scorable as a full calibration because 35 frozen cases lack provider observations.

## Google replication

Model: `gemini-3.5-flash-lite`. Non-gating.

Operational:
- completed 48/48
- successful provider cases: 48
- failed provider cases: 0
- provider attempts: 96/96 completed
- model calls: 96
- input/output/total tokens: 83,892 / 2,950 / 86,842
- active execution: 129,720 ms
- provider/pacing wait: 237,291 ms
- retry wait: 0 ms

Semantic/materialization:
- proposal exact: 37/48 (77.08%)
- raw verifier exact: 28/48 (58.33%)
- effective qualification exact: 45/48
- authority-qualified effective qualification: 47/48
- effective risk misses / spurious: 0 / 0
- authority identity misses: 0
- authority relation misses: 1
- materialized exact: 48/48
- wrong-target Relevant retention: 0
- false relevance rejections: 0
- Relevant -> Ambiguous: 0
- utility misses: 0

The one zero-risk authority relation mismatch was `76_v13_sibling_different_relation_no_cue`:
- expected: `distinct_target + different_relation + none`
- primary: `target=different, relation=exact`
- raw verifier: `distinct_target + different_relation + none`
- effective: `distinct_target + requested_relation + none`
- final disposition: expected Irrelevant, materialized Irrelevant

The existing target-negative terminal dominated the relation disagreement, so this did not produce a safety or utility miss. The remaining all-axis effective mismatches were blocking `context_gap` relation diagnostics on cases `25` and `80`, both still terminally Ambiguous. v22 does not tune semantics to these non-gating observations by default.

## Cross-provider conclusion

v21 fixes the v20 authority-conflict semantic failure on the evidence observed:
- Mistral: complete 48/48 semantic/materialization PASS
- Groq: 13/13 exact before TPD latch, no semantic miss observed
- Google: complete 48/48 materialization PASS with zero correctness/utility misses

The canonical still fails because the required Groq arm was operationally incomplete. The failure is now directly attributable to organization-level TPD exhaustion, and the preceding three-probe readiness check is proven insufficient because it did not measure full-run TPD headroom.

Historical Groq token demand also shows why a small readiness canary is not enough:
- v20 used 97,259 tokens over 39 successful cases before its later latch
- v21 used 31,648 tokens over 13 successful cases before its latch
- both imply roughly 117K–120K tokens for a complete 48-case Groq arm at the current runner shape

A successor should therefore preserve v21 semantics and change canonical admission, not tune labels or prompts to the quota outcome.

## Final decision

Required top-level acceptance is FAIL:
- required operational completeness: false
- required correctness gate: false at aggregate final-gate level because the required set is incomplete
- required utility gate: false at aggregate final-gate level because the required set is incomplete
- required materialization gate: false at aggregate final-gate level because the required set is incomplete
- required qualification gate: false at aggregate final-gate level because the required set is incomplete

v21 is an immutable canonical FAIL. Independent holdout authoring remains prohibited.
