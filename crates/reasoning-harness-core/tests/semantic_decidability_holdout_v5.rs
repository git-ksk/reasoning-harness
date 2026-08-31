use std::{collections::BTreeSet, fs, path::Path};

use reasoning_harness_core::{
    CalibrationLabel, SemanticDecidabilityDisposition, SemanticDecidabilityStudyFixture,
    SemanticDiagnosticKind, SemanticDiagnosticTarget, SoftJudgeCalibrationFixture,
    assess_semantic_decidability,
};

fn load_sources(dir: &str) -> Vec<SoftJudgeCalibrationFixture> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(dir);
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

fn load_manifests() -> Vec<SemanticDecidabilityStudyFixture> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/semantic-decidability-holdout-v5");
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
fn decidability_holdout_v5_is_observation_free_balanced_and_deterministic() {
    let sources = load_sources("fixtures/semantic-judges-holdout-v5");
    let manifests = load_manifests();
    assert_eq!(sources.len(), 24);
    assert_eq!(manifests.len(), 24);

    assert!(sources.iter().all(|source| source.id.starts_with("v5h")));
    assert!(
        sources
            .iter()
            .all(|source| source.request.id.starts_with("holdout-v5-soft-v5h"))
    );
    assert!(
        sources
            .iter()
            .all(|source| source.recorded_observations.is_empty())
    );
    assert!(
        sources
            .iter()
            .all(|source| !source.request.context.is_empty())
    );

    let source_ids = sources
        .iter()
        .map(|source| source.id.as_str())
        .collect::<BTreeSet<_>>();
    let request_ids = sources
        .iter()
        .map(|source| source.request.id.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(source_ids.len(), sources.len());
    assert_eq!(request_ids.len(), sources.len());

    for label in [
        CalibrationLabel::Positive,
        CalibrationLabel::Negative,
        CalibrationLabel::Ambiguous,
    ] {
        assert_eq!(
            sources
                .iter()
                .filter(|source| source.label == label)
                .count(),
            8
        );
    }
    for kind in [
        SemanticDiagnosticKind::Contradiction,
        SemanticDiagnosticKind::UnsupportedPremise,
        SemanticDiagnosticKind::CausalGap,
        SemanticDiagnosticKind::Counterexample,
    ] {
        assert_eq!(
            sources
                .iter()
                .filter(|source| source.request.kind == kind)
                .count(),
            6
        );
    }

    assert_eq!(
        sources
            .iter()
            .filter(|source| matches!(
                source.request.target,
                SemanticDiagnosticTarget::Inference { .. }
            ))
            .count(),
        1
    );

    let fresh_payloads = sources
        .iter()
        .map(|source| serde_json::to_string(&source.request).unwrap())
        .collect::<BTreeSet<_>>();
    for dir in [
        "fixtures/semantic-judges",
        "fixtures/semantic-judges-holdout",
        "fixtures/semantic-judges-holdout-v2",
        "fixtures/semantic-judges-holdout-v3",
    ] {
        for prior in load_sources(dir) {
            assert!(
                !fresh_payloads.contains(&serde_json::to_string(&prior.request).unwrap()),
                "holdout-v5 request duplicates pre-v4 corpus: {}",
                prior.id
            );
        }
    }

    let source_by_id = sources
        .iter()
        .map(|source| (source.id.as_str(), source))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut manifest_source_ids = BTreeSet::new();
    let mut force_variants = 0usize;
    let mut force_kinds = BTreeSet::new();
    let mut mutation_markers = BTreeSet::new();

    for manifest in manifests {
        assert!(manifest.id.starts_with("v5d"));
        assert!(manifest_source_ids.insert(manifest.source_fixture_id.clone()));
        let source = source_by_id[manifest.source_fixture_id.as_str()];
        assert_eq!(manifest.semantic_label, source.label);

        let permit_count = manifest
            .variants
            .iter()
            .filter(|variant| {
                variant.expected_disposition == SemanticDecidabilityDisposition::Permit
            })
            .count();
        let force_count = manifest
            .variants
            .iter()
            .filter(|variant| {
                variant.expected_disposition == SemanticDecidabilityDisposition::ForceAbstain
            })
            .count();
        assert_eq!(permit_count, 1, "{}", manifest.id);
        assert!(force_count <= 1, "{}", manifest.id);
        if force_count == 1 {
            assert_ne!(source.label, CalibrationLabel::Ambiguous, "{}", manifest.id);
            assert_ne!(
                source.request.kind,
                SemanticDiagnosticKind::CausalGap,
                "{}",
                manifest.id
            );
            force_variants += 1;
            force_kinds.insert(source.request.kind);
        }

        for variant in manifest.variants {
            let assessment =
                assess_semantic_decidability(&source.request, &variant.artifact).unwrap();
            assert_eq!(
                assessment.disposition, variant.expected_disposition,
                "{}:{}",
                manifest.id, variant.id
            );
            for marker in [
                "evidence-missing",
                "insufficient-authority",
                "scope-mismatch",
                "stale-evidence",
                "provenance-missing",
                "conflicting-qualified-values",
                "inference-premise-unbound",
                "claim-unbound",
            ] {
                if variant.id.contains(marker) {
                    mutation_markers.insert(marker);
                }
            }
        }
    }

    assert_eq!(
        manifest_source_ids,
        source_ids.into_iter().map(str::to_string).collect()
    );
    assert_eq!(force_variants, 10);
    assert_eq!(force_kinds.len(), 3);
    assert!(!force_kinds.contains(&SemanticDiagnosticKind::CausalGap));
    assert_eq!(mutation_markers.len(), 8);
}
