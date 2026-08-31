# Semantic judge format-invariance research

Issue #59 studies whether the `soft-semantic-v3` semantic decision survives bounded changes to the model-facing output representation. This is a calibration-only research surface. It does not replace the runtime semantic-judge contract and it must not consume historical holdouts v1/v2/v3 for tuning.

## R1a intervention contract

The primary R1a comparison changes only `ModelOutputFormat::JsonSchema` while preserving the v3 primary request's task text, system text, request JSON, kind-specific decision guidance, authority boundary, reasoning preference, token budget, fixture, trial index, and seed.

A design re-review found that a decision-only schema is **not** a pure R1 representation change under the unchanged v3 prompt. The v3 prompt explicitly requires a `finding` to preserve the requested `kind` and `target`; a decision-only or scalar-label schema removes the fields needed to satisfy that instruction. Using those schemas in R1a would therefore conflate representation bias with an instruction/schema conflict and with the R2 materialization hypothesis.

R1a consequently uses only information-equivalent representations:

- `v3_full_json`: the exact current v3 primary request and schema;
- `nested_result_object`: the complete v3 output nested under a `result` field;
- `decision_finding_tuple`: the complete decision and optional finding encoded as a two-element JSON tuple;
- `compact_key_object`: compact top-level keys (`d`, `f`) while retaining canonical decision labels, the complete finding payload, and binding semantics.

Decision-only JSON, scalar labels, and any protocol that removes the echoed finding binding are deferred to R2, where harness-owned materialization can be studied explicitly rather than smuggled into R1.

The baseline request is regression-tested for byte-for-byte equality with `build_soft_judge_model_request`. Every R1a variant is regression-tested to preserve all model request fields except `output_format`, and every parsed variant is passed through the same v3 finding/binding validation.

R1a intentionally has no fallback. All variants request JSON Schema output. A provider capability failure or malformed representation is an operational result, not a semantic decision. Fallback and other enforcement-mechanism changes belong to a separately reported R1b diagnostic and must not be pooled into the pure representation flip estimate.

## Provider enforcement fidelity

Requested output format is not automatically equivalent to effective provider enforcement. The study records both coordinates.

- Mistral maps `JsonSchema` to strict provider-side JSON Schema enforcement.
- Google maps `JsonSchema` to its response JSON schema mechanism.
- The current NVIDIA Hosted NIM adapter maps both `JsonObject` and `JsonSchema` to `json_object` and does not transmit the schema. Running R1a there would therefore be a null representation intervention.

For that reason the first R1a runner accepts Mistral and Google only. NVIDIA is not semantically tuned around; it is excluded because the intervention cannot currently be instantiated. NVIDIA can be revisited under R1b or after a provider capability supports a materially transmitted schema.

Cross-provider results are never pooled as matched observations because Mistral and Google use different effective enforcement mechanisms. R1 matching is within one provider/model only.

## Decision extraction boundary

R1 parsers extract only an untrusted `SoftJudgeDecision` after validating the complete representation against the existing v3 finding contract. They do not construct a new `SoftSemanticFinding`, repair malformed semantic output, or resolve ambiguity. The runtime validation contract remains unchanged. Harness-owned materialization is an R2 hypothesis, not an R1 implementation detail.

One complete JSON value plus non-JSON trailing text follows the existing bounded normalization policy. Multiple JSON values, invalid decision labels, missing required finding payloads, mismatched kind/target bindings, or any output requiring semantic interpretation fail closed.

## Matched comparison

Within one provider/model study, cases are matched by `(fixture_id, trial, seed)`; provider and model are fixed study-level coordinates.

`format_flip_rate` is:

```text
changed semantic decisions / matched successful baseline-variant pairs
```

Operationally incomplete pairs are counted separately and excluded from the semantic denominator. The report preserves the full decision-transition table, so changes such as `abstain -> finding` remain visible rather than collapsing into one scalar.

Per representation, the study reports protocol completion, precision, recall, decision coverage, ambiguous abstention, token usage, and latency. Semantic metrics are emitted only for operationally complete trials. R1a reports fallback as disabled/not applicable rather than silently treating the absence of fallback as a zero-rate runtime observation. Provider responses that fail representation parsing still retain returned token usage for the operational report.

## Calibration-only execution

The research binary canonicalizes the requested path and accepts only this checkout's exact `fixtures/semantic-judges` directory. A renamed/copy/symlinked holdout cannot be substituted as tuning data.

```text
cargo run -p reasoning-harness-cli --bin reason-format-study -- \
  fixtures/semantic-judges \
  --provider mistral \
  --model ministral-8b-latest \
  --representation nested-result-object \
  --fixture 07_causal_positive \
  --fixture 08_causal_negative \
  --fixture 09_causal_ambiguous \
  --seed 1000 \
  --trials 1
```

The dedicated `semantic-format-study` GitHub Actions workflow defaults to that three-fixture causal positive/negative/ambiguous triad. Because the v3 baseline is implicit, the default validation performs six provider calls rather than starting with a full cross-provider matrix. `all-calibration` and repeated trials are explicit later-stage choices.

## Contamination and authority invariants

- holdout-v1/v2/v3 are immutable historical diagnostic evidence, not tuning data;
- holdout-v4 remains blocked until a provider-neutral R1-R3 candidate passes its predeclared calibration gates;
- model-specific semantic prompt/schema branches are prohibited;
- representation disagreement is a risk signal, never a truth vote;
- model outputs remain untrusted/advisory only;
- no model output can create trusted evidence, a hard finding, a verification receipt, epistemic promotion, or verdict authority;
- operational failure is never converted to `no_finding`;
- incomplete trials never enter semantic denominators;
- hidden chain of thought is neither persisted nor evaluated.

## Research anchors

The R1 design is motivated by evidence that model performance can change under format restrictions and even across superficially equivalent structured representations:

- Tam et al., *Let Me Speak Freely?* (EMNLP Industry 2024);
- Long et al., *LLMs Are Biased Towards Output Formats!* (NAACL 2025);
- Schall and de Melo, *The Hidden Cost of Structure* (RANLP 2025);
- Yuan et al., *Quantifying the Impact of Structured Output Format on Large Language Models through Causal Inference* (Findings of EACL 2026);
- Hamilton and Mimno, *Lost in Space: Finding the Right Tokens for Structured Output* (GEM 2026).

These works motivate measurement; they do not override the harness authority boundary or justify provider-specific tuning.
