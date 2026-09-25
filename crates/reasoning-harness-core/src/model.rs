use std::{future::Future, pin::Pin};

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ModelOutputFormat {
    Text,
    JsonObject,
    JsonSchema { name: String, schema: Value },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelReasoningPreference {
    Minimize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelRequest {
    pub task: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    pub output_format: ModelOutputFormat,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub random_seed: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_preference: Option<ModelReasoningPreference>,
}

/// Builds a bounded provider-neutral strict-JSON text fallback for a JSON-Schema request.
///
/// This is intentionally distinct from the JSON-object fallback: providers that reject or
/// repeatedly fail server-side structured generation can still receive the exact same task and
/// schema without enabling a provider JSON response mode. The caller must parse the entire
/// response against its typed contract; extraction, repair, and semantic retries remain forbidden.
pub fn build_strict_json_text_fallback_request(request: &ModelRequest) -> Option<ModelRequest> {
    let ModelOutputFormat::JsonSchema { schema, .. } = &request.output_format else {
        return None;
    };
    let schema = serde_json::to_string_pretty(schema)
        .expect("ModelOutputFormat::JsonSchema value must serialize");

    let mut fallback = request.clone();
    fallback.task = format!(
        "JSON Schema:
{schema}

Original task:
{}

Return exactly one raw JSON object conforming to the supplied JSON Schema. Do not add prose, Markdown fences, commentary, or fields not allowed by the schema.",
        request.task
    );
    fallback.system = Some(match request.system.as_deref() {
        Some(system) => format!(
            "{system}

Strict-JSON text fallback constraint: return exactly one raw JSON object and no prose. Preserve the original task semantics; do not invent missing fields, facts, evidence, identities, or authority."
        ),
        None => "Strict-JSON text fallback constraint: return exactly one raw JSON object and no prose. Preserve the original task semantics; do not invent missing fields, facts, evidence, identities, or authority.".into(),
    });
    fallback.output_format = ModelOutputFormat::Text;
    Some(fallback)
}

/// Builds a bounded provider-neutral fallback for a JSON-Schema request.
///
/// The fallback changes only the structured-output transport contract:
/// - original task/system/budgets/seed/reasoning preference are preserved,
/// - the same JSON Schema is embedded verbatim in the model-visible task,
/// - output mode is relaxed from JsonSchema to JsonObject,
/// - no model output is repaired, extracted, or semantically completed.
///
/// Callers must still parse and validate the fallback response against their typed contract and fail
/// closed if it remains malformed.
pub fn build_json_object_fallback_request(request: &ModelRequest) -> Option<ModelRequest> {
    let ModelOutputFormat::JsonSchema { schema, .. } = &request.output_format else {
        return None;
    };
    let schema = serde_json::to_string_pretty(schema)
        .expect("ModelOutputFormat::JsonSchema value must serialize");

    let mut fallback = request.clone();
    fallback.task = format!(
        "JSON Schema:
{schema}

Original task:
{}

Return exactly one JSON object conforming to the supplied JSON Schema. Do not add prose, Markdown fences, commentary, or fields not allowed by the schema.",
        request.task
    );
    fallback.system = Some(match request.system.as_deref() {
        Some(system) => format!(
            "{system}

Structured-output fallback constraint: return exactly one JSON object and no prose. Preserve the original task semantics; do not invent missing fields, facts, evidence, identities, or authority."
        ),
        None => "Structured-output fallback constraint: return exactly one JSON object and no prose. Preserve the original task semantics; do not invent missing fields, facts, evidence, identities, or authority.".into(),
    });
    fallback.output_format = ModelOutputFormat::JsonObject;
    Some(fallback)
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelUsage {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelResponse {
    pub text: String,
    pub model: String,
    pub usage: ModelUsage,
    /// Number of actual provider HTTP attempts consumed to produce this response, including bounded
    /// adapter-internal retries. Structured-output fallback calls are separate ModelAdapter calls and
    /// their attempt counts must be summed by the caller.
    #[serde(default = "default_provider_attempts")]
    pub provider_attempts: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

fn default_provider_attempts() -> u32 {
    1
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelErrorKind {
    Credentials,
    Transport,
    Provider,
    RateLimit,
    Quota,
    ProviderUnavailable,
    Timeout,
    Protocol,
    UnsupportedCapability,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelError {
    pub kind: ModelErrorKind,
    pub message: String,
    /// Number of actual provider attempts consumed before this terminal failure.
    pub provider_attempts: u32,
}

impl ModelError {
    pub fn new(kind: ModelErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            provider_attempts: 1,
        }
    }

    pub fn with_provider_attempts(mut self, provider_attempts: u32) -> Self {
        self.provider_attempts = provider_attempts.max(1);
        self
    }
}

impl std::fmt::Display for ModelError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

impl std::error::Error for ModelError {}

pub trait ModelAdapter: Send + Sync {
    fn generate<'a>(
        &'a self,
        request: ModelRequest,
    ) -> Pin<Box<dyn Future<Output = Result<ModelResponse, ModelError>> + Send + 'a>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_object_fallback_preserves_request_semantics_and_budgets() {
        let request = ModelRequest {
            task: "decide the bounded target".into(),
            system: Some("server-owned policy".into()),
            output_format: ModelOutputFormat::JsonSchema {
                name: "contract_v1".into(),
                schema: serde_json::json!({
                    "type": "object",
                    "additionalProperties": false,
                    "properties": { "decision": { "type": "string" } },
                    "required": ["decision"]
                }),
            },
            max_tokens: Some(321),
            random_seed: Some(7),
            reasoning_preference: Some(ModelReasoningPreference::Minimize),
        };

        let fallback = build_json_object_fallback_request(&request).expect("schema fallback");

        assert_eq!(fallback.output_format, ModelOutputFormat::JsonObject);
        assert_eq!(fallback.max_tokens, request.max_tokens);
        assert_eq!(fallback.random_seed, request.random_seed);
        assert_eq!(fallback.reasoning_preference, request.reasoning_preference);
        assert!(
            fallback
                .task
                .contains("Original task:\ndecide the bounded target")
        );
        assert!(fallback.task.contains("\"additionalProperties\": false"));
        assert!(fallback.task.contains("exactly one JSON object"));
        assert!(
            fallback
                .system
                .as_deref()
                .unwrap()
                .contains("server-owned policy")
        );
        assert!(
            fallback
                .system
                .as_deref()
                .unwrap()
                .contains("do not invent missing fields")
        );
    }

    #[test]
    fn json_object_fallback_is_only_available_for_json_schema_requests() {
        let request = ModelRequest {
            task: "plain".into(),
            system: None,
            output_format: ModelOutputFormat::JsonObject,
            max_tokens: None,
            random_seed: None,
            reasoning_preference: None,
        };
        assert!(build_json_object_fallback_request(&request).is_none());
    }

    #[test]
    fn strict_json_text_fallback_preserves_schema_semantics_without_provider_json_mode() {
        let request = ModelRequest {
            task: "qualify the local evidence".into(),
            system: Some("server-owned guard".into()),
            output_format: ModelOutputFormat::JsonSchema {
                name: "qualification_v2".into(),
                schema: serde_json::json!({
                    "type": "object",
                    "additionalProperties": false,
                    "properties": { "risk": { "type": "string", "enum": ["absent", "present"] } },
                    "required": ["risk"]
                }),
            },
            max_tokens: Some(192),
            random_seed: Some(11),
            reasoning_preference: Some(ModelReasoningPreference::Minimize),
        };

        let fallback = build_strict_json_text_fallback_request(&request).expect("text fallback");

        assert_eq!(fallback.output_format, ModelOutputFormat::Text);
        assert_eq!(fallback.max_tokens, request.max_tokens);
        assert_eq!(fallback.random_seed, request.random_seed);
        assert_eq!(fallback.reasoning_preference, request.reasoning_preference);
        assert!(
            fallback
                .task
                .contains("Original task:\nqualify the local evidence")
        );
        assert!(fallback.task.contains("exactly one raw JSON object"));
        assert!(
            fallback
                .system
                .as_deref()
                .unwrap()
                .contains("Strict-JSON text fallback constraint")
        );
    }

    #[test]
    fn strict_json_text_fallback_is_only_available_for_json_schema_requests() {
        let request = ModelRequest {
            task: "plain".into(),
            system: None,
            output_format: ModelOutputFormat::Text,
            max_tokens: None,
            random_seed: None,
            reasoning_preference: None,
        };
        assert!(build_strict_json_text_fallback_request(&request).is_none());
    }
}
