use std::{fs, path::PathBuf};

use reasoning_harness_core::{
    AssumptionBenchmarkFixture, aggregate_assumption_benchmark, evaluate_assumption_fixture,
};

#[test]
fn assumption_fixture_suite_is_a_separate_deterministic_regression_baseline() {
    let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/assumptions");
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
            let fixture: AssumptionBenchmarkFixture =
                serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
            evaluate_assumption_fixture(&fixture)
        })
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let aggregate = aggregate_assumption_benchmark(&results);

    assert_eq!(aggregate.cases, 5);
    assert_eq!(aggregate.passed_cases, 5);
    assert_eq!(aggregate.supported_premises, 1);
    assert_eq!(aggregate.explicit_input_assumptions, 1);
    assert_eq!(aggregate.unsupported_premises, 2);
    assert_eq!(aggregate.unbound_premises, 1);
    assert_eq!(aggregate.hard_findings, 2);
    assert_eq!(aggregate.soft_findings, 1);
    assert_eq!(aggregate.unsupported_detection_rate, 1.0);
    assert_eq!(aggregate.explicit_assumption_recognition_rate, 1.0);
}
