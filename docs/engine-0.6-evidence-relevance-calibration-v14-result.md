# Engine 0.6 evidence-target relevance calibration v14 — immutable result

Status: FAIL. This first/only canonical v14 result is immutable. Do not rerun, rescore, relabel, or retag v14.

Freeze:
- commit: 1b29ad7bf752e38070a44d58fbc0300d041bbbc2
- tag: engine-0.6-evidence-relevance-calibration-v14-freeze
- canonical run: 36174639970
- run attempt: 1
- final gate: failure
- fixed core: evidence-relevance-fixed-core-v1 (48 cases)

The fixed calibration core remains closed. Do not append cases in-place because of this result.

## Required Mistral arm

Model: ministral-8b-latest.

Operational:
- completed 48/48
- successful provider cases 48
- provider failures 0
- provider-attempt telemetry complete
- model calls 96
- provider attempts 96
- latency p50/p95/max: 1,294 / 2,811 / 6,717 ms

Semantic / materialization:
- proposal exact 34/48 (70.83%)
- local qualification exact 27/48 (56.25%)
- blocking-reason misses 1
- spurious blocking reasons 2
- materialized exact 35/48 (72.92%)
- wrong-target / false Relevant retention 0
- false relevance rejections 0
- expected Relevant left Ambiguous 11
- utility misses 13

The 13 disposition misses concentrate in four repeated shapes:
1. False explicit_local_absence=present on six clearly positive cases: 01, 05, 07, 65, 69, 74.
2. Positive-target primary under-binding on 04, 08, 11, 12, plus 55 degrading to unresolved target/relation with spurious context_gap.
3. One spurious ownership_scope blocker on 60.
4. One ambiguity-preservation miss on 45, which became Irrelevant.

The dominant semantic residual is now concentrated in two remaining model-authority surfaces: model-authored explicit local absence and positive-evidence target binding.

## Required Groq arm

Model: openai/gpt-oss-120b.

Operational:
- completed 48/48 attempts
- successful provider cases 6
- failed provider cases 42
- all 42 failures: typed quota
- provider-attempt telemetry complete
- no assessment-timeout cascade
- model calls 55
- provider attempts 55

The provider explicitly reported tokens per day (TPD) with a 200,000-token daily limit. Around failure onset, Used was approximately 199,579.

The v14 Groq hardening behaved as intended: daily quota exhaustion failed fast as typed Quota with zero transient rate-limit retries on those calls. The earlier failure mode where long provider waits became 60-second assessment timeouts is gone.

This does not disable retry:
- transient HTTP 429 keeps the bounded provider retry path;
- daily quota exhaustion is intentionally not retried as a short-window limit;
- Groq semantic transport remains strict raw-JSON Text.

The six successful Groq cases were exact, but the required semantic arm is non-scorable because operational completeness failed.

## Google replication

Model: gemini-3.5-flash-lite.

Operational:
- completed 48/48 attempts
- successful provider cases 13
- failed provider cases 35
- all 35 runner failures surfaced as assessment_timeout
- provider-attempt telemetry was incomplete at the runner boundary for interrupted cases
- latency p50/p95/max: 60,001 / 60,002 / 60,002 ms

The first 13 successful cases were semantically exact: proposal, compact guard, and final materialization all matched, with zero safety or utility misses.

Attempt telemetry explains the later failures. Google repeatedly returned HTTP 429 with Retry-After values in the 20–59 second range. The adapter scheduled the existing bounded retries correctly. The outer runner still charged provider wait/retry sleep against the same shared 60-second case deadline, so many cases were cancelled during a valid retry window and surfaced as assessment_timeout.

This is an operational budget-boundary defect, not evidence of semantic regression.

## Cross-provider conclusions

v15 must address three separate issues without growing the fixed core:
1. Remove or deterministically constrain model authority over explicit_local_absence, and improve positive-target binding without weakening the Harness-owned identity floor.
2. Separate provider throttle/retry waiting from semantic execution budget while retaining bounded retries and a finite absolute operational deadline.
3. For required providers with daily quotas, use a full-run capacity preflight rather than a tiny readiness probe that cannot predict whether the full canonical will cross TPD.

No new calibration cases are justified. All observed semantic failures already fit dimensions represented in evidence-relevance-fixed-core-v1.

## Final decision

Required top-level acceptance is FAIL:
- required operational completeness: false
- required correctness gate: false
- required utility gate: false
- required materialization gate: false
- required qualification-safety gate: false

v14 is an immutable canonical FAIL.

Independent holdout authoring remains prohibited.
