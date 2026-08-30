use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    AcceptancePolicy, AdversarialFinding, BenchmarkFixture, CausalBenchmarkFixture, CausalFinding,
    CausalInputError, CausalInspector, CausalRelation, Evidence, FindingStrength, HarnessError,
    Proposition, StrictAcceptancePolicy, Verdict, benchmark::run_benchmark_harness,
};

/// A semantics-preserving transformation family used by deterministic robustness tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetamorphicTransformFamily {
    EvidenceOrder,
    InferenceOrder,
    StableIdRemap,
    IrrelevantEvidence,
    CausalCauseOrder,
    CausalEvidenceOrder,
}

/// Provider-neutral contract for deterministic transformations over benchmark fixtures.
///
/// Implementations may change representation only. They must not change the proposition-level
/// meaning or trusted oracle semantics of the supplied fixture.
pub trait MetamorphicTransform<T>: Send + Sync {
    fn family(&self) -> MetamorphicTransformFamily;
    fn apply(&self, fixture: &T) -> T;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ReverseEvidenceOrder;

impl MetamorphicTransform<BenchmarkFixture> for ReverseEvidenceOrder {
    fn family(&self) -> MetamorphicTransformFamily {
        MetamorphicTransformFamily::EvidenceOrder
    }

    fn apply(&self, fixture: &BenchmarkFixture) -> BenchmarkFixture {
        let mut transformed = fixture.clone();
        transformed.input.evidence.reverse();
        transformed
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ReverseInferenceOrder;

impl MetamorphicTransform<BenchmarkFixture> for ReverseInferenceOrder {
    fn family(&self) -> MetamorphicTransformFamily {
        MetamorphicTransformFamily::InferenceOrder
    }

    fn apply(&self, fixture: &BenchmarkFixture) -> BenchmarkFixture {
        let mut transformed = fixture.clone();
        transformed.recorded_candidate.inferences.reverse();
        transformed
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct StableIdRemap;

impl MetamorphicTransform<BenchmarkFixture> for StableIdRemap {
    fn family(&self) -> MetamorphicTransformFamily {
        MetamorphicTransformFamily::StableIdRemap
    }

    fn apply(&self, fixture: &BenchmarkFixture) -> BenchmarkFixture {
        let mut transformed = fixture.clone();

        let evidence_ids = transformed
            .input
            .evidence
            .iter()
            .map(|item| (item.id.clone(), format!("morph:evidence:{}", item.id)))
            .collect::<BTreeMap<_, _>>();
        let claim_ids = transformed
            .recorded_candidate
            .claims
            .iter()
            .map(|item| (item.id.clone(), format!("morph:claim:{}", item.id)))
            .collect::<BTreeMap<_, _>>();
        let inference_ids = transformed
            .recorded_candidate
            .inferences
            .iter()
            .map(|item| (item.id.clone(), format!("morph:inference:{}", item.id)))
            .collect::<BTreeMap<_, _>>();

        for evidence in &mut transformed.input.evidence {
            evidence.id = remap(&evidence_ids, &evidence.id);
        }
        for claim in &mut transformed.recorded_candidate.claims {
            claim.id = remap(&claim_ids, &claim.id);
            claim.evidence_ids = claim
                .evidence_ids
                .iter()
                .map(|id| remap(&evidence_ids, id))
                .collect();
        }
        for inference in &mut transformed.recorded_candidate.inferences {
            inference.id = remap(&inference_ids, &inference.id);
            inference.premise_claim_ids = inference
                .premise_claim_ids
                .iter()
                .map(|id| remap(&claim_ids, id))
                .collect();
            inference.conclusion_claim_id = remap(&claim_ids, &inference.conclusion_claim_id);
        }
        transformed.bad_inference_ids = transformed
            .bad_inference_ids
            .iter()
            .map(|id| remap(&inference_ids, id))
            .collect();
        for receipt in &mut transformed.verification_receipts {
            receipt.id = format!("morph:receipt:{}", receipt.id);
            receipt.claim_id = receipt.claim_id.as_ref().map(|id| remap(&claim_ids, id));
            receipt.evidence_ids = receipt
                .evidence_ids
                .iter()
                .map(|id| remap(&evidence_ids, id))
                .collect();
        }
        transformed
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct AddIrrelevantEvidence;

impl MetamorphicTransform<BenchmarkFixture> for AddIrrelevantEvidence {
    fn family(&self) -> MetamorphicTransformFamily {
        MetamorphicTransformFamily::IrrelevantEvidence
    }

    fn apply(&self, fixture: &BenchmarkFixture) -> BenchmarkFixture {
        let mut transformed = fixture.clone();
        let mut id = "metamorphic-irrelevant".to_string();
        let mut suffix = 1usize;
        while transformed.input.evidence.iter().any(|item| item.id == id) {
            id = format!("metamorphic-irrelevant-{suffix}");
            suffix += 1;
        }
        transformed.input.evidence.push(Evidence {
            id,
            source: "metamorphic control".into(),
            observation: "An explicitly unrelated control fact is present.".into(),
            facts: BTreeMap::from([("metamorphic.irrelevant".into(), "true".into())]),
        });
        transformed
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ReverseCausalCauseOrder;

impl MetamorphicTransform<CausalBenchmarkFixture> for ReverseCausalCauseOrder {
    fn family(&self) -> MetamorphicTransformFamily {
        MetamorphicTransformFamily::CausalCauseOrder
    }

    fn apply(&self, fixture: &CausalBenchmarkFixture) -> CausalBenchmarkFixture {
        let mut transformed = fixture.clone();
        for inference in &mut transformed.artifact.inferences {
            if inference.method == "causal_forward" {
                inference.premise_claim_ids.reverse();
            }
        }
        for evidence in &mut transformed.evidence {
            evidence.relation.causes.reverse();
        }
        transformed
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ReverseCausalEvidenceOrder;

impl MetamorphicTransform<CausalBenchmarkFixture> for ReverseCausalEvidenceOrder {
    fn family(&self) -> MetamorphicTransformFamily {
        MetamorphicTransformFamily::CausalEvidenceOrder
    }

    fn apply(&self, fixture: &CausalBenchmarkFixture) -> CausalBenchmarkFixture {
        let mut transformed = fixture.clone();
        transformed.evidence.reverse();
        transformed
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MetamorphicCaseResult {
    pub fixture_id: String,
    pub transform: MetamorphicTransformFamily,
    pub final_verdict_invariant: bool,
    pub hard_findings_invariant: bool,
    pub soft_findings_stable: bool,
    pub diagnostic_status_invariant: bool,
    pub changed_diagnostic_ids: Vec<String>,
    pub changed_diagnostic_reasons: Vec<String>,
}

impl MetamorphicCaseResult {
    pub fn hard_outcomes_invariant(&self) -> bool {
        self.final_verdict_invariant
            && self.hard_findings_invariant
            && self.diagnostic_status_invariant
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MetamorphicAggregate {
    pub transformations: usize,
    pub final_verdict_invariance_rate: f64,
    pub hard_finding_invariance_rate: f64,
    pub soft_finding_stability_rate: f64,
    pub diagnostic_status_invariance_rate: f64,
    pub hard_outcome_invariance_rate: f64,
    pub failures_by_transform: BTreeMap<MetamorphicTransformFamily, usize>,
}

#[derive(Debug, Error)]
pub enum MetamorphicEvaluationError {
    #[error("benchmark harness failed during metamorphic evaluation: {0}")]
    Harness(#[from] HarnessError),
    #[error("causal fixture failed during metamorphic evaluation: {0}")]
    Causal(#[from] CausalInputError),
}

pub fn evaluate_benchmark_metamorphic<T>(
    fixture: &BenchmarkFixture,
    transform: &T,
) -> Result<MetamorphicCaseResult, MetamorphicEvaluationError>
where
    T: MetamorphicTransform<BenchmarkFixture>,
{
    let base = benchmark_snapshot(fixture)?;
    let transformed_fixture = transform.apply(fixture);
    let transformed = benchmark_snapshot(&transformed_fixture)?;
    Ok(compare_snapshots(
        &fixture.id,
        transform.family(),
        base,
        transformed,
    ))
}

pub fn evaluate_causal_metamorphic<T>(
    fixture: &CausalBenchmarkFixture,
    transform: &T,
) -> Result<MetamorphicCaseResult, MetamorphicEvaluationError>
where
    T: MetamorphicTransform<CausalBenchmarkFixture>,
{
    let base = causal_snapshot(fixture)?;
    let transformed_fixture = transform.apply(fixture);
    let transformed = causal_snapshot(&transformed_fixture)?;
    Ok(compare_snapshots(
        &fixture.id,
        transform.family(),
        base,
        transformed,
    ))
}

pub fn aggregate_metamorphic(results: &[MetamorphicCaseResult]) -> MetamorphicAggregate {
    let denominator = results.len().max(1) as f64;
    let rate = |predicate: fn(&MetamorphicCaseResult) -> bool| {
        results.iter().filter(|result| predicate(result)).count() as f64 / denominator
    };
    let mut failures_by_transform = BTreeMap::new();
    for result in results {
        let failures = failures_by_transform.entry(result.transform).or_insert(0);
        if !result.hard_outcomes_invariant() {
            *failures += 1;
        }
    }

    MetamorphicAggregate {
        transformations: results.len(),
        final_verdict_invariance_rate: rate(|result| result.final_verdict_invariant),
        hard_finding_invariance_rate: rate(|result| result.hard_findings_invariant),
        soft_finding_stability_rate: rate(|result| result.soft_findings_stable),
        diagnostic_status_invariance_rate: rate(|result| result.diagnostic_status_invariant),
        hard_outcome_invariance_rate: rate(MetamorphicCaseResult::hard_outcomes_invariant),
        failures_by_transform,
    }
}

#[derive(Debug, Clone)]
struct DiagnosticSnapshot {
    verdict: Verdict,
    hard_findings: Vec<String>,
    soft_findings: Vec<String>,
    diagnostic_statuses: Vec<String>,
    diagnostic_ids: BTreeSet<String>,
    diagnostic_reasons: BTreeSet<String>,
}

fn benchmark_snapshot(
    fixture: &BenchmarkFixture,
) -> Result<DiagnosticSnapshot, MetamorphicEvaluationError> {
    let outcome = run_benchmark_harness(fixture, fixture.recorded_candidate.clone())?;
    let mut hard_findings = Vec::new();
    let mut soft_findings = Vec::new();
    let mut diagnostic_ids = BTreeSet::new();
    let mut diagnostic_reasons = outcome
        .artifact
        .candidate_diagnostics
        .iter()
        .map(|diagnostic| format!("candidate:{}", diagnostic.code))
        .collect::<BTreeSet<_>>();

    for finding in &outcome.artifact.adversarial_findings {
        diagnostic_ids.insert(finding.id.clone());
        diagnostic_reasons.insert(format!("adversarial:{:?}", finding.kind));
        let signature = adversarial_signature(finding);
        match finding.strength {
            FindingStrength::Hard => hard_findings.push(signature),
            FindingStrength::Soft => soft_findings.push(signature),
        }
    }
    hard_findings.sort();
    soft_findings.sort();

    Ok(DiagnosticSnapshot {
        verdict: outcome.verdict,
        hard_findings,
        soft_findings,
        diagnostic_statuses: Vec::new(),
        diagnostic_ids,
        diagnostic_reasons,
    })
}

fn causal_snapshot(
    fixture: &CausalBenchmarkFixture,
) -> Result<DiagnosticSnapshot, MetamorphicEvaluationError> {
    let inspection = CausalInspector::new(fixture.evidence.clone())?.inspect(&fixture.artifact);
    let mut hard_findings = Vec::new();
    let mut soft_findings = Vec::new();
    let mut diagnostic_ids = BTreeSet::new();
    let mut diagnostic_reasons = BTreeSet::new();
    let mut diagnostic_statuses = inspection
        .assessments
        .iter()
        .map(|assessment| {
            let relation = assessment
                .relation
                .as_ref()
                .map(relation_signature)
                .unwrap_or_else(|| "unbound".into());
            format!("{:?}|{relation}", assessment.status)
        })
        .collect::<Vec<_>>();

    for finding in &inspection.findings {
        diagnostic_ids.insert(finding.id.clone());
        diagnostic_reasons.insert(format!("causal:{:?}", finding.reason));
        let signature = causal_finding_signature(finding);
        match finding.strength {
            FindingStrength::Hard => hard_findings.push(signature),
            FindingStrength::Soft => soft_findings.push(signature),
        }
    }
    for assessment in &inspection.assessments {
        diagnostic_reasons.insert(format!("causal_status:{:?}", assessment.status));
    }
    hard_findings.sort();
    soft_findings.sort();
    diagnostic_statuses.sort();

    Ok(DiagnosticSnapshot {
        verdict: StrictAcceptancePolicy.decide(&fixture.artifact),
        hard_findings,
        soft_findings,
        diagnostic_statuses,
        diagnostic_ids,
        diagnostic_reasons,
    })
}

fn compare_snapshots(
    fixture_id: &str,
    transform: MetamorphicTransformFamily,
    base: DiagnosticSnapshot,
    transformed: DiagnosticSnapshot,
) -> MetamorphicCaseResult {
    MetamorphicCaseResult {
        fixture_id: fixture_id.into(),
        transform,
        final_verdict_invariant: base.verdict == transformed.verdict,
        hard_findings_invariant: base.hard_findings == transformed.hard_findings,
        soft_findings_stable: base.soft_findings == transformed.soft_findings,
        diagnostic_status_invariant: base.diagnostic_statuses == transformed.diagnostic_statuses,
        changed_diagnostic_ids: symmetric_difference(
            &base.diagnostic_ids,
            &transformed.diagnostic_ids,
        ),
        changed_diagnostic_reasons: symmetric_difference(
            &base.diagnostic_reasons,
            &transformed.diagnostic_reasons,
        ),
    }
}

fn adversarial_signature(finding: &AdversarialFinding) -> String {
    format!(
        "{}|{:?}|{}={}|{}",
        finding.detector,
        finding.kind,
        finding.proposition.key,
        finding.proposition.value,
        finding.message
    )
}

fn causal_finding_signature(finding: &CausalFinding) -> String {
    let relation = finding
        .relation
        .as_ref()
        .map(relation_signature)
        .unwrap_or_else(|| "unbound".into());
    format!(
        "{}|{:?}|{:?}|{relation}|{}",
        finding.detector, finding.kind, finding.reason, finding.message
    )
}

fn relation_signature(relation: &CausalRelation) -> String {
    let mut causes = relation
        .causes
        .iter()
        .map(proposition_signature)
        .collect::<Vec<_>>();
    causes.sort();
    format!(
        "{}->{}",
        causes.join("&"),
        proposition_signature(&relation.effect)
    )
}

fn proposition_signature(proposition: &Proposition) -> String {
    format!("{}={}", proposition.key, proposition.value)
}

fn symmetric_difference(left: &BTreeSet<String>, right: &BTreeSet<String>) -> Vec<String> {
    left.symmetric_difference(right).cloned().collect()
}

fn remap(mapping: &BTreeMap<String, String>, id: &str) -> String {
    mapping.get(id).cloned().unwrap_or_else(|| id.to_string())
}
