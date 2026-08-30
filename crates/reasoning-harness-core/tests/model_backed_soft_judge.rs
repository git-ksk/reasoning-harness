use std::{collections::VecDeque, pin::Pin, sync::Mutex};

use reasoning_harness_core::{
    CausalRelation, ModelAdapter, ModelBackedSoftJudgeError, ModelError, ModelErrorKind,
    ModelOutputFormat, ModelRequest, ModelResponse, ModelUsage, Proposition,
    SemanticDiagnosticKind, SemanticDiagnosticTarget, SoftJudgeDecision, SoftJudgeFallbackReason,
    SoftJudgeIdentity, SoftJudgeRequest, build_soft_judge_json_fallback_request,
    build_soft_judge_model_request, parse_soft_judge_output, run_model_backed_soft_judge,
};

struct SequenceAdapter {
    responses: Mutex<VecDeque<Result<ModelResponse, ModelError>>>,
    requests: Mutex<Vec<ModelRequest>>,
}

impl SequenceAdapter {
    fn new(responses: Vec<Result<ModelResponse, ModelError>>) -> Self {
        Self {
            responses: Mutex::new(responses.into()),
            requests: Mutex::new(Vec::new()),
        }
    }

    fn requests(&self) -> Vec<ModelRequest> {
        self.requests.lock().unwrap().clone()
    }
}

impl ModelAdapter for SequenceAdapter {
    fn generate<'a>(
        &'a self,
        request: ModelRequest,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<ModelResponse, ModelError>> + Send + 'a>>
    {
        Box::pin(async move {
            self.requests.lock().unwrap().push(request);
            self.responses
                .lock()
                .unwrap()
                .pop_front()
                .expect("test adapter response")
        })
    }
}

fn request() -> SoftJudgeRequest {
    SoftJudgeRequest {
        id: "soft-1".into(),
        task: "Does the supplied context contradict the target?".into(),
        kind: SemanticDiagnosticKind::Contradiction,
        target: SemanticDiagnosticTarget::Proposition {
            proposition: Proposition {
                key: "feature.enabled".into(),
                value: "true".into(),
            },
        },
        context: vec!["The feature is disabled in the observed configuration.".into()],
    }
}

fn identity() -> SoftJudgeIdentity {
    SoftJudgeIdentity {
        judge_id: "judge-a".into(),
        model_id: "model-a".into(),
        configuration_id: "semantic-v1".into(),
    }
}

fn response(text: &str, input: u64, output: u64) -> Result<ModelResponse, ModelError> {
    Ok(ModelResponse {
        text: text.into(),
        model: "provider-model".into(),
        usage: ModelUsage {
            input_tokens: Some(input),
            output_tokens: Some(output),
            total_tokens: Some(input + output),
        },
        finish_reason: Some("stop".into()),
    })
}

fn valid_finding_json() -> &'static str {
    r#"{
      "decision":"finding",
      "finding":{
        "kind":"contradiction",
        "target":{
          "type":"proposition",
          "proposition":{"key":"feature.enabled","value":"true"}
        },
        "note":"context states the opposite value"
      }
    }"#
}

#[test]
fn structured_schema_exposes_only_soft_decision_fields() {
    let model_request = build_soft_judge_model_request(&request(), 256, Some(7)).unwrap();
    let ModelOutputFormat::JsonSchema { schema, .. } = model_request.output_format else {
        panic!("expected JSON Schema output")
    };
    let serialized = serde_json::to_string(&schema).unwrap();
    assert!(serialized.contains("decision"));
    assert!(serialized.contains("finding"));
    for forbidden in [
        "verification_receipt",
        "verification_receipts",
        "finding_strength",
        "epistemic_state",
        "verdict",
        "authority_policy",
    ] {
        assert!(
            !serialized.contains(forbidden),
            "forbidden schema field {forbidden}"
        );
    }
    assert!(
        model_request
            .system
            .as_deref()
            .unwrap()
            .contains("advisory only")
    );
}

#[test]
fn parser_accepts_one_object_with_non_json_trailing_text_but_rejects_multiple_objects() {
    let one = format!("{}\n<done>", valid_finding_json());
    let parsed = parse_soft_judge_output(&one).unwrap();
    assert_eq!(parsed.decision, SoftJudgeDecision::Finding);

    let multiple = r#"{"decision":"abstain"}
{"decision":"no_finding"}"#;
    assert!(parse_soft_judge_output(multiple).is_err());
}

