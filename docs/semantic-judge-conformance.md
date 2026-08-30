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
