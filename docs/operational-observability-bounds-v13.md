# Operational observability and conservative paired bounds — metric v13

Metric v13 is a **prospective** release-evaluation identity for the first fresh successor after v29. It does not change, rerun, rescore, or reclassify any frozen v26/v28/v29 result.

## Problem

A released control can terminate operationally after some outcomes are already known. Treating every default `false`/`0` after that terminal as semantic failure gives the candidate unearned credit. Dropping the incomplete cases is also invalid because operational missingness need not be random.

v13 therefore separates each metric value into `observed`, `censored`, or `not_applicable`. Positive monotone witnesses remain observed even if a later provider/protocol/process terminal occurs. Negative claims that require future ordinary completion are censored when that completion never happened. `not_applicable` is a semantic state, not missing data.

## Metric frontier

| Metric | Observed before a later terminal | Censored by an operational terminal | N/A |
| --- | --- | --- | --- |
| target recall | expected target membership was positively witnessed | target was not yet witnessed | non-investigation case |
| tool selection | a qualifying relevant capability executed | no qualifying execution was yet witnessed | non-investigation case |
| false abstention | grounding was positively reached, proving no abstention | final semantic grounding decision was preempted | field is structurally absent |
| avoidable follow-up stall | any action executed, proving `action_count != 0` | zero actions at terminal | non-follow-up case |
| trigger exposure | configured cache returned typed `no_result` | trigger absence was not established before terminal | non-follow-up case |
| continuation eligibility | trigger is observed and eligibility is determined at that trigger state | trigger/eligibility prerequisite is censored | trigger definitively unexposed |
| mechanism conformance | eligible continuation is positively conformant, or a concrete immediate wrong follow-up witnesses nonconformance | eligible continuation outcome was preempted | continuation definitively ineligible |
| downstream utility | conformant follow-up outcome is observed | post-follow-up outcome was preempted | mechanism definitively nonconformant/inapplicable |
| correctness violation | a violation is positively witnessed | zero violations before an incomplete control terminal is not treated as proof | — |

Candidate operational failure remains a release hard failure. Candidate correctness violations remain a separate hard failure. These rules do not weaken authority, admission, verification, finalization, identity, session, or MCP-authority boundaries.

## Conservative bounds

For fixed-denominator binary/count metrics, observed values are exact and every censored case contributes its full admissible binary range. For conditional rates, v13 enumerates all numerator/denominator assignments consistent with the case-level observability states and semantic prerequisites. No complete-case deletion or point imputation is used.

For exact candidate value `C` and control interval `[L, U]`:

- higher-is-better non-regression passes only when `C >= U`; strict improvement passes only when `C > U`;
- lower-is-better non-regression passes only when `C <= L`; strict improvement passes only when `C < L`;
- overlap that cannot prove the required claim is `INCONCLUSIVE`;
- `INCONCLUSIVE` never releases.

## Historical offline characterization

The following is **diagnostic characterization only**. Frozen canonical outcomes remain unchanged.

| Historical control | Operational failures | Target recall | Tool selection | False abstentions | Avoidable stalls | Trigger count |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| v26 Gemini | 1 | `[1.0, 1.0]` | `[0.6, 0.7]` | `[5, 6]` | `[2, 3]` | `[0, 1]` |
| v28 Gemini | 2 | `[1.0, 1.0]` | `[0.6, 0.8]` | `[4, 6]` | `[2, 3]` | `[0, 1]` |
| v28 Gemma | 1 | `[0.8, 0.8]` | `[0.8, 0.9]` | `[4, 5]` | `[0, 1]` | `[2, 3]` |
| v29 Gemini | 2 | `[1.0, 1.0]` | `[0.6, 0.8]` | `[5, 7]` | `[2, 3]` | `[0, 1]` |

This shows the intended frontier: the recurring Gemini terminal cases had already recalled their targets, so recall remains exact; later negative outcomes become bounded uncertainty rather than semantic failures.

As a sanity check only, applying v13-style bounds to the immutable v29 Gemini pair would **not** turn v29 into a pass. Target recall, tool selection, avoidable stalls, and trigger exposure can be conservatively proven non-worse/improved, but false abstentions are candidate `7` versus control `[5, 7]`, so that required non-regression is `INCONCLUSIVE`. Frozen v29 remains its original `VALID RELEASE FAIL` under v12.

## v30 integration boundary

Fresh v30 must lock `config/natural-language-e2e-metric-v13.json` before any provider credential is exposed. Operational-terminal reports must preserve the semantic case envelope, including case kind and coverage contract; missing envelope data makes the observability evaluator fail closed. It must use only symmetric canonical report telemetry for scoring; candidate-only structured-generation diagnostics remain `scoring_input=false`. Mistral remains paired and Groq candidate-only unless separately changed. Gemini and Gemma may execute as separate Google jobs with `max-parallel: 2`, while each model row preserves control-to-candidate canonical discipline.

Progress heartbeats may be written to the Actions log without changing raw evidence. Any per-arm timeout must be predeclared symmetrically, adds no retry, is recorded as an operational failure, and should still preserve the other canonical arm where possible.