#[tokio::test]
async fn successful_model_output_gets_harness_owned_identity_and_usage() {
    let adapter = SequenceAdapter::new(vec![response(valid_finding_json(), 10, 4)]);
    let result = run_model_backed_soft_judge(&adapter, identity(), &request(), 256, Some(3))
        .await
        .unwrap();
    assert_eq!(result.observation.judge, identity());
    assert_eq!(result.observation.decision, SoftJudgeDecision::Finding);
    assert_eq!(result.provider_attempts, 1);
    assert_eq!(result.fallback_reason, SoftJudgeFallbackReason::NotNeeded);
    assert_eq!(result.usage.total_tokens, Some(14));
    assert_eq!(adapter.requests().len(), 1);
}

#[tokio::test]
async fn invalid_primary_output_uses_json_fallback_and_sums_usage() {
    let wrong_target = r#"{
      "decision":"finding",
      "finding":{
        "kind":"contradiction",
        "target":{
          "type":"proposition",
          "proposition":{"key":"feature.enabled","value":"false"}
        }
      }
    }"#;
    let adapter = SequenceAdapter::new(vec![
        response(wrong_target, 10, 3),
        response(r#"{"decision":"abstain"}"#, 12, 2),
    ]);
    let result = run_model_backed_soft_judge(&adapter, identity(), &request(), 256, Some(4))
        .await
        .unwrap();
    assert_eq!(result.observation.decision, SoftJudgeDecision::Abstain);
    assert!(result.observation.finding.is_none());
    assert_eq!(result.provider_attempts, 2);
    assert_eq!(
        result.fallback_reason,
        SoftJudgeFallbackReason::InvalidPrimaryStructuredOutput
    );
    assert_eq!(result.usage.total_tokens, Some(27));
    let requests = adapter.requests();
    assert!(matches!(
        requests[0].output_format,
        ModelOutputFormat::JsonSchema { .. }
    ));
    assert_eq!(requests[1].output_format, ModelOutputFormat::JsonObject);
}

#[tokio::test]
async fn unsupported_schema_mode_falls_back_without_promoting_authority() {
    let adapter = SequenceAdapter::new(vec![
        Err(ModelError::new(
            ModelErrorKind::UnsupportedCapability,
            "schema mode unsupported",
        )),
        response(r#"{"decision":"no_finding"}"#, 9, 2),
    ]);
    let result = run_model_backed_soft_judge(&adapter, identity(), &request(), 128, None)
        .await
        .unwrap();
    assert_eq!(result.observation.decision, SoftJudgeDecision::NoFinding);
    assert_eq!(result.provider_attempts, 2);
    assert_eq!(
        result.fallback_reason,
        SoftJudgeFallbackReason::PrimaryJsonSchemaUnsupported
    );
    assert_eq!(adapter.requests().len(), 2);
}

#[tokio::test]
async fn malformed_fallback_fails_closed() {
    let adapter = SequenceAdapter::new(vec![
        response("not-json", 3, 1),
        response("still-not-json", 4, 1),
    ]);
    let error = run_model_backed_soft_judge(&adapter, identity(), &request(), 128, None)
        .await
        .unwrap_err();
    assert!(matches!(
        error,
        ModelBackedSoftJudgeError::InvalidStructuredOutput(_)
    ));
    assert_eq!(adapter.requests().len(), 2);
}

#[test]
fn model_requests_define_kind_specific_finding_no_finding_and_abstain_semantics() {
    let mut unsupported = request();
    unsupported.kind = SemanticDiagnosticKind::UnsupportedPremise;
    let primary = build_soft_judge_model_request(&unsupported, 128, Some(3)).unwrap();
    assert!(primary.task.contains("unsupported_premise:"));
    assert!(primary.task.contains("introduced without support"));
    assert!(primary.task.contains("no_finding means"));
    assert!(primary.task.contains("abstain only when"));

    let mut causal = request();
    causal.kind = SemanticDiagnosticKind::CausalGap;
    causal.target = SemanticDiagnosticTarget::CausalRelation {
        relation: CausalRelation {
            causes: vec![Proposition {
                key: "queue.depth".into(),
                value: "high".into(),
            }],
            effect: Proposition {
                key: "request.latency".into(),
                value: "high".into(),
            },
        },
    };
    let fallback = build_soft_judge_json_fallback_request(&causal, 128, Some(3)).unwrap();
    assert!(fallback.task.contains("causal_gap:"));
    assert!(fallback.task.contains("only correlation or association"));
    assert!(fallback.task.contains("controlled intervention"));
    assert!(fallback.task.contains("Use only the supplied context"));
}
