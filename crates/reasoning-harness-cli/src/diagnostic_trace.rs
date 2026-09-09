use std::{
    fs,
    future::Future,
    path::Path,
    pin::Pin,
    sync::{
        Mutex,
        atomic::{AtomicU32, Ordering},
    },
    time::Instant,
};

use reasoning_harness_core::{
    FinalAnswerCandidate, FinalizationResult, HarnessOutcome, ModelAdapter, ModelError,
    ModelErrorKind, ModelRequest, ModelResponse, ReasoningCandidate,
};
use serde::Serialize;
use serde_json::Value;

pub const NATURAL_DIAGNOSTIC_TRACE_CONTRACT_ID: &str = "reason-natural-diagnostic-trace-v1";

#[derive(Debug, Clone, Copy)]
pub struct DiagnosticPhase {
    pub name: &'static str,
    pub round: Option<usize>,
}

impl DiagnosticPhase {
    pub const fn new(name: &'static str, round: Option<usize>) -> Self {
        Self { name, round }
    }
}

pub struct DiagnosticModelAdapter<'a> {
    inner: &'a dyn ModelAdapter,
    recorder: Mutex<&'a mut DiagnosticTraceRecorder>,
    phase: DiagnosticPhase,
    provider: &'static str,
    requested_model: &'a str,
    correlation_id: String,
    next_attempt: AtomicU32,
}

impl<'a> DiagnosticModelAdapter<'a> {
    pub fn new(
        inner: &'a dyn ModelAdapter,
        recorder: &'a mut DiagnosticTraceRecorder,
        phase: DiagnosticPhase,
        provider: &'static str,
        requested_model: &'a str,
    ) -> Self {
        let correlation_id = recorder.next_correlation_id(phase);
        Self {
            inner,
            recorder: Mutex::new(recorder),
            phase,
            provider,
            requested_model,
            correlation_id,
            next_attempt: AtomicU32::new(1),
        }
    }
}

impl ModelAdapter for DiagnosticModelAdapter<'_> {
    fn generate<'a>(
        &'a self,
        request: ModelRequest,
    ) -> Pin<Box<dyn Future<Output = Result<ModelResponse, ModelError>> + Send + 'a>> {
        Box::pin(async move {
            let attempt = self.next_attempt.fetch_add(1, Ordering::Relaxed);
            let started = Instant::now();
            {
                let mut recorder = self
                    .recorder
                    .lock()
                    .expect("diagnostic trace recorder mutex poisoned");
                recorder.record_model_request(
                    &self.correlation_id,
                    self.phase,
                    attempt,
                    self.provider,
                    self.requested_model,
                    &request,
                );
            }

            let result = self.inner.generate(request).await;
            {
                let mut recorder = self
                    .recorder
                    .lock()
                    .expect("diagnostic trace recorder mutex poisoned");
                match &result {
                    Ok(response) => recorder.record_model_response(
                        &self.correlation_id,
                        self.phase,
                        attempt,
                        self.provider,
                        started.elapsed().as_millis(),
                        response,
                    ),
                    Err(error) => recorder.record_model_failure(
                        &self.correlation_id,
                        self.phase,
                        attempt,
                        (self.provider, self.requested_model),
                        started.elapsed().as_millis(),
                        error,
                    ),
                }
            }
            result
        })
    }
}

fn diagnostic_model_error_class(kind: ModelErrorKind) -> &'static str {
    match kind {
        ModelErrorKind::Credentials => "credentials",
        ModelErrorKind::Transport => "transport",
        ModelErrorKind::Provider => "provider_error",
        ModelErrorKind::RateLimit => "rate_limit",
        ModelErrorKind::Quota => "quota",
        ModelErrorKind::ProviderUnavailable => "provider_unavailable",
        ModelErrorKind::Timeout => "timeout",
        ModelErrorKind::Protocol => "protocol",
        ModelErrorKind::UnsupportedCapability => "unsupported_capability",
    }
}

