use std::{env, time::Duration};

use reasoning_harness_core::{
    ModelAdapter, ModelError, ModelErrorKind, ModelOutputFormat, ModelRequest, ModelResponse,
    ModelUsage,
};
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const DEFAULT_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta/";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);

/// Google Gemini API / AI Studio adapter for Gemma models.
///
/// This adapter is intentionally limited to untrusted candidate generation. It never
/// participates in harness verification or verdict authority.
pub struct GemmaAdapter {
    client: Client,
    api_key: String,
    base_url: Url,
    model: String,
}

impl GemmaAdapter {
    pub fn from_env(model: impl Into<String>) -> Result<Self, ModelError> {
        let api_key = env::var("GEMINI_API_KEY").map_err(|_| {
            ModelError::new(ModelErrorKind::Credentials, "GEMINI_API_KEY is not set")
        })?;
        Self::new(api_key, model)
    }

    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Result<Self, ModelError> {
        Self::with_base_url(api_key, model, DEFAULT_BASE_URL)
    }

    pub fn with_base_url(
        api_key: impl Into<String>,
        model: impl Into<String>,
        base_url: &str,
    ) -> Result<Self, ModelError> {
        let api_key = api_key.into();
        let model = model.into();
        if api_key.trim().is_empty() {
            return Err(ModelError::new(
                ModelErrorKind::Credentials,
                "Gemini API key must not be empty",
            ));
        }
        if model.trim().is_empty() {
            return Err(ModelError::new(
                ModelErrorKind::Protocol,
                "Gemma model identifier must not be empty",
            ));
        }

        let base_url = Url::parse(base_url).map_err(|error| {
            ModelError::new(
                ModelErrorKind::Protocol,
                format!("invalid Gemini API base URL: {error}"),
            )
        })?;
        let client = Client::builder()
            .timeout(DEFAULT_TIMEOUT)
            .build()
            .map_err(|error| {
                ModelError::new(
                    ModelErrorKind::Transport,
                    format!("failed to build HTTP client: {error}"),
                )
            })?;

        Ok(Self {
            client,
            api_key,
            base_url,
            model,
        })
    }

    async fn generate_inner(&self, request: ModelRequest) -> Result<ModelResponse, ModelError> {
        let endpoint = self.base_url.join("interactions").map_err(|error| {
            ModelError::new(
                ModelErrorKind::Protocol,
                format!("failed to construct Gemini Interactions endpoint: {error}"),
            )
        })?;

        let body = InteractionRequest {
            model: &self.model,
            input: request.task,
            system_instruction: request.system,
            response_format: response_format(request.output_format),
            generation_config: GenerationConfig {
                max_output_tokens: request.max_tokens,
                seed: request.random_seed,
                temperature: 0.0,
            },
            store: false,
        };

        let response = self
            .client
            .post(endpoint)
            .header("x-goog-api-key", &self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|error| {
                ModelError::new(
                    ModelErrorKind::Transport,
                    format!("Gemini API request failed: {error}"),
                )
            })?;

        let status = response.status();
        if !status.is_success() {
            return Err(ModelError::new(
                ModelErrorKind::Provider,
                format!("Gemini API returned HTTP {status}"),
            ));
        }

        let response: InteractionResponse = response.json().await.map_err(|error| {
            ModelError::new(
                ModelErrorKind::Protocol,
                format!("invalid Gemini Interactions response: {error}"),
            )
        })?;
        let text = response.text()?;
        let usage = response.usage.unwrap_or_default();

        Ok(ModelResponse {
            text,
            model: response.model.unwrap_or_else(|| self.model.clone()),
            usage: ModelUsage {
                input_tokens: usage.total_input_tokens,
                output_tokens: usage.total_output_tokens,
                total_tokens: usage.total_tokens,
            },
            finish_reason: response.status,
        })
    }
}

