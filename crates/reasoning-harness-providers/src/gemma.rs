use std::{env, time::Duration};

use reasoning_harness_core::{
    ModelAdapter, ModelError, ModelErrorKind, ModelOutputFormat, ModelRequest, ModelResponse,
    ModelUsage,
};
use reqwest::{Client, StatusCode, Url, header::RETRY_AFTER};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const DEFAULT_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta/";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(180);
const MAX_RATE_LIMIT_RETRIES: usize = 3;
const INITIAL_RATE_LIMIT_BACKOFF: Duration = Duration::from_secs(10);
const GOOGLE_RECOMMENDED_TEMPERATURE: f32 = 1.0;

/// Google Gemini API / AI Studio adapter for Google-hosted text models.
///
/// This adapter is intentionally limited to untrusted candidate generation. It never
/// participates in harness verification or verdict authority.
pub struct GoogleAdapter {
    client: Client,
    api_key: String,
    base_url: Url,
    model: String,
}

impl GoogleAdapter {
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
                "Google model identifier must not be empty",
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
                temperature: GOOGLE_RECOMMENDED_TEMPERATURE,
            },
            store: false,
        };

        let mut rate_limit_retries = 0usize;
        let response = loop {
            let response = self
                .client
                .post(endpoint.clone())
                .header("x-goog-api-key", &self.api_key)
                .json(&body)
                .send()
                .await
                .map_err(|error| {
                    let detail = if error.is_timeout() {
                        "Gemini API request timed out".to_string()
                    } else {
                        format!("Gemini API request failed: {error}")
                    };
                    ModelError::new(ModelErrorKind::Transport, detail)
                })?;

            if response.status() != StatusCode::TOO_MANY_REQUESTS
                || rate_limit_retries >= MAX_RATE_LIMIT_RETRIES
            {
                break response;
            }

            let delay = rate_limit_delay(response.headers(), rate_limit_retries);
            rate_limit_retries += 1;
            tokio::time::sleep(delay).await;
        };

        let status = response.status();
        if !status.is_success() {
            return Err(ModelError::new(
                ModelErrorKind::Provider,
                format!(
                    "Gemini API returned HTTP {status} after {rate_limit_retries} rate-limit retries"
                ),
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

impl ModelAdapter for GoogleAdapter {
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

fn rate_limit_delay(headers: &reqwest::header::HeaderMap, retry_index: usize) -> Duration {
    if let Some(seconds) = headers
        .get(RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
    {
        return Duration::from_secs(seconds.max(1));
    }

    let multiplier = 1u32.checked_shl(retry_index as u32).unwrap_or(u32::MAX);
    INITIAL_RATE_LIMIT_BACKOFF
        .checked_mul(multiplier)
        .unwrap_or(Duration::MAX)
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
    fn uses_google_recommended_sampling_temperature() {
        let value = serde_json::to_value(GenerationConfig {
            max_output_tokens: Some(4096),
            seed: Some(7),
            temperature: GOOGLE_RECOMMENDED_TEMPERATURE,
        })
        .unwrap();
        assert_eq!(value["temperature"], 1.0);
    }

    #[test]
    fn rate_limit_delay_prefers_retry_after_header() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(RETRY_AFTER, reqwest::header::HeaderValue::from_static("7"));
        assert_eq!(rate_limit_delay(&headers, 2), Duration::from_secs(7));
    }

    #[test]
    fn rate_limit_delay_uses_bounded_exponential_fallback() {
        let headers = reqwest::header::HeaderMap::new();
        assert_eq!(rate_limit_delay(&headers, 0), Duration::from_secs(10));
        assert_eq!(rate_limit_delay(&headers, 1), Duration::from_secs(20));
        assert_eq!(rate_limit_delay(&headers, 2), Duration::from_secs(40));
    }

    #[test]
    fn rejects_empty_credentials_without_echoing_them() {
        let error = GoogleAdapter::new("", "gemma-4-26b-a4b-it").err().unwrap();
        assert_eq!(error.kind, ModelErrorKind::Credentials);
        assert!(!error.to_string().contains("x-goog-api-key"));
    }
}
