use std::{fs, path::PathBuf};

use reasoning_harness_core::{
    EvidenceQualificationBenchmarkFixture, aggregate_evidence_qualification_benchmark,
    evaluate_evidence_qualification_fixture,
};

#[test]
fn evidence_qualification_suite_is_a_separate_deterministic_regression_baseline() {
    let fixture_dir =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/evidence-qualification");
    let mut paths = fs::read_dir(fixture_dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect::<Vec<_>>();
    paths.sort();

    let results = paths
        .into_iter()
        .map(|path| {
            let fixture: EvidenceQualificationBenchmarkFixture =
                serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
            evaluate_evidence_qualification_fixture(&fixture)
        })
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let aggregate = aggregate_evidence_qualification_benchmark(&results);

    assert_eq!(aggregate.cases, 8);
    assert_eq!(aggregate.passed_cases, 8);
    assert_eq!(aggregate.qualified_evidence, 3);
    assert_eq!(aggregate.disqualified_evidence, 5);
    assert_eq!(aggregate.unknown_evidence, 1);
    assert_eq!(aggregate.hard_findings, 6);
    assert_eq!(aggregate.soft_findings, 3);
    assert_eq!(aggregate.finding_reason_detection_rate, 1.0);
}
