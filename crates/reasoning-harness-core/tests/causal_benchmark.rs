use std::{fs, path::PathBuf};

use reasoning_harness_core::{
    CausalBenchmarkFixture, aggregate_causal_benchmark, evaluate_causal_fixture,
};

#[test]
fn causal_fixture_suite_is_a_separate_deterministic_regression_baseline() {
    let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/causal");
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
            let fixture: CausalBenchmarkFixture =
                serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
            evaluate_causal_fixture(&fixture)
        })
        .collect::<Vec<_>>();
    let aggregate = aggregate_causal_benchmark(&results);

    assert_eq!(aggregate.cases, 8);
    assert_eq!(aggregate.passed_cases, 8);
    assert_eq!(aggregate.edge_assessments, 8);
    assert_eq!(aggregate.supported_edges, 1);
    assert_eq!(aggregate.refuted_edges, 1);
    assert_eq!(aggregate.unknown_edges, 6);
    assert_eq!(aggregate.hard_findings, 1);
    assert_eq!(aggregate.soft_findings, 6);
}
