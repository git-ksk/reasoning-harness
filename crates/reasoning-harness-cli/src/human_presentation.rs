use std::collections::{BTreeMap, BTreeSet};

use reasoning_harness_core::{
    EpistemicState, EvidenceAdmissionRejection, FinalizationResult, FinalizationStatus,
    ReasoningArtifact, ResolutionAttempt, ResolutionAttemptStatus,
};

use super::NaturalOutput;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct HumanPresentation {
    answer: String,
    status: &'static str,
    verified_facts: Vec<FactView>,
    unresolved: Vec<UnresolvedView>,
    evidence: Vec<EvidenceView>,
    untrusted_context_sources: Vec<String>,
    acquisition_notes: Vec<String>,
    coverage: String,
    safety_configuration_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FactView {
    proposition: String,
    state: &'static str,
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct UnresolvedView {
    proposition: String,
    reason: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct EvidenceView {
    id: String,
    source: String,
    provenance: String,
}

pub(super) fn from_output(output: &NaturalOutput) -> HumanPresentation {
    let artifact = output
        .resolution_rounds
        .last()
        .map(|round| &round.final_artifact)
        .unwrap_or(&output.initial_outcome.artifact);
    let attempts = output
        .resolution_rounds
        .iter()
        .flat_map(|round| round.attempts.iter())
        .cloned()
        .collect::<Vec<_>>();
    from_parts(
        &output.finalization,
        artifact,
        &attempts,
        output.safety_runtime.configuration_id(),
    )
}

pub(super) fn from_parts(
    finalization: &FinalizationResult,
    artifact: &ReasoningArtifact,
    attempts: &[ResolutionAttempt],
    safety_configuration_id: &str,
) -> HumanPresentation {
    let mut supporting_ids = BTreeSet::new();
    let mut verified_facts = Vec::new();
    let mut unresolved = BTreeMap::<String, BTreeSet<String>>::new();
    for claim in &artifact.claims {
        let Some(proposition) = &claim.proposition else {
            continue;
        };
        let canonical = proposition_text(&proposition.key, &proposition.value);
        match claim.state {
            EpistemicState::Known | EpistemicState::Supported => {
                verified_facts.push(FactView {
                    proposition: canonical,
                    state: epistemic_label(claim.state),
                });
                supporting_ids.extend(claim.evidence_ids.iter().cloned());
            }
            EpistemicState::Inferred
            | EpistemicState::Assumed
            | EpistemicState::Contradicted
            | EpistemicState::Unknown => {
                unresolved
                    .entry(canonical)
                    .or_default()
                    .insert(epistemic_reason(claim.state).into());
            }
        }
    }
    for proposition in &finalization.uncovered_propositions {
        unresolved
            .entry(proposition_text(&proposition.key, &proposition.value))
            .or_default()
            .insert("not covered by verified evidence".into());
    }
    for finding in &artifact.evidence_qualification_findings {
        unresolved
            .entry(proposition_text(
                &finding.proposition.key,
                &finding.proposition.value,
            ))
            .or_default()
            .insert(qualification_reason(&finding.reason).into());
    }
    let unresolved = unresolved
        .into_iter()
        .map(|(proposition, reasons)| UnresolvedView {
            proposition,
            reason: reasons.into_iter().collect::<Vec<_>>().join("; "),
        })
        .collect();
    verified_facts.sort_by(|a, b| a.proposition.cmp(&b.proposition));
    verified_facts.dedup_by(|a, b| a.proposition == b.proposition && a.state == b.state);

    let mut evidence = Vec::new();
    let mut untrusted_sources = BTreeSet::new();
    for item in &artifact.evidence {
        let provenance = item
            .metadata
            .provenance_class
            .as_deref()
            .unwrap_or("unspecified");
        if provenance == "untrusted_context" {
            untrusted_sources.insert(item.source.clone());
        } else if supporting_ids.contains(&item.id) {
            evidence.push(EvidenceView {
                id: item.id.clone(),
                source: item.source.clone(),
                provenance: provenance_label(provenance),
            });
        }
    }
    evidence.sort_by(|a, b| (&a.source, &a.id).cmp(&(&b.source, &b.id)));
    let acquisition_notes = attempts
        .iter()
        .filter_map(attempt_note)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();

    HumanPresentation {
        answer: exposed_answer(finalization),
        status: finalization_status_label(finalization.status),
        verified_facts,
        unresolved,
        evidence,
        untrusted_context_sources: untrusted_sources.into_iter().collect(),
        acquisition_notes,
        coverage: format!("{:.3}", finalization.factual_claim_coverage),
        safety_configuration_id: safety_configuration_id.into(),
    }
}

impl HumanPresentation {
    pub(super) fn print_full(&self) {
        println!("Answer\n{}\n", self.answer);
        self.print_status_sections();
        println!();
        self.print_evidence_sections();
        self.print_summary();
    }
    pub(super) fn print_status(&self) {
        self.print_status_sections();
        if !self.acquisition_notes.is_empty() {
            println!("\nAcquisition / verification notes");
            for note in &self.acquisition_notes {
                println!("- {note}");
            }
        }
        self.print_summary();
    }
    pub(super) fn print_evidence(&self) {
        self.print_evidence_sections();
    }
    fn print_status_sections(&self) {
        println!("Verified facts");
        if self.verified_facts.is_empty() {
            println!("- none");
        }
        for fact in &self.verified_facts {
            println!("- {} ({})", fact.proposition, fact.state);
        }
        println!("\nUnresolved / qualified");
        if self.unresolved.is_empty() {
            println!("- none");
        }
        for item in &self.unresolved {
            println!("- {} — {}", item.proposition, item.reason);
        }
    }
    fn print_evidence_sections(&self) {
        println!("Evidence / sources");
        if self.evidence.is_empty() {
            println!("- no supporting source is exposed for the verified facts");
        }
        for item in &self.evidence {
            println!(
                "- {} — provenance={} — evidence_id={}",
                item.source, item.provenance, item.id
            );
        }
        if !self.untrusted_context_sources.is_empty() {
            println!("\nUntrusted context (not supporting authority)");
            for source in &self.untrusted_context_sources {
                println!("- {source}");
            }
        }
        if !self.acquisition_notes.is_empty() {
            println!("\nAcquisition / verification notes");
            for note in &self.acquisition_notes {
                println!("- {note}");
            }
        }
    }
    fn print_summary(&self) {
        println!(
            "\nstatus: {} | verified_facts={} | unresolved_items={} | coverage={} | safety={}",
            self.status,
            self.verified_facts.len(),
            self.unresolved.len(),
            self.coverage,
            self.safety_configuration_id
        );
    }
    #[cfg(test)]
    fn render_for_test(&self) -> String {
        let mut out = format!("Answer\n{}\nVerified facts\n", self.answer);
        for fact in &self.verified_facts {
            out.push_str(&format!("{} {}\n", fact.proposition, fact.state));
        }
        out.push_str("Unresolved / qualified\n");
        for item in &self.unresolved {
            out.push_str(&format!("{} {}\n", item.proposition, item.reason));
        }
        out.push_str("Evidence / sources\n");
        for item in &self.evidence {
            out.push_str(&format!(
                "{} {} {}\n",
                item.source, item.provenance, item.id
            ));
        }
        for source in &self.untrusted_context_sources {
            out.push_str(&format!("untrusted:{source}\n"));
        }
        for note in &self.acquisition_notes {
            out.push_str(&format!("note:{note}\n"));
        }
        out
    }
}

fn proposition_text(key: &str, value: &str) -> String {
    format!("{key}={value}")
}
fn finalization_status_label(s: FinalizationStatus) -> &'static str {
    match s {
        FinalizationStatus::GroundedAnswer => "grounded_answer",
        FinalizationStatus::QualifiedPartialAnswer => "qualified_partial_answer",
        FinalizationStatus::Unresolved => "unresolved",
        FinalizationStatus::Abstain => "abstain",
        FinalizationStatus::RequiresVerification => "requires_verification",
    }
}
fn exposed_answer(f: &FinalizationResult) -> String {
    match f.status {
        FinalizationStatus::GroundedAnswer | FinalizationStatus::QualifiedPartialAnswer => f
            .text
            .clone()
            .unwrap_or_else(|| "No exposed answer text is available.".into()),
        FinalizationStatus::Abstain => {
            "I cannot provide a grounded answer because verified state is contradictory.".into()
        }
        FinalizationStatus::Unresolved | FinalizationStatus::RequiresVerification => {
            "I cannot support a complete answer from the currently verified evidence.".into()
        }
    }
}
fn epistemic_label(s: EpistemicState) -> &'static str {
    match s {
        EpistemicState::Known => "known",
        EpistemicState::Supported => "supported",
        EpistemicState::Inferred => "inferred",
        EpistemicState::Assumed => "assumed",
        EpistemicState::Contradicted => "contradicted",
        EpistemicState::Unknown => "unknown",
    }
}
fn epistemic_reason(s: EpistemicState) -> &'static str {
    match s {
        EpistemicState::Inferred => "inferred but not verified as a fact",
        EpistemicState::Assumed => "assumption, not verified",
        EpistemicState::Contradicted => "contradicted by verified state",
        EpistemicState::Unknown => "not established by verified evidence",
        EpistemicState::Known | EpistemicState::Supported => "verified",
    }
}
fn provenance_label(p: &str) -> String {
    match p {
        "explicit_user_fact" => "explicit user fact".into(),
        "explicit_local_resolver" => "explicit local resolver".into(),
        other => other.into(),
    }
}

