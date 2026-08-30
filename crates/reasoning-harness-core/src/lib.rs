#![cfg_attr(test, allow(clippy::field_reassign_with_default))]

pub mod adversarial;
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
pub mod frameworks;
pub mod generation;
pub mod harness;
pub mod metamorphic;
pub mod model;
pub mod resolution;
pub mod resolution_benchmark;
pub mod schema;
pub mod types;
pub mod validate;
pub mod verification;

pub use adversarial::{
    AdversarialDetector, AdversarialDiscoveryPass, StructuredFactConflictDetector,
    record_soft_finding,
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
    FinalClaimMode, FinalizationPolicy, FinalizationResult, FinalizationStatus, finalize_answer,
};
pub use generation::{build_candidate_json_fallback_request, build_candidate_request};
pub use harness::{HarnessError, HarnessOutcome, Pass, run_harness, run_passes};
pub use metamorphic::{
    AddIrrelevantEvidence, MetamorphicAggregate, MetamorphicCaseResult, MetamorphicEvaluationError,
    MetamorphicTransform, MetamorphicTransformFamily, ReverseCausalCauseOrder,
    ReverseCausalEvidenceOrder, ReverseEvidenceOrder, ReverseInferenceOrder, StableIdRemap,
    aggregate_metamorphic, evaluate_benchmark_metamorphic, evaluate_causal_metamorphic,
};
pub use model::{
    ModelAdapter, ModelError, ModelErrorKind, ModelOutputFormat, ModelRequest, ModelResponse,
    ModelUsage,
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
pub use schema::{reasoning_artifact_schema, reasoning_candidate_schema};
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
