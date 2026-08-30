use std::{collections::BTreeSet, fs, path::Path};

use reasoning_harness_core::{
    CalibrationLabel, SemanticDiagnosticKind, SoftJudgeCalibrationFixture,
};

fn load_holdout_v3() -> Vec<SoftJudgeCalibrationFixture> {
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/semantic-judges-holdout-v3");
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
fn semantic_judge_holdout_v3_is_observation_free_unique_and_stratified() {
    let fixtures = load_holdout_v3();
    assert_eq!(fixtures.len(), 28);
    assert!(fixtures.iter().all(|fixture| fixture.id.starts_with("v3h")));
    assert!(
        fixtures
            .iter()
            .all(|fixture| fixture.request.id.starts_with("holdout-v3-soft-v3h"))
    );
    assert!(
        fixtures
            .iter()
            .all(|fixture| fixture.recorded_observations.is_empty())
    );

    let fixture_ids = fixtures
        .iter()
        .map(|fixture| fixture.id.as_str())
        .collect::<BTreeSet<_>>();
    let request_ids = fixtures
        .iter()
        .map(|fixture| fixture.request.id.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(fixture_ids.len(), fixtures.len());
    assert_eq!(request_ids.len(), fixtures.len());
    assert!(
        fixtures
            .iter()
            .all(|fixture| !fixture.request.context.is_empty())
    );

    assert_eq!(
        fixtures
            .iter()
            .filter(|fixture| fixture.label == CalibrationLabel::Positive)
            .count(),
        8
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
        12
    );

    for kind in [
        SemanticDiagnosticKind::Contradiction,
        SemanticDiagnosticKind::UnsupportedPremise,
        SemanticDiagnosticKind::CausalGap,
        SemanticDiagnosticKind::Counterexample,
    ] {
        assert_eq!(
            fixtures
                .iter()
                .filter(|fixture| fixture.request.kind == kind)
                .count(),
            7
        );
    }
}
