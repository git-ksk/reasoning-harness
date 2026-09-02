#![cfg_attr(test, allow(clippy::field_reassign_with_default))]

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
pub mod evidence_qualification;
pub mod evidence_qualification_benchmark;
pub mod finalization;
pub mod format_invariance;
pub mod frameworks;
pub mod generation;
pub mod harness;
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
    GENERIC_ANSWER_SUFFICIENCY_REQUIREMENT_POLICY_ID, build_answer_sufficiency_request,
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
pub use finalization::{
    CanonicalFinalAnswerRenderer, FinalAnswerCandidate, FinalAnswerClaim, FinalAnswerRenderer,
    FinalClaimMode, FinalizationPolicy, FinalizationResult, FinalizationStatus,
    canonical_verified_target_answer, canonical_verified_target_partial_answer,
    final_answer_candidate_schema, finalize_answer,
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
pub use metamorphic::{
    AddIrrelevantEvidence, MetamorphicAggregate, MetamorphicCaseResult, MetamorphicEvaluationError,
    MetamorphicTransform, MetamorphicTransformFamily, ReverseCausalCauseOrder,
    ReverseCausalEvidenceOrder, ReverseEvidenceOrder, ReverseInferenceOrder, StableIdRemap,
    aggregate_metamorphic, evaluate_benchmark_metamorphic, evaluate_causal_metamorphic,
};
pub use model::{
    ModelAdapter, ModelError, ModelErrorKind, ModelOutputFormat, ModelReasoningPreference,
    ModelRequest, ModelResponse, ModelUsage,
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
    policy_invalidations, replay_thread, validate_thread,
};
pub use resolution::{
    AcquiredEvidence, DefaultResolutionPlanner, EvidenceAdmissionPolicy,
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
