# Natural-language E2E v30 — Investigation Utility & Provider Parity acceptance

Issue #263 uses v30 as the first fresh held-out successor after the prospective v13 evaluation-design change from #328 / PR #329. Frozen v1-v29 evidence is immutable and must not be rerun, rescored, tuned, or rewritten. Cargo workspace version remains `0.4.1` until release closeout.

## Why v30 exists

v29 (`natural-language-e2e-v29-freeze`, commit `a91e16c017efcc14bdd698afab8c588af7e6cbac`, seed `96231`) is an immutable VALID RELEASE FAIL. The v0.4.2 candidate was operationally and correctness-clean across all required model rows, while the released v0.4.1 Gemini control again terminated with structured-generation protocol failures. The evidence-preservation helper introduced by #323 correctly retained the one canonical candidate observation after control failure, so the remaining blocker was evaluation design: the v12 gate treated an operationally incomplete released control as an invalid paired row even where some semantic evidence had already been observed.

#328 / PR #329 added a provider-neutral operational observability frontier and conservative partial-identification bounds. This is prospective only: it does not reclassify v26, v28, or v29. v30 is the first surface allowed to use metric revision `v13`.

v30 pairs released v0.4.1 control `29a9e4be6273dbffeda324e15517dc64930ad315` with candidate `597f5ac8ffaef5c4e66f23f301bfc05c9b73ae3f`. The corpus uses fresh seed `97362` and fresh case IDs, tasks, fact keys, answers, source identities, and fresh markers selected before any live observation.

## Measurement lock

v30 preserves all v11 target/tool/finalization semantics and all v12 continuation-opportunity semantics. The only approved scoring delta is v13 operational observability and conservative bounds, locked by `config/natural-language-e2e-metric-v13.json` and `natural-language-e2e-operational-bounds-v13`.

Each scoring-relevant case metric is classified as `observed`, `censored`, or `not_applicable`. Positive monotone witnesses observed before an operational terminal remain observed. Absence of a later event is not converted into semantic false merely because the run terminated operationally. Censored values are not deleted and are not favorably imputed; the evaluator derives the full admissible control interval.

For higher-is-better metrics, candidate non-regression is proven only when the exact candidate value is at least the control upper bound. For lower-is-better metrics, non-regression is proven only when the candidate is at most the control lower bound. Strict improvement uses the corresponding adverse endpoint. A comparison that cannot be proven becomes `INCONCLUSIVE`, and `INCONCLUSIVE` never releases.

Candidate operational incompleteness remains a hard `FAIL`. All existing correctness/safety zero fields remain hard gates. Candidate mechanism conformance remains 1.0 whenever v12 continuation eligibility is true. Candidate-only #324 diagnostic sidecars remain `scoring_input=false` and are rejected as scoring input by the v13 bounds module.

The v30 metric-lock validator uses immutable v29 as predecessor. It proves the preserved v11/v12 runner scoring functions and pair scrub are AST-equivalent after only identity/seed/candidate-coordinate normalization, then separately validates the exact v13 policy identity and acceptance boundary. No post-observation semantic rewrite is permitted.

## Freshness and pairing

The 13-case composition preserves the logical family coverage: 10 investigation cases and 3 session cases, including three observational exact-target typed-no-result follow-ups and one read-only GitHub MCP nonpromotion lane. Freshness tests check v1-v11 repository roots plus immutable v12-v29 freeze refs. The GitHub MCP case reads `Cargo.toml` at the exact paired coordinate; control/candidate configs differ only by the coordinate ref plus candidate-only `selection_priority` on the three follow-up pairs.

Provider coordinates are fixed before live use: Mistral `ministral-8b-latest`, Google `gemini-3.5-flash-lite`, Google `gemma-4-31b-it`, and Groq `openai/gpt-oss-120b` candidate-only. `planner_max_tokens = 256` and provider `max_tokens = 1024` remain unchanged. Google Gemini and Gemma execute as separate matrix jobs with `max-parallel: 2`, reflecting their model-specific quota lanes, while each paired row preserves strict control-then-candidate canonical order.

## Pre-live gate

Before any provider credential is used, v30 must prove exact control/candidate coordinates, immutable v29 predecessor identity, no `Cargo.toml` / `Cargo.lock` / `crates` runtime diff from candidate, workspace version `0.4.1`, corpus and surface checksums, v13 metric lock, pair validator, validate-only, no-model/no-network preflight, exact CLI capability probes, full deterministic Python tests, full Rust workspace tests, fmt, clippy `-D warnings`, pinned GitHub MCP contract, workflow-policy tests, and ordinary PR CI.

Only after those deterministic gates are green may `natural-language-e2e-v30-freeze` be created. The freeze tag and checksums must exist before provider credentials are exposed.

## Canonical live order

1. Run Mistral paired canonical once on the immutable v30 freeze.
2. Only if the Mistral paired gate is `PASS`, launch the cross-model workflow on the same freeze.
3. Run Gemini paired canonical once and Gemma paired canonical once. Each row executes control first, then candidate exactly once. A nonzero control with preserved canonical evidence is delegated to the v13 acceptance comparator rather than automatically granting or denying release.
4. Run Groq candidate-only canonical once.
5. Preserve raw canonical stdout, reports, orchestration records, and candidate diagnostic sidecars. The paired orchestrator emits non-scoring heartbeat progress to stderr every 45 seconds without altering captured canonical stdout.
6. Evaluate each required model row independently. Cross-model averaging is forbidden.

A `FAIL` or `INCONCLUSIVE` v30 result is immutable evidence. It is not rerun, rescored, or tuned. A new successor requires a separately justified prospective change.
