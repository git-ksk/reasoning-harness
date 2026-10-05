# Engine 0.6 evidence relevance — Google model diagnostic v1

Status: calibration-tuning diagnostic prepared after frozen v4. This is not a canonical calibration, independent holdout, or release gate.

Frozen v4 run `35998574508` left five Google-specific unresolved cases: three 30-second assessment timeouts and two safe-ambiguity utility misses. Before changing Harness semantics again, compare the same frozen v4 contract on exactly those five cases using:

- `gemini-3.5-flash-lite`;
- `gemini-3.1-flash-lite`.

Cases:

- `04_semantic_paraphrase`;
- `16_broad_landing_no_support`;
- `19_relation_mismatch_same_target`;
- `20_prompt_injection_self_declare`;
- `21_unknown_rename`.

The diagnostic uses the unchanged v4 policy budget (2 model calls, 192 output tokens, 30,000 ms elapsed) and the same Harness-owned binding materializer. It is intentionally noncanonical because fixture selection is partial. Results may inform the model panel for a future calibration successor, but may not be treated as independent validation.
