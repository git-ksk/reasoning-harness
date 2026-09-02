# Product dogfood capability matrix

Issue #147 expands product dogfood from the original six-case smoke slice into a broader, pre-observation model-fitness evaluation. This is product evaluation, not a new semantic-authority experiment: the adopted D3 runtime, sufficiency policy, verifier boundary, and frozen research holdouts remain unchanged.

## Three-stage design

### Stage A — 24-case development/product matrix

`fixtures/product-dogfood-v1` remains the original six-case smoke/seed set. `fixtures/product-dogfood-v2` is the development/product capability matrix used for broader model comparison.

The v2 corpus contains exactly 24 cases: eight capability families with three independently worded/domain-varied cases each.

| Capability family | Cases | What it probes |
| --- | ---: | --- |
| `direct_grounding` | 3 | direct reporting of harness-owned structured facts |
| `insufficient_evidence` | 3 | refusal to promote weak/irrelevant observations into the requested conclusion |
| `bounded_resolution` | 3 | acquisition/admission/re-verification of an explicitly configured resolver fact |
| `safe_partial` | 3 | preservation of useful supported non-target facts while the requested target remains unconfirmed |
| `contradiction` | 3 | consistent-control vs conflicting structured records |
| `causal_boundary` | 3 | explicit verified cause vs association/sequence that does not establish causation |
| `temporal_validity` | 3 | valid-window control, stale evidence, and not-yet-valid evidence |
| `scope_entity_boundary` | 3 | exact scope vs region/tenant mismatch |

The outcome mix is deliberately not forced to 50/50: 7 cases are directly grounded, 3 become grounded only after bounded resolution, and 14 are expected to remain unknown. Capability coverage, not a synthetic class balance, determines the corpus size.

Before provider observation, the corpus payload is frozen by `fixtures/product-dogfood-v2.sha256`. Any changed fixture requires a new corpus identity rather than rewriting the observed v2 payload.

All Stage-A models use the same corpus, base seed, max-token limit, and `shared-candidate-initial-render-v1` comparison contract. Results remain workload-specific compatibility/utility evidence, not a universal model leaderboard.

### Stage B — five-run replication

Only operationally complete/useful Stage-A candidates advance. The frozen v2 corpus is rerun across five predeclared base seeds. Comparison is paired by case and seed; aggregate mean target coverage is not sufficient by itself.

At minimum, replication records:

- provider/protocol completion;
- unsupported exposed grounded claims;
- missed target insufficiency;
- target coverage and false target abstention;
- bounded-resolution success;
- safe-partial retention;
- token and latency overhead;
- per-case disagreement across runs.

A provider/protocol failure stays operational evidence and does not enter a semantic denominator as a fabricated abstention.

### Stage C — fresh holdout

Only after Stage-B model/runtime selection is complete will a fresh 12–16 case holdout be authored. Its fixture payload, model list, seeds, and acceptance gates must be frozen before any provider observation. Development-matrix results cannot be used to rewrite the fresh holdout after observation.

## Evaluator hardening required by v2

Temporal and scope cases exposed an evaluation-side weakness in the earlier dogfood helper: raw support accounting checked only structured key/value equality. v2 changes that helper to reuse the same trusted structured-fact verifier selection used by the runtime, including evidence qualification requirements. Stale, out-of-scope, and conflicting evidence therefore cannot be counted as supported merely because a matching key/value appears somewhere in the input.

This change affects evaluation accounting only. It does not grant the evaluator or model new authority and does not modify the adopted runtime policy.

## Workflow

The manual `product-dogfood` workflow explicitly selects `product-dogfood-v1` or `product-dogfood-v2`. v2 is the default for new capability-matrix observations. Before loading provider credentials, the workflow:

1. parses the selected fixture corpus with `reason-product-dogfood --validate-only`;
2. verifies the v2 SHA-256 manifest;
3. requires exactly 24 cases, eight capability families, and three cases per family.

The six-case v1 corpus remains available for fast smoke/regression use and for interpreting historical NL-5 runs.
