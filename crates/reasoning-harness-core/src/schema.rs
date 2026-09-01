use serde_json::Value;

use crate::{ReasoningArtifact, ReasoningCandidate, SoftJudgeOutput};

pub const REASONING_ARTIFACT_CONTRACT_ID: &str = "reasoning-artifact-v1";
pub const REASONING_CANDIDATE_CONTRACT_ID: &str = "reasoning-candidate-v1";

pub fn reasoning_artifact_schema() -> Value {
    serialize_schema(schemars::schema_for!(ReasoningArtifact))
}

pub fn reasoning_candidate_schema() -> Value {
    serialize_schema(schemars::schema_for!(ReasoningCandidate))
}

pub fn soft_judge_output_schema() -> Value {
    serialize_schema(schemars::schema_for!(SoftJudgeOutput))
}

fn serialize_schema(schema: schemars::Schema) -> Value {
    serde_json::to_value(schema).expect("reasoning JSON Schema must be serializable")
}
