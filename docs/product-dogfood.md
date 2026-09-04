# Product dogfood: raw model vs Reasoning Harness

NL-5 evaluates the product path on workloads that are separate from frozen research holdouts.

The runner is `reason-product-dogfood`. It sends the same task/context to the same provider/model in three arms:

```text
raw arm:                   task/context -> model -> structured answer
harness baseline arm:      task/context -> shared candidate -> deterministic pre-render Harness state -> shared initial render -> baseline finalization
harness current safety:    same candidate/state/render -> D3/sufficiency gate -> optional bounded resolution -> successor-only rerender if state changes
```

The v4 comparison contract is `shared-candidate-initial-render-v1`. The two Harness arms share the untrusted candidate, deterministic pre-render state, and exact first final-answer render. Only a successor intervention that changes state may trigger a C-only rerender, so renderer sampling is not attributed to the D3/sufficiency gate.

The original `fixtures/product-dogfood-v1` corpus remains the six-case smoke/seed set. Issue #147 adds the frozen 24-case `fixtures/product-dogfood-v2` capability matrix for broader model-fitness evaluation. See [product dogfood capability matrix](product-dogfood-capability-matrix.md) for the 8×3 taxonomy, pre-observation hash freeze, five-run replication plan, and later fresh-holdout protocol.

The committed `fixtures/product-dogfood-v1` corpus has two workload classes:

- incident analysis;
- architecture review.

Each class contains a directly groundable case, an intentionally insufficient case, and a case that becomes groundable only after bounded resolution. These fixtures are product dogfood, not research calibration or holdout data.

The report contract is `reason-product-dogfood-v10` and records:

- unsupported grounded assertion count/rate;
- correct abstention and missed insufficiency;
- false abstention on expected-grounded cases;
- mean final-claim coverage;
- bounded-resolution attempts and success rate;
- total tokens and latency for all three arms, including incremental D3/sufficiency overhead;
- explicit answer-safety runtime identity and per-target safety observations for the successor arm;
- per-target failure provenance for Harness arms, including exact candidate/verification/resolution/render/finalization state, deterministic recovery eligibility, and a classified expected-grounded miss reason;
- actual provider HTTP-attempt counts, including adapter-internal bounded retries and structured-output fallbacks;
- case-level checkpoint/resume execution telemetry: checkpoint enabled/resume requested, reused completed cases, newly executed cases, and preserved operational-failure history.

The v10 report retains the v9 semantic/finalization measurements unchanged and adds only operational reliability telemetry. The v9 behavior retains v8 target-scoped qualified finalization and evaluates the descriptive current answer-safety profile. Under artifact-global `Unknown`, an exact authorized requested target may be preserved as deterministic target-only `QualifiedPartialAnswer` without promoting the global verdict. The #160 successor additionally recovers a renderer-only downgrade when that same exact requested target is already authorized but the renderer marks it `uncertain`; under `Unknown` this still yields target-only `QualifiedPartialAnswer`, while under `Accept` it may become `GroundedAnswer`. The renderer's mode only selects the recovery path and never supplies authority. The #164 successor adds a separate, stricter `Reject` lane: a target-only `QualifiedPartialAnswer` is eligible only for direct evidence-bound trusted `Supported` target receipts and only when all problematic non-target state is typed and structurally isolated (no same-key blocker, target-local contradiction/qualification/hard adversarial finding, shared evidence, or inference/dependency path; contradicted blockers require their own evidence-bound trusted contradiction receipt). The artifact-global `Reject` remains unchanged. `canonical_recovery_cases` and `target_scoped_partial_cases` remain separate so renderer recovery is not confused with artifact-level non-target blockers.

The v5 report retains the v2 case-level abstention metrics and the v3 target-level measurements unchanged, preserves the v4 shared-render comparison contract, and adds the actual user-visible `exposed_text` for each arm so qualified/unknown outputs can receive the manual comprehension review required by NL-5. The target-level measurements are keyed only to each fixture's harness-owned `input.hypotheses`. A safe partial answer may expose supported non-target facts while still correctly abstaining from the task target. They report grounded-target coverage, missed target insufficiency, false target abstention, and safe-partial retention separately. The first v2 three-arm pilot remains historical evidence and is not rewritten under the new metric.

`user_comprehension` is deliberately reported as `not_automated_manual_review_required`; the project does not fabricate a human-comprehension metric from model output.

