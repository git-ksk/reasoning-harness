use std::{fs, path::PathBuf};

use reasoning_harness_core::{BenchmarkFixture, aggregate_benchmark, evaluate_benchmark_fixture};

#[test]
fn recorded_fixture_suite_is_a_stable_regression_baseline() {
    let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures");
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
            let fixture: BenchmarkFixture =
                serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
            evaluate_benchmark_fixture(&fixture, fixture.recorded_candidate.clone())
        })
        .collect::<Vec<_>>();
    let comparison = aggregate_benchmark(&results);

    assert_eq!(comparison.harness.cases, 7);
    assert_eq!(comparison.baseline.unsupported_accepted_claims, 3);
    assert_eq!(comparison.harness.unsupported_accepted_claims, 0);
    assert_close(comparison.baseline.verdict_accuracy, 2.0 / 7.0);
    assert_close(comparison.harness.verdict_accuracy, 1.0);
    assert_close(comparison.baseline.accept_recall, 1.0);
    assert_close(comparison.harness.accept_recall, 1.0);
    assert_close(comparison.baseline.reject_recall, 0.0);
    assert_close(comparison.harness.reject_recall, 1.0);
    assert_close(comparison.baseline.unknown_recall, 0.25);
    assert_close(comparison.harness.unknown_recall, 1.0);
    assert_close(comparison.baseline.hidden_assumption_exposure_rate, 0.0);
    assert_close(comparison.harness.hidden_assumption_exposure_rate, 1.0);
    assert_close(comparison.harness.contradiction_detection_rate, 1.0);
    assert_close(comparison.harness.counterexample_detection_rate, 1.0);
    assert_eq!(comparison.harness.bad_inference_edges_retained, 0);
    assert_close(comparison.harness.causal_edge_quality, 1.0);
    assert_close(comparison.harness.deterministic_verifier_failure_rate, 0.0);
}

fn assert_close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < f64::EPSILON * 8.0);
}
