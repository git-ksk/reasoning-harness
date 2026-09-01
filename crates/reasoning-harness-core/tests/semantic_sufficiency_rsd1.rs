use std::{fs, path::Path, pin::Pin};

use reasoning_harness_core::{
    EvidenceSufficiencyCalibrationFixture, EvidenceSufficiencyFallbackReason,
    EvidenceSufficiencyLabel, ModelAdapter, ModelError, ModelErrorKind, ModelOutputFormat,
    ModelRequest, ModelResponse, ModelUsage, build_evidence_sufficiency_json_fallback_request,
    build_evidence_sufficiency_model_request, evidence_sufficiency_output_schema,
    parse_evidence_sufficiency_output, run_model_backed_evidence_sufficiency,
};

fn fixture() -> EvidenceSufficiencyCalibrationFixture {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/evidence-sufficiency-rsd0/02_incident-insufficient.json");
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

#[test]
fn output_schema_exposes_only_the_three_way_decision() {
    let schema = evidence_sufficiency_output_schema();
    assert_eq!(schema["additionalProperties"], false);
    let properties = schema["properties"].as_object().unwrap();
    assert_eq!(properties.len(), 1);
    assert!(properties.contains_key("decision"));
    for forbidden in [
        "verification_receipt",
        "evidence_ids",
        "confidence",
        "verdict",
        "proposition",
        "authority",
    ] {
        assert!(!properties.contains_key(forbidden), "{forbidden}");
    }
}

#[test]
fn primary_and_fallback_requests_preserve_the_same_sufficiency_semantics() {
    let fixture = fixture();
    let primary =
        build_evidence_sufficiency_model_request(&fixture.request, &fixture.artifact, 128, Some(7))
            .unwrap();
    let fallback = build_evidence_sufficiency_json_fallback_request(
        &fixture.request,
        &fixture.artifact,
        128,
        Some(7),
    )
    .unwrap();
    assert!(matches!(
        primary.output_format,
        ModelOutputFormat::JsonSchema { .. }
    ));
    assert_eq!(fallback.output_format, ModelOutputFormat::JsonObject);
    for request in [&primary, &fallback] {
        assert!(request.task.contains("sufficient"));
        assert!(request.task.contains("insufficient"));
        assert!(request.task.contains("mixed"));
        assert!(
            request
                .task
                .contains("Do not judge whether the target is ultimately true")
        );
        assert!(request.task.contains("Database latency was elevated"));
        assert!(!request.task.contains(&fixture.rationale));
        assert_eq!(
            request.reasoning_preference,
            Some(reasoning_harness_core::ModelReasoningPreference::Minimize)
        );
    }
}

#[test]
fn parser_rejects_authority_fields_and_multiple_json_values() {
    assert_eq!(
        parse_evidence_sufficiency_output(r#"{"decision":"mixed"}"#)
            .unwrap()
            .decision,
        EvidenceSufficiencyLabel::Mixed
    );
    assert!(
        parse_evidence_sufficiency_output(r#"{"decision":"sufficient","confidence":0.99}"#)
            .is_err()
    );
    assert!(
        parse_evidence_sufficiency_output(
            r#"{"decision":"sufficient"}{"decision":"insufficient"}"#
        )
        .is_err()
    );
}

struct SchemaAdapter;
impl ModelAdapter for SchemaAdapter {
    fn generate<'a>(
        &'a self,
        request: ModelRequest,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<ModelResponse, ModelError>> + Send + 'a>>
    {
        Box::pin(async move {
            assert!(matches!(
                request.output_format,
                ModelOutputFormat::JsonSchema { .. }
            ));
            Ok(ModelResponse {
                text: r#"{"decision":"insufficient"}"#.into(),
                model: "schema-model".into(),
                usage: ModelUsage {
                    input_tokens: Some(10),
                    output_tokens: Some(3),
                    total_tokens: Some(13),
                },
                finish_reason: Some("stop".into()),
            })
        })
    }
}

#[tokio::test]
async fn successful_schema_output_is_observed_without_authority_promotion() {
    let fixture = fixture();
    let observation = run_model_backed_evidence_sufficiency(
        &SchemaAdapter,
        &fixture.request,
        &fixture.artifact,
        128,
        Some(7),
    )
    .await
    .unwrap();
    assert_eq!(observation.decision, EvidenceSufficiencyLabel::Insufficient);
    assert_eq!(observation.provider_attempts, 1);
    assert_eq!(
        observation.fallback_reason,
        EvidenceSufficiencyFallbackReason::NotNeeded
    );
    assert_eq!(observation.model, "schema-model");
}

struct FallbackAdapter;
impl ModelAdapter for FallbackAdapter {
    fn generate<'a>(
        &'a self,
        request: ModelRequest,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<ModelResponse, ModelError>> + Send + 'a>>
    {
        Box::pin(async move {
            match request.output_format {
                ModelOutputFormat::JsonSchema { .. } => Err(ModelError::new(
                    ModelErrorKind::UnsupportedCapability,
                    "schema unsupported",
                )),
                ModelOutputFormat::JsonObject => Ok(ModelResponse {
                    text: r#"{"decision":"mixed"}"#.into(),
                    model: "fallback-model".into(),
                    usage: ModelUsage {
                        input_tokens: Some(12),
                        output_tokens: Some(3),
                        total_tokens: Some(15),
                    },
                    finish_reason: Some("stop".into()),
                }),
                ModelOutputFormat::Text => panic!("unexpected text mode"),
            }
        })
    }
}

#[tokio::test]
async fn unsupported_schema_mode_falls_back_without_changing_the_contract() {
    let fixture = fixture();
    let observation = run_model_backed_evidence_sufficiency(
        &FallbackAdapter,
        &fixture.request,
        &fixture.artifact,
        128,
        Some(7),
    )
    .await
    .unwrap();
    assert_eq!(observation.decision, EvidenceSufficiencyLabel::Mixed);
    assert_eq!(observation.provider_attempts, 1);
    assert_eq!(
        observation.fallback_reason,
        EvidenceSufficiencyFallbackReason::PrimaryJsonSchemaUnsupported
    );
}

struct InvalidAdapter;
impl ModelAdapter for InvalidAdapter {
    fn generate<'a>(
        &'a self,
        _request: ModelRequest,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<ModelResponse, ModelError>> + Send + 'a>>
    {
        Box::pin(async {
            Ok(ModelResponse {
                text: r#"{"decision":"sufficient","verdict":"accept"}"#.into(),
                model: "invalid-model".into(),
                usage: ModelUsage::default(),
                finish_reason: Some("stop".into()),
            })
        })
    }
}

#[tokio::test]
async fn invalid_authority_bearing_output_fails_closed_after_fallback() {
    let fixture = fixture();
    let error = run_model_backed_evidence_sufficiency(
        &InvalidAdapter,
        &fixture.request,
        &fixture.artifact,
        128,
        Some(7),
    )
    .await
    .unwrap_err();
    assert!(matches!(
        error,
        reasoning_harness_core::EvidenceSufficiencyModelError::InvalidOutput { .. }
    ));
}
