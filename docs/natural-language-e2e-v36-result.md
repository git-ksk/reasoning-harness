# Natural-language E2E v36 — canonical v0.4.2 release acceptance

Issue #263 froze v36 as the final fresh metric-v13 successor for the v0.4.2 release gate. The evaluation surface was intentionally kept out of product `main`; the immutable freeze tag is the evidence coordinate.

## Frozen identity

- freeze tag: `natural-language-e2e-v36-freeze`
- freeze commit: `57bea659d472a103cc48d86ddee7dfe4a41de790`
- candidate product commit: `9497b563ad914fada13d33e0c1a7fee549a1f1de`
- released v0.4.1 control: `29a9e4be6273dbffeda324e15517dc64930ad315`
- seed: `738214`
- metric revision: `v13`
- operational bounds: `natural-language-e2e-operational-bounds-v13`
- Cargo workspace version during acceptance: `0.4.1`
- canonical reruns after observation: `0`
- post-freeze mutation: `0`

Before credentials were exposed, pair validation, metric lock, fresh-collision checks, checksums, Google pacing validation, runner-level no-model integration validation, exact provider capability probes, full Python tests, `cargo fmt`, `cargo clippy`, `cargo test`, and normal PR CI were green.

## Canonical results

### Mistral — PASS

Paired canonical Actions run `34564120392`, `mistral/ministral-8b-latest`.

Both released control and candidate completed 13/13 cases with operational failures `0`, generation failures `0`, and correctness-boundary violations `0`. The paired gate passed. Both rows recorded target recall `0.6`, tool-selection success `1.0`, false abstentions `7`, avoidable follow-up stalls `0`, trigger exposure `3`, and mechanism conformance `1.0` on the eligible follow-up cases. The released control was already at the structural follow-up utility ceiling for this frozen Mistral slice, so exact preservation is release-valid under the predeclared gate.

### Groq — PASS

Cross-model Actions run `34564672351`, candidate-only generic provider-parity row `groq/openai/gpt-oss-120b`.

The candidate completed 13/13 with operational failures `0`, generation failures `0`, correctness-boundary violations `0`, target recall `1.0`, tool-selection success `1.0`, mechanism conformance `1.0`, trigger exposure `3`, and avoidable follow-up stalls `0`. This closes the released-v0.4.1 generic-provider gap without adding provider-specific correctness or authority behavior.

### Gemini 3.5 Flash-Lite — PASS

Cross-model Actions run `34564672351`, paired `google/gemini-3.5-flash-lite`.

Both rows completed 13/13 with operational failures `0`, generation failures `0`, and correctness-boundary violations `0`; the paired gate passed.

- control: target recall `1.0`, tool selection `0.6`, false abstentions `7`, trigger exposure `0`, avoidable follow-up stalls `3`;
- candidate: target recall `1.0`, tool selection `1.0`, false abstentions `7`, trigger exposure `3`, avoidable follow-up stalls `0`, mechanism conformance `1.0`.

This is the strict utility-improvement row for the final release gate. The v35 released-control structured-planner JSON EOF failures did not recur, and the v34 free-tier 429 quota failure did not recur under the frozen 6000ms Google request-start floor.

### Gemma 4 31B — PASS

Cross-model Actions run `34564672351`, paired `google/gemma-4-31b-it` with `investigation_workers=2` behind one shared Google pacer.

Both rows completed 13/13 with operational failures `0`, generation failures `0`, and correctness-boundary violations `0`; the paired gate passed. Both recorded target recall `0.8`, tool selection `0.9`, false abstentions `7`, avoidable follow-up stalls `0`, trigger exposure `3`, and mechanism conformance `1.0`.

The v35 evaluation-infrastructure failure did not recur: shared Google request pacing remained `6000ms`, inter-case delay remained independently `3000ms`, and the two-worker Gemma canonical path reached and completed live model execution rather than exiting on the obsolete `pacing == inter_case_delay` invariant.

## Release disposition

**v36 overall v0.4.2 release gate: PASS.**

No cross-model averaging was used. Every required row passed independently, candidate operational failure remained a hard gate, and `INCONCLUSIVE` remained non-releasable. The final evidence therefore supports releasing v0.4.2 while preserving the v0.4.x correctness, authority, admission, verification, finalization, answer-safety, MCP non-promotion, and session-replay boundaries.

The broader finalization/grounding bridge (#248), repeated-trial planner reliability (#282), and deterministic Harness-owned action materialization (#283) remain v0.5.0 work and were not pulled into this patch release.