#[derive(Debug, Serialize)]
pub struct DiagnosticTraceRecorder {
    contract: &'static str,
    provider: String,
    model: String,
    seed: Option<u64>,
    events: Vec<DiagnosticEvent>,
    #[serde(skip)]
    next_sequence: usize,
    #[serde(skip)]
    next_correlation: usize,
}

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum DiagnosticEvent {
    ModelRequest {
        sequence: usize,
        correlation_id: String,
        phase: &'static str,
        #[serde(skip_serializing_if = "Option::is_none")]
        round: Option<usize>,
        attempt: u32,
        provider: String,
        model: String,
        request: Value,
    },
    ModelResponse {
        sequence: usize,
        correlation_id: String,
        phase: &'static str,
        #[serde(skip_serializing_if = "Option::is_none")]
        round: Option<usize>,
        attempt: u32,
        provider: String,
        model: String,
        latency_ms: u128,
        response: DiagnosticStructuredResponse,
    },
    ModelFailure {
        sequence: usize,
        correlation_id: String,
        phase: &'static str,
        #[serde(skip_serializing_if = "Option::is_none")]
        round: Option<usize>,
        attempt: u32,
        provider: String,
        model: String,
        failure_class: String,
        latency_ms: u128,
        message: String,
    },
    CandidateState {
        sequence: usize,
        phase: &'static str,
        #[serde(skip_serializing_if = "Option::is_none")]
        round: Option<usize>,
        candidate: Value,
    },
    GroundingState {
        sequence: usize,
        phase: &'static str,
        #[serde(skip_serializing_if = "Option::is_none")]
        round: Option<usize>,
        outcome: Value,
    },
    EvidenceAdmission {
        sequence: usize,
        phase: &'static str,
        round: usize,
        target_id: String,
        capability_id: String,
        admitted_evidence_ids: Vec<String>,
        fact_keys: Vec<String>,
    },
    FinalRenderState {
        sequence: usize,
        phase: &'static str,
        round: usize,
        source: &'static str,
        candidate: Value,
    },
    FinalizationState {
        sequence: usize,
        phase: &'static str,
        round: usize,
        stage: &'static str,
        result: Value,
    },
    AnswerSafetyState {
        sequence: usize,
        phase: &'static str,
        round: usize,
        observation: Value,
    },
}

#[derive(Debug, Serialize)]
struct DiagnosticStructuredResponse {
    model: String,
    usage: Value,
    provider_attempts: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    finish_reason: Option<String>,
    bytes: usize,
    structured: bool,
    trailing_text_omitted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<Value>,
}

impl DiagnosticTraceRecorder {
    pub fn new(provider: &str, model: &str, seed: Option<u64>) -> Self {
        Self {
            contract: NATURAL_DIAGNOSTIC_TRACE_CONTRACT_ID,
            provider: provider.to_string(),
            model: model.to_string(),
            seed,
            events: Vec::new(),
            next_sequence: 0,
            next_correlation: 0,
        }
    }

    pub fn next_correlation_id(&mut self, phase: DiagnosticPhase) -> String {
        let id = format!(
            "{:04}-{}-r-{}",
            self.next_correlation,
            phase.name,
            phase
                .round
                .map_or_else(|| "na".to_string(), |round| round.to_string())
        );
        self.next_correlation = self.next_correlation.saturating_add(1);
        id
    }

    pub fn record_model_request(
        &mut self,
        correlation_id: &str,
        phase: DiagnosticPhase,
        attempt: u32,
        provider: &str,
        model: &str,
        request: &ModelRequest,
    ) {
        let request = sanitized_value(request);
        let sequence = self.sequence();
        self.events.push(DiagnosticEvent::ModelRequest {
            sequence,
            correlation_id: correlation_id.to_string(),
            phase: phase.name,
            round: phase.round,
            attempt,
            provider: provider.to_string(),
            model: model.to_string(),
            request,
        });
    }

    pub fn record_model_response(
        &mut self,
        correlation_id: &str,
        phase: DiagnosticPhase,
        attempt: u32,
        provider: &str,
        latency_ms: u128,
        response: &ModelResponse,
    ) {
        let (value, trailing_text_omitted) = structured_response_value(&response.text);
        let structured = value.is_some();
        let response_value = value.map(sanitize_value);
        let sequence = self.sequence();
        self.events.push(DiagnosticEvent::ModelResponse {
            sequence,
            correlation_id: correlation_id.to_string(),
            phase: phase.name,
            round: phase.round,
            attempt,
            provider: provider.to_string(),
            model: response.model.clone(),
            latency_ms,
            response: DiagnosticStructuredResponse {
                model: response.model.clone(),
                usage: sanitized_value(&response.usage),
                provider_attempts: response.provider_attempts,
                finish_reason: response.finish_reason.as_deref().map(sanitize_text),
                bytes: response.text.len(),
                structured,
                trailing_text_omitted,
                value: response_value,
            },
        });
    }

