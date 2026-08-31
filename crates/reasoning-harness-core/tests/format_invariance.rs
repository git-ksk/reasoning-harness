use std::{fs, path::Path, pin::Pin};

use reasoning_harness_core::{
    FormatJudgeError, MatchedFormatDecision, ModelAdapter, ModelOutputFormat,
    ModelReasoningPreference, ModelRequest, ModelResponse, ModelUsage, Proposition,
    SemanticDiagnosticKind, SemanticDiagnosticTarget, SoftJudgeCalibrationFixture,
    SoftJudgeDecision, SoftJudgeRepresentation, SoftJudgeRequest, build_soft_judge_model_request,
    build_soft_judge_representation_request, compare_soft_judge_formats,
    parse_soft_judge_representation_decision, run_model_backed_soft_judge_representation,
};

fn request() -> SoftJudgeRequest {
    SoftJudgeRequest {
        id: "format-r1".into(),
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

fn finding_payload() -> &'static str {
    r#"{
      "kind":"contradiction",
      "target":{
        "type":"proposition",
        "proposition":{"key":"feature.enabled","value":"true"}
      },
      "note":"context states the opposite value"
    }"#
}

fn valid_v3_finding() -> String {
    format!(
        r#"{{"decision":"finding","finding":{}}}"#,
        finding_payload()
    )
}

#[test]
fn v3_representation_is_byte_for_byte_the_runtime_primary_request() {
    let runtime = build_soft_judge_model_request(&request(), 256, Some(11)).unwrap();
    let research = build_soft_judge_representation_request(
        &request(),
        SoftJudgeRepresentation::V3FullJson,
        256,
        Some(11),
    )
    .unwrap();
    assert_eq!(research, runtime);
}

#[test]
fn representation_variants_change_only_output_format() {
    let baseline = build_soft_judge_representation_request(
        &request(),
        SoftJudgeRepresentation::V3FullJson,
        256,
        Some(11),
    )
    .unwrap();

    for representation in SoftJudgeRepresentation::ALL
        .into_iter()
        .filter(|representation| *representation != SoftJudgeRepresentation::V3FullJson)
    {
        let variant =
            build_soft_judge_representation_request(&request(), representation, 256, Some(11))
                .unwrap();
        assert_eq!(
            variant.task,
            baseline.task,
            "{} task drifted",
            representation.id()
        );
        assert_eq!(
            variant.system,
            baseline.system,
            "{} system prompt drifted",
            representation.id()
        );
        assert_eq!(variant.max_tokens, baseline.max_tokens);
        assert_eq!(variant.random_seed, baseline.random_seed);
        assert_eq!(
            variant.reasoning_preference,
            Some(ModelReasoningPreference::Minimize)
        );
        assert!(matches!(
            variant.output_format,
            ModelOutputFormat::JsonSchema { .. }
        ));
        assert_ne!(variant.output_format, baseline.output_format);
        assert_eq!(representation.requested_output_format(), "json_schema");
    }
}

