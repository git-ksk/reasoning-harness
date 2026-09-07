# Natural-language E2E v10 — v0.4.1 successor measurement

Issue #252 defines `natural-language-e2e-v10` as a fresh successor to frozen v9 for exact shipped v0.4.1. v9 remains immutable historical evidence at `b42b287b57b3e7e6f19f69464639c5c77a1fe707` / `natural-language-e2e-v9-freeze` / Actions `34079947614` / artifact `10003417402` / digest `sha256:261dd6de3c05053bca3967a29b941cecd6fe4430ecba2dde01c282f5810b72a7`.

v10 does not rerun, rescore, tune, or repair v9. The comparison is structural and descriptive; one successor observation is not a causal effect-size estimate.

## Frozen product coordinate

- tag: `v0.4.1`
- commit: `29a9e4be6273dbffeda324e15517dc64930ad315`
- natural output: `reason-natural-output-v4`
- MCP adapter: `mcp_readonly_v3`
- provider/model: Mistral / `ministral-8b-latest`
- base seed: `53000`
- max tokens: `1024`
- inter-case pacing: `1500ms`

Measurement-only files may differ from the release coordinate. `Cargo.toml`, `Cargo.lock`, and `crates/` must remain byte-diff clean from the released product coordinate.

## Measurement question

v9's Umber case observed cache `no_result` with a still-relevant registry capability, planner calls `4`, registry invocation `0`, stop `round_budget`, correctness violations `0`, and operational failures `0`. That was classified as a utility / avoidable-abstention residual rather than a correctness defect.

v0.4.1 Issue #249 added a narrow Harness-owned continuation after typed `no_result`: only the same exact target, only when exactly one untried explicit read-only capability supports that target's `expected_fact_key`, with no target merging, authority promotion, fact inference, or finalization recovery.

v10 measures whether that shipped mechanism is exercised on a fresh surface while ordinary validation, acquisition, admission, verification, finalization, and answer-safety remain intact.

## Dedicated #249 lane

The fresh Cobalt lane is deliberately an interventional mechanism-exercise fixture. Its natural-language task preserves the v9 construction by naming cache first and registry after `no_result`; this isolates the shipped continuation mechanism from a separate question about whether a model would naturally discover that ordering. It must not be interpreted as evidence about naturalistic planner incidence.

Success for this lane requires all of the following together:

- cache is the first selected capability and returns typed `no_result`;
- registry is the only subsequent capability execution and yields `applied_evidence` or `verification_progress` through the ordinary path;
- `harness_no_result_followup_selections == 1`;
- `planner_calls == 1`, used only as a mechanism invariant that no additional stochastic action-planner call occurred after `no_result`;
- no duplicate action execution;
- correctness-boundary violations and operational failures remain zero.

Telemetry alone is not a success criterion.

## Freshness and contamination discipline

v10 contains eight fresh investigation cases and three fresh session cases. Pre-observation tests mechanically reject reuse of predecessor case IDs, exact task text, target keys, fresh markers, configured source references, and provider base seed across the observed predecessor roots. Frozen historical refs are also checked for fresh-marker reuse. Assertions are not weakened to exempt predecessor identifiers.

The v10 MCP non-promotion lane reads `CHANGELOG.md` at `refs/tags/v0.4.1` through the same digest-pinned official GitHub MCP image while using a fresh v10 case/source/target identity.

## Deterministic pre-observation contracts

Before live credentials are used, v10 requires no-model/no-network fixture and admission preflight plus the relevant v0.4.1 deterministic contracts. The #249 checks include the positive validated continuation and negative boundaries for exact-target sibling isolation, exactly-one remaining explicit capability, non-`no_result` outcomes, keyless/wildcard capabilities, and terminal budgets.

The workflow also verifies frozen v1-v9 surfaces remain untouched and that production source under measurement is unchanged.

## Scoring separation

`hard_correctness_gate_passed` requires correctness-boundary violations `0`.

`measurement_validity_passed` separately requires 11/11 operational completion, operational failures `0`, freshness/scope/authority/identity rejection coverage `4/4`, MCP live coverage `1/1`, the dedicated #249 lane `1/1` with Harness telemetry / planner-call / no-duplicate contracts, valid session persistence/fork behavior with external replay `0`, and token-usage case coverage `>= 0.70`.

Utility metrics remain observational and are not tuned into the gate after observation.

## Freeze and canonical-attempt discipline

Corpus, checksum manifest, evaluator, tests, scoring identity, provider/model, seed, budgets, MCP coordinate, docs, and workflow are frozen before live observation. The live workflow checks out `natural-language-e2e-v10-freeze` directly and requires `HEAD` to equal the freeze-tag commit.

Failures before live-case launch (checkout, identity, deterministic validation, build, pinned-MCP acceptance, or credential presence) may be retried because they have not entered the measurement surface. Once the frozen evaluator launches its first live case with the configured provider, that run is canonical even if it later records an operational failure. The attempt marker, GitHub run identity, logs, and artifacts are preserved. No rerun, rescoring, fixture/evaluator repair, or threshold tuning is allowed for that v10 identity after this boundary; any semantic change requires a successor identity.

## Observation status

No v10 live observation has been performed at freeze preparation time. Results are intentionally absent until the pre-observation freeze is complete.