    pub fn record_model_failure(
        &mut self,
        correlation_id: &str,
        phase: DiagnosticPhase,
        attempt: u32,
        identity: (&str, &str),
        latency_ms: u128,
        error: &ModelError,
    ) {
        let (provider, model) = identity;
        let sequence = self.sequence();
        self.events.push(DiagnosticEvent::ModelFailure {
            sequence,
            correlation_id: correlation_id.to_string(),
            phase: phase.name,
            round: phase.round,
            attempt,
            provider: provider.to_string(),
            model: model.to_string(),
            failure_class: diagnostic_model_error_class(error.kind).to_string(),
            latency_ms,
            message: sanitize_text(&error.to_string()),
        });
    }

    pub fn record_candidate(&mut self, phase: DiagnosticPhase, candidate: &ReasoningCandidate) {
        let sequence = self.sequence();
        self.events.push(DiagnosticEvent::CandidateState {
            sequence,
            phase: phase.name,
            round: phase.round,
            candidate: sanitized_value(candidate),
        });
    }

    pub fn record_grounding(&mut self, phase: DiagnosticPhase, outcome: &HarnessOutcome) {
        let sequence = self.sequence();
        self.events.push(DiagnosticEvent::GroundingState {
            sequence,
            phase: phase.name,
            round: phase.round,
            outcome: sanitized_value(outcome),
        });
    }

    pub fn record_evidence_admission(
        &mut self,
        phase: DiagnosticPhase,
        round: usize,
        target_id: &str,
        capability_id: &str,
        admitted_evidence_ids: Vec<String>,
        mut fact_keys: Vec<String>,
    ) {
        fact_keys.sort();
        fact_keys.dedup();
        let sequence = self.sequence();
        self.events.push(DiagnosticEvent::EvidenceAdmission {
            sequence,
            phase: phase.name,
            round,
            target_id: target_id.to_string(),
            capability_id: capability_id.to_string(),
            admitted_evidence_ids,
            fact_keys,
        });
    }

    pub fn record_final_render(
        &mut self,
        phase: DiagnosticPhase,
        round: usize,
        source: &'static str,
        candidate: &FinalAnswerCandidate,
    ) {
        let sequence = self.sequence();
        self.events.push(DiagnosticEvent::FinalRenderState {
            sequence,
            phase: phase.name,
            round,
            source,
            candidate: sanitized_value(candidate),
        });
    }

    pub fn record_finalization(
        &mut self,
        phase: DiagnosticPhase,
        round: usize,
        stage: &'static str,
        result: &FinalizationResult,
    ) {
        let sequence = self.sequence();
        self.events.push(DiagnosticEvent::FinalizationState {
            sequence,
            phase: phase.name,
            round,
            stage,
            result: sanitized_value(result),
        });
    }

    pub fn record_answer_safety<T: Serialize>(
        &mut self,
        phase: DiagnosticPhase,
        round: usize,
        observation: &T,
    ) {
        let sequence = self.sequence();
        self.events.push(DiagnosticEvent::AnswerSafetyState {
            sequence,
            phase: phase.name,
            round,
            observation: sanitized_value(observation),
        });
    }

    pub fn write_to_path(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "failed to create diagnostic trace directory {}: {error}",
                    parent.display()
                )
            })?;
        }
        let bytes = serde_json::to_vec_pretty(self)
            .map_err(|error| format!("failed to serialize diagnostic trace: {error}"))?;
        let tmp = path.with_extension("tmp");
        fs::write(&tmp, bytes).map_err(|error| {
            format!(
                "failed to write diagnostic trace {}: {error}",
                tmp.display()
            )
        })?;
        fs::rename(&tmp, path).map_err(|error| {
            format!(
                "failed to finalize diagnostic trace {} -> {}: {error}",
                tmp.display(),
                path.display()
            )
        })
    }

    fn sequence(&mut self) -> usize {
        let sequence = self.next_sequence;
        self.next_sequence = self.next_sequence.saturating_add(1);
        sequence
    }
}

fn sanitized_value<T: Serialize>(value: &T) -> Value {
    serde_json::to_value(value)
        .map(sanitize_value)
        .unwrap_or(Value::Null)
}

