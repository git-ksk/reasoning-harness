# Metamorphic reasoning robustness

Point accuracy does not show whether a reasoning harness is sensitive to representation details that should not change meaning. The metamorphic layer applies deterministic, semantics-preserving transformations to committed fixtures and compares the resulting trusted outcomes.

## Authority boundary

A metamorphic transform is a test operation, not a new verifier. It cannot create trusted evidence, change an oracle conclusion, or promote a model-authored statement. Required CI uses only deterministic transforms over committed fixtures; no provider credential or LLM judge is required.

## Initial transform families

- `evidence_order`: reorders harness-owned evidence without changing its content.
- `inference_order`: reorders independent candidate inference edges.
- `stable_id_remap`: consistently changes evidence, claim, inference, and compatible receipt IDs while preserving all references.
- `irrelevant_evidence`: adds a structured fact under an unrelated proposition key.
- `causal_cause_order`: reorders a multi-cause set in both the candidate relation and harness-owned causal evidence.
- `causal_evidence_order`: reorders causal evidence records, including conflicting support/refutation records.

Free-form paraphrase generation is intentionally excluded. Natural-language equivalence would require a separately calibrated soft semantic layer.

## Semantic versus non-semantic fields

For the initial deterministic layer, proposition `key`/`value`, evidence facts, verification conclusions, causal relation membership/direction, inference connectivity, finding kind/reason/strength, and the final verdict are semantic. Changing them is not a valid metamorphic transform unless a future transform defines an independently proven equivalence rule.

Collection order is non-semantic where the contract describes a set or independent records. Stable evidence/claim/inference/receipt identifiers are referential rather than semantic: they may change only when every internal reference is remapped consistently. An added evidence record is non-semantic only when its proposition key is explicitly unrelated to every proposition under test.

Human-readable task, observation, source, and claim prose are not currently declared non-semantic. Deterministic transforms therefore do not rewrite them.

## Reporting

Each base-case/transform pair reports final-verdict invariance, hard-finding invariance, soft-finding stability, diagnostic-status invariance, raw diagnostic-ID changes, and diagnostic reason changes. Raw IDs may legitimately change under stable-ID remapping; semantic finding signatures deliberately exclude those IDs.

`hard_outcome_invariance_rate` combines final verdict, hard findings, and typed diagnostic statuses. It is reported separately from ordinary benchmark accuracy, and transformed cases never replace the original benchmark denominator.