fn qualification_reason(
    r: &reasoning_harness_core::EvidenceQualificationFindingReason,
) -> &'static str {
    use reasoning_harness_core::EvidenceQualificationFindingReason as R;
    match r {
        R::Stale => "supporting evidence is stale",
        R::NotYetValid => "supporting evidence is not yet valid",
        R::ScopeMismatch => "supporting evidence does not match the required scope",
        R::ScopeExpansion => "supporting evidence would require a broader scope than allowed",
        R::InsufficientAuthority => "supporting evidence has insufficient authority",
        R::MissingTemporalMetadata => "supporting evidence is missing required time metadata",
        R::MissingScopeMetadata => "supporting evidence is missing required scope metadata",
        R::MissingProvenanceMetadata => "supporting evidence is missing provenance metadata",
        R::UnknownProvenanceClass => "supporting evidence has an unknown provenance class",
        R::ConflictingQualifiedEvidence => "qualified evidence conflicts",
    }
}
fn admission_reason(r: EvidenceAdmissionRejection) -> &'static str {
    match r {
        EvidenceAdmissionRejection::UntrustedSource => {
            "source is not trusted by the admission policy"
        }
        EvidenceAdmissionRejection::MissingTrustedMetadata => "trusted metadata is missing",
        EvidenceAdmissionRejection::InvalidEvidence => "evidence is invalid",
        EvidenceAdmissionRejection::MissingObservationTime => "observation time is missing",
        EvidenceAdmissionRejection::MissingRetrievalTime => "retrieval time is missing",
        EvidenceAdmissionRejection::MissingScopeMetadata => "scope metadata is missing",
        EvidenceAdmissionRejection::MissingAuthorityClaim => "authority claim is missing",
        EvidenceAdmissionRejection::Stale => "evidence is stale",
        EvidenceAdmissionRejection::NotYetValid => "evidence is not yet valid",
        EvidenceAdmissionRejection::ScopeMismatch => "evidence scope does not match",
        EvidenceAdmissionRejection::ScopeExpansion => "evidence would expand the allowed scope",
        EvidenceAdmissionRejection::UnknownAuthorityClass => "authority class is unknown",
        EvidenceAdmissionRejection::InsufficientAuthority => "authority is insufficient",
        EvidenceAdmissionRejection::AuthorityClaimMismatch => {
            "authority claim does not match policy"
        }
    }
}
fn attempt_status_note(s: ResolutionAttemptStatus) -> Option<&'static str> {
    match s {
        ResolutionAttemptStatus::AppliedEvidence
        | ResolutionAttemptStatus::AppliedCandidateRevision
        | ResolutionAttemptStatus::AppliedVerification
        | ResolutionAttemptStatus::NoResult => None,
        ResolutionAttemptStatus::RejectedUntrustedEvidence => Some("evidence was rejected"),
        ResolutionAttemptStatus::AdapterUnavailable => Some("resolver is unavailable"),
        ResolutionAttemptStatus::MalformedOutput => Some("resolver returned malformed output"),
        ResolutionAttemptStatus::AdapterFailed => Some("resolver failed"),
        ResolutionAttemptStatus::TransportFailure => Some("resolver transport failed"),
        ResolutionAttemptStatus::AuthenticationFailure => Some("resolver authentication failed"),
        ResolutionAttemptStatus::PermissionDenied => Some("resolver permission was denied"),
        ResolutionAttemptStatus::NegotiationFailure => Some("resolver negotiation failed"),
        ResolutionAttemptStatus::SessionFailure => Some("resolver session failed"),
        ResolutionAttemptStatus::ProtocolFailure => Some("resolver protocol failed"),
        ResolutionAttemptStatus::ToolFailed => Some("resolver tool failed"),
        ResolutionAttemptStatus::TimedOut => Some("resolver timed out"),
        ResolutionAttemptStatus::PolicyDenied => Some("resolver request was denied by policy"),
        ResolutionAttemptStatus::HumanReviewRequired => Some("human review is required"),
        ResolutionAttemptStatus::BudgetExceeded => Some("resolution budget was exhausted"),
    }
}
fn attempt_note(a: &ResolutionAttempt) -> Option<String> {
    if let Some(rejection) = a.admission_rejection {
        Some(format!(
            "{}: rejected evidence — {}",
            a.adapter_name,
            admission_reason(rejection)
        ))
    } else {
        attempt_status_note(a.status).map(|reason| format!("{}: {reason}", a.adapter_name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reasoning_harness_core::{
        Claim, Evidence, EvidenceMetadata, EvidenceQualificationFinding,
        EvidenceQualificationFindingKind, EvidenceQualificationFindingReason, FindingStrength,
        Proposition, ResolutionCost, ResolutionReason, ResolutionRequest, ResolutionRequestBudget,
        ResolutionTarget, ResolverClass,
    };
    use std::collections::BTreeMap;

    fn finalization(status: FinalizationStatus) -> FinalizationResult {
        FinalizationResult {
            status,
            text: Some("renderer answer".into()),
            factual_claims: 1,
            covered_claims: 1,
            factual_claim_coverage: 1.0,
            uncovered_propositions: vec![],
        }
    }

    #[test]
    fn verified_facts_use_canonical_propositions_and_do_not_render_raw_prose() {
        let artifact = ReasoningArtifact {
            claims: vec![Claim {
                id: "c1".into(),
                statement: "UNSUPPORTED MODEL PROSE".into(),
                state: EpistemicState::Supported,
                proposition: Some(Proposition {
                    key: "service.status".into(),
                    value: "healthy".into(),
                }),
                evidence_ids: vec!["trusted".into(), "ctx".into()],
            }],
            evidence: vec![
                Evidence {
                    id: "trusted".into(),
                    source: "health-api".into(),
                    observation: "RAW TRUSTED OBSERVATION".into(),
                    facts: BTreeMap::from([("service.status".into(), "healthy".into())]),
                    metadata: EvidenceMetadata {
                        provenance_class: Some("primary".into()),
                        ..Default::default()
                    },
                },
                Evidence {
                    id: "ctx".into(),
                    source: "notes.txt".into(),
                    observation: "UNTRUSTED CONTENT".into(),
                    facts: BTreeMap::new(),
                    metadata: EvidenceMetadata {
                        provenance_class: Some("untrusted_context".into()),
                        ..Default::default()
                    },
                },
            ],
            ..Default::default()
        };
        let rendered = from_parts(
            &finalization(FinalizationStatus::GroundedAnswer),
            &artifact,
            &[],
            "safe-v1",
        )
        .render_for_test();
        assert!(rendered.contains("service.status=healthy supported"));
        assert!(rendered.contains("health-api primary trusted"));
        assert!(rendered.contains("untrusted:notes.txt"));
        assert!(!rendered.contains("UNSUPPORTED MODEL PROSE"));
        assert!(!rendered.contains("RAW TRUSTED OBSERVATION"));
        assert!(!rendered.contains("UNTRUSTED CONTENT"));
    }

    #[test]
    fn unresolved_and_qualification_reasons_are_typed_and_visible() {
        let proposition = Proposition {
            key: "release.region".into(),
            value: "eu-west-1".into(),
        };
        let mut f = finalization(FinalizationStatus::QualifiedPartialAnswer);
        f.uncovered_propositions.push(proposition.clone());
        let artifact = ReasoningArtifact {
            claims: vec![Claim {
                id: "c1".into(),
                statement: "ignored".into(),
                state: EpistemicState::Unknown,
                proposition: Some(proposition.clone()),
                evidence_ids: vec![],
            }],
            evidence_qualification_findings: vec![EvidenceQualificationFinding {
                id: "q1".into(),
                detector: "qualification".into(),
                kind: EvidenceQualificationFindingKind::TemporalMismatch,
                reason: EvidenceQualificationFindingReason::Stale,
                strength: FindingStrength::Hard,
                proposition,
                evidence_ids: vec!["e1".into()],
                message: "INTERNAL DETAIL".into(),
            }],
            ..Default::default()
        };
        let rendered = from_parts(&f, &artifact, &[], "safe-v1").render_for_test();
        assert!(rendered.contains("release.region=eu-west-1"));
        assert!(rendered.contains("supporting evidence is stale"));
        assert!(rendered.contains("not established by verified evidence"));
        assert!(!rendered.contains("INTERNAL DETAIL"));
    }

    #[test]
    fn unresolved_answer_reports_unknown_without_renderer_text() {
        let mut f = finalization(FinalizationStatus::Unresolved);
        f.text = Some("UNSAFE RENDERER TEXT".into());
        f.factual_claim_coverage = 0.0;
        let rendered =
            from_parts(&f, &ReasoningArtifact::default(), &[], "safe-v1").render_for_test();
        assert!(rendered.contains("currently verified evidence"));
        assert!(!rendered.contains("UNSAFE RENDERER TEXT"));
    }

    #[test]
    fn abstain_and_acquisition_rejection_are_explained_without_trace() {
        let attempt = ResolutionAttempt {
            attempt_index: 0,
            request: ResolutionRequest {
                id: "r1".into(),
                reason: ResolutionReason::MissingSupport,
                target: ResolutionTarget::Proposition {
                    proposition: Proposition {
                        key: "region".into(),
                        value: "east".into(),
                    },
                },
                resolver_class: ResolverClass::EvidenceAcquisition,
                budget: ResolutionRequestBudget::default(),
            },
            adapter_name: "region-resolver".into(),
            adapter_config_id: None,
            admission_policy_id: None,
            status: ResolutionAttemptStatus::RejectedUntrustedEvidence,
            cost: ResolutionCost::default(),
            admitted_evidence_ids: vec![],
            verification_receipts: 0,
            admission_rejection: Some(EvidenceAdmissionRejection::InsufficientAuthority),
        };
        let rendered = from_parts(
            &finalization(FinalizationStatus::Abstain),
            &ReasoningArtifact::default(),
            &[attempt],
            "safe-v1",
        )
        .render_for_test();
        assert!(rendered.contains("verified state is contradictory"));
        assert!(
            rendered.contains("region-resolver: rejected evidence — authority is insufficient")
        );
    }
}
