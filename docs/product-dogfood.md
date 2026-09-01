# Product dogfood: raw model vs Reasoning Harness

NL-5 evaluates the product path on workloads that are separate from frozen research holdouts.

The runner is `reason-product-dogfood`. It sends the same task/context to the same provider/model in three arms:

```text
raw arm:                   task/context -> model -> structured answer
harness baseline arm:      task/context -> shared model candidate -> verify -> bounded resolution -> render -> final-claim coverage
harness+D3+sufficiency:    same shared candidate -> same harness -> D3/sufficiency answer gate -> bounded resolution or abstention
```

The committed `fixtures/product-dogfood-v1` corpus has two workload classes:

- incident analysis;
- architecture review.

Each class contains a directly groundable case, an intentionally insufficient case, and a case that becomes groundable only after bounded resolution. These fixtures are product dogfood, not research calibration or holdout data.

The report contract is `reason-product-dogfood-v2` and records:

- unsupported grounded assertion count/rate;
- correct abstention and missed insufficiency;
- false abstention on expected-grounded cases;
- mean final-claim coverage;
- bounded-resolution attempts and success rate;
- total tokens and latency for all three arms, including incremental D3/sufficiency overhead;
- explicit answer-safety runtime identity and per-target safety observations for the successor arm.

`user_comprehension` is deliberately reported as `not_automated_manual_review_required`; the project does not fabricate a human-comprehension metric from model output.

Run locally when a provider key is available:

```bash
cargo run -p reasoning-harness-cli --bin reason-product-dogfood -- \
  --provider mistral \
  --model ministral-8b-latest \
  --fixtures fixtures/product-dogfood-v1 \
  --output /tmp/reason-product-dogfood.json
```

The successor runtime also identifies its harness-owned requirement policy as `generic-answer-sufficiency-requirements-v1`. This is product wiring, not part of the frozen holdout corpus: the holdout validates the classifier conditional on supplied requirements, while NL-5 separately measures whether this fixed product policy is useful without excessive abstention.

The manual `product-dogfood` GitHub Actions workflow uses repository secrets and preserves the JSON report as an artifact. The workflow gates on zero exposed unsupported grounded claims from both harness arms and verifies the baseline/successor runtime identities. `sufficient` is a no-op with respect to authority; only `insufficient`/`mixed` may force verification, bounded resolution, or abstention. A live result is evidence for the tested model/workload slice only; it is not a universal model or correctness claim.
