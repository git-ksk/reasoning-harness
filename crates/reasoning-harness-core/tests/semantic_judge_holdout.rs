use std::{fs, path::Path};

use reasoning_harness_core::{
    CalibrationLabel, SemanticDiagnosticKind, SoftJudgeCalibrationFixture,
};

fn load_holdout() -> Vec<SoftJudgeCalibrationFixture> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/semantic-judges-holdout");
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
fn semantic_judge_holdout_v1_is_separate_observation_free_and_stratified() {
    let fixtures = load_holdout();
    assert_eq!(fixtures.len(), 28);
    assert!(fixtures.iter().all(|fixture| fixture.id.starts_with('h')));
    assert!(
        fixtures
            .iter()
            .all(|fixture| fixture.request.id.starts_with("holdout-soft-h"))
    );
    assert!(
        fixtures
            .iter()
            .all(|fixture| fixture.recorded_observations.is_empty())
    );

    assert_eq!(
        fixtures
            .iter()
            .filter(|fixture| fixture.label == CalibrationLabel::Positive)
            .count(),
        11
    );
    assert_eq!(
        fixtures
            .iter()
            .filter(|fixture| fixture.label == CalibrationLabel::Negative)
            .count(),
        8
    );
    assert_eq!(
        fixtures
            .iter()
            .filter(|fixture| fixture.label == CalibrationLabel::Ambiguous)
            .count(),
        9
    );
    assert_eq!(
        fixtures
            .iter()
            .filter(|fixture| fixture.request.kind == SemanticDiagnosticKind::Contradiction)
            .count(),
        6
    );
    assert_eq!(
        fixtures
            .iter()
            .filter(|fixture| fixture.request.kind == SemanticDiagnosticKind::UnsupportedPremise)
            .count(),
        6
    );
    assert_eq!(
        fixtures
            .iter()
            .filter(|fixture| fixture.request.kind == SemanticDiagnosticKind::CausalGap)
            .count(),
        10
    );
    assert_eq!(
        fixtures
            .iter()
            .filter(|fixture| fixture.request.kind == SemanticDiagnosticKind::Counterexample)
            .count(),
        6
    );
}
