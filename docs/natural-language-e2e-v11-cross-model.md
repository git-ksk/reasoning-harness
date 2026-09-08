# Natural-language E2E v11 cross-model replication

Issue #256 replicates the immutable v0.4.1 `natural-language-e2e-v11` observation across independent model families before any planner/action-selection or downstream-grounding product changes.

## Reference coordinate

- released product: `v0.4.1` / `29a9e4be6273dbffeda324e15517dc64930ad315`
- v11 freeze: `natural-language-e2e-v11-freeze` / `a758af17a998493c1005702365b100e05b05f95d`
- canonical reference: Actions `34129798774`, artifact `10021729999`
- canonical reference model: Mistral `ministral-8b-latest`
- seed / max tokens: `57000` / `1024`
- fixed cases: 13

The canonical Mistral observation is a reference row only and is not rerun. The replication wrapper imports and reuses the frozen v11 evaluator and scoring implementation; the frozen v11 files are not modified.

## Frozen replication targets

- Google `gemma-4-31b-it`
- Google `gemini-3.5-flash-lite`
- Groq `openai/gpt-oss-120b`
- Groq `qwen/qwen3.8-27b`
- Groq `openai/gpt-oss-20b`

Provider-specific transport pacing may differ to respect quotas, but the semantic coordinate remains fixed.

## Reporting contract

Each target reports correctness/operational gates and utility separately. The three no-result follow-up cases are scored as:

1. trigger reachability;
2. conditional Issue #249 mechanism conformance only when the predecessor typed `no_result` trigger is exposed;
3. downstream useful follow-up and grounded-target utility.

A trigger miss is planner/action-selection utility data, not a mechanism failure. A zero trigger denominator is inconclusive. Provider or protocol failure is preserved separately from semantic results.

## Freeze discipline

The first live case launch for each target is canonical for that target. No launched target is silently rerun. Any later semantic change requires a new replication identity. Frozen v9/v10/v11 and the released v0.4.1 product source remain immutable.
