use std::{fs, path::PathBuf};

use reasoning_harness_core::{
    AddIrrelevantEvidence, BenchmarkFixture, CausalBenchmarkFixture, ReverseCausalCauseOrder,
    ReverseCausalEvidenceOrder, ReverseEvidenceOrder, ReverseInferenceOrder, StableIdRemap,
    aggregate_metamorphic, evaluate_benchmark_metamorphic, evaluate_causal_metamorphic,
};

fn fixture(name: &str) -> BenchmarkFixture {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("../../fixtures/{name}"));
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn causal_fixture(name: &str) -> CausalBenchmarkFixture {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("../../fixtures/{name}"));
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

#[test]
fn deterministic_transform_families_preserve_hard_reasoning_outcomes() {
    let contradictory = fixture("contradictory-evidence.json");
    let direct = fixture("direct-region.json");
    let independent = fixture("metamorphic/independent-inferences.json");
    let multi_cause = causal_fixture("metamorphic/causal-multi-cause.json");
    let conflicting = causal_fixture("causal/05_conflicting_evidence.json");

    let results = vec![
        evaluate_benchmark_metamorphic(&contradictory, &ReverseEvidenceOrder).unwrap(),
        evaluate_benchmark_metamorphic(&independent, &ReverseInferenceOrder).unwrap(),
        evaluate_benchmark_metamorphic(&contradictory, &StableIdRemap).unwrap(),
        evaluate_benchmark_metamorphic(&direct, &AddIrrelevantEvidence).unwrap(),
        evaluate_causal_metamorphic(&multi_cause, &ReverseCausalCauseOrder).unwrap(),
        evaluate_causal_metamorphic(&conflicting, &ReverseCausalEvidenceOrder).unwrap(),
    ];

    for result in &results {
        assert!(result.final_verdict_invariant, "{result:?}");
        assert!(result.hard_findings_invariant, "{result:?}");
        assert!(result.soft_findings_stable, "{result:?}");
        assert!(result.diagnostic_status_invariant, "{result:?}");
        assert!(result.hard_outcomes_invariant(), "{result:?}");
        assert!(result.changed_diagnostic_reasons.is_empty(), "{result:?}");
    }

    let id_remap = &results[2];
    assert!(
        !id_remap.changed_diagnostic_ids.is_empty(),
        "stable-ID remapping should prove raw IDs may change while semantic findings remain invariant"
    );

    let aggregate = aggregate_metamorphic(&results);
    assert_eq!(aggregate.transformations, 6);
    assert_eq!(aggregate.final_verdict_invariance_rate, 1.0);
    assert_eq!(aggregate.hard_finding_invariance_rate, 1.0);
    assert_eq!(aggregate.soft_finding_stability_rate, 1.0);
    assert_eq!(aggregate.diagnostic_status_invariance_rate, 1.0);
    assert_eq!(aggregate.hard_outcome_invariance_rate, 1.0);
    assert_eq!(aggregate.failures_by_transform.values().sum::<usize>(), 0);
    assert_eq!(aggregate.failures_by_transform.len(), 6);
}
