use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    AdversarialFindingKind, AssumptionFindingKind, CausalFindingKind, CausalFindingReason,
    CausalInspection, CausalRelation, CausalSupportStatus, EvidenceQualificationFindingKind,
    EvidenceQualificationFindingReason, FindingStrength, Proposition, ReasoningArtifact,
};

const MIN_CI_OBSERVATIONS: usize = 5;
const WILSON_95_Z: f64 = 1.959_963_984_540_054;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "family", rename_all = "snake_case")]
pub enum DiagnosticSignal {
    Adversarial {
        detector: String,
        kind: AdversarialFindingKind,
        strength: FindingStrength,
        proposition: Proposition,
    },
    Causal {
        detector: String,
        kind: CausalFindingKind,
        reason: CausalFindingReason,
        strength: FindingStrength,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        relation: Option<CausalRelation>,
    },
    CausalAssessment {
        status: CausalSupportStatus,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        relation: Option<CausalRelation>,
    },
    Assumption {
        detector: String,
        kind: AssumptionFindingKind,
        strength: FindingStrength,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        proposition: Option<Proposition>,
    },
    EvidenceQualification {
        detector: String,
        kind: EvidenceQualificationFindingKind,
        reason: EvidenceQualificationFindingReason,
        strength: FindingStrength,
        proposition: Proposition,
    },
    Candidate {
        code: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticObservation {
    pub fixture_id: String,
    #[serde(default)]
    pub signals: Vec<DiagnosticSignal>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticTrial {
    pub trial_index: usize,
    pub operationally_complete: bool,
    pub operational_failures: usize,
    #[serde(default)]
    pub observations: Vec<DiagnosticObservation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceIntervalMethod {
    WilsonScore,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProportionConfidenceInterval {
    pub method: ConfidenceIntervalMethod,
    pub confidence_level: f64,
    pub lower: f64,
    pub upper: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiagnosticFrequency {
    pub signal: DiagnosticSignal,
    pub occurrences: usize,
    pub denominator: usize,
    pub frequency: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence_interval_95: Option<ProportionConfidenceInterval>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiagnosticCountDistribution {
    pub count: usize,
    pub mean: f64,
    pub min: f64,
    pub max: f64,
    pub stddev: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiagnosticFamilyDistributions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adversarial: Option<DiagnosticCountDistribution>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub causal: Option<DiagnosticCountDistribution>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assumption: Option<DiagnosticCountDistribution>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_qualification: Option<DiagnosticCountDistribution>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidate: Option<DiagnosticCountDistribution>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FixtureDiagnosticStability {
    pub fixture_id: String,
    pub complete_trial_observations: usize,
    pub missing_complete_trial_observations: usize,
    pub frequencies: Vec<DiagnosticFrequency>,
    pub count_distributions: DiagnosticFamilyDistributions,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepeatedDiagnosticReport {
    pub requested_trials: usize,
    pub complete_trials: usize,
    pub incomplete_trials: usize,
    pub operational_failures: usize,
    pub excluded_incomplete_trial_observations: usize,
    pub fixtures: Vec<FixtureDiagnosticStability>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DiagnosticStabilityError {
    #[error("duplicate diagnostic trial index: {trial_index}")]
    DuplicateTrialIndex { trial_index: usize },
    #[error("complete diagnostic trial {trial_index} reports operational failures")]
    CompleteTrialHasFailures { trial_index: usize },
    #[error("diagnostic observation fixture id must not be empty")]
    EmptyFixtureId,
    #[error("duplicate fixture {fixture_id} in diagnostic trial {trial_index}")]
    DuplicateFixtureObservation {
        trial_index: usize,
        fixture_id: String,
    },
}

pub fn observe_diagnostics(
    fixture_id: impl Into<String>,
    artifact: &ReasoningArtifact,
    causal: Option<&CausalInspection>,
) -> DiagnosticObservation {
    let mut signals = Vec::new();

    signals.extend(artifact.adversarial_findings.iter().map(|finding| {
        DiagnosticSignal::Adversarial {
            detector: finding.detector.clone(),
            kind: finding.kind,
            strength: finding.strength,
            proposition: finding.proposition.clone(),
        }
    }));
    signals.extend(artifact.assumption_findings.iter().map(|finding| {
        DiagnosticSignal::Assumption {
            detector: finding.detector.clone(),
            kind: finding.kind,
            strength: finding.strength,
            proposition: finding.proposition.clone(),
        }
    }));
    signals.extend(
        artifact
            .evidence_qualification_findings
            .iter()
            .map(|finding| DiagnosticSignal::EvidenceQualification {
                detector: finding.detector.clone(),
                kind: finding.kind,
                reason: finding.reason,
                strength: finding.strength,
                proposition: finding.proposition.clone(),
            }),
    );
    signals.extend(artifact.candidate_diagnostics.iter().map(|diagnostic| {
        DiagnosticSignal::Candidate {
            code: diagnostic.code.clone(),
        }
    }));
    if let Some(causal) = causal {
        signals.extend(causal.assessments.iter().map(|assessment| {
            DiagnosticSignal::CausalAssessment {
                status: assessment.status,
                relation: assessment.relation.as_ref().map(canonical_relation),
            }
        }));
        signals.extend(
            causal
                .findings
                .iter()
                .map(|finding| DiagnosticSignal::Causal {
                    detector: finding.detector.clone(),
                    kind: finding.kind,
                    reason: finding.reason,
                    strength: finding.strength,
                    relation: finding.relation.as_ref().map(canonical_relation),
                }),
        );
    }
    signals.sort_by_key(signal_sort_key);

    DiagnosticObservation {
        fixture_id: fixture_id.into(),
        signals,
    }
}

pub fn aggregate_repeated_diagnostics(
    trials: &[DiagnosticTrial],
) -> Result<RepeatedDiagnosticReport, DiagnosticStabilityError> {
    validate_trials(trials)?;

    let requested_trials = trials.len();
    let complete_trials = trials
        .iter()
        .filter(|trial| trial.operationally_complete)
        .count();
    let incomplete_trials = requested_trials.saturating_sub(complete_trials);
    let operational_failures = trials.iter().map(|trial| trial.operational_failures).sum();
    let excluded_incomplete_trial_observations = trials
        .iter()
        .filter(|trial| !trial.operationally_complete)
        .map(|trial| trial.observations.len())
        .sum();

    let fixture_ids = trials
        .iter()
        .flat_map(|trial| trial.observations.iter())
        .map(|observation| observation.fixture_id.clone())
        .collect::<BTreeSet<_>>();
    let mut fixtures = Vec::with_capacity(fixture_ids.len());

    for fixture_id in fixture_ids {
        let observations = trials
            .iter()
            .filter(|trial| trial.operationally_complete)
            .filter_map(|trial| {
                trial
                    .observations
                    .iter()
                    .find(|observation| observation.fixture_id == fixture_id)
            })
            .collect::<Vec<_>>();
        let denominator = observations.len();
        let missing_complete_trial_observations = complete_trials.saturating_sub(denominator);

        let mut frequencies = BTreeMap::<String, (DiagnosticSignal, usize)>::new();
        for observation in &observations {
            let unique = observation
                .signals
                .iter()
                .map(|signal| (signal_sort_key(signal), signal.clone()))
                .collect::<BTreeMap<_, _>>();
            for (key, signal) in unique {
                let entry = frequencies.entry(key).or_insert((signal, 0));
                entry.1 += 1;
            }
        }
        let frequencies = frequencies
            .into_values()
            .map(|(signal, occurrences)| DiagnosticFrequency {
                signal,
                occurrences,
                denominator,
                frequency: if denominator == 0 {
                    0.0
                } else {
                    occurrences as f64 / denominator as f64
                },
                confidence_interval_95: wilson_95(occurrences, denominator),
            })
            .collect();

        let family_counts = |family: fn(&DiagnosticSignal) -> bool| {
            observations.iter().map(move |observation| {
                observation
                    .signals
                    .iter()
                    .filter(|signal| family(signal))
                    .count() as f64
            })
        };
        fixtures.push(FixtureDiagnosticStability {
            fixture_id,
            complete_trial_observations: denominator,
            missing_complete_trial_observations,
            frequencies,
            count_distributions: DiagnosticFamilyDistributions {
                adversarial: scalar_distribution(family_counts(is_adversarial)),
                causal: scalar_distribution(family_counts(is_causal)),
                assumption: scalar_distribution(family_counts(is_assumption)),
                evidence_qualification: scalar_distribution(family_counts(
                    is_evidence_qualification,
                )),
                candidate: scalar_distribution(family_counts(is_candidate)),
            },
        });
    }

    Ok(RepeatedDiagnosticReport {
        requested_trials,
        complete_trials,
        incomplete_trials,
        operational_failures,
        excluded_incomplete_trial_observations,
        fixtures,
    })
}

pub fn wilson_95(successes: usize, denominator: usize) -> Option<ProportionConfidenceInterval> {
    if denominator < MIN_CI_OBSERVATIONS || successes > denominator {
        return None;
    }
    let n = denominator as f64;
    let p = successes as f64 / n;
    let z2 = WILSON_95_Z * WILSON_95_Z;
    let denominator_term = 1.0 + z2 / n;
    let center = (p + z2 / (2.0 * n)) / denominator_term;
    let margin = WILSON_95_Z * ((p * (1.0 - p) + z2 / (4.0 * n)) / n).sqrt() / denominator_term;
    Some(ProportionConfidenceInterval {
        method: ConfidenceIntervalMethod::WilsonScore,
        confidence_level: 0.95,
        lower: (center - margin).max(0.0),
        upper: (center + margin).min(1.0),
    })
}

fn validate_trials(trials: &[DiagnosticTrial]) -> Result<(), DiagnosticStabilityError> {
    let mut trial_indices = BTreeSet::new();
    for trial in trials {
        if !trial_indices.insert(trial.trial_index) {
            return Err(DiagnosticStabilityError::DuplicateTrialIndex {
                trial_index: trial.trial_index,
            });
        }
        if trial.operationally_complete && trial.operational_failures > 0 {
            return Err(DiagnosticStabilityError::CompleteTrialHasFailures {
                trial_index: trial.trial_index,
            });
        }
        let mut fixture_ids = BTreeSet::new();
        for observation in &trial.observations {
            if observation.fixture_id.trim().is_empty() {
                return Err(DiagnosticStabilityError::EmptyFixtureId);
            }
            if !fixture_ids.insert(observation.fixture_id.as_str()) {
                return Err(DiagnosticStabilityError::DuplicateFixtureObservation {
                    trial_index: trial.trial_index,
                    fixture_id: observation.fixture_id.clone(),
                });
            }
        }
    }
    Ok(())
}

fn scalar_distribution(
    values: impl IntoIterator<Item = f64>,
) -> Option<DiagnosticCountDistribution> {
    let values = values.into_iter().collect::<Vec<_>>();
    if values.is_empty() {
        return None;
    }
    let count = values.len();
    let mean = values.iter().sum::<f64>() / count as f64;
    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let variance = values
        .iter()
        .map(|value| {
            let delta = value - mean;
            delta * delta
        })
        .sum::<f64>()
        / count as f64;
    Some(DiagnosticCountDistribution {
        count,
        mean,
        min,
        max,
        stddev: variance.sqrt(),
    })
}

fn is_adversarial(signal: &DiagnosticSignal) -> bool {
    matches!(signal, DiagnosticSignal::Adversarial { .. })
}

fn is_causal(signal: &DiagnosticSignal) -> bool {
    matches!(
        signal,
        DiagnosticSignal::Causal { .. } | DiagnosticSignal::CausalAssessment { .. }
    )
}

fn is_assumption(signal: &DiagnosticSignal) -> bool {
    matches!(signal, DiagnosticSignal::Assumption { .. })
}

fn is_evidence_qualification(signal: &DiagnosticSignal) -> bool {
    matches!(signal, DiagnosticSignal::EvidenceQualification { .. })
}

fn is_candidate(signal: &DiagnosticSignal) -> bool {
    matches!(signal, DiagnosticSignal::Candidate { .. })
}

fn canonical_relation(relation: &CausalRelation) -> CausalRelation {
    let mut relation = relation.clone();
    relation.causes.sort_by(|left, right| {
        left.key
            .cmp(&right.key)
            .then_with(|| left.value.cmp(&right.value))
    });
    relation
}

fn signal_sort_key(signal: &DiagnosticSignal) -> String {
    match signal {
        DiagnosticSignal::Adversarial {
            detector,
            kind,
            strength,
            proposition,
        } => format!(
            "adversarial|{detector}|{kind:?}|{strength:?}|{}={}",
            proposition.key, proposition.value
        ),
        DiagnosticSignal::Causal {
            detector,
            kind,
            reason,
            strength,
            relation,
        } => format!(
            "causal|{detector}|{kind:?}|{reason:?}|{strength:?}|{}",
            relation
                .as_ref()
                .map(relation_sort_key)
                .unwrap_or_else(|| "unbound".into())
        ),
        DiagnosticSignal::CausalAssessment { status, relation } => format!(
            "causal_assessment|{status:?}|{}",
            relation
                .as_ref()
                .map(relation_sort_key)
                .unwrap_or_else(|| "unbound".into())
        ),
        DiagnosticSignal::Assumption {
            detector,
            kind,
            strength,
            proposition,
        } => format!(
            "assumption|{detector}|{kind:?}|{strength:?}|{}",
            proposition
                .as_ref()
                .map(|proposition| format!("{}={}", proposition.key, proposition.value))
                .unwrap_or_else(|| "unbound".into())
        ),
        DiagnosticSignal::EvidenceQualification {
            detector,
            kind,
            reason,
            strength,
            proposition,
        } => format!(
            "evidence_qualification|{detector}|{kind:?}|{reason:?}|{strength:?}|{}={}",
            proposition.key, proposition.value
        ),
        DiagnosticSignal::Candidate { code } => format!("candidate|{code}"),
    }
}

fn relation_sort_key(relation: &CausalRelation) -> String {
    let mut causes = relation
        .causes
        .iter()
        .map(|cause| format!("{}={}", cause.key, cause.value))
        .collect::<Vec<_>>();
    causes.sort();
    format!(
        "{}->{}={}",
        causes.join("&"),
        relation.effect.key,
        relation.effect.value
    )
}

#[cfg(test)]
mod tests {
    use crate::{
        AdversarialFinding, CandidateDiagnostic, CausalEdgeAssessment, CausalFinding,
        CausalFindingKind, CausalFindingReason, CausalInspection, CausalSupportStatus,
        EpistemicState, EvidenceQualificationFinding, EvidenceQualificationFindingKind,
        EvidenceQualificationFindingReason, FindingStrength, ReasoningArtifact,
    };

    use super::*;

    fn observation(id: &str, signals: Vec<DiagnosticSignal>) -> DiagnosticObservation {
        DiagnosticObservation {
            fixture_id: id.into(),
            signals,
        }
    }

    fn candidate(code: &str) -> DiagnosticSignal {
        DiagnosticSignal::Candidate { code: code.into() }
    }

    #[test]
    fn excludes_incomplete_trials_from_frequency_denominator() {
        let trials = vec![
            DiagnosticTrial {
                trial_index: 0,
                operationally_complete: true,
                operational_failures: 0,
                observations: vec![observation("a", vec![candidate("dropped_edge")])],
            },
            DiagnosticTrial {
                trial_index: 1,
                operationally_complete: false,
                operational_failures: 1,
                observations: vec![observation("a", vec![])],
            },
        ];
        let report = aggregate_repeated_diagnostics(&trials).unwrap();
        assert_eq!(report.complete_trials, 1);
        assert_eq!(report.incomplete_trials, 1);
        assert_eq!(report.operational_failures, 1);
        assert_eq!(report.excluded_incomplete_trial_observations, 1);
        let fixture = &report.fixtures[0];
        assert_eq!(fixture.complete_trial_observations, 1);
        assert_eq!(fixture.frequencies[0].occurrences, 1);
        assert_eq!(fixture.frequencies[0].denominator, 1);
        assert_eq!(fixture.frequencies[0].frequency, 1.0);
        assert!(fixture.frequencies[0].confidence_interval_95.is_none());
    }

    #[test]
    fn wilson_interval_is_regression_stable() {
        let interval = wilson_95(5, 10).unwrap();
        assert_eq!(interval.method, ConfidenceIntervalMethod::WilsonScore);
        assert!((interval.lower - 0.236_593_090_512_564).abs() < 1e-12);
        assert!((interval.upper - 0.763_406_909_487_436_1).abs() < 1e-12);
        assert!(wilson_95(2, 4).is_none());
    }

    #[test]
    fn observation_includes_adversarial_candidate_and_causal_reasons() {
        let proposition = Proposition {
            key: "feature.enabled".into(),
            value: "true".into(),
        };
        let relation = CausalRelation {
            causes: vec![Proposition {
                key: "cause".into(),
                value: "true".into(),
            }],
            effect: proposition.clone(),
        };
        let artifact = ReasoningArtifact {
            task: "test".into(),
            evidence: vec![],
            hypotheses: vec![],
            assumptions: vec![],
            evidence_requirements: vec![],
            authority_policy: Default::default(),
            candidate_diagnostics: vec![CandidateDiagnostic {
                code: "dropped_edge".into(),
                message: "fixture".into(),
            }],
            verification_receipts: vec![],
            assumption_findings: vec![],
            evidence_qualification_findings: vec![EvidenceQualificationFinding {
                id: "eq1".into(),
                detector: "evidence_qualification_inspector".into(),
                kind: EvidenceQualificationFindingKind::TemporalMismatch,
                reason: EvidenceQualificationFindingReason::Stale,
                strength: FindingStrength::Hard,
                proposition: proposition.clone(),
                evidence_ids: vec!["e1".into()],
                message: "fixture".into(),
            }],
            adversarial_findings: vec![AdversarialFinding {
                id: "a1".into(),
                detector: "structured_fact_conflict".into(),
                kind: AdversarialFindingKind::Counterexample,
                strength: FindingStrength::Hard,
                claim_id: "c1".into(),
                proposition: proposition.clone(),
                evidence_ids: vec![],
                message: "fixture".into(),
            }],
            claims: vec![crate::Claim {
                id: "c1".into(),
                statement: "test".into(),
                state: EpistemicState::Assumed,
                proposition: Some(proposition.clone()),
                evidence_ids: vec![],
            }],
            inferences: vec![],
        };
        let causal = CausalInspection {
            assessments: vec![CausalEdgeAssessment {
                edge_id: "edge".into(),
                inference_id: "i1".into(),
                premise_claim_ids: vec!["c1".into()],
                conclusion_claim_id: "c1".into(),
                relation: Some(relation.clone()),
                evidence_ids: vec![],
                status: CausalSupportStatus::Unknown,
            }],
            findings: vec![CausalFinding {
                id: "cf1".into(),
                detector: "causal_inspector".into(),
                kind: CausalFindingKind::UnsupportedCausalEdge,
                reason: CausalFindingReason::MissingCausalEvidence,
                strength: FindingStrength::Soft,
                edge_id: "edge".into(),
                relation: Some(relation),
                evidence_ids: vec![],
                message: "fixture".into(),
            }],
        };
        let observation = observe_diagnostics("fixture", &artifact, Some(&causal));
        assert_eq!(observation.signals.len(), 5);
        assert!(observation.signals.iter().any(|signal| matches!(
            signal,
            DiagnosticSignal::EvidenceQualification {
                reason: EvidenceQualificationFindingReason::Stale,
                ..
            }
        )));
        assert!(observation.signals.iter().any(|signal| matches!(
            signal,
            DiagnosticSignal::CausalAssessment {
                status: CausalSupportStatus::Unknown,
                ..
            }
        )));
        assert!(observation.signals.iter().any(|signal| matches!(
            signal,
            DiagnosticSignal::Causal {
                reason: CausalFindingReason::MissingCausalEvidence,
                ..
            }
        )));
    }

    #[test]
    fn repeated_diagnostic_report_serializes_without_provider_identity() {
        let trials = (0..5)
            .map(|trial_index| DiagnosticTrial {
                trial_index,
                operationally_complete: true,
                operational_failures: 0,
                observations: vec![observation("a", vec![candidate("dropped_edge")])],
            })
            .collect::<Vec<_>>();
        let report = aggregate_repeated_diagnostics(&trials).unwrap();
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("\"family\":\"candidate\""));
        assert!(!json.contains("\"provider\""));
    }

    #[test]
    fn per_fixture_count_distribution_includes_zero_finding_trials() {
        let trials = (0..5)
            .map(|trial_index| DiagnosticTrial {
                trial_index,
                operationally_complete: true,
                operational_failures: 0,
                observations: vec![observation(
                    "a",
                    if trial_index < 2 {
                        vec![candidate("dropped_edge")]
                    } else {
                        vec![]
                    },
                )],
            })
            .collect::<Vec<_>>();
        let report = aggregate_repeated_diagnostics(&trials).unwrap();
        let fixture = &report.fixtures[0];
        assert_eq!(fixture.frequencies[0].occurrences, 2);
        assert_eq!(fixture.frequencies[0].denominator, 5);
        assert_eq!(fixture.frequencies[0].frequency, 0.4);
        assert!(fixture.frequencies[0].confidence_interval_95.is_some());
        let distribution = fixture.count_distributions.candidate.as_ref().unwrap();
        assert_eq!(distribution.count, 5);
        assert!((distribution.mean - 0.4).abs() < 1e-12);
        assert_eq!(distribution.min, 0.0);
        assert_eq!(distribution.max, 1.0);
    }
}
