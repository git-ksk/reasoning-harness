use std::{fs, path::Path, pin::Pin};

use reasoning_harness_core::{
    MaterializationError, MaterializationRepresentation, MaterializedDecisionOutput, ModelAdapter,
    ModelOutputFormat, ModelRequest, ModelResponse, ModelUsage, Proposition,
    SemanticDiagnosticKind, SemanticDiagnosticTarget, SoftJudgeDecision, SoftJudgeRequest,
    build_soft_judge_materialization_representation_request,
    build_soft_judge_materialization_request, build_soft_judge_model_request,
    materialize_soft_judge_output, parse_materialized_decision_output,
    parse_materialized_decision_representation_output, run_model_backed_soft_judge_materialization,
};

fn request() -> SoftJudgeRequest {
    SoftJudgeRequest {
        id: "materialization-r2".into(),
        task: "Does context contradict the target?".into(),
        kind: SemanticDiagnosticKind::Contradiction,
        target: SemanticDiagnosticTarget::Proposition {
            proposition: Proposition {
                key: "feature.enabled".into(),
                value: "true".into(),
            },
        },
        context: vec!["The observed feature is disabled.".into()],
    }
}

#[test]
fn materialization_preserves_v3_decision_guidance_and_request_controls() {
    let baseline = build_soft_judge_model_request(&request(), 256, Some(11)).unwrap();
    let materialized = build_soft_judge_materialization_request(&request(), 256, Some(11)).unwrap();

    assert_eq!(materialized.max_tokens, baseline.max_tokens);
    assert_eq!(materialized.random_seed, baseline.random_seed);
    assert_eq!(
        materialized.reasoning_preference,
        baseline.reasoning_preference
    );

    fn decision_guidance(task: &str) -> &str {
        task.split("Decision rule:\n")
            .nth(1)
            .unwrap()
            .split("\n\nUse only the supplied context.")
            .next()
            .unwrap()
    }
    assert_eq!(
        decision_guidance(&materialized.task),
        decision_guidance(&baseline.task)
    );
    assert_ne!(materialized.output_format, baseline.output_format);
}

#[test]
fn materialization_schema_exposes_only_decision_and_optional_note() {
    let request = build_soft_judge_materialization_request(&request(), 256, Some(11)).unwrap();
    let ModelOutputFormat::JsonSchema { name, schema } = request.output_format else {
        panic!("R2 materialization must request JSON schema output")
    };
    assert_eq!(name, "soft_judge_materialized_decision");
    let properties = schema["properties"].as_object().unwrap();
    assert_eq!(properties.len(), 2);
    assert!(properties.contains_key("decision"));
    assert!(properties.contains_key("advisory_note"));
    assert!(!properties.contains_key("finding"));
    assert!(!properties.contains_key("kind"));
    assert!(!properties.contains_key("target"));
    assert!(
        request
            .task
            .contains("harness owns finding kind and target")
    );
    assert!(request.system.unwrap().contains("harness, not the model"));
}

#[test]
fn finding_materialization_copies_request_binding_exactly() {
    let request = request();
    let model_output = MaterializedDecisionOutput {
        decision: SoftJudgeDecision::Finding,
        advisory_note: Some("context states the opposite value".into()),
    };
    let output = materialize_soft_judge_output(&request, &model_output);
    let finding = output
        .finding
        .expect("finding decision must materialize a finding");
    assert_eq!(finding.kind, request.kind);
    assert_eq!(finding.target, request.target);
    assert_eq!(
        finding.note.as_deref(),
        Some("context states the opposite value")
    );
}

#[test]
fn non_finding_decisions_never_materialize_a_finding_even_with_note() {
    for decision in [SoftJudgeDecision::NoFinding, SoftJudgeDecision::Abstain] {
        let output = materialize_soft_judge_output(
            &request(),
            &MaterializedDecisionOutput {
                decision,
                advisory_note: Some("untrusted note remains research telemetry".into()),
            },
        );
        assert_eq!(output.decision, decision);
        assert!(output.finding.is_none());
    }
}

#[test]
fn every_calibration_fixture_builds_r2_request_without_reading_holdouts() {
    let directory = Path::new("../../fixtures/semantic-judges");
    let mut fixtures = fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect::<Vec<_>>();
    fixtures.sort();
    assert!(!fixtures.is_empty());

    for path in fixtures {
        let fixture: reasoning_harness_core::SoftJudgeCalibrationFixture =
            serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
        let built =
            build_soft_judge_materialization_request(&fixture.request, 512, Some(2000)).unwrap();
        assert_eq!(built.random_seed, Some(2000));
        assert!(matches!(
            built.output_format,
            ModelOutputFormat::JsonSchema { .. }
        ));
    }
}