Run locally when a provider key is available:

```bash
cargo run -p reasoning-harness-cli --bin reason-product-dogfood -- \
  --provider mistral \
  --model ministral-8b-latest \
  --fixtures fixtures/product-dogfood-v1 \
  --checkpoint /tmp/reason-product-dogfood.checkpoint.json \
  --output /tmp/reason-product-dogfood.json
```

For long provider-backed runs, `--checkpoint` atomically persists only fully completed cases. If a run stops during case N, that active case is recorded for audit but is **not** reusable; rerun the exact same command with `--resume` to reuse only the completed prefix and execute case N again from its beginning. Resume fails closed unless the complete evaluation identity matches exactly: fixture path and byte fingerprint, any sibling SHA-256 freeze manifest, provider/model, seed, max-token and bounded-resolution settings, comparison contract, answer-safety identity, semantic successor candidate identity, report schema, and executable fingerprint. Changing any of these inputs refuses observation reuse rather than merging runs.

Checkpoint payloads contain typed report/evaluation observations only. They do not contain provider credentials or hidden model reasoning. Provider/protocol failures that interrupted a case remain in `execution.operational_failures` across a later successful resume; they are never converted into semantic abstentions or erased from the operational record. A run without `--resume` remains the clean replication path.

The current answer-safety runtime is `verified-target-answer-gate-v1`, rolling back directly to `d3-sufficiency-answer-gate-v2`. It keeps the claim-local requirement policy and D3 precondition, but exact targets that already have trusted Supported verification, no target qualification finding, and no hard target adversarial finding preserve the baseline without a redundant model sufficiency call. It scopes the product sufficiency question to the individual typed proposition and preferentially selects evidence already bound to that supported/known proposition. The broader task is context, not an extra requirement that every safe partial fact must answer completely. Historical `d3-sufficiency-answer-gate-v1` / `generic-answer-sufficiency-requirements-v1` remains executable as a rollback. Neither product policy is part of the frozen holdout corpus; NL-5 evaluates this wiring independently.

## Final NL-5 acceptance result

The final v5 acceptance runs use the `shared-candidate-initial-render-v1` comparison contract and the claim-local `d3-sufficiency-answer-gate-v2` successor:

- Ministral 8B: Actions run `33576517724`;
- Gemma 4 31B: Actions run `33576520136`;
- Gemini 3.5 Flash-Lite follow-up: Actions run `33613604519`.

Both Harness arms exposed zero unsupported grounded claims and zero missed task-target insufficiency in both model slices. On Gemma, baseline and successor both reached 1.0 mean grounded-target coverage across expected-grounded cases, zero false target abstentions, 2/2 resolution success, and retained one useful safe-partial unknown case with two supported non-target grounded facts. The successor added about 45.3% tokens and 12.2% latency over the baseline Harness in that run. On Ministral, baseline and successor were behaviorally identical at the target boundary, but both had a 0.75 false-target-abstention rate and 0/2 resolution success; this was retained as a model-specific utility limitation rather than attributed to the successor gate and was tracked in #139; see the current successor revalidation below.

Manual comprehension review used the v5 `exposed_text` field. Gemma's qualified root-cause answer clearly states that the database cause is unconfirmed, preserves the verified HTTP 503 and seven connection-error observations, and does not turn correlation into causation; baseline and successor text are identical. Ministral's raw unknown answers clearly explain missing evidence, while its Harness arms frequently withhold final text entirely. That is safe but less informative and matches the measured false-abstention limitation. These observations are product-slice evidence, not universal model-quality claims.

Gemini 3.5 Flash-Lite was then run on the same v5/shared-render product workload from current `main`. Both Harness arms again exposed zero unsupported grounded claims and zero missed task-target insufficiency, reached 1.0 mean grounded-target coverage on the four expected-grounded cases, had zero false target abstentions, and resolved 2/2 configured-resolution cases. Both expected-unknown cases remained correct target abstentions while retaining safe partial state; the report records two safe-partial unknown cases with four supported non-target grounded claims. Manual review of the exposed text remained understandable and conservative, including the incident root-cause case that preserves HTTP 503 plus seven database connection errors while explicitly leaving database causation unconfirmed. The successor cost was materially higher in this run: about +58.4% tokens and +156.4% latency versus the baseline Harness.