fn sanitize_value(value: Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.into_iter()
                .map(|(key, value)| {
                    let lowered = key.to_ascii_lowercase();
                    let value = if hidden_reasoning_key(&lowered) {
                        Value::String("[OMITTED_HIDDEN_REASONING]".into())
                    } else if secret_key(&lowered) {
                        Value::String("[REDACTED]".into())
                    } else {
                        sanitize_value(value)
                    };
                    (key, value)
                })
                .collect(),
        ),
        Value::Array(values) => Value::Array(values.into_iter().map(sanitize_value).collect()),
        Value::String(text) => Value::String(sanitize_text(&text)),
        other => other,
    }
}

fn secret_key(key: &str) -> bool {
    [
        "authorization",
        "api_key",
        "apikey",
        "access_key",
        "secret",
        "credential",
        "private_key",
        "password",
    ]
    .iter()
    .any(|needle| key.contains(needle))
}

fn hidden_reasoning_key(key: &str) -> bool {
    [
        "chain_of_thought",
        "chain-of-thought",
        "internal_reasoning",
        "hidden_reasoning",
        "reasoning_content",
        "thoughts",
        "scratchpad",
    ]
    .iter()
    .any(|needle| key.contains(needle))
        || matches!(key, "analysis" | "reasoning" | "thinking" | "thought")
}

