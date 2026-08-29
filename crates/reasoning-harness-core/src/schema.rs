use serde_json::Value;

use crate::{ReasoningArtifact, ReasoningCandidate};

pub fn reasoning_artifact_schema() -> Value {
    serialize_schema(schemars::schema_for!(ReasoningArtifact))
}

pub fn reasoning_candidate_schema() -> Value {
    serialize_schema(schemars::schema_for!(ReasoningCandidate))
}

fn serialize_schema(schema: schemars::Schema) -> Value {
    serde_json::to_value(schema).expect("reasoning JSON Schema must be serializable")
}
