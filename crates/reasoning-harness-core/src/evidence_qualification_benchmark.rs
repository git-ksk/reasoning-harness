use serde::{Deserialize, Serialize};

use crate::{
    EvidenceQualificationFindingReason, EvidenceQualificationInspector, EvidenceQualificationPass,
    EvidenceQualificationStatus, FindingStrength, HarnessError, HarnessInput, ReasoningCandidate,
    StrictAcceptancePolicy, run_harness,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceQualificationBenchmarkFixture {
    pub id: String,
    pub description: String,
    pub input: HarnessInput,
    #[serde(default)]
    pub expected_statuses: Vec<EvidenceQualificationStatus>,
    #[serde(default)]
    pub expected_reasons: Vec<EvidenceQualificationFindingReason>,
    pub expected_hard_findings: usize,
    pub expected_soft_findings: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EvidenceQualificationBenchmarkCaseResult {
    pub fixture_id: String,
    pub statuses: Vec<EvidenceQualificationStatus>,
    pub reasons: Vec<EvidenceQualificationFindingReason>,
    pub hard_findings: usize,
    pub soft_findings: usize,
    pub expected_reasons: usize,
    pub detected_expected_reasons: usize,
    pub expectations_met: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct EvidenceQualificationBenchmarkAggregate {
    pub cases: usize,
    pub passed_cases: usize,
    pub qualified_evidence: usize,
    pub disqualified_evidence: usize,
    pub unknown_evidence: usize,
    pub hard_findings: usize,
    pub soft_findings: usize,
    pub finding_reason_detection_rate: f64,
}

pub fn evaluate_evidence_qualification_fixture(
    fixture: &EvidenceQualificationBenchmarkFixture,
) -> Result<EvidenceQualificationBenchmarkCaseResult, HarnessError> {
    let passes: Vec<Box<dyn crate::Pass>> = vec![Box::new(EvidenceQualificationPass)];
    let outcome = run_harness(
        fixture.input.clone(),
        ReasoningCandidate::default(),
        &passes,
        &StrictAcceptancePolicy,
    )?;
    let inspection = EvidenceQualificationInspector.inspect(&outcome.artifact);
    let statuses = inspection
        .assessments
        .iter()
        .map(|assessment| assessment.status)
        .collect::<Vec<_>>();
    let mut reasons = inspection
        .findings
        .iter()
        .map(|finding| finding.reason)
        .collect::<Vec<_>>();
    reasons.sort();
    let mut expected_reasons = fixture.expected_reasons.clone();
    expected_reasons.sort();
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
    let detected_expected_reasons = expected_reasons
        .iter()
        .filter(|reason| reasons.contains(reason))
        .count();

    Ok(EvidenceQualificationBenchmarkCaseResult {
        fixture_id: fixture.id.clone(),
        expectations_met: statuses == fixture.expected_statuses
            && reasons == expected_reasons
            && hard_findings == fixture.expected_hard_findings
            && soft_findings == fixture.expected_soft_findings,
        statuses,
        reasons,
        hard_findings,
        soft_findings,
        expected_reasons: expected_reasons.len(),
        detected_expected_reasons,
    })
}

pub fn aggregate_evidence_qualification_benchmark(
    results: &[EvidenceQualificationBenchmarkCaseResult],
) -> EvidenceQualificationBenchmarkAggregate {
    let statuses = results
        .iter()
        .flat_map(|result| result.statuses.iter().copied())
        .collect::<Vec<_>>();
    let expected_reasons = results.iter().map(|result| result.expected_reasons).sum();
    let detected_reasons = results
        .iter()
        .map(|result| result.detected_expected_reasons)
        .sum();

    EvidenceQualificationBenchmarkAggregate {
        cases: results.len(),
        passed_cases: results
            .iter()
            .filter(|result| result.expectations_met)
            .count(),
        qualified_evidence: count_status(&statuses, EvidenceQualificationStatus::Qualified),
        disqualified_evidence: count_status(&statuses, EvidenceQualificationStatus::Disqualified),
        unknown_evidence: count_status(&statuses, EvidenceQualificationStatus::Unknown),
        hard_findings: results.iter().map(|result| result.hard_findings).sum(),
        soft_findings: results.iter().map(|result| result.soft_findings).sum(),
        finding_reason_detection_rate: rate(detected_reasons, expected_reasons),
    }
}

fn count_status(
    statuses: &[EvidenceQualificationStatus],
    target: EvidenceQualificationStatus,
) -> usize {
    statuses.iter().filter(|status| **status == target).count()
}

fn rate(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        1.0
    } else {
        numerator as f64 / denominator as f64
    }
}
