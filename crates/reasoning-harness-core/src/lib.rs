pub mod benchmark;
pub mod candidate;
pub mod decision;
pub mod eval;
pub mod frameworks;
pub mod generation;
pub mod harness;
pub mod model;
pub mod schema;
pub mod types;
pub mod validate;
pub mod verification;

pub use benchmark::{
    BenchmarkAggregate, BenchmarkArmResult, BenchmarkCaseResult, BenchmarkComparison,
    BenchmarkFixture, aggregate_benchmark, evaluate_benchmark_fixture,
};
pub use candidate::materialize_candidate;
pub use decision::{AcceptancePolicy, StrictAcceptancePolicy};
pub use eval::{EvalMetrics, evaluate};
pub use generation::{build_candidate_json_fallback_request, build_candidate_request};
pub use harness::{HarnessError, HarnessOutcome, Pass, run_harness, run_passes};
pub use model::{
    ModelAdapter, ModelError, ModelErrorKind, ModelOutputFormat, ModelRequest, ModelResponse,
    ModelUsage,
};
pub use schema::{reasoning_artifact_schema, reasoning_candidate_schema};
pub use types::{
    CandidateClaim, Claim, EpistemicState, Evidence, HarnessInput, Inference, ReasoningArtifact,
    ReasoningCandidate, Verdict, VerificationConclusion, VerificationReceipt,
};
pub use validate::{Diagnostic, ValidationReport, validate_artifact};
pub use verification::TrustedVerificationPass;
