use std::collections::BTreeMap;

use reasoning_harness_core::{
    CandidateClaim, Claim, EpistemicState, Evidence, EvidenceAuthorityPolicy, EvidenceMetadata,
    FinalizationResult, FinalizationStatus, PolicyEvidenceConstraints, PolicyInvalidation,
    PolicyInvalidationReason, PolicyInvalidationTarget, Proposition, ReasoningArtifact,
    ReasoningCandidate, ReasoningPolicy, ReasoningPolicyLayer, ReasoningThread,
    ReasoningThreadError, ReasoningThreadEvent, ReasoningThreadEventKind, ReasoningThreadStatus,
    ResolutionAttempt, ResolutionAttemptStatus, ResolutionCost, ResolutionReason,
    ResolutionRequest, ResolutionRequestBudget, ResolutionTarget, ResolverClass, SoftJudgeDecision,
    SoftJudgeIdentity, SoftJudgeObservation, TemporalValidity, Verdict, VerificationConclusion,
    VerificationReceipt, apply_reasoning_policy, compose_reasoning_policy, replay_thread,
    validate_thread,
};

fn proposition() -> Proposition {
    Proposition {
        key: "feature.enabled".into(),
        value: "true".into(),
    }
}

fn candidate() -> ReasoningCandidate {
    ReasoningCandidate {
        claims: vec![CandidateClaim {
            id: "c1".into(),
            statement: "feature.enabled = true".into(),
            proposed_state: EpistemicState::Supported,
            proposition: Some(proposition()),
            evidence_ids: vec![],
        }],
        inferences: vec![],
    }
}

fn artifact() -> ReasoningArtifact {
    ReasoningArtifact {
        task: "is feature enabled".into(),
        evidence: vec![Evidence {
            id: "e1".into(),
            source: "fixture".into(),
            observation: "enabled".into(),
            facts: BTreeMap::from([("feature.enabled".into(), "true".into())]),
            metadata: EvidenceMetadata {
                temporal: Some(TemporalValidity {
                    effective_from_unix_seconds: Some(100),
                    effective_until_unix_seconds: Some(300),
                }),
                scope: None,
                provenance_class: Some("secondary".into()),
            },
        }],
        hypotheses: vec![proposition()],
        assumptions: vec![],
        evidence_requirements: vec![],
        authority_policy: EvidenceAuthorityPolicy {
            ranks: BTreeMap::from([("secondary".into(), 1), ("primary".into(), 2)]),
        },
        candidate_diagnostics: vec![],
        verification_receipts: vec![VerificationReceipt {
            id: "receipt-1".into(),
            verifier: "fixture".into(),
            claim_statement: None,
            proposition: Some(proposition()),
            claim_id: Some("c1".into()),
            conclusion: VerificationConclusion::Supported,
            evidence_ids: vec!["e1".into()],
        }],
        adversarial_findings: vec![],
        assumption_findings: vec![],
        evidence_qualification_findings: vec![],
        claims: vec![Claim {
            id: "c1".into(),
            statement: "feature.enabled = true".into(),
            state: EpistemicState::Supported,
            proposition: Some(proposition()),
            evidence_ids: vec!["e1".into()],
        }],
        inferences: vec![],
    }
}

fn policy(version: &str, authority: &str) -> ReasoningPolicy {
    compose_reasoning_policy(
        version,
        &[ReasoningPolicyLayer {
            layer_id: "run".into(),
            evidence: PolicyEvidenceConstraints {
                minimum_authority_class: Some(authority.into()),
                ..Default::default()
            },
            ..Default::default()
        }],
        &artifact().authority_policy,
    )
    .unwrap()
}

fn resolution_attempt() -> ResolutionAttempt {
    ResolutionAttempt {
        attempt_index: 0,
        request: ResolutionRequest {
            id: "request-1".into(),
            reason: ResolutionReason::ExplicitRequest,
            target: ResolutionTarget::Proposition {
                proposition: proposition(),
            },
            resolver_class: ResolverClass::EvidenceAcquisition,
            budget: ResolutionRequestBudget::default(),
        },
        adapter_name: "external-resolver-that-must-not-run-on-replay".into(),
        status: ResolutionAttemptStatus::AppliedEvidence,
        cost: ResolutionCost {
            added_tokens: 12,
            elapsed_ms: 34,
        },
        admitted_evidence_ids: vec!["e1".into()],
        verification_receipts: 1,
        admission_rejection: None,
    }
}