NVIDIA Hosted NIM `nvidia/nemotron-3.5-lightning-30b-a3b` was also attempted under the same product workload in Actions run `33613607389`. It completed the first fixture, entered the second, then failed during structured candidate generation with `invalid structured output after fallback` (`expected value at line 1 column 1`). No aggregate report was produced. This is operational/protocol evidence only, not a semantic score; it is consistent with the previously recorded Nemotron structured-protocol incompatibility and does not justify provider-specific prompt/schema relaxation.

### Expanded product-model matrix

The same six-case v5/shared-render workload was subsequently run across the additional Mistral/Google models already used by the research benchmark. The table below reports the **Harness task-target boundary**, not raw-model quality. `Target coverage` is mean grounded-target coverage over the four expected-grounded cases; `resolution` is success on the two configured-resolution cases. Every completed Harness slice still exposed zero unsupported grounded claims and zero missed target insufficiency.

| Model | Run | Completed | Target coverage | False target abstention | Resolution | Product observation |
| --- | ---: | :---: | ---: | ---: | ---: | --- |
| Gemma 4 31B | `33576520136` | yes | **1.00** | **0.00** | **2/2** | strongest complete Google-hosted Gemma slice |
| Gemini 3.5 Flash-Lite | `33613604519` | yes | **1.00** | **0.00** | **2/2** | strong utility; successor overhead was comparatively high |
| Mistral Small | `33618436419` | yes | 0.75 | 0.25 | 1/2 | materially better coverage than the Ministral 8B/14B slices on this workload |
| Gemini 3.1 Flash-Lite | `33618442500` | yes | 0.75 | 0.25 | 1/2 | safe but less complete than Gemini 3.5 Flash-Lite |
| Ministral 8B | `33576517724` | yes | 0.25 | 0.75 | 0/2 | historical pre-successor limitation; #139 closeout is recorded below |
| Ministral 14B | `33618430680` | yes | 0.25 | 0.75 | 0/2 | larger parameter count did not improve product utility here |
| Ministral 3B | `33618424552` | yes | 0.00 | 1.00 | 0/2 | maximally conservative/withholding on all expected-grounded target cases |
| Gemma 4 26B A4B | `33618449494` | **no** | n/a | n/a | n/a | second fixture failed with invalid structured output after fallback |
| Nemotron 3.5 Lightning 30B A3B | `33613607389` | **no** | n/a | n/a | n/a | second fixture failed with invalid structured output after fallback |

### Current successor revalidation: Ministral 8B (#139 closeout)

The historical table above remains unchanged provenance for the pre-successor runs. After #159/#160/#164 and the #126 operational hardening landed, the same six-case `product-dogfood-v1` workload was rerun on current main `5c5701f77df9dd507c3949294708f8c07a054064` with Ministral 8B, max tokens 1024, and base seed 12000 (Actions run `33822567155`). The raw arm again reached only 0.25 mean grounded-target coverage with false target abstention 0.75. Both Harness arms reached **1.00 target coverage**, **0.00 false target abstention**, unsupported grounded claims **0**, and missed target insufficiency **0**. All four expected-grounded cases exposed the requested target; the two expected-unknown cases remained correct target abstentions. One resolved root-cause case remained artifact-global `Unknown` but safely exposed the exact verified target as a target-scoped `QualifiedPartialAnswer`, preserving the global verdict.

This closes the original #139 grounded-coverage failure without relaxing the adopted semantic/sufficiency boundary. The product report intentionally leaves `exposed_text` absent for unresolved finalization because that field represents finalization text, not presentation fallback. The human `reason` CLI separately renders deterministic evidence-insufficiency guidance for `Unresolved` / `RequiresVerification` states, so a user is not left with a blank terminal while no unsupported factual text is invented.

This is a workload-specific compatibility/utility matrix, not a general model leaderboard. In particular, parameter count does not predict product fitness here: strict structured-output adherence, candidate materialization, final rendering, and bounded-resolution behavior all matter.

The manual `product-dogfood` GitHub Actions workflow uses repository secrets and preserves both the v10 JSON report and its atomic checkpoint as artifacts. The workflow gates on zero exposed unsupported grounded claims from both harness arms and verifies the baseline/successor runtime identities. `sufficient` is a no-op with respect to authority; only `insufficient`/`mixed` may force verification, bounded resolution, or abstention. A live result is evidence for the tested model/workload slice only; it is not a universal model or correctness claim.
