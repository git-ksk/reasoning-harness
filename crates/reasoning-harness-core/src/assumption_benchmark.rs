use serde::{Deserialize, Serialize};

use crate::{
    AssumptionInspector, AssumptionSupportStatus, BenchmarkFixture, FindingStrength, HarnessError,
    HarnessInput, ReasoningCandidate, Verdict, VerificationReceipt,
    benchmark::run_benchmark_harness,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssumptionBenchmarkFixture {
    pub id: String,
    pub description: String,
    pub input: HarnessInput,
    pub recorded_candidate: ReasoningCandidate,
    #[serde(default)]
    pub verification_receipts: Vec<VerificationReceipt>,
    #[serde(default)]
    pub expected_statuses: Vec<AssumptionSupportStatus>,
    pub expected_hard_findings: usize,
    pub expected_soft_findings: usize,
    #[serde(default)]
    pub expected_finding_inference_references: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AssumptionBenchmarkCaseResult {
    pub fixture_id: String,
    pub statuses: Vec<AssumptionSupportStatus>,
    pub hard_findings: usize,
    pub soft_findings: usize,
    pub finding_inference_references: Vec<usize>,
    pub unsupported_detected: usize,
    pub expected_unsupported: usize,
    pub explicit_assumptions_recognized: usize,
    pub expected_explicit_assumptions: usize,
    pub expectations_met: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct AssumptionBenchmarkAggregate {
    pub cases: usize,
    pub passed_cases: usize,
    pub supported_premises: usize,
    pub explicit_input_assumptions: usize,
    pub unsupported_premises: usize,
    pub unbound_premises: usize,
    pub hard_findings: usize,
    pub soft_findings: usize,
    pub unsupported_detection_rate: f64,
    pub explicit_assumption_recognition_rate: f64,
}

pub fn evaluate_assumption_fixture(
    fixture: &AssumptionBenchmarkFixture,
) -> Result<AssumptionBenchmarkCaseResult, HarnessError> {
    let benchmark_fixture = BenchmarkFixture {
        id: fixture.id.clone(),
        description: fixture.description.clone(),
        input: fixture.input.clone(),
        recorded_candidate: fixture.recorded_candidate.clone(),
        expected_verdict: Verdict::Unknown,
        unsupported_propositions: vec![],
        hidden_assumption_propositions: vec![],
        expect_contradiction_finding: false,
        expect_counterexample_finding: false,
        bad_inference_ids: vec![],
        verification_receipts: fixture.verification_receipts.clone(),
    };
    let outcome = run_benchmark_harness(&benchmark_fixture, fixture.recorded_candidate.clone())?;
    let inspection = AssumptionInspector.inspect(&outcome.artifact);
    let statuses = inspection
        .assessments
        .iter()
        .map(|assessment| assessment.status)
        .collect::<Vec<_>>();
    let hard_findings = inspection
        .findings
        .iter()
        .filter(|finding| finding.strength == FindingStrength::Hard)
        .count();
    let soft_findings = inspection
        .findings
        .iter()
        .filter(|finding| finding.strength == FindingStrength::Soft)
        .count();
    let finding_inference_references = inspection
        .findings
        .iter()
        .map(|finding| finding.inference_ids.len())
        .collect::<Vec<_>>();
    let unsupported_detected = statuses
        .iter()
        .zip(&fixture.expected_statuses)
        .filter(|(actual, expected)| {
            **expected == AssumptionSupportStatus::Unsupported
                && **actual == AssumptionSupportStatus::Unsupported
        })
        .count();
    let expected_unsupported = fixture
        .expected_statuses
        .iter()
        .filter(|status| **status == AssumptionSupportStatus::Unsupported)
        .count();
    let explicit_assumptions_recognized = statuses
        .iter()
        .zip(&fixture.expected_statuses)
        .filter(|(actual, expected)| {
            **expected == AssumptionSupportStatus::ExplicitInputAssumption
                && **actual == AssumptionSupportStatus::ExplicitInputAssumption
        })
        .count();
    let expected_explicit_assumptions = fixture
        .expected_statuses
        .iter()
        .filter(|status| **status == AssumptionSupportStatus::ExplicitInputAssumption)
        .count();

    Ok(AssumptionBenchmarkCaseResult {
        fixture_id: fixture.id.clone(),
        expectations_met: statuses == fixture.expected_statuses
            && hard_findings == fixture.expected_hard_findings
            && soft_findings == fixture.expected_soft_findings
            && finding_inference_references == fixture.expected_finding_inference_references,
        statuses,
        hard_findings,
        soft_findings,
        finding_inference_references,
        unsupported_detected,
        expected_unsupported,
        explicit_assumptions_recognized,
        expected_explicit_assumptions,
    })
}

pub fn aggregate_assumption_benchmark(
    results: &[AssumptionBenchmarkCaseResult],
) -> AssumptionBenchmarkAggregate {
    let statuses = results
        .iter()
        .flat_map(|result| result.statuses.iter().copied())
        .collect::<Vec<_>>();
    let expected_unsupported = results
        .iter()
        .map(|result| result.expected_unsupported)
        .sum();
    let unsupported_detected = results
        .iter()
        .map(|result| result.unsupported_detected)
        .sum();
    let expected_explicit = results
        .iter()
        .map(|result| result.expected_explicit_assumptions)
        .sum();
    let explicit_recognized = results
        .iter()
        .map(|result| result.explicit_assumptions_recognized)
        .sum();

    AssumptionBenchmarkAggregate {
        cases: results.len(),
        passed_cases: results
            .iter()
            .filter(|result| result.expectations_met)
            .count(),
        supported_premises: count_status(&statuses, AssumptionSupportStatus::Supported),
        explicit_input_assumptions: count_status(
            &statuses,
            AssumptionSupportStatus::ExplicitInputAssumption,
        ),
        unsupported_premises: count_status(&statuses, AssumptionSupportStatus::Unsupported),
        unbound_premises: count_status(&statuses, AssumptionSupportStatus::Unbound),
        hard_findings: results.iter().map(|result| result.hard_findings).sum(),
        soft_findings: results.iter().map(|result| result.soft_findings).sum(),
        unsupported_detection_rate: rate(unsupported_detected, expected_unsupported),
        explicit_assumption_recognition_rate: rate(explicit_recognized, expected_explicit),
    }
}

fn count_status(statuses: &[AssumptionSupportStatus], target: AssumptionSupportStatus) -> usize {
    statuses.iter().filter(|status| **status == target).count()
}

fn rate(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        1.0
    } else {
        numerator as f64 / denominator as f64
    }
}
