# Cross-model semantic judge conformance

This document tracks portability of the advisory semantic judge across model implementations. It does not rank models and model agreement is never treated as truth.

## Authority boundary

All live semantic outputs remain untrusted observations. A model-backed `SoftJudgeOutput` cannot create a verification receipt, hard finding, verdict, trusted evidence, epistemic promotion, or final-answer authority. Provider/protocol failures remain operational failures and never become `no_finding`; incomplete trials remain outside semantic denominators. Hidden chain of thought is neither persisted nor graded.

## `soft-semantic-v3` holdout-v2 matrix

Holdout-v2 is frozen and already observed. These results diagnose v3 portability only; they are not a tuning target for a successor contract.

| Model | Operational / protocol | Precision | Recall | Coverage | Ambiguous abstention | Fallback | Main conformance signal |
| --- | --- | ---: | ---: | ---: | ---: | ---: | --- |
| `ministral-8b-latest` | 140/140, 5/5 complete | 1.000 | 1.000 | 0.700 | 0.933 | 0.371 | strongest observed adherence to the v3 uncertainty boundary |
| `mistral-small-latest` | 140/140, 5/5 complete | 0.982 | 1.000 | 1.000 | 0.000 | 0.071 | systematic over-assertion / abstention collapse |
| `gemini-3.1-flash-lite` | 140/140, 5/5 complete | 1.000 | 1.000 | 0.800 | 0.622 | 0.000 | intermediate conservatism without a protocol issue |
| `ministral-14b-latest` | 135/140; same protocol failure in all five trials | n/a | n/a | n/a | n/a | n/a | repeated non-finding decision with a finding object |
| `nvidia/nemotron-3.5-lightning-30b-a3b` | initial holdout result invalid for semantic interpretation; calibration after reasoning minimization reached 14/18 | n/a | n/a | n/a | n/a | 0.929 on successful calibration calls | finding bias plus typed-output/schema-completeness failures |

Relevant run IDs are `33318380199` (Ministral 8B), `33319598306` (Mistral Small), `33318691626` (Gemini 3.1 Flash-Lite), `33318689080` (Ministral 14B), and `33321109608` (initial Nemotron holdout). Nemotron's initial 0/140 result is not a semantic score: content-free diagnostics showed reasoning-token truncation. After provider-neutral reasoning minimization, run `33340942700` removed the length-truncation confound but left semantic/protocol failures. Bounded-reasoning experiments on `research/51-bounded-reasoning` were worse and are intentionally not merged.

The matrix is evidence about contract portability, not a scalar capability ordering. A change that only improves one row is presumptively model overfit; a harness improvement should generalize across model/provider families.

## `soft-semantic-v4` successor

Issue #53 simplifies the representation while preserving v3 semantic intent.

Global rule:

- `finding`: supplied context affirmatively supports the requested diagnostic concern;
- `no_finding`: supplied context affirmatively resolves or negates the concern;
- `abstain`: neither conclusion is sufficiently supported because binding, scope, applicability, authority, or evidence adequacy remains unresolved, mixed, or partial.

Diagnostic-kind text only defines the requested concern. It does not repeat a separate three-way policy per kind. Causal-gap semantics retain correlation-only, temporal/mechanism-only without direction, explicit confounding, and undistinguished viable reverse direction as affirmative gap evidence; merely imperfect, partial, or scoped evidence does not by itself establish a gap.

The model-facing schema is also stricter without weakening validation. Structured output is a discriminated union:

- `finding` requires a typed finding;
- `no_finding` permits no finding object;
- `abstain` permits no finding object.

The parsed model DTO converts into the existing internal `SoftJudgeOutput`, after which the existing exact kind/target validation still runs. The public soft/hard authority surface is unchanged.

## Frozen v4 compatibility criteria

These criteria were fixed before any v4 provider run on holdout-v3.

A model is `conformant` only when all five holdout-v3 trials complete, protocol conformance is 100%, aggregate precision and recall are at least 0.95, each complete trial has precision and recall at least 0.90, aggregate ambiguous abstention is at least 0.80, each complete trial ambiguous abstention is at least 0.70, and no labelled fixture directly oscillates between `finding` and `no_finding` across complete trials.