fn sanitize_text(text: &str) -> String {
    text.lines()
        .map(|line| {
            let lowered = line.to_ascii_lowercase();
            if lowered.contains("authorization:")
                || lowered.contains("bearer ")
                || lowered.contains("api_key")
                || lowered.contains("api-key")
                || lowered.contains("apikey")
                || lowered.contains("access_key")
                || lowered.contains("private_key")
                || lowered.contains("password=")
                || lowered.contains("password:")
                || lowered.contains("credential")
                || lowered.contains("secret=")
                || lowered.contains("secret:")
            {
                "[REDACTED]".to_string()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn structured_response_value(text: &str) -> (Option<Value>, bool) {
    if let Ok(value) = serde_json::from_str::<Value>(text) {
        return (Some(value), false);
    }
    let mut stream = serde_json::Deserializer::from_str(text).into_iter::<Value>();
    let Some(Ok(value)) = stream.next() else {
        return (None, false);
    };
    let remainder = text[stream.byte_offset()..].trim();
    (Some(value), !remainder.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use reasoning_harness_core::{ModelOutputFormat, ModelUsage};

    #[test]
    fn request_and_structured_response_are_sanitized_without_raw_hidden_text() {
        let mut recorder = DiagnosticTraceRecorder::new("fixture", "model", Some(7));
        let phase = DiagnosticPhase::new("generation", Some(1));
        let correlation = recorder.next_correlation_id(phase);
        let request = ModelRequest {
            task: "task\nPassword: redacted-value\nvisible".into(),
            system: Some("system".into()),
            output_format: ModelOutputFormat::JsonObject,
            max_tokens: Some(32),
            random_seed: Some(7),
            reasoning_preference: None,
        };
        recorder.record_model_request(&correlation, phase, 1, "fixture", "model", &request);
        let response = ModelResponse {
            text: r#"{"claims":[],"api_key":"redacted-value","chain_of_thought":"private","visible":"ok"}
ignored prose"#.into(),
            model: "model".into(),
            usage: ModelUsage::default(),
            provider_attempts: 1,
            finish_reason: Some("stop".into()),
        };
        recorder.record_model_response(&correlation, phase, 1, "fixture", 0, &response);

        let serialized = serde_json::to_string(&recorder).unwrap();
        assert!(!serialized.contains("redacted-value"));
        assert!(!serialized.contains("ignored prose"));
        assert!(!serialized.contains("\"private\""));
        assert!(!serialized.contains("private-analysis"));
        assert!(!serialized.contains("private-reasoning"));
        assert!(serialized.contains("[REDACTED]"));
        assert!(serialized.contains("[OMITTED_HIDDEN_REASONING]"));
        assert!(serialized.contains("\"visible\":\"ok\""));
        assert!(serialized.contains("\"trailing_text_omitted\":true"));
    }

    struct CapturingAdapter {
        request: Mutex<Option<ModelRequest>>,
        response: ModelResponse,
    }

    impl ModelAdapter for CapturingAdapter {
        fn generate<'a>(
            &'a self,
            request: ModelRequest,
        ) -> Pin<Box<dyn Future<Output = Result<ModelResponse, ModelError>> + Send + 'a>> {
            *self.request.lock().unwrap() = Some(request);
            let response = self.response.clone();
            Box::pin(async move { Ok(response) })
        }
    }

    #[tokio::test]
    async fn diagnostic_wrapper_forwards_request_without_semantic_mutation() {
        let request = ModelRequest {
            task: "task payload".into(),
            system: Some("system payload".into()),
            output_format: ModelOutputFormat::JsonObject,
            max_tokens: Some(64),
            random_seed: Some(17),
            reasoning_preference: None,
        };
        let expected_bytes = serde_json::to_vec(&request).unwrap();
        let expected_response = ModelResponse {
            text: r#"{"claims":[],"inferences":[]}"#.into(),
            model: "model".into(),
            usage: ModelUsage::default(),
            provider_attempts: 1,
            finish_reason: Some("stop".into()),
        };
        let inner = CapturingAdapter {
            request: Mutex::new(None),
            response: expected_response.clone(),
        };
        let mut recorder = DiagnosticTraceRecorder::new("fixture", "model", Some(17));
        let wrapped = DiagnosticModelAdapter::new(
            &inner,
            &mut recorder,
            DiagnosticPhase::new("generation", None),
            "fixture",
            "model",
        );

        let actual_response = wrapped.generate(request).await.unwrap();
        let forwarded = inner.request.lock().unwrap().clone().unwrap();
        assert_eq!(serde_json::to_vec(&forwarded).unwrap(), expected_bytes);
        assert_eq!(actual_response, expected_response);
    }

    #[test]
    fn admitted_evidence_then_claim_drop_is_reconstructable_from_trace() {
        use reasoning_harness_core::{FinalizationStatus, ReasoningArtifact, Verdict};

        let mut recorder = DiagnosticTraceRecorder::new("fixture", "model", Some(23));
        recorder.record_evidence_admission(
            DiagnosticPhase::new("evidence_admission", Some(1)),
            1,
            "target",
            "catalog",
            vec!["evidence-1".into()],
            vec!["fixture.answer".into()],
        );
        recorder.record_candidate(
            DiagnosticPhase::new("post_investigation_regeneration", Some(1)),
            &ReasoningCandidate::default(),
        );
        recorder.record_grounding(
            DiagnosticPhase::new("post_investigation_grounding", Some(1)),
            &HarnessOutcome {
                verdict: Verdict::Unknown,
                artifact: ReasoningArtifact::default(),
            },
        );
        recorder.record_final_render(
            DiagnosticPhase::new("final_render", Some(1)),
            1,
            "model",
            &FinalAnswerCandidate::default(),
        );
        recorder.record_finalization(
            DiagnosticPhase::new("finalization", Some(1)),
            1,
            "renderer_candidate",
            &FinalizationResult {
                status: FinalizationStatus::Unresolved,
                text: None,
                factual_claims: 0,
                covered_claims: 0,
                factual_claim_coverage: 0.0,
                uncovered_propositions: vec![],
            },
        );

        let value = serde_json::to_value(&recorder).unwrap();
        let events = value["events"].as_array().unwrap();
        let kinds = events
            .iter()
            .map(|event| event["kind"].as_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            kinds,
            vec![
                "evidence_admission",
                "candidate_state",
                "grounding_state",
                "final_render_state",
                "finalization_state",
            ]
        );
        assert_eq!(events[0]["admitted_evidence_ids"][0], "evidence-1");
        assert_eq!(events[0]["fact_keys"][0], "fixture.answer");
        assert_eq!(
            events[1]["candidate"]["claims"].as_array().unwrap().len(),
            0
        );
        assert_eq!(events[4]["result"]["status"], "unresolved");
    }

    #[test]
    fn invalid_unstructured_response_content_is_not_persisted() {
        let mut recorder = DiagnosticTraceRecorder::new("fixture", "model", None);
        let phase = DiagnosticPhase::new("generation", None);
        let correlation = recorder.next_correlation_id(phase);
        let response = ModelResponse {
            text: "internal unstructured model text".into(),
            model: "model".into(),
            usage: ModelUsage::default(),
            provider_attempts: 1,
            finish_reason: None,
        };
        recorder.record_model_response(&correlation, phase, 1, "fixture", 0, &response);
        let serialized = serde_json::to_string(&recorder).unwrap();
        assert!(!serialized.contains("internal unstructured model text"));
        assert!(serialized.contains("\"structured\":false"));
    }
}
