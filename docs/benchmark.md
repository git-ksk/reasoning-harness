# Benchmark design

## Goal

Measure the reliability difference caused by the harness itself rather than accidentally measuring a different model, provider, or prompt budget.

The initial comparison therefore uses the **same `ReasoningCandidate` in both arms**:

- **naive baseline**: treat the model-proposed epistemic states as authority and apply no deterministic verification process;
- **harness arm**: combine the same candidate with harness-owned evidence, downgrade unverified strong states, run required passes, and apply the harness acceptance policy.

This isolates process effects. A later supplementary experiment may compare free-form direct answers, but it must be reported separately because output format and prompt differences become confounders.

## Recorded fixtures vs live model runs

Committed fixtures contain synthetic recorded candidates designed to exercise specific failure modes. They are **deterministic regression tests, not empirical evidence of model quality**.

A live model study uses the same fixture inputs but replaces `recorded_candidate` with fresh provider output:

```bash
reason eval fixtures --provider mistral --model ministral-8b-latest --trials 5
```

Use `--seed` when a provider supports it. Trial N uses `base_seed + N`. Live runs are intentionally not part of the required CI gate because network availability, provider behavior, quota, and cost are external variables.

## Initial fixtures

- sufficient direct evidence;
- intentionally missing evidence;
- misleading evidence from a different scope;
- contradictory evidence;
- 5 Whys symptom restatement;
- a supplied counterexample to a universal claim;
- a case where `unknown` is the correct answer.

## Metrics

| Metric | Basis | CI-safe |
| --- | --- | --- |
| unsupported accepted claims | golden fixture labels + typed epistemic state | yes |
| evidence coverage | deterministic structural measurement | yes |
| verdict accuracy / accept / reject / unknown recall | golden fixture verdict | yes |
| hidden assumption exposure | golden fixture labels + typed state | yes |
| contradiction detection | golden fixture labels + typed verdict/state | yes |
| causal edge quality | golden bad-edge labels | yes |
| deterministic verifier failure rate | runtime result | yes |
| token usage | provider usage metadata | live only |
| latency | local wall-clock around provider call | live only |
| provider cost | token usage + explicit caller-supplied price rates | live only |
| model-judge semantic score | soft evidence, not yet implemented | no hard gate |

Provider pricing is not hard-coded into the runtime because prices change independently of harness semantics. A live run can supply explicit rates:

```bash
reason eval fixtures \
  --provider mistral \
  --input-cost-per-million <usd> \
  --output-cost-per-million <usd>
```

The resulting report records token counts, latency, and calculated cost when all required metadata is available.

## Current recorded-fixture baseline

With fixture-owned trusted verification receipts for cases where a deterministic golden oracle is available, the seven recorded fixtures currently produce:

- naive baseline verdict accuracy: 2/7;
- harness verdict accuracy under fixture-oracle coverage: 7/7;
- unsupported accepted claims: 3 → 0;
- accept recall: 1.0 → 1.0;
- reject recall: 0.0 → 1.0;
- unknown recall: 0.25 → 1.0;
- hidden assumption exposure: 0.0 → 1.0;
- contradiction detection: 0.0 → 1.0 where a trusted contradiction receipt exists;
- known bad Five Whys edges retained: 1 → 0.

This **is not 100% generic model reasoning accuracy**. It is a deterministic process regression result under explicit golden-oracle coverage. The support and contradiction fixtures contain trusted receipts that the model cannot see or create. Cases without oracle authority still resolve conservatively to `unknown`. Generic semantic contradiction discovery and counterexample verification remain research gaps.

A first live Mistral run before receipt-backed verification was introduced used seven generations, 6,022 tokens, and roughly 17.2 seconds of provider latency. It confirmed the core safety trade-off: untrusted candidate states could accept an unsupported claim, while the harness eliminated unsupported acceptance but was over-conservative. Future live runs should be compared as stochastic research observations, never as a replacement for the committed deterministic fixture baseline.

## Regression policy

`cargo test --workspace` contains a snapshot-style regression test for the recorded fixture aggregate. Intentional semantic changes must update both the implementation and the expected benchmark baseline. Live provider results never silently rewrite the committed baseline.
