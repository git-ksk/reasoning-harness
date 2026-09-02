# Product dogfood: raw model vs Reasoning Harness

NL-5 evaluates the product path on workloads that are separate from frozen research holdouts.

The runner is `reason-product-dogfood`. It sends the same task/context to the same provider/model in three arms:

```text
raw arm:                   task/context -> model -> structured answer
harness baseline arm:      task/context -> shared candidate -> deterministic pre-render Harness state -> shared initial render -> baseline finalization
harness+D3+sufficiency:    same candidate/state/render -> D3/sufficiency gate -> optional bounded resolution -> successor-only rerender if state changes
```

The v4 comparison contract is `shared-candidate-initial-render-v1`. The two Harness arms share the untrusted candidate, deterministic pre-render state, and exact first final-answer render. Only a successor intervention that changes state may trigger a C-only rerender, so renderer sampling is not attributed to the D3/sufficiency gate.

The committed `fixtures/product-dogfood-v1` corpus has two workload classes:

- incident analysis;
- architecture review.

Each class contains a directly groundable case, an intentionally insufficient case, and a case that becomes groundable only after bounded resolution. These fixtures are product dogfood, not research calibration or holdout data.

The report contract is `reason-product-dogfood-v5` and records:

- unsupported grounded assertion count/rate;
- correct abstention and missed insufficiency;
- false abstention on expected-grounded cases;
- mean final-claim coverage;
- bounded-resolution attempts and success rate;
- total tokens and latency for all three arms, including incremental D3/sufficiency overhead;
- explicit answer-safety runtime identity and per-target safety observations for the successor arm.

The v5 report retains the v2 case-level abstention metrics and the v3 target-level measurements unchanged, preserves the v4 shared-render comparison contract, and adds the actual user-visible `exposed_text` for each arm so qualified/unknown outputs can receive the manual comprehension review required by NL-5. The target-level measurements are keyed only to each fixture's harness-owned `input.hypotheses`. A safe partial answer may expose supported non-target facts while still correctly abstaining from the task target. They report grounded-target coverage, missed target insufficiency, false target abstention, and safe-partial retention separately. The first v2 three-arm pilot remains historical evidence and is not rewritten under the new metric.

`user_comprehension` is deliberately reported as `not_automated_manual_review_required`; the project does not fabricate a human-comprehension metric from model output.

Run locally when a provider key is available:

```bash
cargo run -p reasoning-harness-cli --bin reason-product-dogfood -- \
  --provider mistral \
  --model ministral-8b-latest \
  --fixtures fixtures/product-dogfood-v1 \
  --output /tmp/reason-product-dogfood.json
```

The current successor runtime is `d3-sufficiency-answer-gate-v2` with requirement policy `claim-local-answer-sufficiency-requirements-v1`. It scopes the product sufficiency question to the individual typed proposition and preferentially selects evidence already bound to that supported/known proposition. The broader task is context, not an extra requirement that every safe partial fact must answer completely. Historical `d3-sufficiency-answer-gate-v1` / `generic-answer-sufficiency-requirements-v1` remains executable as a rollback. Neither product policy is part of the frozen holdout corpus; NL-5 evaluates this wiring independently.

The manual `product-dogfood` GitHub Actions workflow uses repository secrets and preserves the JSON report as an artifact. The workflow gates on zero exposed unsupported grounded claims from both harness arms and verifies the baseline/successor runtime identities. `sufficient` is a no-op with respect to authority; only `insufficient`/`mixed` may force verification, bounded resolution, or abstention. A live result is evidence for the tested model/workload slice only; it is not a universal model or correctness claim.
