#![cfg_attr(test, allow(clippy::field_reassign_with_default))]

pub mod adversarial;
pub mod benchmark;
pub mod candidate;
pub mod causal;
pub mod causal_benchmark;
pub mod decision;
pub mod diagnostic_stability;
pub mod eval;
pub mod frameworks;
pub mod generation;
pub mod harness;
pub mod metamorphic;
pub mod model;
pub mod schema;
pub mod types;
pub mod validate;
pub mod verification;

pub use adversarial::{
    AdversarialDetector, AdversarialDiscoveryPass, StructuredFactConflictDetector,
    record_soft_finding,
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
pub use decision::{AcceptancePolicy, StrictAcceptancePolicy};
pub use diagnostic_stability::{
    ConfidenceIntervalMethod, DiagnosticCountDistribution, DiagnosticFamilyDistributions,
    DiagnosticFrequency, DiagnosticObservation, DiagnosticSignal, DiagnosticStabilityError,
    DiagnosticTrial, FixtureDiagnosticStability, ProportionConfidenceInterval,
    RepeatedDiagnosticReport, aggregate_repeated_diagnostics, observe_diagnostics, wilson_95,
};
pub use eval::{EvalMetrics, evaluate};
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
pub use schema::{reasoning_artifact_schema, reasoning_candidate_schema};
pub use types::{
    AdversarialFinding, AdversarialFindingKind, CandidateClaim, CandidateDiagnostic, Claim,
    EpistemicState, Evidence, FindingStrength, HarnessInput, Inference, Proposition,
    ReasoningArtifact, ReasoningCandidate, Verdict, VerificationConclusion, VerificationReceipt,
};
pub use validate::{Diagnostic, ValidationReport, validate_artifact};
pub use verification::{
    StructuredFactVerifier, TrustedVerificationPass, VerificationPass, Verifier,
};
