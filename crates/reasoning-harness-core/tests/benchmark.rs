use std::{fs, path::PathBuf};

use reasoning_harness_core::{
    BenchmarkArmResult, BenchmarkCaseResult, BenchmarkFixture, Verdict, aggregate_benchmark,
    evaluate_benchmark_fixture,
};

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

    assert_eq!(comparison.harness.cases, 20);
    assert_eq!(comparison.baseline.unsupported_accepted_claims, 8);
    assert_eq!(comparison.harness.unsupported_accepted_claims, 0);
    assert_eq!(comparison.baseline.unsafe_accept_cases, 8);
    assert_eq!(comparison.harness.unsafe_accept_cases, 0);
    assert_close(comparison.baseline.verdict_accuracy, 0.4);
    assert_close(comparison.harness.verdict_accuracy, 1.0);
    assert_close(comparison.baseline.accept_recall, 1.0);
    assert_close(comparison.harness.accept_recall, 1.0);
    assert_close(comparison.baseline.reject_recall, 0.0);
    assert_close(comparison.harness.reject_recall, 1.0);
    assert_close(comparison.baseline.unknown_recall, 1.0 / 3.0);
    assert_close(comparison.harness.unknown_recall, 1.0);
    assert_close(
        comparison.baseline.hidden_assumption_exposure_rate,
        1.0 / 9.0,
    );
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

#[test]
fn detection_rates_ignore_findings_outside_labeled_cases() {
    let empty_arm = |counterexamples_detected| BenchmarkArmResult {
        verdict: Some(Verdict::Unknown),
        claims: 0,
        claims_with_evidence: 0,
        inference_edges: 0,
        verdict_correct: true,
        evidence_coverage: 0.0,
        unsupported_accepted_claims: 0,
        unsafe_accept: false,
        hidden_assumptions_exposed: 0,
        contradiction_claims_detected: 0,
        counterexamples_detected,
        hard_adversarial_findings: counterexamples_detected,
        soft_adversarial_findings: 0,
        bad_inference_edges_retained: 0,
        deterministic_failure: false,
        deterministic_failure_reason: None,
    };

    let labeled = BenchmarkCaseResult {
        fixture_id: "labeled".into(),
        expected_verdict: Verdict::Unknown,
        expected_hidden_assumptions: 0,
        expected_contradiction: false,
        expected_counterexample: true,
        baseline: empty_arm(0),
        harness: empty_arm(1),
    };
    let unlabeled = BenchmarkCaseResult {
        fixture_id: "unlabeled".into(),
        expected_verdict: Verdict::Unknown,
        expected_hidden_assumptions: 0,
        expected_contradiction: false,
        expected_counterexample: false,
        baseline: empty_arm(0),
        harness: empty_arm(1),
    };

    let comparison = aggregate_benchmark(&[labeled, unlabeled]);
    assert_close(comparison.harness.counterexample_detection_rate, 1.0);
}
