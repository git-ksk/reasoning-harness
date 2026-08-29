use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::{
    AcceptancePolicy, AdversarialDiscoveryPass, AdversarialFindingKind, Claim, EpistemicState,
    FindingStrength, HarnessInput, ReasoningArtifact, ReasoningCandidate, StrictAcceptancePolicy,
    StructuredFactConflictDetector, StructuredFactVerifier, TrustedVerificationPass, Verdict,
    VerificationPass, evaluate, frameworks::five_whys::FiveWhysRestatementPass, run_harness,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BenchmarkFixture {
    pub id: String,
    pub description: String,
    pub input: HarnessInput,
    pub recorded_candidate: ReasoningCandidate,
    pub expected_verdict: Verdict,
    #[serde(default)]
    pub unsupported_claim_ids: Vec<String>,
    #[serde(default)]
    pub hidden_assumption_claim_ids: Vec<String>,
    #[serde(default)]
    pub contradiction_claim_ids: Vec<String>,
    #[serde(default)]
    pub counterexample_claim_ids: Vec<String>,
    #[serde(default)]
    pub bad_inference_ids: Vec<String>,
    #[serde(default)]
    pub verification_receipts: Vec<crate::VerificationReceipt>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BenchmarkArmResult {
    pub verdict: Option<Verdict>,
    pub claims: usize,
    pub claims_with_evidence: usize,
    pub inference_edges: usize,
    pub verdict_correct: bool,
    pub evidence_coverage: f64,
    pub unsupported_accepted_claims: usize,
    pub hidden_assumptions_exposed: usize,
    pub contradiction_claims_detected: usize,
    pub counterexamples_detected: usize,
    pub hard_adversarial_findings: usize,
    pub soft_adversarial_findings: usize,
    pub bad_inference_edges_retained: usize,
    pub deterministic_failure: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deterministic_failure_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BenchmarkCaseResult {
    pub fixture_id: String,
    pub expected_verdict: Verdict,
    pub expected_hidden_assumptions: usize,
    pub expected_contradictions: usize,
    pub expected_counterexamples: usize,
    pub baseline: BenchmarkArmResult,
    pub harness: BenchmarkArmResult,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BenchmarkAggregate {
    pub cases: usize,
    pub verdict_accuracy: f64,
    pub accept_recall: f64,
    pub reject_recall: f64,
    pub unknown_recall: f64,
    pub evidence_coverage: f64,
    pub unsupported_accepted_claims: usize,
    pub hidden_assumption_exposure_rate: f64,
    pub contradiction_detection_rate: f64,
    pub counterexample_detection_rate: f64,
    pub bad_inference_edges_retained: usize,
    pub causal_edge_quality: f64,
    pub deterministic_verifier_failure_rate: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BenchmarkComparison {
    pub baseline: BenchmarkAggregate,
    pub harness: BenchmarkAggregate,
}

pub fn evaluate_benchmark_fixture(
    fixture: &BenchmarkFixture,
    candidate: ReasoningCandidate,
) -> BenchmarkCaseResult {
    let baseline_artifact = naive_materialize(fixture.input.clone(), candidate.clone());
    let baseline_verdict = StrictAcceptancePolicy.decide(&baseline_artifact);
    let baseline = arm_result(
        fixture,
        Some(&baseline_artifact),
        Some(baseline_verdict),
        false,
        None,
    );

    let passes: Vec<Box<dyn crate::Pass>> = vec![
        Box::new(AdversarialDiscoveryPass::new(vec![Box::new(
            StructuredFactConflictDetector,
        )])),
        Box::new(VerificationPass::new(vec![Box::new(
            StructuredFactVerifier,
        )])),
        Box::new(TrustedVerificationPass::new(
            fixture.verification_receipts.clone(),
        )),
        Box::new(FiveWhysRestatementPass),
    ];
    let harness_run = run_harness(
        fixture.input.clone(),
        candidate,
        &passes,
        &StrictAcceptancePolicy,
    );
    let harness = match harness_run {
        Ok(outcome) => arm_result(
            fixture,
            Some(&outcome.artifact),
            Some(outcome.verdict),
            false,
            None,
        ),
        Err(error) => arm_result(fixture, None, None, true, Some(error.to_string())),
    };

    BenchmarkCaseResult {
        fixture_id: fixture.id.clone(),
        expected_verdict: fixture.expected_verdict,
        expected_hidden_assumptions: fixture.hidden_assumption_claim_ids.len(),
        expected_contradictions: fixture.contradiction_claim_ids.len(),
        expected_counterexamples: fixture.counterexample_claim_ids.len(),
        baseline,
        harness,
    }
}

pub fn aggregate_benchmark(results: &[BenchmarkCaseResult]) -> BenchmarkComparison {
    BenchmarkComparison {
        baseline: aggregate_arm(results, |result| &result.baseline),
        harness: aggregate_arm(results, |result| &result.harness),
    }
}

fn naive_materialize(input: HarnessInput, candidate: ReasoningCandidate) -> ReasoningArtifact {
    ReasoningArtifact {
        task: input.task,
        evidence: input.evidence,
        candidate_diagnostics: Vec::new(),
        verification_receipts: Vec::new(),
        adversarial_findings: Vec::new(),
        claims: candidate
            .claims
            .into_iter()
            .map(|claim| Claim {
                id: claim.id,
                statement: claim.statement,
                state: claim.proposed_state,
                proposition: claim.proposition,
                evidence_ids: claim.evidence_ids,
            })
            .collect(),
        inferences: candidate.inferences,
    }
}

fn arm_result(
    fixture: &BenchmarkFixture,
    artifact: Option<&ReasoningArtifact>,
    verdict: Option<Verdict>,
    deterministic_failure: bool,
    deterministic_failure_reason: Option<String>,
) -> BenchmarkArmResult {
    let unsupported_ids: HashSet<&str> = fixture
        .unsupported_claim_ids
        .iter()
        .map(String::as_str)
        .collect();
    let assumption_ids: HashSet<&str> = fixture
        .hidden_assumption_claim_ids
        .iter()
        .map(String::as_str)
        .collect();
    let contradiction_ids: HashSet<&str> = fixture
        .contradiction_claim_ids
        .iter()
        .map(String::as_str)
        .collect();
    let counterexample_ids: HashSet<&str> = fixture
        .counterexample_claim_ids
        .iter()
        .map(String::as_str)
        .collect();
    let bad_inference_ids: HashSet<&str> = fixture
        .bad_inference_ids
        .iter()
        .map(String::as_str)
        .collect();

    let claims = artifact.map(|artifact| artifact.claims.len()).unwrap_or(0);
    let claims_with_evidence = artifact
        .map(|artifact| {
            artifact
                .claims
                .iter()
                .filter(|claim| !claim.evidence_ids.is_empty())
                .count()
        })
        .unwrap_or(0);
    let inference_edges = artifact
        .map(|artifact| artifact.inferences.len())
        .unwrap_or(0);
    let evidence_coverage = artifact
        .map(|artifact| evaluate(artifact).evidence_coverage)
        .unwrap_or(0.0);
    let unsupported_accepted_claims = artifact
        .map(|artifact| {
            artifact
                .claims
                .iter()
                .filter(|claim| {
                    unsupported_ids.contains(claim.id.as_str())
                        && matches!(
                            claim.state,
                            EpistemicState::Known
                                | EpistemicState::Supported
                                | EpistemicState::Inferred
                        )
                })
                .count()
        })
        .unwrap_or(0);
    let hidden_assumptions_exposed = artifact
        .map(|artifact| {
            artifact
                .claims
                .iter()
                .filter(|claim| {
                    assumption_ids.contains(claim.id.as_str())
                        && matches!(
                            claim.state,
                            EpistemicState::Assumed
                                | EpistemicState::Unknown
                                | EpistemicState::Contradicted
                        )
                })
                .count()
        })
        .unwrap_or(0);
    let contradiction_claims_detected = artifact
        .map(|artifact| {
            if verdict == Some(Verdict::Reject) {
                contradiction_ids.len()
            } else {
                artifact
                    .claims
                    .iter()
                    .filter(|claim| {
                        contradiction_ids.contains(claim.id.as_str())
                            && claim.state == EpistemicState::Contradicted
                    })
                    .count()
            }
        })
        .unwrap_or(0);
    let bad_inference_edges_retained = artifact
        .map(|artifact| {
            artifact
                .inferences
                .iter()
                .filter(|inference| bad_inference_ids.contains(inference.id.as_str()))
                .count()
        })
        .unwrap_or(0);
    let counterexamples_detected = artifact
        .map(|artifact| {
            artifact
                .adversarial_findings
                .iter()
                .filter(|finding| {
                    finding.kind == AdversarialFindingKind::Counterexample
                        && counterexample_ids.contains(finding.claim_id.as_str())
                })
                .count()
        })
        .unwrap_or(0);
    let hard_adversarial_findings = artifact
        .map(|artifact| {
            artifact
                .adversarial_findings
                .iter()
                .filter(|finding| finding.strength == FindingStrength::Hard)
                .count()
        })
        .unwrap_or(0);
    let soft_adversarial_findings = artifact
        .map(|artifact| {
            artifact
                .adversarial_findings
                .iter()
                .filter(|finding| finding.strength == FindingStrength::Soft)
                .count()
        })
        .unwrap_or(0);

    BenchmarkArmResult {
        verdict,
        claims,
        claims_with_evidence,
        inference_edges,
        verdict_correct: verdict == Some(fixture.expected_verdict),
        evidence_coverage,
        unsupported_accepted_claims,
        hidden_assumptions_exposed,
        contradiction_claims_detected,
        counterexamples_detected,
        hard_adversarial_findings,
        soft_adversarial_findings,
        bad_inference_edges_retained,
        deterministic_failure,
        deterministic_failure_reason,
    }
}

fn aggregate_arm<'a>(
    results: &'a [BenchmarkCaseResult],
    select: impl Fn(&'a BenchmarkCaseResult) -> &'a BenchmarkArmResult,
) -> BenchmarkAggregate {
    let cases = results.len();
    let denominator = cases.max(1) as f64;
    let correct = results
        .iter()
        .filter(|result| select(result).verdict_correct)
        .count();
    let accept_cases = results
        .iter()
        .filter(|result| result.expected_verdict == Verdict::Accept)
        .count();
    let accept_correct = results
        .iter()
        .filter(|result| {
            result.expected_verdict == Verdict::Accept
                && select(result).verdict == Some(Verdict::Accept)
        })
        .count();
    let reject_cases = results
        .iter()
        .filter(|result| result.expected_verdict == Verdict::Reject)
        .count();
    let reject_correct = results
        .iter()
        .filter(|result| {
            result.expected_verdict == Verdict::Reject
                && select(result).verdict == Some(Verdict::Reject)
        })
        .count();
    let unknown_cases = results
        .iter()
        .filter(|result| result.expected_verdict == Verdict::Unknown)
        .count();
    let unknown_correct = results
        .iter()
        .filter(|result| {
            result.expected_verdict == Verdict::Unknown
                && select(result).verdict == Some(Verdict::Unknown)
        })
        .count();

    let total_claims: usize = results.iter().map(|result| select(result).claims).sum();
    let total_claims_with_evidence: usize = results
        .iter()
        .map(|result| select(result).claims_with_evidence)
        .sum();
    let evidence_coverage = if total_claims == 0 {
        0.0
    } else {
        total_claims_with_evidence as f64 / total_claims as f64
    };

    let unsupported_accepted_claims = results
        .iter()
        .map(|result| select(result).unsupported_accepted_claims)
        .sum();
    let hidden_exposed: usize = results
        .iter()
        .map(|result| select(result).hidden_assumptions_exposed)
        .sum();
    let expected_hidden: usize = results
        .iter()
        .map(|result| result.expected_hidden_assumptions)
        .sum();
    let hidden_assumption_exposure_rate = if expected_hidden == 0 {
        1.0
    } else {
        hidden_exposed as f64 / expected_hidden as f64
    };

    let contradiction_detected: usize = results
        .iter()
        .map(|result| select(result).contradiction_claims_detected)
        .sum();
    let expected_contradictions: usize = results
        .iter()
        .map(|result| result.expected_contradictions)
        .sum();
    let contradiction_detection_rate = if expected_contradictions == 0 {
        1.0
    } else {
        contradiction_detected as f64 / expected_contradictions as f64
    };

    let counterexamples_detected: usize = results
        .iter()
        .map(|result| select(result).counterexamples_detected)
        .sum();
    let expected_counterexamples: usize = results
        .iter()
        .map(|result| result.expected_counterexamples)
        .sum();
    let counterexample_detection_rate = if expected_counterexamples == 0 {
        1.0
    } else {
        counterexamples_detected as f64 / expected_counterexamples as f64
    };

    let total_inference_edges: usize = results
        .iter()
        .map(|result| select(result).inference_edges)
        .sum();
    let bad_inference_edges_retained: usize = results
        .iter()
        .map(|result| select(result).bad_inference_edges_retained)
        .sum();
    let causal_edge_quality = if total_inference_edges == 0 {
        1.0
    } else {
        (total_inference_edges.saturating_sub(bad_inference_edges_retained)) as f64
            / total_inference_edges as f64
    };

    let deterministic_failures = results
        .iter()
        .filter(|result| select(result).deterministic_failure)
        .count();

    BenchmarkAggregate {
        cases,
        verdict_accuracy: correct as f64 / denominator,
        accept_recall: if accept_cases == 0 {
            1.0
        } else {
            accept_correct as f64 / accept_cases as f64
        },
        reject_recall: if reject_cases == 0 {
            1.0
        } else {
            reject_correct as f64 / reject_cases as f64
        },
        unknown_recall: if unknown_cases == 0 {
            1.0
        } else {
            unknown_correct as f64 / unknown_cases as f64
        },
        evidence_coverage,
        unsupported_accepted_claims,
        hidden_assumption_exposure_rate,
        contradiction_detection_rate,
        counterexample_detection_rate,
        bad_inference_edges_retained,
        causal_edge_quality,
        deterministic_verifier_failure_rate: deterministic_failures as f64 / denominator,
    }
}
