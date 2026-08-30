use crate::{HarnessInput, ModelOutputFormat, ModelRequest, reasoning_candidate_schema};

pub fn build_candidate_request(
    input: &HarnessInput,
    max_tokens: Option<u32>,
    random_seed: Option<u64>,
) -> Result<ModelRequest, serde_json::Error> {
    let evidence = serde_json::to_string_pretty(&input.evidence)?;
    let hypotheses = serde_json::to_string_pretty(&input.hypotheses)?;
    let assumptions = serde_json::to_string_pretty(&input.assumptions)?;
    Ok(ModelRequest {
        system: Some(
            "You are a candidate generator inside a reasoning harness. Return only the requested structured candidate. Epistemic states are proposals, not verdicts. Use only evidence IDs supplied by the harness; do not invent evidence, sources, or observations. When harness evidence contains structured facts, attach a proposition only for a direct key=value claim that can be checked against those facts. If the supplied evidence cannot support a claim, propose unknown or assumed instead of fabricating support."
                .into(),
        ),
        task: format!(
            "Task:\n{}\n\nHarness-owned hypotheses:\n{}\n\nHarness-owned explicit assumptions:\n{}\n\nHarness-owned evidence:\n{}\n\nGenerate candidate claims and inference edges. Evaluate supplied hypotheses when present; do not alter their key/value pair. Explicit assumptions may be used as premises, but do not treat them as verified facts.",
            input.task, hypotheses, assumptions, evidence
        ),
        output_format: ModelOutputFormat::JsonSchema {
            name: "reasoning_candidate".into(),
            schema: reasoning_candidate_schema(),
        },
        max_tokens,
        random_seed,
    })
}

pub fn build_candidate_json_fallback_request(
    input: &HarnessInput,
    max_tokens: Option<u32>,
    random_seed: Option<u64>,
) -> Result<ModelRequest, serde_json::Error> {
    let evidence = serde_json::to_string_pretty(&input.evidence)?;
    let hypotheses = serde_json::to_string_pretty(&input.hypotheses)?;
    let assumptions = serde_json::to_string_pretty(&input.assumptions)?;
    let schema = serde_json::to_string_pretty(&reasoning_candidate_schema())?;
    Ok(ModelRequest {
        system: Some(
            "You are a candidate generator inside a reasoning harness. Return exactly one JSON object and no prose. The object must conform to the supplied JSON Schema. Epistemic states are proposals, not verdicts. Use only evidence IDs supplied by the harness; do not invent evidence, sources, or observations. When harness evidence contains structured facts, attach a proposition only for a direct key=value claim that can be checked against those facts. If the supplied evidence cannot support a claim, propose unknown or assumed instead of fabricating support."
                .into(),
        ),
        task: format!(
            "JSON Schema:\n{}\n\nTask:\n{}\n\nHarness-owned hypotheses:\n{}\n\nHarness-owned explicit assumptions:\n{}\n\nHarness-owned evidence:\n{}\n\nGenerate candidate claims and inference edges as one JSON object conforming to the schema. Evaluate supplied hypotheses when present; do not alter their key/value pair. Explicit assumptions may be used as premises, but do not treat them as verified facts.",
            schema, input.task, hypotheses, assumptions, evidence
        ),
        output_format: ModelOutputFormat::JsonObject,
        max_tokens,
        random_seed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Evidence;

    #[test]
    fn json_fallback_embeds_schema_and_uses_json_object_mode() {
        let input = HarnessInput {
            task: "decide".into(),
            evidence: vec![Evidence {
                id: "e1".into(),
                observation: "observed".into(),
                facts: Default::default(),
                source: "fixture".into(),
            }],
            hypotheses: vec![],
            assumptions: vec![crate::Proposition {
                key: "planning.mode".into(),
                value: "dry_run".into(),
            }],
        };
        let request = build_candidate_json_fallback_request(&input, Some(512), Some(7)).unwrap();

        assert_eq!(request.output_format, ModelOutputFormat::JsonObject);
        assert!(request.task.contains("JSON Schema:"));
        assert!(request.task.contains("proposed_state"));
        assert!(request.task.contains("e1"));
        assert!(request.task.contains("Harness-owned explicit assumptions:"));
        assert!(request.task.contains("planning.mode"));
        assert!(request.task.contains("do not treat them as verified facts"));
        assert_eq!(request.max_tokens, Some(512));
        assert_eq!(request.random_seed, Some(7));
    }
}
