use std::{env, time::Duration};

use reasoning_harness_core::{
    ModelAdapter, ModelError, ModelErrorKind, ModelOutputFormat, ModelRequest, ModelResponse,
    ModelUsage,
};
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use serde_json::Value;

const DEFAULT_BASE_URL: &str = "https://api.mistral.ai/v1/";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);

pub struct MistralAdapter {
    client: Client,
    api_key: String,
    base_url: Url,
    model: String,
}

impl MistralAdapter {
    pub fn from_env(model: impl Into<String>) -> Result<Self, ModelError> {
        let api_key = env::var("MISTRAL_API_KEY").map_err(|_| {
            ModelError::new(ModelErrorKind::Credentials, "MISTRAL_API_KEY is not set")
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
                "Mistral API key must not be empty",
            ));
        }

        if model.trim().is_empty() {
            return Err(ModelError::new(
                ModelErrorKind::Protocol,
                "Mistral model identifier must not be empty",
            ));
        }

        let base_url = Url::parse(base_url).map_err(|error| {
            ModelError::new(
                ModelErrorKind::Protocol,
                format!("invalid Mistral base URL: {error}"),
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
        let endpoint = self.base_url.join("chat/completions").map_err(|error| {
            ModelError::new(
                ModelErrorKind::Protocol,
                format!("failed to construct Mistral endpoint: {error}"),
            )
        })?;

        let mut messages = Vec::new();
        if let Some(system) = request.system {
            messages.push(Message {
                role: "system",
                content: system,
            });
        }
        messages.push(Message {
            role: "user",
            content: request.task,
        });

        let body = ChatRequest {
            model: &self.model,
            messages,
            response_format: response_format(request.output_format),
            max_tokens: request.max_tokens,
            random_seed: request.random_seed,
            temperature: 0.0,
        };

        let response = self
            .client
            .post(endpoint)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|error| {
                ModelError::new(
                    ModelErrorKind::Transport,
                    format!("Mistral request failed: {error}"),
                )
            })?;

        let status = response.status();
        if !status.is_success() {
            return Err(ModelError::new(
                ModelErrorKind::Provider,
                format!("Mistral returned HTTP {status}"),
            ));
        }

        let response: ChatResponse = response.json().await.map_err(|error| {
            ModelError::new(
                ModelErrorKind::Protocol,
                format!("invalid Mistral response: {error}"),
            )
        })?;
        let choice = response.choices.into_iter().next().ok_or_else(|| {
            ModelError::new(
                ModelErrorKind::Protocol,
                "Mistral response contained no choices",
            )
        })?;

        Ok(ModelResponse {
            text: choice.message.content.into_text()?,
            model: response.model,
            usage: ModelUsage {
                input_tokens: response
                    .usage
                    .as_ref()
                    .and_then(|usage| usage.prompt_tokens),
                output_tokens: response
                    .usage
                    .as_ref()
                    .and_then(|usage| usage.completion_tokens),
                total_tokens: response.usage.and_then(|usage| usage.total_tokens),
            },
        })
    }
}

impl ModelAdapter for MistralAdapter {
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
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<Message>,
    response_format: ResponseFormat,
    max_tokens: Option<u32>,
    random_seed: Option<u64>,
    temperature: f32,
}

#[derive(Debug, Serialize)]
struct Message {
    role: &'static str,
    content: String,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ResponseFormat {
    Text,
    JsonObject,
    JsonSchema { json_schema: JsonSchema },
}

#[derive(Debug, Serialize)]
struct JsonSchema {
    name: String,
    schema: Value,
    strict: bool,
}

fn response_format(format: ModelOutputFormat) -> ResponseFormat {
    match format {
        ModelOutputFormat::Text => ResponseFormat::Text,
        ModelOutputFormat::JsonObject => ResponseFormat::JsonObject,
        ModelOutputFormat::JsonSchema { name, schema } => ResponseFormat::JsonSchema {
            json_schema: JsonSchema {
                name,
                schema,
                strict: true,
            },
        },
    }
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    model: String,
    choices: Vec<Choice>,
    usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Debug, Deserialize)]
struct ResponseMessage {
    content: ResponseContent,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ResponseContent {
    Text(String),
    Chunks(Vec<ResponseChunk>),
}

impl ResponseContent {
    fn into_text(self) -> Result<String, ModelError> {
        match self {
            Self::Text(text) => Ok(text),
            Self::Chunks(chunks) => {
                let text = chunks
                    .into_iter()
                    .filter_map(|chunk| chunk.text)
                    .collect::<String>();
                if text.is_empty() {
                    Err(ModelError::new(
                        ModelErrorKind::Protocol,
                        "Mistral response contained no text content",
                    ))
                } else {
                    Ok(text)
                }
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct ResponseChunk {
    #[allow(dead_code)]
    #[serde(rename = "type")]
    kind: Option<String>,
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Usage {
    prompt_tokens: Option<u64>,
    completion_tokens: Option<u64>,
    total_tokens: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_json_schema_without_provider_authority_fields() {
        let value = serde_json::to_value(response_format(ModelOutputFormat::JsonSchema {
            name: "artifact".into(),
            schema: serde_json::json!({"type": "object"}),
        }))
        .unwrap();

        assert_eq!(value["type"], "json_schema");
        assert_eq!(value["json_schema"]["name"], "artifact");
        assert_eq!(value["json_schema"]["strict"], true);
        assert!(value.get("verdict").is_none());
    }

    #[test]
    fn rejects_empty_credentials_without_echoing_them() {
        let error = MistralAdapter::new("", "ministral-8b-latest")
            .err()
            .unwrap();
        assert_eq!(error.kind, ModelErrorKind::Credentials);
        assert!(!error.to_string().contains("Bearer"));
    }
}
