use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use reasoning_harness_core::{
    CalibrationLabel, SemanticDiagnosticKind, SoftJudgeCalibrationFixture,
};

fn load_dir(path: PathBuf) -> Vec<SoftJudgeCalibrationFixture> {
    let mut paths = fs::read_dir(path)
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
fn semantic_judge_holdout_v4_is_frozen_observation_free_unique_and_stratified() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let fixtures = load_dir(manifest.join("fixtures/semantic-judges-holdout-v4"));
    assert_eq!(fixtures.len(), 28);
    assert!(fixtures.iter().all(|f| f.id.starts_with("v4h-")));
    assert!(
        fixtures
            .iter()
            .all(|f| f.request.id.starts_with("holdout-v4-soft-v4h-"))
    );
    assert!(fixtures.iter().all(|f| f.recorded_observations.is_empty()));
    assert_eq!(
        fixtures
            .iter()
            .map(|f| f.id.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        28
    );
    assert_eq!(
        fixtures
            .iter()
            .map(|f| f.request.id.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        28
    );
    assert!(fixtures.iter().all(|f| !f.request.context.is_empty()));
    assert_eq!(
        fixtures
            .iter()
            .filter(|f| f.label == CalibrationLabel::Positive)
            .count(),
        8
    );
    assert_eq!(
        fixtures
            .iter()
            .filter(|f| f.label == CalibrationLabel::Negative)
            .count(),
        8
    );
    assert_eq!(
        fixtures
            .iter()
            .filter(|f| f.label == CalibrationLabel::Ambiguous)
            .count(),
        12
    );
    for kind in [
        SemanticDiagnosticKind::Contradiction,
        SemanticDiagnosticKind::UnsupportedPremise,
        SemanticDiagnosticKind::CausalGap,
        SemanticDiagnosticKind::Counterexample,
    ] {
        assert_eq!(
            fixtures.iter().filter(|f| f.request.kind == kind).count(),
            7
        );
    }

    let frozen_payloads = fixtures
        .iter()
        .map(|f| serde_json::to_string(&f.request).unwrap())
        .collect::<BTreeSet<_>>();
    for dir in [
        "fixtures/semantic-judges",
        "fixtures/semantic-judges-holdout",
        "fixtures/semantic-judges-holdout-v2",
        "fixtures/semantic-judges-holdout-v3",
    ] {
        for prior in load_dir(manifest.join(dir)) {
            assert!(
                !frozen_payloads.contains(&serde_json::to_string(&prior.request).unwrap()),
                "holdout-v4 request duplicates prior corpus: {}",
                prior.id
            );
        }
    }
}
