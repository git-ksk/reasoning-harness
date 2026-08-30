use std::{fs, path::Path};

use reasoning_harness_core::{
    CalibrationLabel, SoftJudgeCalibrationFixture, SoftJudgeDecision,
    aggregate_soft_judge_calibration, validate_calibration_fixtures,
};

fn load_fixtures() -> Vec<SoftJudgeCalibrationFixture> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/semantic-judges");
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
fn semantic_judge_calibration_corpus_preserves_labels_disagreement_and_abstention() {
    let fixtures = load_fixtures();
    assert_eq!(fixtures.len(), 18);
    assert_eq!(
        fixtures
            .iter()
            .filter(|fixture| fixture.label == CalibrationLabel::Positive)
            .count(),
        5
    );
    assert_eq!(
        fixtures
            .iter()
            .filter(|fixture| fixture.label == CalibrationLabel::Negative)
            .count(),
        6
    );
    assert_eq!(
        fixtures
            .iter()
            .filter(|fixture| fixture.label == CalibrationLabel::Ambiguous)
            .count(),
        7
    );
    validate_calibration_fixtures(&fixtures).unwrap();

    let report = aggregate_soft_judge_calibration(&fixtures).unwrap();
    assert_eq!(report.cases, 18);
    assert_eq!(report.judges.len(), 3);
    assert!(report.agreement.disagreeing_pairs > 0);
    assert!(report.agreement.abstain_votes > 0);
    assert!(report.judges.iter().any(|metrics| metrics.abstentions > 0));
    assert!(
        report
            .judges
            .iter()
            .all(|metrics| metrics.labelled_cases == 11)
    );
    assert!(
        report
            .judges
            .iter()
            .all(|metrics| metrics.ambiguous_cases == 7)
    );
    assert!(report.judges.iter().all(|metrics| {
        metrics.ambiguous_abstention_rate.is_some()
            && metrics.ambiguous_abstentions <= metrics.ambiguous_cases
    }));

    for fixture in &fixtures {
        for observation in &fixture.recorded_observations {
            if observation.decision == SoftJudgeDecision::Finding {
                let finding = observation.finding.as_ref().unwrap();
                assert_eq!(finding.kind, fixture.request.kind);
                assert_eq!(finding.target, fixture.request.target);
            }
        }
    }
}
