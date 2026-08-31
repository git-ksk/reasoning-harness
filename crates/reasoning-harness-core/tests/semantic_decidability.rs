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
    use reasoning_harness_core::SemanticDiagnosticKind;
    assert_eq!(families.len(), 7);
    assert_eq!(kinds.len(), 3);
    assert!(kinds.contains(&SemanticDiagnosticKind::Contradiction));
    assert!(kinds.contains(&SemanticDiagnosticKind::UnsupportedPremise));
    assert!(kinds.contains(&SemanticDiagnosticKind::Counterexample));
    assert!(!kinds.contains(&SemanticDiagnosticKind::CausalGap));
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
fn causal_endpoint_requirement_is_not_promoted_to_relation_sufficiency() {
    use reasoning_harness_core::{SemanticDecidabilityStudyFixture, SoftJudgeCalibrationFixture};

    let source_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/semantic-judges/08_causal_negative.json");
    let source: SoftJudgeCalibrationFixture =
        serde_json::from_slice(&fs::read(source_path).unwrap()).unwrap();
    let study_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/semantic-decidability-d2/07_08_causal_negative.json");
    let study: SemanticDecidabilityStudyFixture =
        serde_json::from_slice(&fs::read(study_path).unwrap()).unwrap();
    let variant = study
        .variants
        .into_iter()
        .find(|variant| variant.id == "permit-only-one-qualified-value")
        .unwrap();

    assert!(!variant.artifact.evidence_requirements.is_empty());
    let assessment = assess_semantic_decidability(&source.request, &variant.artifact).unwrap();
    assert_eq!(
        assessment.disposition,
        SemanticDecidabilityDisposition::Permit
    );
    assert!(assessment.reasons.is_empty());
}

#[test]
fn d2_study_manifest_keeps_semantic_and_eligibility_labels_separate() {
    use reasoning_harness_core::{
        CalibrationLabel, SemanticDecidabilityStudyFixture, SemanticDiagnosticKind,
        SoftJudgeCalibrationFixture,
    };

    let manifest_root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/semantic-decidability-d2");
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/semantic-judges");

    let mut source_paths = fs::read_dir(source_root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect::<Vec<_>>();
    source_paths.sort();
    let sources = source_paths
        .into_iter()
        .map(|path| {
            let fixture: SoftJudgeCalibrationFixture =
                serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
            (fixture.id.clone(), fixture)
        })
        .collect::<std::collections::BTreeMap<_, _>>();

    let mut manifest_paths = fs::read_dir(manifest_root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect::<Vec<_>>();
    manifest_paths.sort();
    let manifests = manifest_paths
        .into_iter()
        .map(|path| {
            serde_json::from_slice::<SemanticDecidabilityStudyFixture>(&fs::read(path).unwrap())
                .unwrap()
        })
        .collect::<Vec<_>>();

    assert_eq!(manifests.len(), 15);
    let mut manifest_ids = BTreeSet::new();
    let mut source_ids = BTreeSet::new();
    let mut all_kinds = BTreeSet::new();
    let mut force_kinds = BTreeSet::new();
    let mut force_variants = 0usize;
    let mut eligible_positive = 0usize;
    let mut eligible_negative = 0usize;
    let mut eligible_ambiguous = 0usize;

    for manifest in manifests {
        assert!(manifest_ids.insert(manifest.id.clone()));
        assert!(source_ids.insert(manifest.source_fixture_id.clone()));
        let source = sources.get(&manifest.source_fixture_id).unwrap();
        assert_eq!(manifest.semantic_label, source.label, "{}", manifest.id);
        all_kinds.insert(source.request.kind);

        let permit_count = manifest
            .variants
            .iter()
            .filter(|variant| {
                variant.expected_disposition == SemanticDecidabilityDisposition::Permit
            })
            .count();
        let force_count = manifest.variants.len() - permit_count;
        assert_eq!(
            permit_count, 1,
            "{} must have exactly one permit control",
            manifest.id
        );
        assert!(
            force_count <= 1,
            "{} has more than one D2 force variant",
            manifest.id
        );

        match manifest.semantic_label {
            CalibrationLabel::Positive => eligible_positive += 1,
            CalibrationLabel::Negative => eligible_negative += 1,
            CalibrationLabel::Ambiguous => eligible_ambiguous += 1,
        }

        for variant in manifest.variants {
            let assessment =
                assess_semantic_decidability(&source.request, &variant.artifact).unwrap();
            assert_eq!(
                assessment.disposition, variant.expected_disposition,
                "{}:{}",
                manifest.id, variant.id
            );
            if variant.expected_disposition == SemanticDecidabilityDisposition::ForceAbstain {
                force_variants += 1;
                force_kinds.insert(source.request.kind);
                assert_ne!(
                    manifest.semantic_label,
                    CalibrationLabel::Ambiguous,
                    "typed insufficiency is deliberately separate from semantic ambiguity in D2 v1"
                );
            }
        }
    }

    assert_eq!(force_variants, 7);
    assert_eq!(eligible_positive, 5);
    assert_eq!(eligible_negative, 6);
    assert_eq!(eligible_ambiguous, 4);
    assert_eq!(all_kinds.len(), 4);
    assert_eq!(force_kinds.len(), 3);
    assert!(all_kinds.contains(&SemanticDiagnosticKind::Contradiction));
    assert!(all_kinds.contains(&SemanticDiagnosticKind::UnsupportedPremise));
    assert!(all_kinds.contains(&SemanticDiagnosticKind::CausalGap));
    assert!(all_kinds.contains(&SemanticDiagnosticKind::Counterexample));
    assert!(!force_kinds.contains(&SemanticDiagnosticKind::CausalGap));
}
