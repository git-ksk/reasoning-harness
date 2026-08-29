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