fn finalization() -> FinalizationResult {
    FinalizationResult {
        status: FinalizationStatus::GroundedAnswer,
        text: Some("feature.enabled = true".into()),
        factual_claims: 1,
        covered_claims: 1,
        factual_claim_coverage: 1.0,
        uncovered_propositions: vec![],
    }
}

fn base_thread() -> ReasoningThread {
    let mut thread = ReasoningThread::new("thread-root").unwrap();
    thread
        .record_task("event-task", "task-1", "is feature enabled")
        .unwrap();
    thread
        .record_candidate("event-candidate-1", "candidate-1", None, candidate())
        .unwrap();
    thread
        .record_accepted_artifact("event-artifact", artifact(), Verdict::Accept)
        .unwrap();
    thread
        .record_resolution_attempt("event-attempt", resolution_attempt())
        .unwrap();
    thread
}

#[test]
fn checkpoint_interrupt_resume_reconstructs_equivalent_harness_state_without_replaying_side_effects()
 {
    let mut thread = base_thread();
    let checkpoint = thread
        .create_checkpoint("event-checkpoint", "checkpoint-1")
        .unwrap();
    assert_eq!(checkpoint.schema_version, thread.schema_version);
    assert_eq!(checkpoint.policy_version, None);
    let accepted_snapshot = checkpoint.snapshot.clone();
    let attempt_count = accepted_snapshot.resolution_attempts.len();

    thread.interrupt("event-interrupt", "checkpoint-1").unwrap();
    let interrupted = replay_thread(&thread).unwrap();
    assert_eq!(
        interrupted.snapshot.status,
        ReasoningThreadStatus::Interrupted
    );
    assert!(interrupted.snapshot.finalization.is_none());
    assert_eq!(
        interrupted.snapshot.resolution_attempts.len(),
        attempt_count
    );

    thread.resume("event-resume", "checkpoint-1").unwrap();
    let resumed = replay_thread(&thread).unwrap();
    assert_eq!(resumed.snapshot, accepted_snapshot);
    assert_eq!(resumed.snapshot.resolution_attempts.len(), 1);
    assert_eq!(
        resumed.snapshot.resolution_attempts[0].adapter_name,
        "external-resolver-that-must-not-run-on-replay"
    );
}

#[test]
fn fork_is_non_destructive_and_preserves_candidate_lineage_from_checkpoint() {
    let mut source = base_thread();
    source
        .create_checkpoint("event-checkpoint", "checkpoint-1")
        .unwrap();
    let source_before = source.clone();

    let mut fork = source
        .fork_from_checkpoint("checkpoint-1", "thread-fork", "event-fork")
        .unwrap();
    assert_eq!(source, source_before);
    assert_eq!(fork.lineage.root_thread_id, "thread-root");
    assert_eq!(
        fork.lineage.parent_thread_id.as_deref(),
        Some("thread-root")
    );
    assert_eq!(
        fork.lineage.forked_from_checkpoint_id.as_deref(),
        Some("checkpoint-1")
    );

    let forked = replay_thread(&fork).unwrap();
    assert_eq!(
        forked
            .snapshot
            .current_candidate
            .as_ref()
            .unwrap()
            .candidate_id,
        "candidate-1"
    );
    fork.record_candidate(
        "event-candidate-2",
        "candidate-2",
        Some("candidate-1".into()),
        candidate(),
    )
    .unwrap();
    assert_eq!(
        replay_thread(&fork)
            .unwrap()
            .snapshot
            .current_candidate
            .unwrap()
            .candidate_id,
        "candidate-2"
    );
    assert_eq!(
        replay_thread(&source)
            .unwrap()
            .snapshot
            .current_candidate
            .unwrap()
            .candidate_id,
        "candidate-1"
    );
}

