use serde::{Deserialize, Serialize};

use crate::{
    CausalEvidence, CausalInputError, CausalInspector, CausalSupportStatus, FindingStrength,
    ReasoningArtifact,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CausalBenchmarkFixture {
    pub id: String,
    pub description: String,
    pub artifact: ReasoningArtifact,
    #[serde(default)]
    pub evidence: Vec<CausalEvidence>,
    #[serde(default)]
    pub expected_statuses: Vec<CausalSupportStatus>,
    pub expected_hard_findings: usize,
    pub expected_soft_findings: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CausalBenchmarkCaseResult {
    pub fixture_id: String,
    pub edge_assessments: usize,
    pub supported_edges: usize,
    pub refuted_edges: usize,
    pub unknown_edges: usize,
    pub hard_findings: usize,
    pub soft_findings: usize,
    pub expectations_met: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct CausalBenchmarkAggregate {
    pub cases: usize,
    pub passed_cases: usize,
    pub edge_assessments: usize,
    pub supported_edges: usize,
    pub refuted_edges: usize,
    pub unknown_edges: usize,
    pub hard_findings: usize,
    pub soft_findings: usize,
}

pub fn evaluate_causal_fixture(
    fixture: &CausalBenchmarkFixture,
) -> Result<CausalBenchmarkCaseResult, CausalInputError> {
    let inspection = CausalInspector::new(fixture.evidence.clone())?.inspect(&fixture.artifact);
    let statuses = inspection
        .assessments
        .iter()
        .map(|assessment| assessment.status)
        .collect::<Vec<_>>();
    let supported_edges = statuses
        .iter()
        .filter(|status| **status == CausalSupportStatus::Supported)
        .count();
    let refuted_edges = statuses
        .iter()
        .filter(|status| **status == CausalSupportStatus::Refuted)
        .count();
    let unknown_edges = statuses
        .iter()
        .filter(|status| **status == CausalSupportStatus::Unknown)
        .count();
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

    Ok(CausalBenchmarkCaseResult {
        fixture_id: fixture.id.clone(),
        edge_assessments: statuses.len(),
        supported_edges,
        refuted_edges,
        unknown_edges,
        hard_findings,
        soft_findings,
        expectations_met: statuses == fixture.expected_statuses
            && hard_findings == fixture.expected_hard_findings
            && soft_findings == fixture.expected_soft_findings,
    })
}

pub fn aggregate_causal_benchmark(
    results: &[CausalBenchmarkCaseResult],
) -> CausalBenchmarkAggregate {
    CausalBenchmarkAggregate {
        cases: results.len(),
        passed_cases: results
            .iter()
            .filter(|result| result.expectations_met)
            .count(),
        edge_assessments: results.iter().map(|result| result.edge_assessments).sum(),
        supported_edges: results.iter().map(|result| result.supported_edges).sum(),
        refuted_edges: results.iter().map(|result| result.refuted_edges).sum(),
        unknown_edges: results.iter().map(|result| result.unknown_edges).sum(),
        hard_findings: results.iter().map(|result| result.hard_findings).sum(),
        soft_findings: results.iter().map(|result| result.soft_findings).sum(),
    }
}