impl ModelAdapter for GemmaAdapter {
    fn generate<'a>(
        &'a self,
        request: ModelRequest,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<ModelResponse, ModelError>> + Send + 'a>,
    > {
        Box::pin(self.generate_inner(request))
    }
}

#[derive(Debug, Serialize)]
struct InteractionRequest<'a> {
    model: &'a str,
    input: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<String>,
    response_format: ResponseFormat,
    generation_config: GenerationConfig,
    store: bool,
}

#[derive(Debug, Serialize)]
struct GenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<u64>,
    temperature: f32,
}

#[derive(Debug, Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    kind: &'static str,
    mime_type: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    schema: Option<Value>,
}

fn response_format(format: ModelOutputFormat) -> ResponseFormat {
    match format {
        ModelOutputFormat::Text => ResponseFormat {
            kind: "text",
            mime_type: "text/plain",
            schema: None,
        },
        ModelOutputFormat::JsonObject => ResponseFormat {
            kind: "text",
            mime_type: "application/json",
            schema: Some(json!({"type": "object"})),
        },
        ModelOutputFormat::JsonSchema { schema, .. } => ResponseFormat {
            kind: "text",
            mime_type: "application/json",
            schema: Some(schema),
        },
    }
}

#[derive(Debug, Deserialize)]
struct InteractionResponse {
    model: Option<String>,
    status: Option<String>,
    #[serde(default)]
    steps: Vec<InteractionStep>,
    usage: Option<Usage>,
}

impl InteractionResponse {
    fn text(&self) -> Result<String, ModelError> {
        let text = self
            .steps
            .iter()
            .filter(|step| step.kind.as_deref() == Some("model_output"))
            .flat_map(|step| step.content.iter())
            .filter(|content| content.kind.as_deref() == Some("text"))
            .filter_map(|content| content.text.as_deref())
            .collect::<String>();
        if text.is_empty() {
            Err(ModelError::new(
                ModelErrorKind::Protocol,
                "Gemini Interactions response contained no model text output",
            ))
        } else {
            Ok(text)
        }
    }
}

#[derive(Debug, Deserialize)]
struct InteractionStep {
    #[serde(rename = "type")]
    kind: Option<String>,
    #[serde(default)]
    content: Vec<InteractionContent>,
}

#[derive(Debug, Deserialize)]
struct InteractionContent {
    #[serde(rename = "type")]
    kind: Option<String>,
    text: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct Usage {
    total_input_tokens: Option<u64>,
    total_output_tokens: Option<u64>,
    total_tokens: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_interactions_json_schema_without_authority_fields() {
        let value = serde_json::to_value(response_format(ModelOutputFormat::JsonSchema {
            name: "candidate".into(),
            schema: json!({"type": "object", "properties": {"claims": {"type": "array"}}}),
        }))
        .unwrap();

        assert_eq!(value["type"], "text");
        assert_eq!(value["mime_type"], "application/json");
        assert_eq!(value["schema"]["type"], "object");
        assert!(value.get("verdict").is_none());
    }

    #[test]
    fn parses_model_output_text_and_usage() {
        let response: InteractionResponse = serde_json::from_value(json!({
            "model": "gemma-4-26b-a4b-it",
            "status": "completed",
            "steps": [{
                "type": "model_output",
                "content": [{"type": "text", "text": "{\"claims\":[]}"}]
            }],
            "usage": {
                "total_input_tokens": 10,
                "total_output_tokens": 4,
                "total_tokens": 14
            }
        }))
        .unwrap();
        assert_eq!(response.text().unwrap(), "{\"claims\":[]}");
        assert_eq!(response.usage.unwrap().total_tokens, Some(14));
    }

    #[test]
    fn rejects_empty_credentials_without_echoing_them() {
        let error = GemmaAdapter::new("", "gemma-4-26b-a4b-it").err().unwrap();
        assert_eq!(error.kind, ModelErrorKind::Credentials);
        assert!(!error.to_string().contains("x-goog-api-key"));
    }
}
