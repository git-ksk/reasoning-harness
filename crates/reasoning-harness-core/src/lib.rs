#![cfg_attr(test, allow(clippy::field_reassign_with_default))]

pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod adversarial;
pub mod answer_safety;
pub mod assumption;
pub mod assumption_benchmark;
pub mod benchmark;
pub mod candidate;
pub mod causal;
pub mod causal_benchmark;
pub mod corpus;
pub mod decision;
pub mod diagnostic_stability;
pub mod eval;
pub mod evidence_need;
pub mod evidence_qualification;
pub mod evidence_qualification_benchmark;
pub mod evidence_relevance;
pub mod finalization;
pub mod format_invariance;
pub mod frameworks;
pub mod generation;
pub mod harness;
pub mod investigation;
pub mod metamorphic;
pub mod model;
pub mod reasoning_policy;
pub mod reasoning_thread;
pub mod resolution;
pub mod resolution_benchmark;
pub mod schema;
pub mod semantic_decidability;
pub mod semantic_judge;
pub mod semantic_materialization;
pub mod semantic_runtime;
pub mod semantic_stability;
pub mod semantic_sufficiency;
pub mod types;
pub mod validate;
pub mod verification;

pub use adversarial::{
    AdversarialDetector, AdversarialDiscoveryPass, StructuredFactConflictDetector,
    record_soft_finding,
};
pub use answer_safety::{
    ANSWER_SAFETY_IDENTITY_VERSION, AnswerSafetyDisposition, AnswerSafetyError,
    AnswerSafetyIdentity, AnswerSafetyObservation, AnswerSafetyProfile, AnswerSafetyReason,
    BASELINE_ANSWER_SAFETY_CONFIGURATION_ID, CLAIM_LOCAL_ANSWER_SUFFICIENCY_REQUIREMENT_POLICY_ID,
    D3_SUFFICIENCY_ANSWER_SAFETY_CONFIGURATION_ID,
    D3_SUFFICIENCY_V2_ANSWER_SAFETY_CONFIGURATION_ID, EVIDENCE_SUFFICIENCY_RSD1_CONTRACT_ID,
    GENERIC_ANSWER_SUFFICIENCY_REQUIREMENT_POLICY_ID,
    VERIFIED_TARGET_ANSWER_SAFETY_CONFIGURATION_ID, build_answer_sufficiency_request,
    build_answer_sufficiency_request_for_profile, run_answer_safety_gate,
};
pub use assumption::{
    AssumptionAssessment, AssumptionDiscoveryPass, AssumptionFinding, AssumptionFindingKind,
    AssumptionInspection, AssumptionInspector, AssumptionSupportStatus,
};
pub use assumption_benchmark::{
    AssumptionBenchmarkAggregate, AssumptionBenchmarkCaseResult, AssumptionBenchmarkFixture,
    aggregate_assumption_benchmark, evaluate_assumption_fixture,
};
pub use benchmark::{
    BenchmarkAggregate, BenchmarkArmResult, BenchmarkCaseResult, BenchmarkComparison,
    BenchmarkEvaluation, BenchmarkFixture, aggregate_benchmark, evaluate_benchmark_fixture,
    evaluate_benchmark_fixture_with_diagnostics,
};
pub use candidate::materialize_candidate;
pub use causal::{
    CausalEdgeAssessment, CausalEvidence, CausalEvidenceConclusion, CausalFinding,
    CausalFindingKind, CausalFindingReason, CausalInputError, CausalInspection, CausalInspector,
    CausalRelation, CausalSupportStatus,
};
pub use causal_benchmark::{
    CausalBenchmarkAggregate, CausalBenchmarkCaseResult, CausalBenchmarkFixture,
    aggregate_causal_benchmark, evaluate_causal_fixture,
};
pub use corpus::{
    ClaimCorpusSummary, CorpusCaseMetadata, CorpusCaseStatus, CorpusDifficulty, CorpusError,
    CorpusManifest, CorpusRedistribution, CorpusScoringMode, CorpusSliceComparison, CorpusSuite,
    aggregate_claim_corpus, validate_corpus_manifest,
};
pub use decision::{AcceptancePolicy, StrictAcceptancePolicy};
pub use diagnostic_stability::{
    ConfidenceIntervalMethod, DiagnosticCountDistribution, DiagnosticFamilyDistributions,
    DiagnosticFrequency, DiagnosticObservation, DiagnosticSignal, DiagnosticStabilityError,
    DiagnosticTrial, FixtureDiagnosticStability, ProportionConfidenceInterval,
    RepeatedDiagnosticReport, aggregate_repeated_diagnostics, observe_diagnostics, wilson_95,
};
pub use eval::{EvalMetrics, evaluate};
pub use evidence_need::{
    ContextSufficiency, EVIDENCE_NEED_MATERIALIZATION_POLICY_ID,
    EVIDENCE_NEED_PROPOSAL_CONTRACT_ID, EvidenceAcquisitionDisposition, EvidenceNeedDecision,
    EvidenceNeedError, EvidenceNeedMaterializationReason, EvidenceNeedMode, EvidenceNeedProposal,
    EvidenceNeedTargetKind, EvidenceNeedTargetPolicy, ExistingEvidenceReuseStatus,
    SuppliedContextState, build_evidence_need_proposal_request, evidence_need_proposal_schema,
    materialize_evidence_need, materialize_evidence_needs, parse_evidence_need_proposal,
};
pub use evidence_qualification::{
    EvidenceQualificationAssessment, EvidenceQualificationFinding,
    EvidenceQualificationFindingKind, EvidenceQualificationFindingReason,
    EvidenceQualificationInspection, EvidenceQualificationInspector, EvidenceQualificationPass,
    EvidenceQualificationStatus,
};
pub use evidence_qualification_benchmark::{
    EvidenceQualificationBenchmarkAggregate, EvidenceQualificationBenchmarkCaseResult,
    EvidenceQualificationBenchmarkFixture, aggregate_evidence_qualification_benchmark,
    evaluate_evidence_qualification_fixture,
};
pub use evidence_relevance::{
    EVIDENCE_RELEVANCE_BINDING_MATERIALIZATION_POLICY_ID,
    EVIDENCE_RELEVANCE_BINDING_MATERIALIZATION_POLICY_V3_ID,
    EVIDENCE_RELEVANCE_BINDING_MATERIALIZATION_POLICY_V4_ID,
    EVIDENCE_RELEVANCE_BINDING_MATERIALIZATION_POLICY_V5_ID,
    EVIDENCE_RELEVANCE_BINDING_MATERIALIZATION_POLICY_V6_ID,
    EVIDENCE_RELEVANCE_BINDING_MATERIALIZATION_POLICY_V7_ID,
    EVIDENCE_RELEVANCE_BINDING_MATERIALIZATION_POLICY_V8_ID,
    EVIDENCE_RELEVANCE_BINDING_MATERIALIZATION_POLICY_V9_ID,
    EVIDENCE_RELEVANCE_BINDING_MATERIALIZATION_POLICY_V10_ID,
    EVIDENCE_RELEVANCE_BINDING_MATERIALIZATION_POLICY_V11_ID,
    EVIDENCE_RELEVANCE_BINDING_PROPOSAL_CONTRACT_ID,
    EVIDENCE_RELEVANCE_BINDING_PROPOSAL_V3_CONTRACT_ID,
    EVIDENCE_RELEVANCE_BINDING_PROPOSAL_V4_CONTRACT_ID,
    EVIDENCE_RELEVANCE_BINDING_PROPOSAL_V5_CONTRACT_ID,
    EVIDENCE_RELEVANCE_LOCAL_QUALIFICATION_CONTRACT_ID,
    EVIDENCE_RELEVANCE_LOCAL_QUALIFICATION_V2_CONTRACT_ID,
    EVIDENCE_RELEVANCE_LOCAL_QUALIFICATION_V3_CONTRACT_ID,
    EVIDENCE_RELEVANCE_LOCAL_QUALIFICATION_V4_CONTRACT_ID,
    EVIDENCE_RELEVANCE_LOCAL_QUALIFICATION_V5_CONTRACT_ID,
    EVIDENCE_RELEVANCE_LOCAL_QUALIFICATION_V6_CONTRACT_ID,
    EVIDENCE_RELEVANCE_MATERIALIZATION_POLICY_ID,
    EVIDENCE_RELEVANCE_NEGATIVE_SAFETY_DECISION_CONTRACT_ID,
    EVIDENCE_RELEVANCE_NEGATIVE_TARGET_CONFIRMATION_CONTRACT_ID,
    EVIDENCE_RELEVANCE_NEGATIVE_TARGET_CONFIRMATION_V2_CONTRACT_ID,
    EVIDENCE_RELEVANCE_POSITIVE_SAFETY_DECISION_CONTRACT_ID,
    EVIDENCE_RELEVANCE_POSITIVE_TARGET_CONFIRMATION_CONTRACT_ID,
    EVIDENCE_RELEVANCE_PROPOSAL_CONTRACT_ID, EvidenceBlockingCue, EvidenceExplicitLocalAbsence,
    EvidenceLocalBindingConfirmation, EvidenceLocalBlockingReason, EvidenceLocalIdentityScope,
    EvidenceLocalQualification, EvidenceLocalQualificationV3, EvidenceLocalQualificationV4,
    EvidenceLocalQualificationV5, EvidenceLocalQualificationV6, EvidenceLocalRelationScope,
    EvidenceLocalSupport, EvidenceNegativeSafetyDecision, EvidenceNegativeSafetyDecisionProposal,
    EvidenceNegativeTargetConfirmation, EvidenceNegativeTargetConfirmationProposal,
    EvidenceNegativeTargetConfirmationV2, EvidencePositiveSafetyDecision,
    EvidencePositiveSafetyDecisionProposal, EvidencePositiveTargetConfirmation,
    EvidenceQualificationRisk, EvidenceRelevanceAssessment, EvidenceRelevanceAssessmentBudget,
    EvidenceRelevanceAssessmentPath, EvidenceRelevanceBinding, EvidenceRelevanceBindingProposal,
    EvidenceRelevanceCandidate, EvidenceRelevanceDisposition, EvidenceRelevanceError,
    EvidenceRelevanceIdentityRequirement, EvidenceRelevanceProposal, EvidenceRelevanceReason,
    EvidenceRelevanceRelationKind, EvidenceRelevanceSignal, EvidenceRelevanceSignalKind,
    EvidenceRelevanceTargetPolicy, EvidenceTargetEntityIdentity,
    build_evidence_local_qualification_request, build_evidence_local_qualification_v3_request,
    build_evidence_local_qualification_v4_request, build_evidence_local_qualification_v5_request,
    build_evidence_local_qualification_v6_request, build_evidence_negative_safety_decision_request,
    build_evidence_negative_target_confirmation_request,
    build_evidence_negative_target_confirmation_v2_request,
    build_evidence_positive_safety_decision_request,
    build_evidence_positive_target_confirmation_request,
    build_evidence_relevance_binding_proposal_request,
    build_evidence_relevance_binding_proposal_v3_request,
    build_evidence_relevance_binding_proposal_v4_request,
    build_evidence_relevance_binding_proposal_v5_request,
    build_evidence_relevance_proposal_request, evidence_local_qualification_schema,
    evidence_local_qualification_v3_schema, evidence_local_qualification_v4_schema,
    evidence_local_qualification_v5_schema, evidence_local_qualification_v6_schema,
    evidence_negative_safety_decision_schema, evidence_negative_target_confirmation_schema,
    evidence_positive_safety_decision_schema, evidence_relevance_binding_proposal_schema,
    evidence_relevance_proposal_schema, materialize_evidence_relevance,
    materialize_evidence_relevance_v2, materialize_evidence_relevance_v3,
    materialize_evidence_relevance_v4, materialize_evidence_relevance_v5,
    materialize_evidence_relevance_v6, materialize_evidence_relevance_v7,
    materialize_evidence_relevance_v8, materialize_evidence_relevance_v9,
    materialize_evidence_relevance_v10, materialize_evidence_relevance_v11,
    parse_evidence_local_qualification, parse_evidence_local_qualification_v3,
    parse_evidence_local_qualification_v4, parse_evidence_local_qualification_v5,
    parse_evidence_local_qualification_v6, parse_evidence_negative_safety_decision,
    parse_evidence_negative_target_confirmation, parse_evidence_negative_target_confirmation_v2,
    parse_evidence_positive_safety_decision, parse_evidence_positive_target_confirmation,
    parse_evidence_relevance_binding_proposal, parse_evidence_relevance_proposal,
};
pub use finalization::{
    CanonicalFinalAnswerRenderer, FinalAnswerCandidate, FinalAnswerClaim, FinalAnswerRenderer,
    FinalClaimMode, FinalizationPolicy, FinalizationResult, FinalizationStatus,
    canonical_verified_target_answer, canonical_verified_target_partial_answer,
    canonical_verified_target_reject_partial_answer, final_answer_candidate_schema,
    finalize_answer, recover_verified_target_renderer_downgrade,
};
pub use format_invariance::{
    FormatComparisonError, FormatDecisionTransition, FormatFlipReport, FormatJudgeError,
    FormatJudgeObservation, MatchedFormatDecision, SoftJudgeRepresentation,
    build_soft_judge_representation_request, compare_soft_judge_formats,
    parse_soft_judge_representation_decision, run_model_backed_soft_judge_representation,
};
pub use generation::{
    build_candidate_json_fallback_request, build_candidate_request,
    build_final_answer_json_fallback_request, build_final_answer_request,
    parse_final_answer_candidate,
};
pub use harness::{HarnessError, HarnessOutcome, Pass, run_harness, run_passes};
pub use investigation::{
    INVESTIGATION_ACTION_CONTRACT_ID, INVESTIGATION_INTENT_CONTRACT_ID,
    INVESTIGATION_MATERIALIZATION_POLICY_ID, INVESTIGATION_PLAN_CONTRACT_ID,
    INVESTIGATION_RUNTIME_ID, InvestigationAction, InvestigationActionKind,
    InvestigationActionProposal, InvestigationActionRecord, InvestigationActionRejection,
    InvestigationCapability, InvestigationIntentKind, InvestigationIntentProposal,
    InvestigationIntentRejection, InvestigationIntentRejectionRecord,
    InvestigationObservationStatus, InvestigationPlanProposal, InvestigationPolicy,
    InvestigationState, InvestigationStopReason, InvestigationTarget, InvestigationTargetOrigin,
    InvestigationTargetProposal, InvestigationTelemetry, admit_investigation_plan,
    build_investigation_action_request, build_investigation_intent_request,
    build_investigation_plan_request, investigation_action_schema, investigation_intent_schema,
    investigation_plan_schema, parse_investigation_action, parse_investigation_intent,
    parse_investigation_plan,
};
pub use metamorphic::{
    AddIrrelevantEvidence, MetamorphicAggregate, MetamorphicCaseResult, MetamorphicEvaluationError,
    MetamorphicTransform, MetamorphicTransformFamily, ReverseCausalCauseOrder,
    ReverseCausalEvidenceOrder, ReverseEvidenceOrder, ReverseInferenceOrder, StableIdRemap,
    aggregate_metamorphic, evaluate_benchmark_metamorphic, evaluate_causal_metamorphic,
};
pub use model::{
    ModelAdapter, ModelError, ModelErrorKind, ModelExecutionBudget,
    ModelExecutionTelemetrySnapshot, ModelOutputFormat, ModelReasoningPreference, ModelRequest,
    ModelResponse, ModelUsage, build_json_object_fallback_request,
    build_strict_json_text_fallback_request,
};
pub use reasoning_policy::{
    PolicyEscalation, PolicyEscalationAction, PolicyEvidenceConstraints, PolicyInvalidation,
    PolicyInvalidationReason, PolicyInvalidationTarget, ReasoningPolicy, ReasoningPolicyError,
    ReasoningPolicyLayer, ReasoningPolicyTransition, SoftFindingEscalation, apply_reasoning_policy,
    compose_reasoning_policy, constrain_resolution_policy, escalation_for_soft_observation,
};
pub use reasoning_thread::{
    REASONING_THREAD_SCHEMA_VERSION, ReasoningCheckpoint, ReasoningThread, ReasoningThreadError,
    ReasoningThreadEvent, ReasoningThreadEventKind, ReasoningThreadLineage, ReasoningThreadReplay,
    ReasoningThreadSnapshot, ReasoningThreadStatus, ReasoningThreadStore, ThreadCandidateState,
    ThreadInputChange, policy_invalidations, replay_thread, validate_thread,
};
pub use resolution::{
    AcquiredEvidence, AcquiredEvidenceMetadata, DefaultResolutionPlanner, EvidenceAdmissionPolicy,
    EvidenceAdmissionRejection, FinalizationPolicyConfig, GroundedResolutionOutcome,
    GroundedResolutionPolicy, GroundedResolutionRuntime, GroundingPipeline,
    RejectAllEvidenceAdmission, ResolutionAdapterError, ResolutionAdapterErrorKind,
    ResolutionAttempt, ResolutionAttemptStatus, ResolutionBudget, ResolutionCost, ResolutionError,
    ResolutionPlanner, ResolutionReason, ResolutionRequest, ResolutionRequestBudget,
    ResolutionResolver, ResolutionResolverContribution, ResolutionResolverOutput, ResolutionTarget,
    ResolutionTerminalStatus, ResolutionUsage, ResolverClass, StandardGroundingPipeline,
    TrustedResolutionVerifier, TrustedVerifierResolutionOutput,
    default_grounded_resolution_runtime,
};
pub use resolution_benchmark::{
    ResolutionBenchmarkAggregate, ResolutionBenchmarkCaseResult, ResolutionBenchmarkFixture,
    ResolutionFixtureStep, ResolutionFixtureStepResult, aggregate_resolution_benchmark,
    evaluate_resolution_fixture,
};
pub use schema::{
    REASONING_ARTIFACT_CONTRACT_ID, REASONING_CANDIDATE_CONTRACT_ID, reasoning_artifact_schema,
    reasoning_candidate_schema, soft_judge_output_schema,
};
pub use semantic_decidability::{
    SemanticDecidabilityAssessment, SemanticDecidabilityCalibrationFixture,
    SemanticDecidabilityDisposition, SemanticDecidabilityError, SemanticDecidabilityReason,
    SemanticDecidabilityStudyFixture, SemanticDecidabilityStudyVariant,
    assess_semantic_decidability, compose_semantic_decidability,
};
pub use semantic_judge::{
    CalibrationLabel, ModelBackedSoftJudge, ModelBackedSoftJudgeError,
    ModelBackedSoftJudgeObservation, SemanticDiagnosticKind, SemanticDiagnosticTarget,
    SoftDiagnosticJudge, SoftJudgeAgreement, SoftJudgeCalibrationError,
    SoftJudgeCalibrationFixture, SoftJudgeCalibrationReport, SoftJudgeDecision, SoftJudgeError,
    SoftJudgeFallbackReason, SoftJudgeIdentity, SoftJudgeMetrics, SoftJudgeObservation,
    SoftJudgeOutput, SoftJudgeRequest, SoftSemanticFinding, aggregate_soft_judge_calibration,
    build_soft_judge_json_fallback_request, build_soft_judge_model_request,
    parse_soft_judge_output, run_model_backed_soft_judge, run_soft_judge,
    validate_calibration_fixtures,
};
pub use semantic_materialization::{
    MaterializationCapabilityPreflight, MaterializationError, MaterializationFailureClass,
    MaterializationObservation, MaterializationRepresentation, MaterializedDecisionOutput,
    R2_MATERIALIZATION_CAPABILITY_ID, build_soft_judge_materialization_representation_request,
    build_soft_judge_materialization_request, classify_materialization_failure,
    materialize_soft_judge_output, parse_materialized_decision_output,
    parse_materialized_decision_representation_output, run_materialization_capability_preflight,
    run_model_backed_soft_judge_materialization,
    run_model_backed_soft_judge_materialization_representation,
};
pub use semantic_runtime::{
    D3_DECIDABILITY_CONTRACT_ID, DEFAULT_SEMANTIC_RUNTIME_PROFILE, MATERIALIZATION_R2_CONTRACT_ID,
    SEMANTIC_DECIDABILITY_D3_CONFIGURATION_ID, SEMANTIC_RUNTIME_IDENTITY_VERSION,
    SOFT_SEMANTIC_V3_CONFIGURATION_ID, SemanticRuntimeError, SemanticRuntimeIdentity,
    SemanticRuntimeObservation, SemanticRuntimeProfile, default_semantic_runtime_profile,
    run_default_semantic_runtime, run_semantic_runtime,
};
pub use semantic_stability::{
    SelectiveAbstentionOutcome, SelectiveAbstentionPolicy, SoftDecisionProbe,
    SoftDecisionStabilityAssessment, StabilityRiskSignal, apply_selective_abstention,
    assess_soft_decision_stability,
};
pub use semantic_sufficiency::{
    EvidenceSufficiencyCalibrationFixture, EvidenceSufficiencyFallbackReason,
    EvidenceSufficiencyFixtureError, EvidenceSufficiencyLabel, EvidenceSufficiencyModelError,
    EvidenceSufficiencyModelOutput, EvidenceSufficiencyObservation, EvidenceSufficiencyRequest,
    build_evidence_sufficiency_json_fallback_request, build_evidence_sufficiency_model_request,
    evidence_sufficiency_output_schema, parse_evidence_sufficiency_output,
    run_model_backed_evidence_sufficiency, validate_evidence_sufficiency_fixture,
};
pub use types::{
    AdversarialFinding, AdversarialFindingKind, ApplicabilityScope, CandidateClaim,
    CandidateDiagnostic, Claim, EpistemicState, Evidence, EvidenceAuthorityPolicy,
    EvidenceMetadata, EvidenceRequirement, FindingStrength, HarnessInput, Inference, Proposition,
    ReasoningArtifact, ReasoningCandidate, ScopeCoverage, TemporalValidity, Verdict,
    VerificationConclusion, VerificationReceipt,
};
pub use validate::{Diagnostic, ValidationReport, validate_artifact};
pub use verification::{
    QualifiedStructuredFactVerifier, StructuredFactVerifier, TrustedVerificationPass,
    VerificationPass, Verifier, structured_fact_verifier_for_input,
};