#[test]
fn policy_change_and_invalidation_are_separate_typed_events_and_checkpoint_records_policy_version()
{
    let mut thread = base_thread();
    let policy_v1 = policy("policy-v1", "secondary");
    let transition_v1 = apply_reasoning_policy(&artifact(), None, &policy_v1).unwrap();
    thread
        .record_policy_transition(
            "event-policy-v1",
            "event-invalidation-v1",
            "transition-v1",
            transition_v1,
        )
        .unwrap();
    let checkpoint = thread
        .create_checkpoint("event-checkpoint-v1", "checkpoint-v1")
        .unwrap();
    assert_eq!(checkpoint.policy_version.as_deref(), Some("policy-v1"));

    let policy_v2 = policy("policy-v2", "primary");
    let current_artifact = replay_thread(&thread).unwrap().snapshot.artifact.unwrap();
    let transition_v2 =
        apply_reasoning_policy(&current_artifact, Some(&policy_v1), &policy_v2).unwrap();
    assert!(!transition_v2.invalidations.is_empty());
    thread
        .record_policy_transition(
            "event-policy-v2",
            "event-invalidation-v2",
            "transition-v2",
            transition_v2,
        )
        .unwrap();

    assert!(
        thread
            .events
            .iter()
            .any(|event| matches!(event.kind, ReasoningThreadEventKind::PolicyChanged { .. }))
    );
    assert!(thread.events.iter().any(|event| matches!(
        event.kind,
        ReasoningThreadEventKind::StateInvalidated { .. }
    )));
    let replay = replay_thread(&thread).unwrap();
    assert_eq!(replay.snapshot.policy.unwrap().version_id, "policy-v2");
    assert_eq!(replay.snapshot.verdict, Some(Verdict::Unknown));
    assert!(replay.snapshot.finalization.is_none());
}

#[test]
fn incomplete_policy_transition_and_interrupted_thread_cannot_finalize() {
    let mut thread = base_thread();
    let policy_v1 = policy("policy-v1", "secondary");
    thread.events.push(ReasoningThreadEvent {
        sequence: thread.events.len() as u64 + 1,
        event_id: "event-policy-only".into(),
        causation_event_id: None,
        kind: ReasoningThreadEventKind::PolicyChanged {
            transition_id: "transition-incomplete".into(),
            previous_policy_version: None,
            policy: policy_v1,
        },
    });
    let pending = replay_thread(&thread).unwrap();
    assert_eq!(
        pending.snapshot.status,
        ReasoningThreadStatus::NeedsReevaluation
    );
    assert!(pending.snapshot.finalization.is_none());
    assert!(matches!(
        thread.record_finalization("event-final", finalization()),
        Err(ReasoningThreadError::FinalizationNotAllowed(
            ReasoningThreadStatus::NeedsReevaluation
        ))
    ));
    assert!(matches!(
        thread.create_checkpoint("event-cp", "checkpoint-pending"),
        Err(ReasoningThreadError::UnsafeCheckpointBoundary)
    ));

    let mut interrupted = base_thread();
    interrupted
        .create_checkpoint("event-checkpoint", "checkpoint-1")
        .unwrap();
    interrupted
        .interrupt("event-interrupt", "checkpoint-1")
        .unwrap();
    assert!(matches!(
        interrupted.record_finalization("event-final", finalization()),
        Err(ReasoningThreadError::FinalizationNotAllowed(
            ReasoningThreadStatus::Interrupted
        ))
    ));
    assert!(matches!(
        interrupted.record_candidate("event-illegal", "candidate-illegal", None, candidate()),
        Err(ReasoningThreadError::InterruptedThreadIsFrozen)
    ));
}

