# Natural-language E2E v31 — Investigation Utility & Provider Parity acceptance

Issue #263 uses v31 as a fresh held-out successor after immutable v30 and the independently justified product fix #331 / PR #332. Frozen v1-v30 evidence is immutable and must not be rerun, rescored, tuned, or rewritten. Cargo workspace version remains `0.4.1` until release closeout.

## Why v31 exists

v30 (`natural-language-e2e-v30-freeze`, commit `cf0cada8f4cf666f75b8dfb6c012a6ca63fb43a3`, seed `97362`) is an immutable VALID RELEASE FAIL. Mistral paired passed and Groq candidate-only passed. Gemini paired was `INCONCLUSIVE` under the predeclared v13 conservative bounds because released-control false abstentions were only identified as `[6,8]` while candidate was `7`. Gemma paired was a hard `FAIL` because the v0.4.2 candidate had one operational protocol failure in `scope-ratube-index`; correctness/safety violations remained zero.

The saved Gemma trace showed a complete typed `{"action":"stop"}` object followed by trailing non-JSON output in the primary structured response. The planner decoder treated the whole response as invalid and invoked its existing JSON-object fallback, which then returned `{}` and terminalized as protocol failure. This exposed a provider-neutral decoder inconsistency: candidate and final-render paths already accepted one complete typed JSON value followed only by non-JSON suffix text, while investigation plan/action decoding did not.

#331 / PR #332 fixed that product inconsistency before v31 was constructed. The shared typed decoder now accepts exactly one complete type-valid JSON value with clearly non-JSON trailing text, but remains fail-closed for a second JSON value, JSON-like trailing fragments, truncation, missing discriminants, missing IDs, unknown fields, or other typed-data errors. It does not infer or repair semantic fields, add semantic/generic protocol retries, add provider/model-specific correctness branches, or implement #283 deterministic action materialization. The merged candidate coordinate is `6bde9227d56ba237cb305777adf452461a0df864`.

v31 therefore evaluates a separately justified product change on a new held-out corpus. It is **not** a rerun of v30 and does not change the v13 ruler. The corpus seed is fixed at `98473`; case identities, tasks, fact keys, answers, source identities, and fresh markers were fixed before live provider use and mechanically checked for collisions against frozen predecessors.

### Surface revision r2

The first v31 freeze (`natural-language-e2e-v31-freeze` @ `4542eecc8a656eb2cd2f625f6cfbf4e72045ae78`) exposed an evaluation-infrastructure packaging defect before any canonical arm launched. Run `34442213199` passed every frozen/precredential gate, then the paired step exited from `argparse` because the workflow referenced paired-orchestrator v2 flags while the branch still contained the older v1 helper from `main`. No control/candidate attempt marker, report, acceptance report, or orchestration record was created; no model observation was launched. The r1 tag and failed run remain immutable evidence of that infrastructure defect and are not rerun.

Because no live arm was observed, r2 keeps the already predeclared seed `98473` and the exact same v31 corpus. The only r2 correction is evaluation infrastructure: carry forward the provider-neutral `paired-canonical-observation-v2` helper and tests from immutable v30, add an explicit helper CLI-surface regression, use distinct r2 workflow labels/artifact names, and freeze a new tag `natural-language-e2e-v31-freeze-r2`. No product runtime, scoring, metric, prompt/case semantics, provider coordinate, retry policy, or candidate coordinate changes.

## Measurement lock

v31 preserves the v11 target/tool/finalization semantics, the v12 continuation-opportunity semantics, and the v13 operational-observability / conservative-bound semantics unchanged. `config/natural-language-e2e-metric-v13.json` remains the policy source of truth, with bound identity `natural-language-e2e-operational-bounds-v13`. Its historical `first_allowed_successor` remains v30 because v30 was the first prospective surface to use v13.

Each scoring-relevant case metric is classified as `observed`, `censored`, or `not_applicable`. Positive monotone witnesses observed before an operational terminal remain observed. Absence of a later event is not converted into semantic false merely because the run terminated operationally. Censored values are neither deleted nor favorably imputed; the evaluator derives the full admissible control interval.

For higher-is-better metrics, candidate non-regression is proven only when the exact candidate value is at least the control upper bound. For lower-is-better metrics, non-regression is proven only when candidate is at most the control lower bound. Strict improvement uses the corresponding candidate-adverse endpoint. A comparison that cannot be proven is `INCONCLUSIVE`, and `INCONCLUSIVE` never releases.

Candidate operational incompleteness remains a hard `FAIL`. All existing correctness/safety zero fields remain hard gates. Candidate mechanism conformance remains 1.0 whenever v12 continuation eligibility is true. Candidate-only #324 diagnostic sidecars remain `scoring_input=false` and cannot affect observability or bounds.

The v31 metric-lock validator uses immutable v30 as predecessor. It proves the preserved runner scoring functions and pair scrub are AST-equivalent after only v30/v31 identity, fresh seed, and candidate-coordinate normalization, then separately verifies the unchanged v13 policy and acceptance boundary. No post-observation metric or semantic rewrite is permitted.

## Freshness and pairing

The 13-case composition preserves logical family coverage: 10 investigation cases and 3 session cases, including three observational exact-target typed-`no_result` follow-ups and one read-only GitHub MCP nonpromotion lane. Freshness tests check v1-v11 repository roots plus immutable v12-v30 freeze refs. The GitHub MCP case reads `Cargo.toml` at the exact paired coordinate; control/candidate configs differ only by the coordinate ref plus candidate-only `selection_priority` on the three follow-up pairs.

Provider coordinates are fixed before live use: Mistral `ministral-8b-latest`, Google `gemini-3.5-flash-lite`, Google `gemma-4-31b-it`, and Groq `openai/gpt-oss-120b` candidate-only. `planner_max_tokens = 256` and provider `max_tokens = 1024` remain unchanged. Google Gemini and Gemma execute as separate matrix jobs with `max-parallel: 2`, while each paired row preserves strict control-then-candidate canonical order.

## Pre-live gate

Before any provider credential is used, v31 must prove exact control/candidate coordinates, immutable v30 predecessor identity, no `Cargo.toml` / `Cargo.lock` / `crates` runtime diff from the candidate, workspace version `0.4.1`, corpus/surface checksums, v13 metric lock, pair validator, validate-only, no-model/no-network preflight, exact CLI capability probes, full deterministic Python tests, full Rust workspace tests, fmt, clippy `-D warnings`, pinned GitHub MCP contract, workflow-policy tests, and ordinary PR CI.

Only after those gates are green may `natural-language-e2e-v31-freeze-r2` be created. The freeze tag and checksums must exist before provider credentials are exposed.

## Canonical live order

1. Run Mistral paired canonical once on the immutable v31 r2 freeze.
2. Only if the Mistral paired gate is `PASS`, launch the cross-model workflow on the same freeze.
3. Run Gemini paired canonical once and Gemma paired canonical once. Each row executes control first, then candidate exactly once. A nonzero control with preserved canonical evidence is delegated to the v13 acceptance comparator rather than automatically granting or denying release.
4. Run Groq candidate-only canonical once.
5. Preserve raw canonical stdout, reports, orchestration records, and candidate diagnostic sidecars. The paired orchestrator emits non-scoring heartbeat progress to stderr every 45 seconds without altering captured canonical stdout.
6. Evaluate every required model row independently. Cross-model averaging is forbidden.

A `FAIL` or `INCONCLUSIVE` v31 result is immutable evidence. It is not rerun, rescored, or tuned. Another successor requires a separately justified prospective product or evaluation change.