#[test]
fn parser_accepts_only_one_decision_object_and_fails_closed_on_extra_authority_fields() {
    let parsed = parse_materialized_decision_output(
        r#"{"decision":"finding","advisory_note":"bounded explanation"}"#,
    )
    .unwrap();
    assert_eq!(parsed.decision, SoftJudgeDecision::Finding);

    for malformed in [
        r#"{"decision":"finding","finding":{"kind":"contradiction"}}"#,
        r#"{"decision":"finding","kind":"contradiction"}"#,
        r#"{"decision":"finding","target":{"type":"claim","claim_id":"x"}}"#,
        r#"{"decision":"finding","verdict":"accepted"}"#,
        r#"{"decision":"X"}"#,
        r#"{"decision":"finding"}{"decision":"abstain"}"#,
    ] {
        assert!(
            parse_materialized_decision_output(malformed).is_err(),
            "accepted malformed or authority-expanding output: {malformed}"
        );
    }
}

struct StaticAdapter;

impl ModelAdapter for StaticAdapter {
    fn generate<'a>(
        &'a self,
        _request: ModelRequest,
    ) -> Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<ModelResponse, reasoning_harness_core::ModelError>,
                > + Send
                + 'a,
        >,
    > {
        Box::pin(async {
            Ok(ModelResponse {
                text: r#"{"decision":"finding","finding":{}}"#.into(),
                model: "provider-model".into(),
                usage: ModelUsage {
                    input_tokens: Some(10),
                    output_tokens: Some(4),
                    total_tokens: Some(14),
                },
                provider_attempts: 1,
                finish_reason: Some("stop".into()),
            })
        })
    }
}

#[tokio::test]
async fn invalid_model_payload_retains_provider_usage_without_semantic_repair() {
    let error =
        run_model_backed_soft_judge_materialization(&StaticAdapter, &request(), 256, Some(11))
            .await
            .unwrap_err();

    let MaterializationError::InvalidOutput {
        model,
        usage,
        finish_reason,
        ..
    } = error
    else {
        panic!("expected invalid materialization output")
    };
    assert_eq!(model, "provider-model");
    assert_eq!(usage.total_tokens, Some(14));
    assert_eq!(finish_reason.as_deref(), Some("stop"));
}

#[test]
fn r2_representation_variants_change_only_output_format() {
    let baseline = build_soft_judge_materialization_representation_request(
        &request(),
        MaterializationRepresentation::DecisionNoteObject,
        256,
        Some(11),
    )
    .unwrap();
    for representation in MaterializationRepresentation::ALL
        .into_iter()
        .filter(|representation| {
            *representation != MaterializationRepresentation::DecisionNoteObject
        })
    {
        let variant = build_soft_judge_materialization_representation_request(
            &request(),
            representation,
            256,
            Some(11),
        )
        .unwrap();
        assert_eq!(variant.task, baseline.task);
        assert_eq!(variant.system, baseline.system);
        assert_eq!(variant.max_tokens, baseline.max_tokens);
        assert_eq!(variant.random_seed, baseline.random_seed);
        assert_eq!(variant.reasoning_preference, baseline.reasoning_preference);
        assert_ne!(variant.output_format, baseline.output_format);
    }
}

#[test]
fn r2_representation_parsers_preserve_the_same_decision_contract() {
    let samples = [
        (
            MaterializationRepresentation::DecisionNoteObject,
            r#"{"decision":"finding","advisory_note":"soft note"}"#,
        ),
        (
            MaterializationRepresentation::CompactDecisionNoteObject,
            r#"{"d":"finding","n":"soft note"}"#,
        ),
        (
            MaterializationRepresentation::NestedDecisionNoteObject,
            r#"{"result":{"decision":"finding","advisory_note":"soft note"}}"#,
        ),
    ];
    for (representation, text) in samples {
        let parsed =
            parse_materialized_decision_representation_output(representation, text).unwrap();
        assert_eq!(parsed.decision, SoftJudgeDecision::Finding);
        let materialized = materialize_soft_judge_output(&request(), &parsed);
        assert_eq!(materialized.decision, SoftJudgeDecision::Finding);
        let finding = materialized
            .finding
            .expect("finding decision is harness-materialized");
        assert_eq!(finding.kind, request().kind);
        assert_eq!(finding.target, request().target);
    }
}

