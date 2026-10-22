//! Acquisition obligations restrict presentation; model plans never grant truth authority.
use std::collections::BTreeSet;

use reasoning_harness_core::{
    EpistemicState, Evidence, FinalAnswerClaim, FinalClaimMode, FinalizationResult,
    FinalizationStatus, FindingStrength, InvestigationStopReason, InvestigationTarget, Proposition,
    ReasoningArtifact, SemanticDecidabilityDisposition, SemanticDiagnosticKind,
    SemanticDiagnosticTarget, SoftJudgeRequest, VerificationConclusion,
    assess_semantic_decidability,
};
use serde::{Deserialize, Serialize};

pub(super) const POLICY_ID: &str = "harness-investigation-coverage-v1";
pub(super) const SOURCED_POLICY_ID: &str = "harness-source-and-investigation-coverage-v1";
pub(super) const OUTPUT_ID: &str = "reason-natural-output-v6";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct EvidenceBinding {
    pub evidence_id: String,
    pub evidence_sha256: String,
    pub fact_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Obligation {
    pub target: InvestigationTarget,
    pub bindings: Vec<EvidenceBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Ledger {
    pub plan_admitted: bool,
    pub targets: Vec<Obligation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Disposition {
    NotAcquired,
    Unverified,
    ConflictingOrBlocked,
    Verified,
    VerifiedNotSelected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TargetView {
    pub target: InvestigationTarget,
    pub disposition: Disposition,
    pub verified_proposition: Option<Proposition>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct View {
    pub policy_id: String,
    pub ledger: Ledger,
    /// Already gated source-context display, if enabled; never a model text.
    pub base_finalization: FinalizationResult,
    pub selected_claims: Vec<FinalAnswerClaim>,
    pub targets: Vec<TargetView>,
    pub plan_evidence_complete: bool,
    pub plan_answer_complete: bool,
    pub stop_reason: Option<InvestigationStopReason>,
}

fn clean(s: &str) -> bool {
    !s.trim().is_empty()
        && !s.chars().any(|c| {
            c.is_control() || matches!(c, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
        })
}

impl Ledger {
    pub fn unavailable() -> Self {
        Self {
            plan_admitted: false,
            targets: vec![],
        }
    }

    pub fn new(targets: Vec<InvestigationTarget>) -> Result<Self, String> {
        let ledger = Self {
            plan_admitted: true,
            targets: targets
                .into_iter()
                .map(|target| Obligation {
                    target,
                    bindings: vec![],
                })
                .collect(),
        };
        ledger.validate()?;
        Ok(ledger)
    }

    pub fn validate(&self) -> Result<(), String> {
        let mut ids = BTreeSet::new();
        if self.plan_admitted == self.targets.is_empty()
            || self.targets.iter().any(|o| {
                !clean(&o.target.id)
                    || !clean(&o.target.question)
                    || o.target
                        .expected_fact_key
                        .as_ref()
                        .is_some_and(|k| !clean(k))
                    || !ids.insert(&o.target.id)
                    || o.bindings.iter().any(|b| {
                        !clean(&b.evidence_id)
                            || b.evidence_sha256.len() != 64
                            || !b
                                .evidence_sha256
                                .bytes()
                                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
                            || b.fact_key.as_ref().is_some_and(|k| {
                                !clean(k)
                                    || o.target
                                        .expected_fact_key
                                        .as_ref()
                                        .is_some_and(|expected| expected != k)
                            })
                    })
            })
        {
            return Err("invalid frozen investigation coverage ledger".into());
        }
        Ok(())
    }

    /// Initial trusted evidence can satisfy an exact, unique key without a new acquisition.
    /// Same-key sibling questions are not silently collapsed into one obligation.
    pub fn bind_existing(&mut self, artifact: &ReasoningArtifact) {
        self.bind_evidence(&artifact.evidence);
    }

    pub fn bind_evidence(&mut self, evidence: &[Evidence]) {
        let unique_keys = self
            .targets
            .iter()
            .filter_map(|o| o.target.expected_fact_key.clone())
            .filter(|key| {
                self.targets
                    .iter()
                    .filter(|o| o.target.expected_fact_key.as_ref() == Some(key))
                    .count()
                    == 1
            })
            .collect::<BTreeSet<_>>();
        for o in &mut self.targets {
            if let Some(key) = o
                .target
                .expected_fact_key
                .as_ref()
                .filter(|k| unique_keys.contains(*k))
            {
                for e in evidence {
                    if e.facts.contains_key(key) {
                        let binding = EvidenceBinding {
                            evidence_id: e.id.clone(),
                            evidence_sha256: super::answer_context::evidence_hash(e),
                            fact_key: Some(key.clone()),
                        };
                        if !o.bindings.contains(&binding) {
                            o.bindings.push(binding);
                        }
                    }
                }
            }
        }
    }

    pub fn record(
        &mut self,
        target_id: &str,
        fact_key: Option<&str>,
        artifact: &ReasoningArtifact,
        admitted_ids: &[String],
    ) -> Result<(), String> {
        let o = self
            .targets
            .iter_mut()
            .find(|o| o.target.id == target_id)
            .ok_or("coverage target absent from frozen plan")?;
        for e in &artifact.evidence {
            if admitted_ids.contains(&e.id) {
                let binding = EvidenceBinding {
                    evidence_id: e.id.clone(),
                    evidence_sha256: super::answer_context::evidence_hash(e),
                    fact_key: fact_key.map(str::to_owned),
                };
                if !o.bindings.contains(&binding) {
                    o.bindings.push(binding);
                }
            }
        }
        self.validate()
    }

    pub fn targets(&self, artifact: &ReasoningArtifact) -> Vec<TargetView> {
        self.targets
            .iter()
            .map(|o| target_view(o, artifact))
            .collect()
    }

    pub fn complete(&self, artifact: &ReasoningArtifact) -> bool {
        !self.targets.is_empty()
            && self
                .targets(artifact)
                .iter()
                .all(|t| t.disposition == Disposition::Verified)
    }

    /// Proposals from exact admitted bytes still enter grounding as Assumed.
    pub fn materialization_facts(&self, evidence: &[Evidence]) -> Vec<(Proposition, Vec<String>)> {
        let mut result = vec![];
        for o in &self.targets {
            let (mut props, ids) = bound_facts(o, evidence);
            if props.len() == 1 && !ids.is_empty() {
                let p = props.remove(0);
                if !result.iter().any(|(q, _)| q == &p) {
                    result.push((p, ids));
                }
            }
        }
        result
    }

    pub fn verified_targets(&self, artifact: &ReasoningArtifact) -> Vec<Proposition> {
        let mut result = vec![];
        for t in self.targets(artifact) {
            if let Some(p) = t.verified_proposition
                && !result.contains(&p)
            {
                result.push(p);
            }
        }
        result
    }
}

fn bound_facts(o: &Obligation, evidence: &[Evidence]) -> (Vec<Proposition>, Vec<String>) {
    let mut propositions = vec![];
    let mut matching_ids = vec![];
    for b in &o.bindings {
        let Some(key) = b.fact_key.as_ref() else {
            continue;
        };
        let matches = evidence
            .iter()
            .filter(|e| e.id == b.evidence_id)
            .collect::<Vec<_>>();
        let [e] = matches.as_slice() else {
            continue;
        };
        if e.metadata
            .provenance_class
            .as_deref()
            .is_none_or(|p| p == "untrusted_context" || p == "untrusted")
            || super::answer_context::evidence_hash(e) != b.evidence_sha256
        {
            continue;
        }
        if let Some(value) = e.facts.get(key) {
            let p = Proposition {
                key: key.clone(),
                value: value.clone(),
            };
            if !propositions.contains(&p) {
                propositions.push(p);
            }
            matching_ids.push(e.id.clone());
        }
    }
    (propositions, matching_ids)
}

fn target_view(o: &Obligation, artifact: &ReasoningArtifact) -> TargetView {
    let mut view = TargetView {
        target: o.target.clone(),
        disposition: Disposition::NotAcquired,
        verified_proposition: None,
    };
    if o.bindings.is_empty() {
        return view;
    }
    view.disposition = Disposition::Unverified;
    let (propositions, matching_ids) = bound_facts(o, &artifact.evidence);
    if propositions.len() > 1 {
        view.disposition = Disposition::ConflictingOrBlocked;
        return view;
    }
    let [p] = propositions.as_slice() else {
        return view;
    };
    if artifact.claims.iter().any(|c| {
        c.proposition.as_ref().is_some_and(|q| {
            q.key == p.key && (q.value != p.value || c.state == EpistemicState::Contradicted)
        })
    }) || artifact.verification_receipts.iter().any(|r| {
        r.proposition.as_ref().is_some_and(|q| q.key == p.key)
            && r.conclusion == VerificationConclusion::Contradicted
    }) || artifact
        .evidence_qualification_findings
        .iter()
        .any(|f| f.proposition.key == p.key)
        || artifact
            .adversarial_findings
            .iter()
            .any(|f| f.proposition.key == p.key && f.strength == FindingStrength::Hard)
    {
        view.disposition = Disposition::ConflictingOrBlocked;
        return view;
    }
    let claims = artifact
        .claims
        .iter()
        .filter(|c| c.proposition.as_ref() == Some(p))
        .collect::<Vec<_>>();
    if claims.is_empty()
        || claims
            .iter()
            .any(|c| !matches!(c.state, EpistemicState::Known | EpistemicState::Supported))
    {
        return view;
    }
    let verifier = reasoning_harness_core::structured_fact_verifier_for_input(
        &super::input_from_artifact(artifact),
    );
    let verified = claims.iter().any(|c| {
        let current = verifier.verify(c, &artifact.evidence);
        current.as_ref().is_some_and(|r| {
            r.conclusion == VerificationConclusion::Supported
                && r.evidence_ids.iter().any(|id| matching_ids.contains(id))
        }) && artifact.verification_receipts.iter().any(|r| {
            matches!(
                r.verifier.as_str(),
                "structured_fact_equality" | "qualified_structured_fact_equality"
            ) && r.conclusion == VerificationConclusion::Supported
                && r.claim_id.as_deref() == Some(c.id.as_str())
                && r.proposition.as_ref() == Some(p)
                && r.claim_statement.as_ref().is_none_or(|s| s == &c.statement)
                && r.evidence_ids
                    .iter()
                    .any(|id| matching_ids.contains(id) && c.evidence_ids.contains(id))
                && r.evidence_ids
                    .iter()
                    .all(|id| artifact.evidence.iter().filter(|e| &e.id == id).count() == 1)
        })
    });
    if !verified {
        return view;
    }
    let request = SoftJudgeRequest {
        id: format!("coverage:{}", o.target.id),
        task: artifact.task.clone(),
        kind: SemanticDiagnosticKind::UnsupportedPremise,
        target: SemanticDiagnosticTarget::Proposition {
            proposition: p.clone(),
        },
        context: vec![],
    };
    if !assess_semantic_decidability(&request, artifact)
        .is_ok_and(|d| d.disposition == SemanticDecidabilityDisposition::Permit)
    {
        view.disposition = Disposition::ConflictingOrBlocked;
        return view;
    }
    view.disposition = Disposition::Verified;
    view.verified_proposition = Some(p.clone());
    view
}

pub(super) fn compose(
    ledger: &Ledger,
    artifact: &ReasoningArtifact,
    selected: &[FinalAnswerClaim],
    base: FinalizationResult,
    stop_reason: Option<InvestigationStopReason>,
) -> Result<(FinalizationResult, View), String> {
    ledger.validate()?;
    let mut targets = ledger.targets(artifact);
    let evidence_complete = ledger.plan_admitted
        && targets
            .iter()
            .all(|t| t.disposition == Disposition::Verified);
    for t in &mut targets {
        if let Some(p) = &t.verified_proposition
            && !selected
                .iter()
                .any(|c| c.mode == FinalClaimMode::Grounded && &c.proposition == p)
        {
            t.disposition = Disposition::VerifiedNotSelected;
        }
    }
    let answer_exposed = matches!(
        base.status,
        FinalizationStatus::GroundedAnswer | FinalizationStatus::QualifiedPartialAnswer
    ) && base.text.is_some();
    let answer_complete = ledger.plan_admitted
        && answer_exposed
        && targets
            .iter()
            .all(|t| t.disposition == Disposition::Verified);
    let view = View {
        policy_id: POLICY_ID.into(),
        ledger: ledger.clone(),
        base_finalization: base.clone(),
        selected_claims: selected.to_vec(),
        targets,
        plan_evidence_complete: evidence_complete,
        plan_answer_complete: answer_complete,
        stop_reason,
    };
    let mut result = base;
    if !answer_complete {
        let mut text = if answer_exposed {
            result.text.take().unwrap()
        } else {
            String::new()
        };
        if !text.is_empty() {
            text.push('\n');
        }
        text.push_str("Investigation coverage (admitted plan only; not proof that the whole question is answered):");
        if !ledger.plan_admitted {
            text.push_str(
                "\nNo admitted plan is available; task coverage has not been established.",
            );
        }
        for t in view
            .targets
            .iter()
            .filter(|t| t.disposition != Disposition::Verified || !answer_exposed)
        {
            text.push_str(&format!(
                "\n- target {} / question {} / selector {}: {}",
                serde_json::to_string(&t.target.id).unwrap(),
                serde_json::to_string(&t.target.question).unwrap(),
                serde_json::to_string(&t.target.expected_fact_key).unwrap(),
                if !answer_exposed && t.disposition == Disposition::Verified {
                    "evidence_verified_but_answer_not_released".into()
                } else {
                    serde_json::to_string(&t.disposition).unwrap()
                }
            ));
        }
        text.push_str(&format!(
            "\nInvestigation stopped: {}. Unestablished targets have no asserted value.",
            stop_reason
                .map(|reason| serde_json::to_string(&reason).unwrap())
                .unwrap_or_else(|| "not run in this turn".into())
        ));
        result.text = Some(text);
        if result.status == FinalizationStatus::GroundedAnswer {
            result.status = FinalizationStatus::QualifiedPartialAnswer;
        }
    }
    Ok((result, view))
}

#[cfg(test)]
mod tests {
    use super::*;
    use reasoning_harness_core::InvestigationTargetOrigin;

    fn target(id: &str, key: &str) -> InvestigationTarget {
        InvestigationTarget {
            id: id.into(),
            question: format!("Explain {key}"),
            expected_fact_key: Some(key.into()),
            origin: InvestigationTargetOrigin::ModelProposedUntrusted,
        }
    }

    #[test]
    fn partial_plan_preserves_source_spans_and_reports_unknown_target_without_value() {
        let (config, artifact, selected, hard) = super::super::answer_context::test_fixture();
        let mut ledger = Ledger::new(vec![
            target("present", "feature.rule"),
            target("missing", "other.rule"),
        ])
        .unwrap();
        ledger.bind_existing(&artifact);
        assert!(!ledger.complete(&artifact));
        let (source, source_view) =
            super::super::answer_context::compose(&config, &artifact, &selected, hard.clone())
                .unwrap();
        let (display, view) = compose(
            &ledger,
            &artifact,
            &selected,
            source.clone(),
            Some(InvestigationStopReason::PlannerStop),
        )
        .unwrap();
        assert!(!view.plan_answer_complete);
        assert_eq!(view.targets[1].disposition, Disposition::NotAcquired);
        assert!(view.targets[1].verified_proposition.is_none());
        assert_eq!(display.factual_claims, hard.factual_claims);
        assert_eq!(display.uncovered_propositions, hard.uncovered_propositions);
        let text = display.text.unwrap();
        assert!(text.starts_with(source.text.as_ref().unwrap()));
        assert!(text.contains("other.rule") && text.contains("planner_stop"));
        let citation = &source_view.citations[0];
        assert_eq!(
            serde_json::from_str::<String>(
                &text[citation.quote_start_byte..citation.quote_end_byte]
            )
            .unwrap(),
            citation.quote
        );
    }

    #[test]
    fn coverage_rechecks_current_evidence_receipts_requirements_and_selection() {
        let (_, original, selected, hard) = super::super::answer_context::test_fixture();
        let mut ledger = Ledger::new(vec![target("t", "feature.rule")]).unwrap();
        ledger.bind_existing(&original);
        assert!(ledger.complete(&original));
        for mutation in 0..4 {
            let mut artifact = original.clone();
            match mutation {
                0 => artifact.evidence[0].observation.push(' '),
                1 => artifact.verification_receipts.clear(),
                2 => artifact.evidence_requirements[0].as_of_unix_seconds = Some(400),
                _ => artifact.claims.clear(),
            }
            assert!(!ledger.complete(&artifact), "mutation={mutation}");
        }
        let (_, omitted) = compose(&ledger, &original, &[], hard.clone(), None).unwrap();
        assert!(omitted.plan_evidence_complete && !omitted.plan_answer_complete);
        assert_eq!(
            omitted.targets[0].disposition,
            Disposition::VerifiedNotSelected
        );
        let mut blocked = hard;
        blocked.status = FinalizationStatus::RequiresVerification;
        blocked.text = None;
        let (display, _) = compose(
            &ledger,
            &original,
            &selected,
            blocked,
            Some(InvestigationStopReason::OperationalTerminal),
        )
        .unwrap();
        assert_eq!(display.status, FinalizationStatus::RequiresVerification);
        assert!(!display.text.as_ref().unwrap().contains("limited"));
        assert!(display.text.unwrap().contains("operational_terminal"));
    }

    #[test]
    fn same_key_siblings_and_missing_plan_do_not_claim_completion() {
        let (_, artifact, selected, hard) = super::super::answer_context::test_fixture();
        let mut ledger = Ledger::new(vec![
            target("first", "feature.rule"),
            target("second", "feature.rule"),
        ])
        .unwrap();
        ledger.bind_existing(&artifact);
        assert!(ledger.targets.iter().all(|t| t.bindings.is_empty()));
        ledger
            .record(
                "first",
                Some("feature.rule"),
                &artifact,
                &[artifact.evidence[0].id.clone()],
            )
            .unwrap();
        assert!(!ledger.complete(&artifact));
        assert_eq!(
            ledger.targets(&artifact)[1].disposition,
            Disposition::NotAcquired
        );
        let (display, view) =
            compose(&Ledger::unavailable(), &artifact, &selected, hard, None).unwrap();
        assert_eq!(display.status, FinalizationStatus::QualifiedPartialAnswer);
        assert!(!view.plan_answer_complete && !view.plan_evidence_complete);
        assert!(display.text.unwrap().contains("No admitted plan"));
    }
}