A model is `usable_with_limitations` only when all five trials complete, protocol conformance is 100%, aggregate precision and recall are at least 0.90, aggregate ambiguous abstention is at least 0.50, and intentionally ambiguous cases do not collapse family-wide to one assertive decision. External provider availability that prevents complete measurement leaves the semantic tier unassigned rather than manufacturing a semantic failure score.

Fallback dependence is reported but is not a semantic gate because provider JSON-Schema capability differs.

Adopt v4 as a portability improvement only if at least two conformant models come from distinct provider families, at least one additional model is conformant or usable with limitations, no model/provider-specific semantic branch is introduced, and deterministic hard/resolution safety gates remain green.

## Independent holdout-v3 freeze

`fixtures/semantic-judges-holdout-v3/` is the independent observation-free corpus for v4. It was frozen before any v4 provider measurement and contains 28 fixtures: seven per diagnostic kind, with 8 positive, 8 negative, and 12 intentionally ambiguous labels. Source `recorded_observations` remain empty.

Holdout-v1 and holdout-v2 remain historical/diagnostic corpora. Once v4 results are observed on holdout-v3, a material contract or schema change requires a new configuration identity and another independently frozen holdout; holdout-v3 must not be tuned in place.

## `soft-semantic-v4` holdout-v3 result

The independent matrix used merged commit `3774e4f19db9da11cfd2ea065792b78b53b0c9dd`, five sequential trials per model, 256 output tokens, and provider-safe fixture concurrency. The frozen compatibility thresholds were not changed after observation.

| Model | Run | Operational / protocol | Precision | Recall | Coverage | Ambiguous abstention | Fallback | Frozen tier |
| --- | ---: | --- | ---: | ---: | ---: | ---: | ---: | --- |
| `ministral-8b-latest` | `33342332130` | 140/140, 5/5 complete | 0.889 | 1.000 | 0.714 | 0.667 | 0.429 | non-conformant |
| `mistral-small-latest` | `33342547879` | 140/140, 5/5 complete | 1.000 | 1.000 | 1.000 | 0.000 | 0.050 | non-conformant |
| `gemini-3.1-flash-lite` | `33342334655` | 140/140, 5/5 complete | 1.000 | 1.000 | 0.821 | 0.417 | 0.000 | non-conformant |
| `ministral-14b-latest` | `33342335857` | 140/140, 5/5 complete | 0.800 | 1.000 | 0.786 | 0.500 | 0.543 | non-conformant |
| `nvidia/nemotron-3.5-lightning-30b-a3b` | `33342337031` | 71/140 success, 69 protocol failures, 0/5 complete | n/a | n/a | n/a | n/a | 1.000 on successes | non-conformant |

The adoption gate failed decisively: there are zero conformant models and zero usable-with-limitations models. The successor is not adopted.

### What the failed successor established

- Simplifying the three-way semantics weakened uncertainty calibration across provider families rather than reducing only model-specific interpretation noise.
- Mistral Small's v3 abstention collapse persisted unchanged in kind, while 8B/Gemini/14B also became too assertive on intentionally ambiguous unsupported-premise and causal-scope cases.
- The stricter discriminated schema improved a real protocol property for Ministral 14B: the repeated v3 non-finding-plus-finding violation disappeared and all 140 calls became protocol-valid. This is a protocol result, not evidence for the simplified semantic wording.
- Nemotron's truncation confound stayed removed, but the remaining incompatibility was severe: 69 protocol failures and `finding` on every successful call.
- Some Mistral failures were decision-mapping errors even when the structured note semantically described agreement rather than conflict, so the issue is not reducible to model knowledge or task comprehension.

### Runtime decision

Issue #55 restores the exact previously characterized `soft-semantic-v3` model request/schema behavior as the runtime baseline. The v4 commit, holdout-v3 fixtures, run IDs, and this result remain immutable research history. No v4 wording is tuned from observed holdout-v3 cases.

A future successor must separate semantic wording from protocol/schema experiments on the calibration corpus and must freeze a new holdout-v4 before any provider measurement of a materially changed configuration.