#[test]
fn compact_representation_preserves_note_and_rejects_model_owned_binding_fields() {
    let parsed = parse_materialized_decision_representation_output(
        MaterializationRepresentation::CompactDecisionNoteObject,
        r#"{"d":"finding","n":"soft note"}"#,
    )
    .unwrap();
    assert_eq!(parsed.advisory_note.as_deref(), Some("soft note"));

    for text in [
        r#"{"d":"finding","kind":"contradiction"}"#,
        r#"{"decision":"finding","advisory_note":"wrong keys"}"#,
    ] {
        assert!(
            parse_materialized_decision_representation_output(
                MaterializationRepresentation::CompactDecisionNoteObject,
                text,
            )
            .is_err()
        );
    }
}

struct PreflightAdapter {
    text: &'static str,
    finish_reason: &'static str,
}

impl ModelAdapter for PreflightAdapter {
    fn generate<'a>(
        &'a self,
        _request: ModelRequest,
    ) -> Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<ModelResponse, reasoning_harness_core::ModelError>,
                > + Send
                + 'a,
        >,
    > {
        let text = self.text.to_string();
        let finish_reason = self.finish_reason.to_string();
        Box::pin(async move {
            Ok(ModelResponse {
                text,
                model: "preflight-model".into(),
                usage: ModelUsage::default(),
                provider_attempts: 1,
                finish_reason: Some(finish_reason),
            })
        })
    }
}

#[tokio::test]
async fn r2_capability_preflight_checks_protocol_not_semantic_correctness() {
    use reasoning_harness_core::run_materialization_capability_preflight;

    for decision in ["finding", "no_finding", "abstain"] {
        let text = format!(r#"{{"decision":"{decision}"}}"#);
        struct OwnedAdapter(String);
        impl ModelAdapter for OwnedAdapter {
            fn generate<'a>(
                &'a self,
                _request: ModelRequest,
            ) -> Pin<
                Box<
                    dyn std::future::Future<
                            Output = Result<ModelResponse, reasoning_harness_core::ModelError>,
                        > + Send
                        + 'a,
                >,
            > {
                let text = self.0.clone();
                Box::pin(async move {
                    Ok(ModelResponse {
                        text,
                        model: "preflight-model".into(),
                        usage: ModelUsage::default(),
                        provider_attempts: 1,
                        finish_reason: Some("stop".into()),
                    })
                })
            }
        }
        let result = run_materialization_capability_preflight(&OwnedAdapter(text), 128, Some(0))
            .await
            .unwrap();
        assert!(result.protocol_compatible);
        assert_eq!(result.materialization_contract, "materialization-r2-v1");
    }
}

#[tokio::test]
async fn r2_capability_preflight_rejects_model_owned_binding_fields() {
    use reasoning_harness_core::{
        MaterializationFailureClass, classify_materialization_failure,
        run_materialization_capability_preflight,
    };

    let adapter = PreflightAdapter {
        text: r#"{"decision":"finding","finding":{"kind":"contradiction"}}"#,
        finish_reason: "stop",
    };
    let error = run_materialization_capability_preflight(&adapter, 128, Some(0))
        .await
        .unwrap_err();
    assert_eq!(
        classify_materialization_failure(&error),
        MaterializationFailureClass::MaterializationProtocol
    );
}

#[test]
fn materialization_failure_classification_is_typed() {
    use reasoning_harness_core::{
        MaterializationFailureClass, ModelError, ModelErrorKind, classify_materialization_failure,
    };

    let quota = MaterializationError::Model(ModelError::new(ModelErrorKind::Quota, "quota"));
    assert_eq!(
        classify_materialization_failure(&quota),
        MaterializationFailureClass::Quota
    );

    let timeout = MaterializationError::Model(ModelError::new(ModelErrorKind::Timeout, "timeout"));
    assert_eq!(
        classify_materialization_failure(&timeout),
        MaterializationFailureClass::Timeout
    );

    let truncated = MaterializationError::InvalidOutput {
        message: "invalid".into(),
        model: "m".into(),
        usage: ModelUsage::default(),
        provider_attempts: 1,
        finish_reason: Some("max_tokens".into()),
    };
    assert_eq!(
        classify_materialization_failure(&truncated),
        MaterializationFailureClass::TruncationProtocol
    );
}
