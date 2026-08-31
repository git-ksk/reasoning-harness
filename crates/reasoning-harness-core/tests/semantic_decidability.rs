use std::{collections::BTreeSet, fs, path::Path};

use reasoning_harness_core::{
    SemanticDecidabilityCalibrationFixture, SemanticDecidabilityDisposition,
    SemanticDecidabilityReason, SoftJudgeDecision, assess_semantic_decidability,
    compose_semantic_decidability,
};

fn load_fixtures() -> Vec<SemanticDecidabilityCalibrationFixture> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/semantic-decidability-calibration");
    let mut paths = fs::read_dir(root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect::<Vec<_>>();
    paths.sort();
    paths
        .into_iter()
        .map(|path| serde_json::from_slice(&fs::read(path).unwrap()).unwrap())
        .collect()
}

#[test]
fn calibration_pairs_are_deterministic_and_monotone() {
    let fixtures = load_fixtures();
    assert_eq!(fixtures.len(), 14);

    let mut pair_results = std::collections::BTreeMap::<String, Vec<_>>::new();
    let mut kinds = BTreeSet::new();
    let mut families = BTreeSet::new();

    for fixture in fixtures {
        let assessment = assess_semantic_decidability(&fixture.request, &fixture.artifact).unwrap();
        assert_eq!(
            assessment.disposition, fixture.expected_disposition,
            "{}",
            fixture.id
        );
        pair_results
            .entry(fixture.pair_id.clone())
            .or_default()
            .push(assessment.disposition);
        kinds.insert(fixture.request.kind);
        families.insert(fixture.mutation_family);
    }

    assert_eq!(pair_results.len(), 7);
    for (pair_id, mut dispositions) in pair_results {
        dispositions.sort();
        assert_eq!(
            dispositions,
            vec![
                SemanticDecidabilityDisposition::Permit,
                SemanticDecidabilityDisposition::ForceAbstain,
            ],
            "{pair_id}"
        );
    }
    assert_eq!(families.len(), 7);
    assert_eq!(kinds.len(), 4);
}

#[test]
fn composition_can_only_preserve_or_abstain() {
    let permit = reasoning_harness_core::SemanticDecidabilityAssessment {
        disposition: SemanticDecidabilityDisposition::Permit,
        reasons: vec![],
    };
    let force = reasoning_harness_core::SemanticDecidabilityAssessment {
        disposition: SemanticDecidabilityDisposition::ForceAbstain,
        reasons: vec![SemanticDecidabilityReason::MissingPropositionBinding],
    };

    for decision in [
        SoftJudgeDecision::Finding,
        SoftJudgeDecision::NoFinding,
        SoftJudgeDecision::Abstain,
    ] {
        assert_eq!(compose_semantic_decidability(decision, &permit), decision);
        assert_eq!(
            compose_semantic_decidability(decision, &force),
            SoftJudgeDecision::Abstain
        );
    }
}

#[test]
fn missing_claim_target_fails_closed_without_inventing_binding() {
    let mut fixture = load_fixtures()
        .into_iter()
        .find(|fixture| fixture.id == "01_binding_control")
        .unwrap();
    fixture.artifact.claims.clear();
    let assessment = assess_semantic_decidability(&fixture.request, &fixture.artifact).unwrap();
    assert_eq!(
        assessment.disposition,
        SemanticDecidabilityDisposition::ForceAbstain
    );
    assert_eq!(
        assessment.reasons,
        vec![SemanticDecidabilityReason::MissingTargetBinding]
    );
}

#[test]
fn invalid_artifact_is_operational_error_not_semantic_abstention() {
    let mut fixture = load_fixtures().remove(0);
    fixture.artifact.task.clear();
    let error = assess_semantic_decidability(&fixture.request, &fixture.artifact).unwrap_err();
    assert!(error.to_string().contains("empty_task"));
}

#[test]
fn causal_target_without_explicit_requirement_is_not_blocked_by_default() {
    let mut fixture = load_fixtures()
        .into_iter()
        .find(|fixture| fixture.id == "13_conflict_control")
        .unwrap();
    fixture.artifact.evidence_requirements.clear();
    fixture.artifact.evidence.clear();
    let assessment = assess_semantic_decidability(&fixture.request, &fixture.artifact).unwrap();
    assert_eq!(
        assessment.disposition,
        SemanticDecidabilityDisposition::Permit
    );
    assert!(assessment.reasons.is_empty());
}
