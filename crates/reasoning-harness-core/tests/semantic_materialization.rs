use std::{fs, path::Path, pin::Pin};

use reasoning_harness_core::{
    MaterializationError, MaterializedDecisionOutput, ModelAdapter, ModelOutputFormat,
    ModelRequest, ModelResponse, ModelUsage, Proposition, SemanticDiagnosticKind,
    SemanticDiagnosticTarget, SoftJudgeDecision, SoftJudgeRequest,
    build_soft_judge_materialization_request, build_soft_judge_model_request,
    materialize_soft_judge_output, parse_materialized_decision_output,
    run_model_backed_soft_judge_materialization,
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
