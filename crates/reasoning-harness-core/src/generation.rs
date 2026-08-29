use crate::{HarnessInput, ModelOutputFormat, ModelRequest, reasoning_candidate_schema};

pub fn build_candidate_request(
    input: &HarnessInput,
    max_tokens: Option<u32>,
    random_seed: Option<u64>,
) -> Result<ModelRequest, serde_json::Error> {
    let evidence = serde_json::to_string_pretty(&input.evidence)?;
    Ok(ModelRequest {
        system: Some(
            "You are a candidate generator inside a reasoning harness. Return only the requested structured candidate. Epistemic states are proposals, not verdicts. Use only evidence IDs supplied by the harness; do not invent evidence, sources, or observations. If the supplied evidence cannot support a claim, propose unknown or assumed instead of fabricating support."
                .into(),
        ),
        task: format!(
            "Task:\n{}\n\nHarness-owned evidence:\n{}\n\nGenerate candidate claims and inference edges.",
            input.task, evidence
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
    let schema = serde_json::to_string_pretty(&reasoning_candidate_schema())?;
    Ok(ModelRequest {
        system: Some(
            "You are a candidate generator inside a reasoning harness. Return exactly one JSON object and no prose. The object must conform to the supplied JSON Schema. Epistemic states are proposals, not verdicts. Use only evidence IDs supplied by the harness; do not invent evidence, sources, or observations. If the supplied evidence cannot support a claim, propose unknown or assumed instead of fabricating support."
                .into(),
        ),
        task: format!(
            "JSON Schema:\n{}\n\nTask:\n{}\n\nHarness-owned evidence:\n{}\n\nGenerate candidate claims and inference edges as one JSON object conforming to the schema.",
            schema, input.task, evidence
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
                source: "fixture".into(),
            }],
        };
        let request = build_candidate_json_fallback_request(&input, Some(512), Some(7)).unwrap();

        assert_eq!(request.output_format, ModelOutputFormat::JsonObject);
        assert!(request.task.contains("JSON Schema:"));
        assert!(request.task.contains("proposed_state"));
        assert!(request.task.contains("e1"));
        assert_eq!(request.max_tokens, Some(512));
        assert_eq!(request.random_seed, Some(7));
    }
}
