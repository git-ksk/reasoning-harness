use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    FinalizationResult, FinalizationStatus, PolicyInvalidation, ReasoningArtifact,
    ReasoningCandidate, ReasoningPolicy, ReasoningPolicyTransition, ResolutionAttempt,
    SoftJudgeObservation, Verdict, apply_reasoning_policy, validate_artifact,
};

pub const REASONING_THREAD_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReasoningThreadLineage {
    pub root_thread_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_thread_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forked_from_checkpoint_id: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningThreadStatus {
    #[default]
    Active,
    NeedsReevaluation,
    Interrupted,
    Finalized,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThreadCandidateState {
    pub candidate_id: String,
    pub candidate: ReasoningCandidate,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReasoningThreadSnapshot {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_candidate: Option<ThreadCandidateState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact: Option<ReasoningArtifact>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy: Option<ReasoningPolicy>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verdict: Option<Verdict>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finalization: Option<FinalizationResult>,
    #[serde(default)]
    pub resolution_attempts: Vec<ResolutionAttempt>,
    #[serde(default)]
    pub soft_observations: Vec<SoftJudgeObservation>,
    #[serde(default)]
    pub status: ReasoningThreadStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReasoningCheckpoint {
    pub checkpoint_id: String,
    pub thread_id: String,
    pub schema_version: u32,
    pub event_sequence: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_version: Option<String>,
    pub snapshot: ReasoningThreadSnapshot,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReasoningThreadEvent {
    pub sequence: u64,
    pub event_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub causation_event_id: Option<String>,
    pub kind: ReasoningThreadEventKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ReasoningThreadEventKind {
    TaskReceived {
        task_id: String,
        task: String,
    },
    CandidateRecorded {
        candidate_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        replaces_candidate_id: Option<String>,
        candidate: ReasoningCandidate,
    },
    ArtifactAccepted {
        artifact: Box<ReasoningArtifact>,
        verdict: Verdict,
    },
    SoftFindingRecorded {
        observation: SoftJudgeObservation,
    },
    ResolutionAttemptRecorded {
        attempt: ResolutionAttempt,
    },
    PolicyChanged {
        transition_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        previous_policy_version: Option<String>,
        policy: ReasoningPolicy,
    },
    StateInvalidated {
        transition_id: String,
        transition: Box<ReasoningPolicyTransition>,
    },
    CheckpointCreated {
        checkpoint_id: String,
    },
    Interrupted {
        checkpoint_id: String,
    },
    Resumed {
        checkpoint_id: String,
    },
    ForkedFrom {
        source_thread_id: String,
        source_root_thread_id: String,
        source_checkpoint_id: String,
        snapshot: Box<ReasoningThreadSnapshot>,
    },
    AnswerFinalized {
        finalization: FinalizationResult,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReasoningThread {
    pub thread_id: String,
    pub schema_version: u32,
    pub lineage: ReasoningThreadLineage,
    #[serde(default)]
    pub events: Vec<ReasoningThreadEvent>,
    #[serde(default)]
    pub checkpoints: Vec<ReasoningCheckpoint>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReasoningThreadReplay {
    pub snapshot: ReasoningThreadSnapshot,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_sequence: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interrupted_checkpoint_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pending_policy_transition_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ReasoningThreadError {
    #[error("reasoning thread id must not be empty")]
    EmptyThreadId,
    #[error("reasoning thread schema version {actual} is unsupported; expected {expected}")]
    UnsupportedSchemaVersion { expected: u32, actual: u32 },
    #[error("reasoning thread lineage root id must not be empty")]
    EmptyRootThreadId,
    #[error("reasoning event id must not be empty")]
    EmptyEventId,
    #[error("duplicate reasoning event id: {0}")]
    DuplicateEventId(String),
    #[error("reasoning event sequence mismatch: expected {expected}, got {actual}")]
    EventSequenceMismatch { expected: u64, actual: u64 },
    #[error("reasoning event causation id does not refer to an earlier event: {0}")]
    UnknownCausationEvent(String),
    #[error("task identity must not be empty")]
    EmptyTaskId,
    #[error("task text must not be empty")]
    EmptyTask,
    #[error("thread already has a different task")]
    TaskMismatch,
    #[error("candidate identity must not be empty")]
    EmptyCandidateId,
    #[error("replacement candidate references unknown prior candidate {0}")]
    UnknownReplacedCandidate(String),
    #[error("accepted artifact is invalid: {0:?}")]
    InvalidArtifact(Vec<String>),
    #[error("accepted artifact task does not match the thread task")]
    ArtifactTaskMismatch,
    #[error("policy transition id must not be empty")]
    EmptyPolicyTransitionId,
    #[error("policy transition {0} is already pending")]
    PolicyTransitionAlreadyPending(String),
    #[error("state invalidation does not match pending policy transition {0}")]
    PolicyTransitionMismatch(String),
    #[error("policy transition previous version does not match current thread policy")]
    PolicyVersionMismatch,
    #[error("policy transition could not be deterministically re-evaluated: {0}")]
    PolicyTransitionReevaluationFailed(String),
    #[error("policy transition event does not match deterministic #27 re-evaluation")]
    PolicyTransitionReplayMismatch,
    #[error("accepted artifact is not already admissible under the active reasoning policy")]
    ArtifactNotAdmissibleUnderCurrentPolicy,
    #[error("checkpoint id must not be empty")]
    EmptyCheckpointId,
    #[error("duplicate checkpoint id: {0}")]
    DuplicateCheckpointId(String),
    #[error("checkpoint {0} does not exist")]
    MissingCheckpoint(String),
    #[error("checkpoint {0} belongs to another thread")]
    CheckpointThreadMismatch(String),
    #[error("checkpoint {0} has an unsupported schema version")]
    CheckpointSchemaMismatch(String),
    #[error("checkpoint can only be created at an active accepted-state boundary")]
    UnsafeCheckpointBoundary,
    #[error("checkpoint snapshot does not match deterministic replay at its event sequence")]
    CheckpointReplayMismatch,
    #[error("interrupt requires the latest safe checkpoint")]
    InterruptRequiresLatestCheckpoint,
    #[error("resume requires an interrupted thread")]
    ResumeRequiresInterruptedThread,
    #[error("resume checkpoint does not match the interrupt checkpoint")]
    ResumeCheckpointMismatch,
    #[error("interrupted thread accepts only resume")]
    InterruptedThreadIsFrozen,
    #[error("policy change is pending deterministic re-evaluation")]
    PolicyReevaluationPending,
    #[error("finalized thread is immutable; fork from a checkpoint to continue")]
    FinalizedThreadIsImmutable,
    #[error("finalization is not allowed while thread status is {0:?}")]
    FinalizationNotAllowed(ReasoningThreadStatus),
    #[error("requires_verification is not a finalized answer")]
    RequiresVerificationIsNotFinal,
    #[error("fork thread id must differ from source thread id")]
    ForkMustUseNewThreadId,
    #[error("fork checkpoint must contain an active accepted-state snapshot")]
    UnsafeForkCheckpoint,
    #[error("fork lineage does not match its source event")]
    InvalidForkLineage,
}

/// Persistence is intentionally abstract. Core defines serializable thread state and this
/// minimal load/save boundary but owns no filesystem, database, or cloud backend.
pub trait ReasoningThreadStore {
    type Error;

    fn load(&self, thread_id: &str) -> Result<Option<ReasoningThread>, Self::Error>;
    fn save(&mut self, thread: &ReasoningThread) -> Result<(), Self::Error>;
}

impl ReasoningThread {
    pub fn new(thread_id: impl Into<String>) -> Result<Self, ReasoningThreadError> {
        let thread_id = thread_id.into();
        if thread_id.trim().is_empty() {
            return Err(ReasoningThreadError::EmptyThreadId);
        }
        Ok(Self {
            thread_id: thread_id.clone(),
            schema_version: REASONING_THREAD_SCHEMA_VERSION,
            lineage: ReasoningThreadLineage {
                root_thread_id: thread_id,
                parent_thread_id: None,
                forked_from_checkpoint_id: None,
            },
            events: Vec::new(),
            checkpoints: Vec::new(),
        })
    }

    pub fn record_task(
        &mut self,
        event_id: impl Into<String>,
        task_id: impl Into<String>,
        task: impl Into<String>,
    ) -> Result<(), ReasoningThreadError> {
        self.push_event(
            event_id.into(),
            None,
            ReasoningThreadEventKind::TaskReceived {
                task_id: task_id.into(),
                task: task.into(),
            },
        )
    }

    pub fn record_candidate(
        &mut self,
        event_id: impl Into<String>,
        candidate_id: impl Into<String>,
        replaces_candidate_id: Option<String>,
        candidate: ReasoningCandidate,
    ) -> Result<(), ReasoningThreadError> {
        self.push_event(
            event_id.into(),
            None,
            ReasoningThreadEventKind::CandidateRecorded {
                candidate_id: candidate_id.into(),
                replaces_candidate_id,
                candidate,
            },
        )
    }

    pub fn record_accepted_artifact(
        &mut self,
        event_id: impl Into<String>,
        artifact: ReasoningArtifact,
        verdict: Verdict,
    ) -> Result<(), ReasoningThreadError> {
        self.push_event(
            event_id.into(),
            None,
            ReasoningThreadEventKind::ArtifactAccepted {
                artifact: Box::new(artifact),
                verdict,
            },
        )
    }

    pub fn record_soft_observation(
        &mut self,
        event_id: impl Into<String>,
        observation: SoftJudgeObservation,
    ) -> Result<(), ReasoningThreadError> {
        self.push_event(
            event_id.into(),
            None,
            ReasoningThreadEventKind::SoftFindingRecorded { observation },
        )
    }

    pub fn record_resolution_attempt(
        &mut self,
        event_id: impl Into<String>,
        attempt: ResolutionAttempt,
    ) -> Result<(), ReasoningThreadError> {
        self.push_event(
            event_id.into(),
            None,
            ReasoningThreadEventKind::ResolutionAttemptRecorded { attempt },
        )
    }

    pub fn record_policy_transition(
        &mut self,
        change_event_id: impl Into<String>,
        invalidation_event_id: impl Into<String>,
        transition_id: impl Into<String>,
        transition: ReasoningPolicyTransition,
    ) -> Result<(), ReasoningThreadError> {
        let change_event_id = change_event_id.into();
        let invalidation_event_id = invalidation_event_id.into();
        let transition_id = transition_id.into();
        let before = self.clone();
        self.push_event(
            change_event_id.clone(),
            None,
            ReasoningThreadEventKind::PolicyChanged {
                transition_id: transition_id.clone(),
                previous_policy_version: transition.previous_policy_version.clone(),
                policy: transition.policy.clone(),
            },
        )?;
        if let Err(error) = self.push_event(
            invalidation_event_id,
            Some(change_event_id),
            ReasoningThreadEventKind::StateInvalidated {
                transition_id,
                transition: Box::new(transition),
            },
        ) {
            *self = before;
            return Err(error);
        }
        Ok(())
    }

    pub fn create_checkpoint(
        &mut self,
        event_id: impl Into<String>,
        checkpoint_id: impl Into<String>,
    ) -> Result<ReasoningCheckpoint, ReasoningThreadError> {
        validate_thread_header(self)?;
        let checkpoint_id = checkpoint_id.into();
        if checkpoint_id.trim().is_empty() {
            return Err(ReasoningThreadError::EmptyCheckpointId);
        }
        if self
            .checkpoints
            .iter()
            .any(|checkpoint| checkpoint.checkpoint_id == checkpoint_id)
        {
            return Err(ReasoningThreadError::DuplicateCheckpointId(checkpoint_id));
        }
        let replay = replay_thread(self)?;
        if replay.snapshot.status != ReasoningThreadStatus::Active
            || replay.snapshot.artifact.is_none()
            || replay.pending_policy_transition_id.is_some()
        {
            return Err(ReasoningThreadError::UnsafeCheckpointBoundary);
        }
        let event_id = event_id.into();
        validate_new_event_id(self, &event_id)?;
        let sequence = next_sequence(self);
        let checkpoint = ReasoningCheckpoint {
            checkpoint_id: checkpoint_id.clone(),
            thread_id: self.thread_id.clone(),
            schema_version: self.schema_version,
            event_sequence: sequence,
            policy_version: replay
                .snapshot
                .policy
                .as_ref()
                .map(|policy| policy.version_id.clone()),
            snapshot: replay.snapshot,
        };
        self.checkpoints.push(checkpoint.clone());
        if let Err(error) = self.push_event(
            event_id,
            None,
            ReasoningThreadEventKind::CheckpointCreated { checkpoint_id },
        ) {
            self.checkpoints.pop();
            return Err(error);
        }
        Ok(checkpoint)
    }

    pub fn interrupt(
        &mut self,
        event_id: impl Into<String>,
        checkpoint_id: impl Into<String>,
    ) -> Result<(), ReasoningThreadError> {
        let checkpoint_id = checkpoint_id.into();
        let replay = replay_thread(self)?;
        if replay.snapshot.status != ReasoningThreadStatus::Active {
            return Err(ReasoningThreadError::UnsafeCheckpointBoundary);
        }
        let checkpoint = checkpoint(self, &checkpoint_id)?;
        if checkpoint.event_sequence != replay.last_sequence.unwrap_or_default() {
            return Err(ReasoningThreadError::InterruptRequiresLatestCheckpoint);
        }
        self.push_event(
            event_id.into(),
            None,
            ReasoningThreadEventKind::Interrupted { checkpoint_id },
        )
    }

    pub fn resume(
        &mut self,
        event_id: impl Into<String>,
        checkpoint_id: impl Into<String>,
    ) -> Result<(), ReasoningThreadError> {
        let checkpoint_id = checkpoint_id.into();
        let replay = replay_thread(self)?;
        if replay.snapshot.status != ReasoningThreadStatus::Interrupted {
            return Err(ReasoningThreadError::ResumeRequiresInterruptedThread);
        }
        if replay.interrupted_checkpoint_id.as_deref() != Some(checkpoint_id.as_str()) {
            return Err(ReasoningThreadError::ResumeCheckpointMismatch);
        }
        self.push_event(
            event_id.into(),
            None,
            ReasoningThreadEventKind::Resumed { checkpoint_id },
        )
    }

    pub fn record_finalization(
        &mut self,
        event_id: impl Into<String>,
        finalization: FinalizationResult,
    ) -> Result<(), ReasoningThreadError> {
        if finalization.status == FinalizationStatus::RequiresVerification {
            return Err(ReasoningThreadError::RequiresVerificationIsNotFinal);
        }
        let replay = replay_thread(self)?;
        if replay.snapshot.status != ReasoningThreadStatus::Active
            || replay.pending_policy_transition_id.is_some()
            || replay.snapshot.artifact.is_none()
            || replay.snapshot.verdict.is_none()
        {
            return Err(ReasoningThreadError::FinalizationNotAllowed(
                replay.snapshot.status,
            ));
        }
        self.push_event(
            event_id.into(),
            None,
            ReasoningThreadEventKind::AnswerFinalized { finalization },
        )
    }

    pub fn fork_from_checkpoint(
        &self,
        checkpoint_id: &str,
        new_thread_id: impl Into<String>,
        event_id: impl Into<String>,
    ) -> Result<Self, ReasoningThreadError> {
        validate_thread(self)?;
        let new_thread_id = new_thread_id.into();
        if new_thread_id == self.thread_id {
            return Err(ReasoningThreadError::ForkMustUseNewThreadId);
        }
        let checkpoint = checkpoint(self, checkpoint_id)?;
        if checkpoint.snapshot.status != ReasoningThreadStatus::Active
            || checkpoint.snapshot.artifact.is_none()
        {
            return Err(ReasoningThreadError::UnsafeForkCheckpoint);
        }
        let mut fork = ReasoningThread::new(new_thread_id)?;
        fork.lineage = ReasoningThreadLineage {
            root_thread_id: self.lineage.root_thread_id.clone(),
            parent_thread_id: Some(self.thread_id.clone()),
            forked_from_checkpoint_id: Some(checkpoint_id.to_string()),
        };
        fork.push_event(
            event_id.into(),
            None,
            ReasoningThreadEventKind::ForkedFrom {
                source_thread_id: self.thread_id.clone(),
                source_root_thread_id: self.lineage.root_thread_id.clone(),
                source_checkpoint_id: checkpoint_id.to_string(),
                snapshot: Box::new(checkpoint.snapshot.clone()),
            },
        )?;
        Ok(fork)
    }

    fn push_event(
        &mut self,
        event_id: String,
        causation_event_id: Option<String>,
        kind: ReasoningThreadEventKind,
    ) -> Result<(), ReasoningThreadError> {
        validate_thread_header(self)?;
        validate_new_event_id(self, &event_id)?;
        if let Some(causation) = &causation_event_id {
            if !self.events.iter().any(|event| &event.event_id == causation) {
                return Err(ReasoningThreadError::UnknownCausationEvent(
                    causation.clone(),
                ));
            }
        }
        self.events.push(ReasoningThreadEvent {
            sequence: next_sequence(self),
            event_id,
            causation_event_id,
            kind,
        });
        if let Err(error) = replay_thread(self) {
            self.events.pop();
            return Err(error);
        }
        Ok(())
    }
}

pub fn replay_thread(
    thread: &ReasoningThread,
) -> Result<ReasoningThreadReplay, ReasoningThreadError> {
    validate_thread_header(thread)?;
    validate_checkpoint_headers(thread)?;

    let mut snapshot = ReasoningThreadSnapshot::default();
    let mut event_ids = BTreeSet::new();
    let mut candidate_ids = BTreeSet::new();
    let mut interrupted_checkpoint_id = None;
    let mut pending_policy_transition_id: Option<String> = None;
    let mut pending_previous_policy: Option<ReasoningPolicy> = None;

    for (index, event) in thread.events.iter().enumerate() {
        let expected = index as u64 + 1;
        if event.sequence != expected {
            return Err(ReasoningThreadError::EventSequenceMismatch {
                expected,
                actual: event.sequence,
            });
        }
        if event.event_id.trim().is_empty() {
            return Err(ReasoningThreadError::EmptyEventId);
        }
        if !event_ids.insert(event.event_id.clone()) {
            return Err(ReasoningThreadError::DuplicateEventId(
                event.event_id.clone(),
            ));
        }
        if let Some(causation) = &event.causation_event_id {
            if !event_ids.contains(causation) || causation == &event.event_id {
                return Err(ReasoningThreadError::UnknownCausationEvent(
                    causation.clone(),
                ));
            }
        }

        match snapshot.status {
            ReasoningThreadStatus::Interrupted
                if !matches!(event.kind, ReasoningThreadEventKind::Resumed { .. }) =>
            {
                return Err(ReasoningThreadError::InterruptedThreadIsFrozen);
            }
            ReasoningThreadStatus::NeedsReevaluation
                if !matches!(
                    event.kind,
                    ReasoningThreadEventKind::StateInvalidated { .. }
                ) =>
            {
                return Err(ReasoningThreadError::PolicyReevaluationPending);
            }
            ReasoningThreadStatus::Finalized => {
                return Err(ReasoningThreadError::FinalizedThreadIsImmutable);
            }
            _ => {}
        }

        match &event.kind {
            ReasoningThreadEventKind::TaskReceived { task_id, task } => {
                if task_id.trim().is_empty() {
                    return Err(ReasoningThreadError::EmptyTaskId);
                }
                if task.trim().is_empty() {
                    return Err(ReasoningThreadError::EmptyTask);
                }
                match (&snapshot.task_id, &snapshot.task) {
                    (None, None) => {
                        snapshot.task_id = Some(task_id.clone());
                        snapshot.task = Some(task.clone());
                    }
                    (Some(existing_id), Some(existing_task))
                        if existing_id == task_id && existing_task == task => {}
                    _ => return Err(ReasoningThreadError::TaskMismatch),
                }
            }
            ReasoningThreadEventKind::CandidateRecorded {
                candidate_id,
                replaces_candidate_id,
                candidate,
            } => {
                if candidate_id.trim().is_empty() {
                    return Err(ReasoningThreadError::EmptyCandidateId);
                }
                if let Some(replaced) = replaces_candidate_id {
                    if !candidate_ids.contains(replaced) {
                        return Err(ReasoningThreadError::UnknownReplacedCandidate(
                            replaced.clone(),
                        ));
                    }
                }
                candidate_ids.insert(candidate_id.clone());
                snapshot.current_candidate = Some(ThreadCandidateState {
                    candidate_id: candidate_id.clone(),
                    candidate: candidate.clone(),
                });
            }
            ReasoningThreadEventKind::ArtifactAccepted { artifact, verdict } => {
                validate_accepted_artifact(artifact)?;
                if snapshot
                    .task
                    .as_ref()
                    .is_some_and(|task| task != &artifact.task)
                {
                    return Err(ReasoningThreadError::ArtifactTaskMismatch);
                }
                if let Some(policy) = snapshot.policy.as_ref() {
                    let reevaluated = apply_reasoning_policy(artifact, Some(policy), policy)
                        .map_err(|error| {
                            ReasoningThreadError::PolicyTransitionReevaluationFailed(
                                error.to_string(),
                            )
                        })?;
                    if reevaluated.artifact != **artifact
                        || reevaluated.verdict_after_re_evaluation != *verdict
                        || !reevaluated.invalidations.is_empty()
                    {
                        return Err(ReasoningThreadError::ArtifactNotAdmissibleUnderCurrentPolicy);
                    }
                }
                snapshot.artifact = Some((**artifact).clone());
                snapshot.verdict = Some(*verdict);
                snapshot.finalization = None;
                snapshot.status = ReasoningThreadStatus::Active;
                pending_policy_transition_id = None;
                pending_previous_policy = None;
            }
            ReasoningThreadEventKind::SoftFindingRecorded { observation } => {
                snapshot.soft_observations.push(observation.clone());
            }
            ReasoningThreadEventKind::ResolutionAttemptRecorded { attempt } => {
                // This is a historical observation only. Replay never invokes the named adapter.
                snapshot.resolution_attempts.push(attempt.clone());
            }
            ReasoningThreadEventKind::PolicyChanged {
                transition_id,
                previous_policy_version,
                policy,
            } => {
                if transition_id.trim().is_empty() {
                    return Err(ReasoningThreadError::EmptyPolicyTransitionId);
                }
                if pending_policy_transition_id.is_some() {
                    return Err(ReasoningThreadError::PolicyTransitionAlreadyPending(
                        transition_id.clone(),
                    ));
                }
                let current_version = snapshot.policy.as_ref().map(|policy| &policy.version_id);
                if previous_policy_version.as_ref() != current_version {
                    return Err(ReasoningThreadError::PolicyVersionMismatch);
                }
                pending_previous_policy = snapshot.policy.clone();
                snapshot.policy = Some(policy.clone());
                snapshot.finalization = None;
                snapshot.status = ReasoningThreadStatus::NeedsReevaluation;
                pending_policy_transition_id = Some(transition_id.clone());
            }
            ReasoningThreadEventKind::StateInvalidated {
                transition_id,
                transition,
            } => {
                if pending_policy_transition_id.as_deref() != Some(transition_id.as_str()) {
                    return Err(ReasoningThreadError::PolicyTransitionMismatch(
                        transition_id.clone(),
                    ));
                }
                if snapshot.policy.as_ref() != Some(&transition.policy)
                    || transition.previous_policy_version
                        != pending_previous_policy
                            .as_ref()
                            .map(|policy| policy.version_id.clone())
                {
                    return Err(ReasoningThreadError::PolicyVersionMismatch);
                }
                let source_artifact = snapshot
                    .artifact
                    .as_ref()
                    .ok_or(ReasoningThreadError::UnsafeCheckpointBoundary)?;
                let reevaluated = apply_reasoning_policy(
                    source_artifact,
                    pending_previous_policy.as_ref(),
                    &transition.policy,
                )
                .map_err(|error| {
                    ReasoningThreadError::PolicyTransitionReevaluationFailed(error.to_string())
                })?;
                if reevaluated != **transition {
                    return Err(ReasoningThreadError::PolicyTransitionReplayMismatch);
                }
                validate_accepted_artifact(&transition.artifact)?;
                snapshot.artifact = Some(transition.artifact.clone());
                snapshot.verdict = Some(transition.verdict_after_re_evaluation);
                if transition.finalization_invalidated {
                    snapshot.finalization = None;
                }
                snapshot.status = ReasoningThreadStatus::Active;
                pending_policy_transition_id = None;
                pending_previous_policy = None;
            }
            ReasoningThreadEventKind::CheckpointCreated { checkpoint_id } => {
                let checkpoint = checkpoint(thread, checkpoint_id)?;
                if checkpoint.event_sequence != event.sequence {
                    return Err(ReasoningThreadError::CheckpointReplayMismatch);
                }
                if checkpoint.snapshot != snapshot {
                    return Err(ReasoningThreadError::CheckpointReplayMismatch);
                }
            }
            ReasoningThreadEventKind::Interrupted { checkpoint_id } => {
                if snapshot.status != ReasoningThreadStatus::Active
                    || pending_policy_transition_id.is_some()
                {
                    return Err(ReasoningThreadError::UnsafeCheckpointBoundary);
                }
                let checkpoint = checkpoint(thread, checkpoint_id)?;
                if checkpoint.event_sequence + 1 != event.sequence {
                    return Err(ReasoningThreadError::InterruptRequiresLatestCheckpoint);
                }
                snapshot = checkpoint.snapshot.clone();
                snapshot.status = ReasoningThreadStatus::Interrupted;
                snapshot.finalization = None;
                interrupted_checkpoint_id = Some(checkpoint_id.clone());
            }
            ReasoningThreadEventKind::Resumed { checkpoint_id } => {
                if snapshot.status != ReasoningThreadStatus::Interrupted {
                    return Err(ReasoningThreadError::ResumeRequiresInterruptedThread);
                }
                if interrupted_checkpoint_id.as_deref() != Some(checkpoint_id.as_str()) {
                    return Err(ReasoningThreadError::ResumeCheckpointMismatch);
                }
                let checkpoint = checkpoint(thread, checkpoint_id)?;
                snapshot = checkpoint.snapshot.clone();
                snapshot.status = ReasoningThreadStatus::Active;
                snapshot.finalization = None;
                interrupted_checkpoint_id = None;
            }
            ReasoningThreadEventKind::ForkedFrom {
                source_thread_id,
                source_root_thread_id,
                source_checkpoint_id,
                snapshot: fork_snapshot,
            } => {
                if event.sequence != 1
                    || thread.lineage.parent_thread_id.as_deref() != Some(source_thread_id.as_str())
                    || thread.lineage.forked_from_checkpoint_id.as_deref()
                        != Some(source_checkpoint_id.as_str())
                    || thread.lineage.root_thread_id != *source_root_thread_id
                {
                    return Err(ReasoningThreadError::InvalidForkLineage);
                }
                if fork_snapshot.status != ReasoningThreadStatus::Active
                    || fork_snapshot.artifact.is_none()
                {
                    return Err(ReasoningThreadError::UnsafeForkCheckpoint);
                }
                snapshot = (**fork_snapshot).clone();
                snapshot.status = ReasoningThreadStatus::Active;
                snapshot.finalization = None;
                if let Some(candidate) = &snapshot.current_candidate {
                    candidate_ids.insert(candidate.candidate_id.clone());
                }
            }
            ReasoningThreadEventKind::AnswerFinalized { finalization } => {
                if snapshot.status != ReasoningThreadStatus::Active
                    || pending_policy_transition_id.is_some()
                    || snapshot.artifact.is_none()
                    || snapshot.verdict.is_none()
                {
                    return Err(ReasoningThreadError::FinalizationNotAllowed(
                        snapshot.status,
                    ));
                }
                if finalization.status == FinalizationStatus::RequiresVerification {
                    return Err(ReasoningThreadError::RequiresVerificationIsNotFinal);
                }
                snapshot.finalization = Some(finalization.clone());
                snapshot.status = ReasoningThreadStatus::Finalized;
            }
        }
    }

    // Every stored checkpoint must match the state that deterministic replay reaches at the
    // corresponding CheckpointCreated event. This detects stale or externally mutated snapshots.
    for checkpoint in &thread.checkpoints {
        let event = thread
            .events
            .iter()
            .find(|event| event.sequence == checkpoint.event_sequence);
        if !matches!(
            event.map(|event| &event.kind),
            Some(ReasoningThreadEventKind::CheckpointCreated { checkpoint_id })
                if checkpoint_id == &checkpoint.checkpoint_id
        ) {
            return Err(ReasoningThreadError::CheckpointReplayMismatch);
        }
    }

    Ok(ReasoningThreadReplay {
        snapshot,
        last_sequence: thread.events.last().map(|event| event.sequence),
        interrupted_checkpoint_id,
        pending_policy_transition_id,
    })
}

pub fn validate_thread(thread: &ReasoningThread) -> Result<(), ReasoningThreadError> {
    replay_thread(thread).map(|_| ())
}

fn validate_thread_header(thread: &ReasoningThread) -> Result<(), ReasoningThreadError> {
    if thread.thread_id.trim().is_empty() {
        return Err(ReasoningThreadError::EmptyThreadId);
    }
    if thread.schema_version != REASONING_THREAD_SCHEMA_VERSION {
        return Err(ReasoningThreadError::UnsupportedSchemaVersion {
            expected: REASONING_THREAD_SCHEMA_VERSION,
            actual: thread.schema_version,
        });
    }
    if thread.lineage.root_thread_id.trim().is_empty() {
        return Err(ReasoningThreadError::EmptyRootThreadId);
    }
    Ok(())
}

fn validate_checkpoint_headers(thread: &ReasoningThread) -> Result<(), ReasoningThreadError> {
    let mut ids = BTreeSet::new();
    for checkpoint in &thread.checkpoints {
        if checkpoint.checkpoint_id.trim().is_empty() {
            return Err(ReasoningThreadError::EmptyCheckpointId);
        }
        if !ids.insert(checkpoint.checkpoint_id.clone()) {
            return Err(ReasoningThreadError::DuplicateCheckpointId(
                checkpoint.checkpoint_id.clone(),
            ));
        }
        if checkpoint.thread_id != thread.thread_id {
            return Err(ReasoningThreadError::CheckpointThreadMismatch(
                checkpoint.checkpoint_id.clone(),
            ));
        }
        if checkpoint.schema_version != thread.schema_version {
            return Err(ReasoningThreadError::CheckpointSchemaMismatch(
                checkpoint.checkpoint_id.clone(),
            ));
        }
        if checkpoint.policy_version
            != checkpoint
                .snapshot
                .policy
                .as_ref()
                .map(|policy| policy.version_id.clone())
        {
            return Err(ReasoningThreadError::CheckpointReplayMismatch);
        }
    }
    Ok(())
}

fn validate_new_event_id(
    thread: &ReasoningThread,
    event_id: &str,
) -> Result<(), ReasoningThreadError> {
    if event_id.trim().is_empty() {
        return Err(ReasoningThreadError::EmptyEventId);
    }
    if thread.events.iter().any(|event| event.event_id == event_id) {
        return Err(ReasoningThreadError::DuplicateEventId(event_id.into()));
    }
    Ok(())
}

fn next_sequence(thread: &ReasoningThread) -> u64 {
    thread.events.len() as u64 + 1
}

fn checkpoint<'a>(
    thread: &'a ReasoningThread,
    checkpoint_id: &str,
) -> Result<&'a ReasoningCheckpoint, ReasoningThreadError> {
    thread
        .checkpoints
        .iter()
        .find(|checkpoint| checkpoint.checkpoint_id == checkpoint_id)
        .ok_or_else(|| ReasoningThreadError::MissingCheckpoint(checkpoint_id.into()))
}

fn validate_accepted_artifact(artifact: &ReasoningArtifact) -> Result<(), ReasoningThreadError> {
    let report = validate_artifact(artifact);
    if report.is_ok() {
        Ok(())
    } else {
        Err(ReasoningThreadError::InvalidArtifact(
            report
                .diagnostics
                .into_iter()
                .map(|diagnostic| format!("{}: {}", diagnostic.code, diagnostic.message))
                .collect(),
        ))
    }
}

/// Converts #27 invalidation details into a standalone typed event payload when callers need
/// to expose them independently from the full policy transition event.
pub fn policy_invalidations(transition: &ReasoningPolicyTransition) -> &[PolicyInvalidation] {
    &transition.invalidations
}
