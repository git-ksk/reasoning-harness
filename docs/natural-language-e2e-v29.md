# Natural-language E2E v29 — Investigation Utility & Provider Parity acceptance

Issue #263 uses v29 as the fresh held-out successor for the v0.4.2 release gate. Frozen v1-v28 evidence is immutable and must not be rerun, rescored, tuned, or rewritten. Cargo workspace version remains `0.4.1` until release closeout.

## Why v29 exists

v28 (`natural-language-e2e-v28-freeze`, commit `ccce3e56b3093746450db05a07ac2dacc473fa1b`, seed `95100`) is a VALID RELEASE FAIL. Mistral paired and Groq candidate-only provided positive evidence, but the required Gemini and Gemma paired rows were incomplete because released v0.4.1 control hit protocol-level structured-generation operational failures. The old workflow stopped after the nonzero control result, so the candidate coordinate was not canonically observed for those Google rows.

Two generic, independently justified changes landed after that immutable result. #323 / PR #325 added `scripts/paired_canonical_observation.py`, which launches control once and candidate once even when control exits nonzero, runs acceptance once when both reports exist, and preserves the original hard-fail result. It adds no semantic retry, whole-run retry, or gate relaxation. #324 / PR #326 added candidate-side structured-generation diagnostics (`structured_mode`, terminal status/finish information, byte count, `parse_class`, and `status_class`) without changing provider requests, fallback counts, retry policy, planner/action budgets, scoring, or failure classification.

v29 pairs released v0.4.1 control `29a9e4be6273dbffeda324e15517dc64930ad315` with candidate `64f6669b872094577a77bcb4137a161aadb66f6a`. The corpus uses fresh seed `96231` and fresh case IDs, tasks, fact keys, answers, source identities, and fresh markers.

## Measurement lock

v29 introduces **no metric revision**. `metric_revision` remains `v12`, with the same scoring semantics as v28. The v29 metric-lock validator compares scoring-relevant AST against immutable v28 and permits normalization only for v28/v29 identity/path tokens, the fresh base seed, and the candidate coordinate. It verifies the locked runner scoring functions, acceptance scoring functions, pair scrub logic, acceptance `ZERO`, and pair `PRESERVED` / `V12` module policies. Any scoring-semantic diff fails the validator.

The v12 semantics remain unchanged: target recall, tool-selection success, false abstentions, follow-up stalls, trigger exposure, continuation eligibility, mechanism conformance, correctness/safety boundaries, and zero-eligible `inconclusive` classification retain their v28 definitions. Candidate mechanism conformance remains 1.0 when a legal continuation opportunity exists. Released-control mechanism conformance remains baseline observation rather than a row-validity gate.

## Freshness and pairing

The 13-case composition remains logically equivalent to v28: 10 investigation cases and 3 session cases, including three observational exact-target no-result follow-ups and one read-only GitHub MCP nonpromotion lane. Freshness tests check v1-v11 repository roots plus immutable v12-v28 freeze refs. The GitHub MCP case reads `Cargo.toml` at the exact paired coordinate, with control/candidate differing only by coordinate ref plus candidate-only `selection_priority` on the three follow-up pairs.

Provider conditions remain frozen: Mistral `ministral-8b-latest`, Google `gemini-3.5-flash-lite`, Google `gemma-4-31b-it`, and Groq `openai/gpt-oss-120b` candidate-only. `planner_max_tokens = 256`, provider `max_tokens = 1024`, and cross-model concurrency policy are unchanged.

## Pre-live gate

Before any provider credential is used, v29 must prove exact control/candidate coordinates, no `Cargo.toml` / `Cargo.lock` / `crates` runtime diff from candidate, workspace version `0.4.1`, corpus checksum and surface checksum, v12 metric-lock no-diff, pair validator, validate-only, no-model/no-network preflight, exact CLI capability probes (control 3/3, candidate 4/4), full Rust tests, provider tests, deterministic Python eval tests, paired orchestration tests, concurrency policy, fmt, clippy `-D warnings`, and normal PR CI.

Only after those gates pass may `natural-language-e2e-v29-freeze` be created.

## Canonical live order

1. Mistral paired canonical once.
2. Only if the Mistral gate passes, run the cross-model workflow.
3. Gemini paired canonical once and Gemma paired canonical once, both through `paired_canonical_observation.py`; control failure does not suppress the one canonical candidate launch.
4. Groq candidate-only canonical once.
5. Preserve reports, orchestration records, and candidate diagnostic sidecars as artifacts.
6. Evaluate the unchanged v29 release gate without cross-model averaging.

A v29 failure is immutable evidence. It is not rerun, rescored, or tuned. A fresh successor is allowed only after an independently justified product change.