#[test]
fn finalized_source_is_immutable_but_can_fork_from_prior_checkpoint() {
    let mut thread = base_thread();
    thread
        .create_checkpoint("event-checkpoint", "checkpoint-1")
        .unwrap();
    thread
        .record_finalization("event-finalized", finalization())
        .unwrap();
    assert_eq!(
        replay_thread(&thread).unwrap().snapshot.status,
        ReasoningThreadStatus::Finalized
    );
    assert!(matches!(
        thread.record_candidate("event-after-final", "candidate-2", None, candidate()),
        Err(ReasoningThreadError::FinalizedThreadIsImmutable)
    ));

    let fork = thread
        .fork_from_checkpoint("checkpoint-1", "thread-after-final", "event-fork")
        .unwrap();
    assert_eq!(
        replay_thread(&fork).unwrap().snapshot.status,
        ReasoningThreadStatus::Active
    );
}

#[test]
fn tampered_checkpoint_or_event_sequence_fails_deterministic_replay() {
    let mut thread = base_thread();
    thread
        .create_checkpoint("event-checkpoint", "checkpoint-1")
        .unwrap();
    let mut tampered_checkpoint = thread.clone();
    tampered_checkpoint.checkpoints[0].snapshot.verdict = Some(Verdict::Reject);
    assert!(matches!(
        validate_thread(&tampered_checkpoint),
        Err(ReasoningThreadError::CheckpointReplayMismatch)
    ));

    let mut tampered_sequence = thread;
    tampered_sequence.events[1].sequence = 99;
    assert!(matches!(
        validate_thread(&tampered_sequence),
        Err(ReasoningThreadError::EventSequenceMismatch { .. })
    ));
}

#[test]
fn policy_transition_replay_rejects_tampered_authority_state_and_rolls_back_append() {
    let mut thread = base_thread();
    let policy_v1 = policy("policy-v1", "secondary");
    let mut transition = apply_reasoning_policy(&artifact(), None, &policy_v1).unwrap();
    transition.invalidations.push(PolicyInvalidation {
        target: PolicyInvalidationTarget::Finalization,
        reason: PolicyInvalidationReason::UpstreamStateChanged,
    });
    let before = thread.clone();
    assert!(matches!(
        thread.record_policy_transition(
            "event-policy-tampered",
            "event-invalidation-tampered",
            "transition-tampered",
            transition,
        ),
        Err(ReasoningThreadError::PolicyTransitionReplayMismatch)
    ));
    assert_eq!(thread, before);
}

#[test]
fn accepted_artifact_cannot_bypass_active_policy_after_policy_is_recorded() {
    let mut thread = base_thread();
    let policy_v1 = policy("policy-v1", "secondary");
    let transition_v1 = apply_reasoning_policy(&artifact(), None, &policy_v1).unwrap();
    thread
        .record_policy_transition(
            "event-policy-v1",
            "event-invalidation-v1",
            "transition-v1",
            transition_v1,
        )
        .unwrap();

    let policy_v2 = policy("policy-v2", "primary");
    let current_artifact = replay_thread(&thread).unwrap().snapshot.artifact.unwrap();
    let transition_v2 =
        apply_reasoning_policy(&current_artifact, Some(&policy_v1), &policy_v2).unwrap();
    thread
        .record_policy_transition(
            "event-policy-v2",
            "event-invalidation-v2",
            "transition-v2",
            transition_v2,
        )
        .unwrap();

    assert!(matches!(
        thread.record_accepted_artifact("event-bypass", artifact(), Verdict::Accept),
        Err(ReasoningThreadError::ArtifactNotAdmissibleUnderCurrentPolicy)
    ));
}

#[test]
fn persisted_contract_contains_only_explicit_typed_runtime_state_not_hidden_chain_of_thought() {
    let mut thread = base_thread();
    thread
        .record_soft_observation(
            "event-soft",
            SoftJudgeObservation {
                judge: SoftJudgeIdentity {
                    judge_id: "judge".into(),
                    model_id: "model".into(),
                    configuration_id: "config".into(),
                },
                request_id: "request".into(),
                decision: SoftJudgeDecision::Abstain,
                finding: None,
            },
        )
        .unwrap();
    let json = serde_json::to_string(&thread).unwrap();
    assert!(!json.contains("chain_of_thought"));
    assert!(!json.contains("hidden_reasoning"));
    assert!(!json.contains("reasoning_text"));
    assert!(json.contains("soft_finding_recorded"));
}