#[test]
fn isomorphic_representation_parsers_preserve_binding_and_fail_closed() {
    let request = request();
    let finding = finding_payload();
    let samples = [
        (SoftJudgeRepresentation::V3FullJson, valid_v3_finding()),
        (
            SoftJudgeRepresentation::NestedResultObject,
            format!(r#"{{"result":{{"decision":"finding","finding":{finding}}}}}"#),
        ),
        (
            SoftJudgeRepresentation::DecisionFindingTuple,
            format!(r#"["finding",{finding}]"#),
        ),
        (
            SoftJudgeRepresentation::CompactKeyObject,
            format!(r#"{{"d":"finding","f":{finding}}}"#),
        ),
    ];
    for (representation, sample) in samples {
        assert_eq!(
            parse_soft_judge_representation_decision(&request, representation, &sample).unwrap(),
            SoftJudgeDecision::Finding,
            "{} changed the bound decision",
            representation.id()
        );
    }

    for (representation, malformed) in [
        (
            SoftJudgeRepresentation::NestedResultObject,
            r#"{"result":{"decision":"finding"}}"#,
        ),
        (
            SoftJudgeRepresentation::DecisionFindingTuple,
            r#"["no_finding",{"kind":"contradiction","target":{"type":"proposition","proposition":{"key":"feature.enabled","value":"true"}}}]"#,
        ),
        (
            SoftJudgeRepresentation::CompactKeyObject,
            r#"{"d":"finding"}"#,
        ),
        (
            SoftJudgeRepresentation::CompactKeyObject,
            r#"{"d":"X","f":null}"#,
        ),
    ] {
        assert!(
            parse_soft_judge_representation_decision(&request, representation, malformed).is_err(),
            "{} accepted invalid or semantically inconsistent output",
            representation.id()
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
                text: r#"{"result":{"decision":"finding"}}"#.into(),
                model: "provider-model".into(),
                usage: ModelUsage {
                    input_tokens: Some(10),
                    output_tokens: Some(4),
                    total_tokens: Some(14),
                },
                finish_reason: Some("length".into()),
            })
        })
    }
}

#[tokio::test]
async fn protocol_failure_retains_provider_usage_for_operational_measurement() {
    let error = run_model_backed_soft_judge_representation(
        &StaticAdapter,
        &request(),
        SoftJudgeRepresentation::NestedResultObject,
        256,
        Some(11),
    )
    .await
    .unwrap_err();

    // The caller can distinguish truncation from a malformed complete response.
    assert_eq!(error.finish_reason(), Some("length"));
    let FormatJudgeError::InvalidRepresentation { model, usage, .. } = error else {
        panic!("expected representation protocol failure")
    };
    assert_eq!(model, "provider-model");
    assert_eq!(usage.total_tokens, Some(14));
}

#[test]
fn format_flip_rate_uses_only_matched_successful_pairs() {
    let baseline = vec![
        matched("a", 0, 7, Some(SoftJudgeDecision::Abstain)),
        matched("b", 0, 7, Some(SoftJudgeDecision::Finding)),
        matched("c", 0, 7, None),
    ];
    let variant = vec![
        matched("a", 0, 7, Some(SoftJudgeDecision::Finding)),
        matched("b", 0, 7, Some(SoftJudgeDecision::Finding)),
        matched("c", 0, 7, Some(SoftJudgeDecision::NoFinding)),
    ];
    let report = compare_soft_judge_formats(
        SoftJudgeRepresentation::V3FullJson,
        &baseline,
        SoftJudgeRepresentation::NestedResultObject,
        &variant,
    )
    .unwrap();

    assert_eq!(report.matched_keys, 3);
    assert_eq!(report.matched_successful_pairs, 2);
    assert_eq!(report.operationally_incomplete_pairs, 1);
    assert_eq!(report.changed_decisions, 1);
    assert_eq!(report.format_flip_rate, Some(0.5));
    assert!(report.transitions.iter().any(|transition| {
        transition.from == SoftJudgeDecision::Abstain
            && transition.to == SoftJudgeDecision::Finding
            && transition.count == 1
    }));
}

#[test]
fn every_calibration_fixture_builds_every_r1_representation_without_reading_holdouts() {
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
        let fixture: SoftJudgeCalibrationFixture =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        for representation in SoftJudgeRepresentation::ALL {
            let built = build_soft_judge_representation_request(
                &fixture.request,
                representation,
                256,
                Some(17),
            )
            .unwrap();
            assert_eq!(built.random_seed, Some(17));
            assert!(matches!(
                built.output_format,
                ModelOutputFormat::JsonSchema { .. }
            ));
        }
    }
}

fn matched(
    fixture_id: &str,
    trial: usize,
    seed: u64,
    decision: Option<SoftJudgeDecision>,
) -> MatchedFormatDecision {
    MatchedFormatDecision {
        fixture_id: fixture_id.into(),
        trial,
        seed: Some(seed),
        decision,
    }
}
