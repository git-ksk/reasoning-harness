# Engine 0.6 source-attribution holdout v2 — immutable result

Status: PASS. This canonical result is immutable. Do not rerun, rescore, relabel, or move/recreate the freeze tag.

Freeze:
- candidate commit: `bb8d43616ee60370c099db6dda35f9a7f6d209f4`
- tag: `engine-0.6-source-attribution-holdout-v2-freeze`
- canonical run: `37326666360`
- run attempt: 1
- final gate: success

## Required providers

Mistral / `ministral-8b-2512`:
- completed 18/18
- useful attribution 6/6 (100%)
- avoidable abstention 0/6
- citation coverage 11/11 exposed cases (100%)
- model calls / provider attempts: 10 / 10
- total tokens: 5,049
- provider failures: 0
- all seven hard gates: 0
- PASS

Google / `gemini-3.5-flash-lite`:
- completed 18/18
- useful attribution 6/6 (100%)
- avoidable abstention 0/6
- citation coverage 11/11 exposed cases (100%)
- model calls / provider attempts: 11 / 11
- total tokens: 5,863
- provider failures: 0
- all seven hard gates: 0
- PASS

Groq / `openai/gpt-oss-120b`:
- completed 18/18
- useful attribution 6/6 (100%)
- avoidable abstention 0/6
- citation coverage 11/11 exposed cases (100%)
- model calls / provider attempts: 13 / 13
- total tokens: 10,045
- provider failures: 0
- all seven hard gates: 0
- PASS

## Groq v1 postmortem outcome

The v2 workflow explicitly restored the Groq adapter's existing bounded structured-output retry policy (`REASON_GROQ_STRUCTURED_OUTPUT_RETRIES=2`) because source attribution has no evidence-relevance-style strict-Text fallback. In this canonical v2 run, however, Groq used exactly 13 provider attempts for 13 Harness model calls, so no structured-output retry was actually consumed. Retained operational telemetry shows HTTP 200 on every observed attempt and no quota/rate-limit error.

Therefore the immutable v1 `UnsupportedCapability` event is not reproduced under the independently fresh v2 corpus. The evidence supports treating it as a transient provider structured-generation failure, not as quota exhaustion, a source-attribution semantic defect, or persistent inability of the Groq model to satisfy the contract.

## Acceptance decision

The independently authored, separately frozen holdout passes on all three required providers with zero truth-promotion/source-binding/renderer/strengthening/wrong-target/citation/replay hard-gate violations. Together with the immutable development-v6 PASS, this supplies the Issue #463 calibration + independent holdout acceptance evidence.

This result does not by itself authorize an Engine release or silently mutate Engine 0.5.0. Promotion/merge/release remains a separate reviewed decision under an explicit Engine 0.6 coordinate.
