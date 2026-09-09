use crate::{
    FinalAnswerCandidate, HarnessInput, ModelOutputFormat, ModelRequest, ReasoningArtifact,
    Verdict, final_answer_candidate_schema, reasoning_candidate_schema,
};

pub fn build_candidate_request(
    input: &HarnessInput,
    max_tokens: Option<u32>,
    random_seed: Option<u64>,
) -> Result<ModelRequest, serde_json::Error> {
    let evidence = serde_json::to_string_pretty(&input.evidence)?;
    let hypotheses = serde_json::to_string_pretty(&input.hypotheses)?;
    let assumptions = serde_json::to_string_pretty(&input.assumptions)?;
    let evidence_requirements = serde_json::to_string_pretty(&input.evidence_requirements)?;
    let authority_policy = serde_json::to_string_pretty(&input.authority_policy)?;
    Ok(ModelRequest {
        system: Some(
            "You are a candidate generator inside a reasoning harness. Return only the requested structured candidate. Epistemic states are proposals, not verdicts. Use only evidence IDs supplied by the harness; do not invent evidence, sources, or observations. When harness evidence contains structured facts, attach a proposition only for a direct key=value claim that can be checked against those facts. If the supplied evidence cannot support a claim, propose unknown or assumed instead of fabricating support."
                .into(),
        ),
        task: format!(
            "Task:\n{}\n\nHarness-owned hypotheses:\n{}\n\nHarness-owned explicit assumptions:\n{}\n\nHarness-owned evidence requirements:\n{}\n\nHarness-owned authority policy:\n{}\n\nHarness-owned evidence:\n{}\n\nGenerate candidate claims and inference edges. Evaluate supplied hypotheses when present; do not alter their key/value pair. Explicit assumptions may be used as premises, but do not treat them as verified facts. Evidence qualification metadata and authority policy are context only; never claim authority over them.",
            input.task, hypotheses, assumptions, evidence_requirements, authority_policy, evidence
        ),
        output_format: ModelOutputFormat::JsonSchema {
            name: "reasoning_candidate".into(),
            schema: reasoning_candidate_schema(),
        },
        max_tokens,
        random_seed,
        reasoning_preference: None,
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
    let evidence_requirements = serde_json::to_string_pretty(&input.evidence_requirements)?;
    let authority_policy = serde_json::to_string_pretty(&input.authority_policy)?;
    let schema = serde_json::to_string_pretty(&reasoning_candidate_schema())?;
    Ok(ModelRequest {
        system: Some(
            "You are a candidate generator inside a reasoning harness. Return exactly one JSON object and no prose. The object must conform to the supplied JSON Schema. Epistemic states are proposals, not verdicts. Use only evidence IDs supplied by the harness; do not invent evidence, sources, or observations. When harness evidence contains structured facts, attach a proposition only for a direct key=value claim that can be checked against those facts. If the supplied evidence cannot support a claim, propose unknown or assumed instead of fabricating support."
                .into(),
        ),
        task: format!(
            "JSON Schema:\n{}\n\nTask:\n{}\n\nHarness-owned hypotheses:\n{}\n\nHarness-owned explicit assumptions:\n{}\n\nHarness-owned evidence requirements:\n{}\n\nHarness-owned authority policy:\n{}\n\nHarness-owned evidence:\n{}\n\nGenerate candidate claims and inference edges as one JSON object conforming to the schema. Evaluate supplied hypotheses when present; do not alter their key/value pair. Explicit assumptions may be used as premises, but do not treat them as verified facts. Evidence qualification metadata and authority policy are context only; never claim authority over them.",
            schema, input.task, hypotheses, assumptions, evidence_requirements, authority_policy, evidence
        ),
        output_format: ModelOutputFormat::JsonObject,
        max_tokens,
        random_seed,
        reasoning_preference: None,
    })
}

fn final_answer_model_schema() -> serde_json::Value {
    let mut schema = final_answer_candidate_schema();
    let required = schema
        .get_mut("required")
        .and_then(serde_json::Value::as_array_mut)
        .expect("FinalAnswerCandidate schema must declare required fields");
    if !required.iter().any(|field| field == "factual_claims") {
        required.push(serde_json::Value::String("factual_claims".into()));
    }
    schema
}

pub fn build_final_answer_request(
    task: &str,
    artifact: &ReasoningArtifact,
    verdict: Verdict,
    max_tokens: Option<u32>,
    random_seed: Option<u64>,
) -> Result<ModelRequest, serde_json::Error> {
    let artifact = serde_json::to_string_pretty(artifact)?;
    Ok(ModelRequest {
        system: Some(
            "You are the final-answer renderer inside a reasoning harness. Return only the requested structured final-answer candidate. You may summarize verified artifact state, but you do not own truth authority. Every factual proposition you intend to communicate must be listed in factual_claims. Mark a proposition grounded only when the artifact state is known or supported; otherwise mark it uncertain. Do not invent evidence, receipts, facts, or authority. The text field is advisory renderer output and is not the correctness-guaranteed exposed answer; the Harness deterministically constructs that surface from accepted factual_claims. If the artifact cannot support a useful factual answer, say so plainly and return no unsupported grounded claim."
                .into(),
        ),
        task: format!(
            "User task:\n{task}\n\nHarness verdict:\n{verdict:?}\n\nVerified reasoning artifact:\n{artifact}\n\nRender a concise natural-language answer and declare every intended factual proposition in factual_claims. The Harness will expose only its deterministic rendering of accepted factual_claims; renderer text is advisory and cannot add or strengthen exposed assertions."
        ),
        output_format: ModelOutputFormat::JsonSchema {
            name: "final_answer_candidate".into(),
            schema: final_answer_model_schema(),
        },
        max_tokens,
        random_seed,
        reasoning_preference: None,
    })
}

pub fn build_final_answer_json_fallback_request(
    task: &str,
    artifact: &ReasoningArtifact,
    verdict: Verdict,
    max_tokens: Option<u32>,
    random_seed: Option<u64>,
) -> Result<ModelRequest, serde_json::Error> {
    let artifact = serde_json::to_string_pretty(artifact)?;
    let schema = serde_json::to_string_pretty(&final_answer_model_schema())?;
    Ok(ModelRequest {
        system: Some(
            "You are the final-answer renderer inside a reasoning harness. Return exactly one JSON object and no prose. The object must conform to the supplied JSON Schema. You do not own truth authority. Every factual proposition you intend to communicate must be listed in factual_claims. Mark grounded only when the artifact state is known or supported; otherwise mark uncertain. Do not invent evidence, receipts, facts, or authority. The text field is advisory; the Harness deterministically constructs the correctness-guaranteed exposed answer from accepted factual_claims."
                .into(),
        ),
        task: format!(
            "JSON Schema:\n{schema}\n\nUser task:\n{task}\n\nHarness verdict:\n{verdict:?}\n\nVerified reasoning artifact:\n{artifact}\n\nRender one concise final-answer candidate conforming to the schema. Any new factual proposition will be blocked and sent back through verification before it can be exposed as grounded output."
        ),
        output_format: ModelOutputFormat::JsonObject,
        max_tokens,
        random_seed,
        reasoning_preference: None,
    })
}

pub fn parse_final_answer_candidate(text: &str) -> Result<FinalAnswerCandidate, serde_json::Error> {
    serde_json::from_str(text)
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
                metadata: Default::default(),
            }],
            hypotheses: vec![],
            assumptions: vec![crate::Proposition {
                key: "planning.mode".into(),
                value: "dry_run".into(),
            }],
            evidence_requirements: vec![crate::EvidenceRequirement {
                proposition: crate::Proposition {
                    key: "feature.enabled".into(),
                    value: "true".into(),
                },
                as_of_unix_seconds: Some(150),
                scope: None,
                minimum_authority_class: Some("primary".into()),
            }],
            authority_policy: crate::EvidenceAuthorityPolicy {
                ranks: std::collections::BTreeMap::from([("primary".into(), 20)]),
            },
        };
        let request = build_candidate_json_fallback_request(&input, Some(512), Some(7)).unwrap();

        assert_eq!(request.output_format, ModelOutputFormat::JsonObject);
        assert!(request.task.contains("JSON Schema:"));
        assert!(request.task.contains("proposed_state"));
        assert!(request.task.contains("e1"));
        assert!(request.task.contains("Harness-owned explicit assumptions:"));
        assert!(request.task.contains("planning.mode"));
        assert!(
            request
                .task
                .contains("Harness-owned evidence requirements:")
        );
        assert!(request.task.contains("feature.enabled"));
        assert!(request.task.contains("Harness-owned authority policy:"));
        assert!(request.task.contains("primary"));
        assert!(request.task.contains("do not treat them as verified facts"));
        assert_eq!(request.max_tokens, Some(512));
        assert_eq!(request.random_seed, Some(7));
    }

    #[test]
    fn final_answer_request_is_schema_constrained_and_non_authoritative() {
        let artifact = crate::ReasoningArtifact {
            task: "review".into(),
            evidence: vec![],
            hypotheses: vec![],
            assumptions: vec![],
            evidence_requirements: vec![],
            authority_policy: Default::default(),
            candidate_diagnostics: vec![],
            verification_receipts: vec![],
            adversarial_findings: vec![],
            assumption_findings: vec![],
            evidence_qualification_findings: vec![],
            claims: vec![],
            inferences: vec![],
        };
        let request = build_final_answer_request(
            "review this",
            &artifact,
            crate::Verdict::Unknown,
            Some(256),
            Some(9),
        )
        .unwrap();
        let ModelOutputFormat::JsonSchema { schema, .. } = &request.output_format else {
            panic!("final answer request must use JSON Schema");
        };
        let required = schema["required"]
            .as_array()
            .expect("final answer schema must declare required fields");
        assert!(required.iter().any(|field| field == "text"));
        assert!(required.iter().any(|field| field == "factual_claims"));
        assert!(request.task.contains(
            "The Harness will expose only its deterministic rendering of accepted factual_claims"
        ));
        assert!(
            request
                .system
                .unwrap()
                .contains("do not own truth authority")
        );
    }
    #[test]
    fn final_answer_runtime_parser_remains_broad_when_factual_claims_are_omitted() {
        let parsed = parse_final_answer_candidate(r#"{"text":"No supported factual answer."}"#)
            .expect("runtime parser must preserve backward-compatible omission");
        assert_eq!(parsed.text, "No supported factual answer.");
        assert!(parsed.factual_claims.is_empty());

        let explicit = parse_final_answer_candidate(
            r#"{"text":"No supported factual answer.","factual_claims":[]}"#,
        )
        .expect("explicit empty factual_claims must remain valid");
        assert!(explicit.factual_claims.is_empty());
    }
}
