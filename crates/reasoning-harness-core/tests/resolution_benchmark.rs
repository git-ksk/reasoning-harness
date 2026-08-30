use std::{fs, path::PathBuf};

use reasoning_harness_core::{
    BenchmarkFixture, CorpusManifest, ResolutionBenchmarkFixture, aggregate_resolution_benchmark,
    evaluate_resolution_fixture,
};

#[test]
fn controlled_resolution_suite_preserves_base_case_identity_and_authority_boundaries() {
    let fixture_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures");
    let manifest: CorpusManifest =
        serde_json::from_slice(&fs::read(fixture_root.join("corpus/v1.json")).unwrap()).unwrap();
    let mut paths = fs::read_dir(fixture_root.join("resolution"))
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
            let fixture: ResolutionBenchmarkFixture =
                serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
            let metadata = manifest
                .cases
                .iter()
                .find(|case| case.case_id == fixture.base_case_id)
                .expect("resolution base case must exist in corpus v1");
            assert_eq!(metadata.fixture_path, fixture.base_fixture_path);
            let base: BenchmarkFixture = serde_json::from_slice(
                &fs::read(fixture_root.join(&fixture.base_fixture_path)).unwrap(),
            )
            .unwrap();
            assert_eq!(base.id, metadata.fixture_id);
            let result = evaluate_resolution_fixture(&fixture, &base).unwrap();
            assert_eq!(result.base_case_id, fixture.base_case_id);
            assert!(result.expectations_met, "scenario {}", fixture.id);
            result
        })
        .collect::<Vec<_>>();

    let aggregate = aggregate_resolution_benchmark(&results);
    assert_eq!(aggregate.cases, 9);
    assert_eq!(aggregate.passed_cases, 9);
    assert_eq!(aggregate.initially_unknown_cases, 9);
    assert_eq!(aggregate.recovered_supported_cases, 1);
    assert!((aggregate.recovery_rate - (1.0 / 9.0)).abs() < 1e-12);
    assert_eq!(aggregate.resolved_refuted_cases, 1);
    assert_eq!(aggregate.exhausted_cases, 7);
    assert_eq!(aggregate.unavailable_cases, 0);
    assert_eq!(aggregate.human_review_required_cases, 0);
    assert_eq!(aggregate.unsafe_final_answers, 0);
    assert_eq!(aggregate.blocked_unverified_finalizations, 0);
    assert_eq!(aggregate.mean_factual_claim_coverage, 1.0);
    assert_eq!(aggregate.total_attempts, 10);
}
