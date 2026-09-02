use std::{collections::BTreeMap, fs, path::Path};

use reasoning_harness_core::{
    EvidenceSufficiencyCalibrationFixture, EvidenceSufficiencyLabel,
    SemanticDecidabilityDisposition, SemanticDiagnosticKind, SemanticDiagnosticTarget,
    SoftJudgeRequest, assess_semantic_decidability, validate_artifact,
    validate_evidence_sufficiency_fixture,
};

fn root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/evidence-sufficiency-rsd0")
}

fn fixtures() -> Vec<EvidenceSufficiencyCalibrationFixture> {
    let mut paths = fs::read_dir(root())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect::<Vec<_>>();
    paths.sort();
    paths
        .into_iter()
        .map(|path| serde_json::from_slice(&fs::read(path).unwrap()).unwrap())
        .collect()
}

fn d3_request(fixture: &EvidenceSufficiencyCalibrationFixture) -> SoftJudgeRequest {
    let context = fixture
        .request
        .evidence_ids
        .iter()
        .map(|id| {
            let evidence = fixture
                .artifact
                .evidence
                .iter()
                .find(|evidence| evidence.id == *id)
                .unwrap();
            format!("{}: {}", evidence.id, evidence.observation)
        })
        .collect();
    SoftJudgeRequest {
        id: format!("rsd0-d3-baseline:{}", fixture.id),
        task: fixture.request.task.clone(),
        kind: SemanticDiagnosticKind::UnsupportedPremise,
        target: SemanticDiagnosticTarget::Proposition {
            proposition: fixture.request.target.clone(),
        },
        context,
    }
}

#[test]
fn rsd0_is_fresh_pre_observation_and_exposes_a_measurable_d3_residual_gap() {
    let fixtures = fixtures();
    assert_eq!(fixtures.len(), 12);

    let mut family_labels = BTreeMap::<String, Vec<EvidenceSufficiencyLabel>>::new();
    let mut label_counts = BTreeMap::<EvidenceSufficiencyLabel, usize>::new();
    let mut residual_non_sufficient = 0usize;

    for fixture in fixtures {
        validate_evidence_sufficiency_fixture(&fixture).unwrap();
        let artifact_validation = validate_artifact(&fixture.artifact);
        assert!(
            artifact_validation.is_ok(),
            "{}: {:?}",
            fixture.id,
            artifact_validation.diagnostics
        );
        // RSD0 must measure a gap *beyond* D3's typed contract. If a fixture is already blocked by
        // D3, it belongs to the existing decidability corpus instead of this residual corpus.
        let d3 = assess_semantic_decidability(&d3_request(&fixture), &fixture.artifact).unwrap();
        assert_eq!(
            d3.disposition,
            SemanticDecidabilityDisposition::Permit,
            "{} is not residual to D3: {:?}",
            fixture.id,
            d3.reasons
        );
        assert!(d3.reasons.is_empty(), "{}", fixture.id);

        family_labels
            .entry(fixture.family.clone())
            .or_default()
            .push(fixture.label);
        *label_counts.entry(fixture.label).or_default() += 1;
        if fixture.label != EvidenceSufficiencyLabel::Sufficient {
            residual_non_sufficient += 1;
        }
    }

    assert_eq!(family_labels.len(), 4);
    for (family, mut labels) in family_labels {
        labels.sort();
        assert_eq!(
            labels,
            vec![
                EvidenceSufficiencyLabel::Sufficient,
                EvidenceSufficiencyLabel::Insufficient,
                EvidenceSufficiencyLabel::Mixed,
            ],
            "{family}"
        );
    }
    assert_eq!(label_counts[&EvidenceSufficiencyLabel::Sufficient], 4);
    assert_eq!(label_counts[&EvidenceSufficiencyLabel::Insufficient], 4);
    assert_eq!(label_counts[&EvidenceSufficiencyLabel::Mixed], 4);
    assert_eq!(residual_non_sufficient, 8);
}

#[test]
fn rsd0_fixture_loader_is_scoped_away_from_frozen_holdouts() {
    let path = root();
    let text = path.to_string_lossy();
    assert!(text.contains("evidence-sufficiency-rsd0"));
    assert!(!text.contains("holdout-v4"));
    assert!(!text.contains("holdout-v5"));
}

#[test]
fn rsd0_contract_rejects_typed_requirements_so_it_cannot_duplicate_d3() {
    let mut fixture = fixtures().remove(0);
    fixture
        .artifact
        .evidence_requirements
        .push(reasoning_harness_core::EvidenceRequirement {
            proposition: fixture.request.target.clone(),
            as_of_unix_seconds: None,
            scope: None,
            minimum_authority_class: None,
        });
    assert!(matches!(
        validate_evidence_sufficiency_fixture(&fixture),
        Err(reasoning_harness_core::EvidenceSufficiencyFixtureError::HasTypedEvidenceRequirement)
    ));
}
