use std::{fs, path::Path, pin::Pin};

use reasoning_harness_core::{
    DEFAULT_SEMANTIC_RUNTIME_PROFILE, ModelAdapter, ModelRequest, ModelResponse, ModelUsage,
    SEMANTIC_DECIDABILITY_D3_CONFIGURATION_ID, SOFT_SEMANTIC_V3_CONFIGURATION_ID,
    SemanticDecidabilityCalibrationFixture, SemanticDecidabilityDisposition,
    SemanticRuntimeProfile, SoftJudgeDecision, run_semantic_runtime,
};

struct DecisionOnlyAdapter;

impl ModelAdapter for DecisionOnlyAdapter {
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
                text: r#"{"decision":"finding"}"#.into(),
                model: "compatible-model".into(),
                usage: ModelUsage::default(),
                finish_reason: Some("stop".into()),
            })
        })
    }
}

struct ForbiddenBindingAdapter;

impl ModelAdapter for ForbiddenBindingAdapter {
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
                text: r#"{"decision":"finding","finding":{"kind":"contradiction"}}"#.into(),
                model: "incompatible-model".into(),
                usage: ModelUsage::default(),
                finish_reason: Some("stop".into()),
            })
        })
    }
}

fn calibration_fixture(id: &str) -> SemanticDecidabilityCalibrationFixture {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/semantic-decidability-calibration");
    let path = fs::read_dir(root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(id))
        })
        .unwrap();
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

#[test]
fn runtime_identity_and_rollback_are_frozen() {
    assert_eq!(
        DEFAULT_SEMANTIC_RUNTIME_PROFILE,
        SemanticRuntimeProfile::SoftSemanticV3
    );
    let baseline = SemanticRuntimeProfile::SoftSemanticV3.identity();
    assert_eq!(
        baseline.configuration_id(),
        SOFT_SEMANTIC_V3_CONFIGURATION_ID
    );
    assert_eq!(
        baseline.semantic_baseline(),
        SOFT_SEMANTIC_V3_CONFIGURATION_ID
    );
    assert!(baseline.materialization_contract().is_none());
    assert!(baseline.decidability_contract().is_none());
    assert!(baseline.rollback_configuration_id().is_none());

    let d3 = SemanticRuntimeProfile::SemanticDecidabilityD3V1.identity();
    assert_eq!(
        d3.configuration_id(),
        SEMANTIC_DECIDABILITY_D3_CONFIGURATION_ID
    );
    assert_eq!(
        d3.rollback_configuration_id(),
        Some(SOFT_SEMANTIC_V3_CONFIGURATION_ID)
    );
    assert_eq!(
        SemanticRuntimeProfile::SemanticDecidabilityD3V1.rollback_profile(),
        Some(SemanticRuntimeProfile::SoftSemanticV3)
    );
}

#[tokio::test]
async fn d3_runtime_only_preserves_or_forces_abstention() {
    let fixture = calibration_fixture("02_binding_missing");
    assert_eq!(
        fixture.expected_disposition,
        SemanticDecidabilityDisposition::ForceAbstain
    );

    let result = run_semantic_runtime(
        SemanticRuntimeProfile::SemanticDecidabilityD3V1,
        &DecisionOnlyAdapter,
        "compatible-model",
        &fixture.request,
        &fixture.artifact,
        256,
        Some(7),
    )
    .await
    .unwrap();

    assert_eq!(result.base_decision, SoftJudgeDecision::Finding);
    assert_eq!(result.observation.decision, SoftJudgeDecision::Abstain);
    assert!(result.observation.finding.is_none());
    assert_eq!(
        result.decidability.unwrap().disposition,
        SemanticDecidabilityDisposition::ForceAbstain
    );
    assert_eq!(
        result.runtime.configuration_id(),
        SEMANTIC_DECIDABILITY_D3_CONFIGURATION_ID
    );
}

#[tokio::test]
async fn d3_runtime_preserves_permitted_base_decision() {
    let fixture = calibration_fixture("01_binding_control");
    assert_eq!(
        fixture.expected_disposition,
        SemanticDecidabilityDisposition::Permit
    );

    let result = run_semantic_runtime(
        SemanticRuntimeProfile::SemanticDecidabilityD3V1,
        &DecisionOnlyAdapter,
        "compatible-model",
        &fixture.request,
        &fixture.artifact,
        256,
        Some(7),
    )
    .await
    .unwrap();

    assert_eq!(result.base_decision, SoftJudgeDecision::Finding);
    assert_eq!(result.observation.decision, SoftJudgeDecision::Finding);
    assert!(result.observation.finding.is_some());
}

#[tokio::test]
async fn d3_runtime_never_repairs_materialization_protocol_failure_into_abstention() {
    let fixture = calibration_fixture("02_binding_missing");
    let error = run_semantic_runtime(
        SemanticRuntimeProfile::SemanticDecidabilityD3V1,
        &ForbiddenBindingAdapter,
        "incompatible-model",
        &fixture.request,
        &fixture.artifact,
        256,
        Some(7),
    )
    .await
    .unwrap_err();

    assert!(matches!(
        error,
        reasoning_harness_core::SemanticRuntimeError::Materialization(_)
    ));
}

#[test]
fn serialized_runtime_identity_rejects_spoofed_configuration() {
    let identity = SemanticRuntimeProfile::SemanticDecidabilityD3V1.identity();
    let mut value = serde_json::to_value(&identity).unwrap();
    value["configuration_id"] = serde_json::Value::String("spoofed-d3".into());
    assert!(
        serde_json::from_value::<reasoning_harness_core::SemanticRuntimeIdentity>(value).is_err()
    );

    let round_trip = serde_json::from_value::<reasoning_harness_core::SemanticRuntimeIdentity>(
        serde_json::to_value(identity).unwrap(),
    )
    .unwrap();
    assert_eq!(
        round_trip.configuration_id(),
        SEMANTIC_DECIDABILITY_D3_CONFIGURATION_ID
    );
}
