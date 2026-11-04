//! Private evaluation sink. The operator passes an already-created private FD;
//! no path is chosen from model output, receipts, user config or environment.
use std::{
    collections::BTreeSet, fs::File, future::Future, io::Write, pin::Pin, sync::Mutex,
    time::Duration,
};

use reasoning_harness_core::ModelError;
use reasoning_harness_providers::http_observation::{
    HttpObservation, HttpObservationSink, observation_error, sha256,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::time::Instant;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HttpLimits {
    contract: String,
    max_request_utf8_json_bytes: u64,
    input_overhead_tokens: u64,
    max_input_tokens_per_generation: u64,
    max_output_tokens_per_generation: u64,
    max_input_tokens_total: u64,
    max_output_tokens_total: u64,
    max_generations: u32,
    min_generation_start_interval_ms: u64,
}

impl HttpLimits {
    pub(crate) fn parse(raw: &str, loopback_control: bool) -> Result<Self, String> {
        if raw.len() > 4096 {
            return Err("oversized native HTTP limits".into());
        }
        let limits: Self =
            serde_json::from_str(raw).map_err(|_| "invalid native HTTP limits".to_string())?;
        if limits.contract != "reason-cli-normal-v3-http-limits-v1"
            || !(1..=12000).contains(&limits.max_request_utf8_json_bytes)
            || limits.input_overhead_tokens != 512
            || !(1..=12512).contains(&limits.max_input_tokens_per_generation)
            || !(1..=1024).contains(&limits.max_output_tokens_per_generation)
            || !(1..=24000).contains(&limits.max_input_tokens_total)
            || !(1..=4096).contains(&limits.max_output_tokens_total)
            || !(1..=24).contains(&limits.max_generations)
            || limits.min_generation_start_interval_ms > 60000
            || (!loopback_control && limits.min_generation_start_interval_ms < 10500)
        {
            return Err("native HTTP limits weaken the frozen profile".into());
        }
        Ok(limits)
    }
}

pub(crate) struct EvaluationHttpJournal {
    binding: String,
    max_attempts: u32,
    reserve: u64,
    limits: Option<HttpLimits>,
    clock: Instant,
    state: Mutex<State>,
}

struct State {
    file: File,
    sequence: u64,
    previous: String,
    attempts: u32,
    used: u64,
    used_input: u64,
    used_output: u64,
    preparing: Option<Preflight>,
    last_start: Option<Instant>,
    pending: Option<String>,
    finalizing: Option<String>,
    seen: BTreeSet<String>,
    halted: bool,
}

#[derive(Clone)]
struct Preflight {
    request_id: String,
    input: u64,
    output: u64,
}

/// Cancellation while asynchronously pacing must burn the reservation and
/// latch stop even though no PendingHttpObservation/physical HTTP exists yet.
struct PacingGuard<'a> {
    journal: &'a EvaluationHttpJournal,
    request_id: String,
    armed: bool,
}
impl Drop for PacingGuard<'_> {
    fn drop(&mut self) {
        if self.armed {
            let _ = self.journal.record(HttpObservation::Aborted {
                request_id: self.request_id.clone(),
            });
        }
    }
}

pub(crate) fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

impl EvaluationHttpJournal {
    #[cfg(unix)]
    pub fn from_fd(
        fd: i32,
        binding: &str,
        max_attempts: u32,
        reserve: u64,
        limits: Option<HttpLimits>,
    ) -> Result<Self, ModelError> {
        use std::os::{fd::FromRawFd, unix::fs::MetadataExt};
        if fd < 3 || !is_sha256(binding) || max_attempts == 0 || reserve == 0 {
            return Err(observation_error());
        }
        // An inherited capability is supplied by the operator's private
        // create-only launcher. Never take ownership of an arbitrary raw FD.
        let duplicated = unsafe { libc::fcntl(fd, libc::F_DUPFD_CLOEXEC, 3) };
        if duplicated < 0 {
            return Err(observation_error());
        }
        let file = unsafe { File::from_raw_fd(duplicated) };
        let metadata = file.metadata().map_err(|_| observation_error())?;
        let current_uid = unsafe { libc::geteuid() };
        if !metadata.is_file()
            || metadata.uid() != current_uid
            || metadata.mode() & 0o777 != 0o600
            || metadata.nlink() != 1
            || metadata.len() != 0
            || unsafe { libc::lseek(duplicated, 0, libc::SEEK_CUR) } != 0
        {
            return Err(observation_error());
        }
        // External resolver children must not inherit the original capability.
        if unsafe { libc::fcntl(fd, libc::F_SETFD, libc::FD_CLOEXEC) } < 0 {
            return Err(observation_error());
        }
        if limits
            .as_ref()
            .is_some_and(|p| max_attempts != p.max_generations)
        {
            return Err(observation_error());
        }
        Self::new(file, binding, max_attempts, reserve, limits)
    }

    #[cfg(not(unix))]
    pub fn from_fd(
        _fd: i32,
        _binding: &str,
        _max_attempts: u32,
        _reserve: u64,
        _limits: Option<HttpLimits>,
    ) -> Result<Self, ModelError> {
        Err(observation_error())
    }

    fn new(
        file: File,
        binding: &str,
        max_attempts: u32,
        reserve: u64,
        limits: Option<HttpLimits>,
    ) -> Result<Self, ModelError> {
        let sink = Self {
            binding: binding.to_owned(),
            max_attempts,
            reserve,
            limits,
            clock: Instant::now(),
            state: Mutex::new(State {
                file,
                sequence: 0,
                previous: "0".repeat(64),
                attempts: 0,
                used: 0,
                used_input: 0,
                used_output: 0,
                preparing: None,
                last_start: None,
                pending: None,
                finalizing: None,
                seen: BTreeSet::new(),
                halted: false,
            }),
        };
        {
            let mut state = sink.state.lock().map_err(|_| observation_error())?;
            let mut binding_event =
                json!({"kind":"binding", "max_attempts":max_attempts, "token_reserve":reserve});
            if let Some(limits) = &sink.limits {
                binding_event["limits"] =
                    serde_json::to_value(limits).map_err(|_| observation_error())?;
            }
            sink.append(&mut state, binding_event)?;
        }
        Ok(sink)
    }

    fn append(&self, state: &mut State, event: Value) -> Result<(), ModelError> {
        let contract = if self.limits.is_some() {
            "reason-cli-evaluation-http-journal-v2"
        } else {
            "reason-cli-evaluation-http-journal-v1"
        };
        let row = json!({"contract":contract, "sequence":state.sequence,
            "previous_sha256":state.previous, "binding_sha256":self.binding, "event":event});
        let mut bytes = serde_json::to_vec(&row).map_err(|_| observation_error())?;
        let hash = sha256(&bytes);
        bytes.push(b'\n');
        if state
            .file
            .write_all(&bytes)
            .and_then(|()| state.file.sync_all())
            .is_err()
        {
            state.halted = true;
            return Err(observation_error());
        }
        state.sequence += 1;
        state.previous = hash;
        Ok(())
    }

    fn prepare(&self, event: &HttpObservation) -> Result<Duration, ModelError> {
        let mut state = self.state.lock().map_err(|_| observation_error())?;
        let limits = self.limits.as_ref().ok_or_else(observation_error)?;
        let HttpObservation::Request {
            request_id,
            context,
            ..
        } = event
        else {
            return Err(observation_error());
        };
        let measured = context.budget_input.as_ref();
        let input = measured.and_then(|m| {
            m.request_utf8_json_bytes
                .checked_add(limits.input_overhead_tokens)
        });
        let output = measured.and_then(|m| m.max_output_tokens).map(u64::from);
        let valid = !state.halted
            && state.preparing.is_none()
            && state.pending.is_none()
            && state.finalizing.is_none()
            && state.attempts < limits.max_generations
            && !state.seen.contains(request_id)
            && measured
                .is_some_and(|m| m.request_utf8_json_bytes <= limits.max_request_utf8_json_bytes)
            && input.is_some_and(|n| {
                n <= limits.max_input_tokens_per_generation
                    && state
                        .used_input
                        .checked_add(n)
                        .is_some_and(|total| total <= limits.max_input_tokens_total)
            })
            && output.is_some_and(|n| {
                n > 0
                    && n <= limits.max_output_tokens_per_generation
                    && state
                        .used_output
                        .checked_add(n)
                        .is_some_and(|total| total <= limits.max_output_tokens_total)
            })
            && input
                .zip(output)
                .and_then(|(i, o)| i.checked_add(o))
                .and_then(|n| state.used.checked_add(n))
                .is_some_and(|n| n <= self.reserve);
        if !valid {
            state.halted = true;
            self.append(&mut state, json!({"kind":"blocked", "request_id":request_id, "reason":"http_limits_preflight"}))?;
            return Err(observation_error());
        }
        let preflight = Preflight {
            request_id: request_id.clone(),
            input: input.unwrap(),
            output: output.unwrap(),
        };
        let wait = state
            .last_start
            .map(|start| {
                (start + Duration::from_millis(limits.min_generation_start_interval_ms))
                    .saturating_duration_since(Instant::now())
            })
            .unwrap_or_default();
        let value = json!({"kind":"preflight", "request_id":request_id, "context":context,
            "request_utf8_json_bytes":measured.unwrap().request_utf8_json_bytes,
            "input_reserve_tokens":preflight.input, "requested_output_tokens":preflight.output,
            "used_input_tokens":state.used_input, "used_output_tokens":state.used_output});
        state.preparing = Some(preflight);
        self.append(&mut state, value)?;
        Ok(wait)
    }
}

impl HttpObservationSink for EvaluationHttpJournal {
    fn before_request<'a>(
        &'a self,
        event: HttpObservation,
    ) -> Pin<Box<dyn Future<Output = Result<(), ModelError>> + Send + 'a>> {
        Box::pin(async move {
            if self.limits.is_none() {
                return self.record(event);
            }
            let wait = self.prepare(&event)?;
            let HttpObservation::Request { request_id, .. } = &event else {
                return Err(observation_error());
            };
            let mut guard = PacingGuard {
                journal: self,
                request_id: request_id.clone(),
                armed: true,
            };
            tokio::time::sleep(wait).await;
            self.record(event)?;
            guard.armed = false;
            Ok(())
        })
    }

    fn record(&self, event: HttpObservation) -> Result<(), ModelError> {
        let mut state = self.state.lock().map_err(|_| observation_error())?;
        let mut start_elapsed_ns = None;
        match &event {
            HttpObservation::Request {
                request_id,
                context,
                provider,
                model,
                wire_request_sha256,
            } => {
                let valid_id = !context.correlation_id.is_empty()
                    && context.correlation_id.len() <= 120
                    && context
                        .correlation_id
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"-_".contains(&b));
                if state.halted
                    || state.pending.is_some()
                    || state.finalizing.is_some()
                    || !valid_id
                    || context.logical_attempt == 0
                    || !is_sha256(&context.request_sha256)
                    || !is_sha256(wire_request_sha256)
                    || *provider != "mistral"
                    || model.is_empty()
                    || model.len() > 200
                    || state.attempts >= self.max_attempts
                    || state.used >= self.reserve
                    || (self.limits.is_some()
                        && state
                            .preparing
                            .as_ref()
                            .is_none_or(|p| p.request_id != *request_id))
                    || !state.seen.insert(request_id.clone())
                {
                    state.halted = true;
                    return Err(observation_error());
                }
                state.attempts += 1;
                state.pending = Some(request_id.clone());
                if self.limits.is_some() {
                    let now = Instant::now();
                    state.last_start = Some(now);
                    start_elapsed_ns = Some(now.duration_since(self.clock).as_nanos() as u64);
                }
            }
            HttpObservation::Response {
                request_id,
                http_status,
                response_sha256,
                usage,
                failure_class,
            } => {
                if state.pending.as_ref() != Some(request_id) {
                    state.halted = true;
                    return Err(observation_error());
                }
                let tokens = usage
                    .input_tokens
                    .zip(usage.output_tokens)
                    .zip(usage.total_tokens)
                    .filter(|((input, output), total)| input.checked_add(*output) == Some(*total))
                    .and_then(|(_, total)| state.used.checked_add(total));
                if let Some(limits) = &self.limits {
                    let reserved = state.preparing.as_ref();
                    let within =
                        usage
                            .input_tokens
                            .zip(usage.output_tokens)
                            .is_some_and(|(i, o)| {
                                reserved.is_some_and(|p| i <= p.input && o <= p.output)
                                    && state
                                        .used_input
                                        .checked_add(i)
                                        .is_some_and(|v| v <= limits.max_input_tokens_total)
                                    && state
                                        .used_output
                                        .checked_add(o)
                                        .is_some_and(|v| v <= limits.max_output_tokens_total)
                            });
                    state.halted |= !within;
                }
                state.halted |= !(200..300).contains(http_status)
                    || !is_sha256(response_sha256)
                    || failure_class.is_some()
                    || tokens.is_none_or(|total| total > self.reserve);
                if let Some(tokens) = tokens {
                    state.used = tokens;
                }
                if let Some(input) = usage
                    .input_tokens
                    .and_then(|n| state.used_input.checked_add(n))
                {
                    state.used_input = input;
                }
                if let Some(output) = usage
                    .output_tokens
                    .and_then(|n| state.used_output.checked_add(n))
                {
                    state.used_output = output;
                }
                state.preparing = None;
                state.pending = None;
                state.finalizing = Some(request_id.clone());
            }
            HttpObservation::Aborted { request_id } => {
                if state.pending.as_ref() != Some(request_id)
                    && state.finalizing.as_ref() != Some(request_id)
                    && state.preparing.as_ref().map(|p| &p.request_id) != Some(request_id)
                {
                    state.halted = true;
                    return Err(observation_error());
                }
                state.halted = true;
                state.pending = None;
                state.finalizing = None;
                state.preparing = None;
            }
            HttpObservation::Completed { request_id } => {
                if state.halted
                    || state.pending.is_some()
                    || state.finalizing.as_ref() != Some(request_id)
                {
                    state.halted = true;
                    return Err(observation_error());
                }
                state.finalizing = None;
            }
        }
        let mut value = serde_json::to_value(event).map_err(|_| observation_error())?;
        if let Some(time) = start_elapsed_ns {
            value["start_elapsed_ns"] = json!(time);
        }
        self.append(&mut state, value)?;
        if state.halted {
            Err(observation_error())
        } else {
            Ok(())
        }
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use reasoning_harness_core::{ModelAdapter, ModelOutputFormat, ModelRequest};
    use reasoning_harness_providers::{
        MistralAdapter,
        http_observation::{HttpObservationContext, with_http_observation_context},
    };
    use std::{
        io::Read,
        net::TcpListener,
        os::{fd::AsRawFd, unix::fs::OpenOptionsExt},
        path::PathBuf,
        sync::{
            Arc,
            atomic::{AtomicBool, AtomicU64, Ordering},
        },
        thread,
        time::{Duration, SystemTime, UNIX_EPOCH},
    };

    struct Server {
        url: String,
        requests: Arc<Mutex<Vec<Vec<u8>>>>,
        stop: Arc<AtomicBool>,
        worker: Option<thread::JoinHandle<()>>,
    }
    impl Server {
        fn new(status: u16, body: String, headers: &'static str) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            listener.set_nonblocking(true).unwrap();
            let url = format!("http://{}/v1/", listener.local_addr().unwrap());
            let stop = Arc::new(AtomicBool::new(false));
            let requests = Arc::new(Mutex::new(Vec::new()));
            let (end, captured) = (stop.clone(), requests.clone());
            let worker = thread::spawn(move || {
                while !end.load(Ordering::Relaxed) {
                    let (mut stream, _) = match listener.accept() {
                        Ok(s) => s,
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            thread::sleep(Duration::from_millis(2));
                            continue;
                        }
                        Err(e) => panic!("{e}"),
                    };
                    stream.set_nonblocking(false).unwrap();
                    stream
                        .set_read_timeout(Some(Duration::from_secs(2)))
                        .unwrap();
                    let mut bytes = Vec::new();
                    let mut buffer = [0; 4096];
                    loop {
                        let n = stream.read(&mut buffer).unwrap();
                        assert!(n > 0);
                        bytes.extend_from_slice(&buffer[..n]);
                        if let Some(end) = bytes.windows(4).position(|p| p == b"\r\n\r\n") {
                            let size: usize = String::from_utf8_lossy(&bytes[..end])
                                .lines()
                                .find_map(|l| {
                                    l.to_ascii_lowercase()
                                        .strip_prefix("content-length:")
                                        .map(|v| v.trim().parse().unwrap())
                                })
                                .unwrap();
                            if bytes.len() >= end + 4 + size {
                                captured
                                    .lock()
                                    .unwrap()
                                    .push(bytes[end + 4..end + 4 + size].to_vec());
                                break;
                            }
                        }
                    }
                    let head = format!(
                        "HTTP/1.1 {status} fixture\r\nContent-Length: {}\r\nConnection: close\r\n{headers}\r\n",
                        body.len()
                    );
                    let _ = stream.write_all(head.as_bytes());
                    let _ = stream.write_all(body.as_bytes());
                }
            });
            Self {
                url,
                requests,
                stop,
                worker: Some(worker),
            }
        }
    }
    impl Drop for Server {
        fn drop(&mut self) {
            self.stop.store(true, Ordering::Relaxed);
            let _ = self.worker.take().unwrap().join();
        }
    }
    fn fixture() -> (PathBuf, File, Arc<EvaluationHttpJournal>) {
        limited_fixture(None)
    }
    fn limited_fixture(limits: Option<HttpLimits>) -> (PathBuf, File, Arc<EvaluationHttpJournal>) {
        static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "reason-http-{}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&path)
            .unwrap();
        let attempts = limits.as_ref().map(|p| p.max_generations).unwrap_or(3);
        let reserve = if limits.is_some() { 50000 } else { 20 };
        let sink = Arc::new(
            EvaluationHttpJournal::from_fd(
                file.as_raw_fd(),
                &"a".repeat(64),
                attempts,
                reserve,
                limits,
            )
            .unwrap(),
        );
        (path, file, sink)
    }
    fn request() -> ModelRequest {
        ModelRequest {
            system: None,
            task: "private task".into(),
            output_format: ModelOutputFormat::Text,
            max_tokens: Some(10),
            random_seed: None,
            reasoning_preference: None,
        }
    }
    async fn call(
        adapter: &MistralAdapter,
        attempt: u32,
    ) -> Result<reasoning_harness_core::ModelResponse, ModelError> {
        with_http_observation_context(
            HttpObservationContext {
                correlation_id: "control".into(),
                logical_attempt: attempt,
                request_sha256: "b".repeat(64),
                budget_input: None,
            },
            adapter.generate(request()),
        )
        .await
    }
    fn response(usage: Value, model: &str) -> String {
        json!({"model":model,"choices":[{"message":{"content":"{}"},"finish_reason":"stop"}],"usage":usage}).to_string()
    }

    #[tokio::test]
    async fn actual_http_bytes_are_hashed_and_reserve_exhaustion_blocks_next_send() {
        let raw = response(
            json!({"prompt_tokens":8,"completion_tokens":12,"total_tokens":20}),
            "fixture",
        );
        let server = Server::new(200, raw.clone(), "");
        let (path, file, sink) = fixture();
        let mut adapter =
            MistralAdapter::with_base_url("secret-fixture-key", "fixture", &server.url).unwrap();
        adapter.set_http_observer(sink.clone());
        assert_eq!(
            call(&adapter, 1).await.unwrap().usage.total_tokens,
            Some(20)
        );
        assert!(call(&adapter, 2).await.is_err());
        assert_eq!(server.requests.lock().unwrap().len(), 1);
        let data = std::fs::read_to_string(&path).unwrap();
        let rows: Vec<Value> = data
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
        assert_eq!(
            rows[1]["event"]["wire_request_sha256"],
            sha256(&server.requests.lock().unwrap()[0])
        );
        assert_eq!(rows[2]["event"]["response_sha256"], sha256(raw.as_bytes()));
        assert!(!data.contains("secret-fixture-key") && !data.contains("private task"));
        drop(sink);
        drop(adapter);
        drop(file);
        std::fs::remove_file(path).unwrap();
    }

    #[tokio::test]
    async fn unknown_usage_wrong_model_and_http_failures_latch_actual_transport() {
        for (status, raw) in [
            (
                200,
                response(json!({"prompt_tokens":2,"completion_tokens":3}), "fixture"),
            ),
            (
                200,
                response(
                    json!({"prompt_tokens":2,"completion_tokens":3,"total_tokens":99}),
                    "fixture",
                ),
            ),
            (
                200,
                response(
                    json!({"prompt_tokens":2,"completion_tokens":3,"total_tokens":5}),
                    "foreign",
                ),
            ),
            (429, "{}".into()),
            (200, "{invalid".into()),
        ] {
            let server = Server::new(status, raw, "");
            let (path, file, sink) = fixture();
            let mut adapter =
                MistralAdapter::with_base_url("fixture-key", "fixture", &server.url).unwrap();
            adapter.set_http_observer(sink.clone());
            assert!(call(&adapter, 1).await.is_err());
            assert!(call(&adapter, 2).await.is_err());
            assert_eq!(server.requests.lock().unwrap().len(), 1);
            drop(sink);
            drop(adapter);
            drop(file);
            std::fs::remove_file(path).unwrap();
        }
    }

    #[tokio::test]
    async fn journal_write_failure_prevents_even_the_first_http_send() {
        let server = Server::new(200, "{}".into(), "");
        let (path, file, sink) = fixture();
        sink.state.lock().unwrap().file = File::open(&path).unwrap(); // injected read-only writer
        let mut adapter =
            MistralAdapter::with_base_url("fixture-key", "fixture", &server.url).unwrap();
        adapter.set_http_observer(sink.clone());
        assert!(call(&adapter, 1).await.is_err());
        assert!(call(&adapter, 2).await.is_err());
        assert!(server.requests.lock().unwrap().is_empty());
        drop(sink);
        drop(adapter);
        drop(file);
        std::fs::remove_file(path).unwrap();
    }

    #[tokio::test]
    async fn cancelling_after_http_response_during_pacing_still_latches_stop() {
        let server = Server::new(
            200,
            response(
                json!({"prompt_tokens":2,"completion_tokens":3,"total_tokens":5}),
                "fixture",
            ),
            "x-ratelimit-remaining-tokens-minute: 0\r\nx-ratelimit-limit-tokens-minute: 1000\r\nx-ratelimit-reset-tokens-minute: 2\r\n",
        );
        let (path, file, sink) = fixture();
        let mut adapter =
            MistralAdapter::with_base_url("fixture-key", "fixture", &server.url).unwrap();
        adapter.set_http_observer(sink.clone());
        {
            let generation = call(&adapter, 1);
            tokio::pin!(generation);
            let response_saved = async {
                loop {
                    if std::fs::read_to_string(&path)
                        .unwrap()
                        .contains("\"kind\":\"response\"")
                    {
                        break;
                    }
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
            };
            tokio::select! {
                _ = &mut generation => panic!("call completed before the post-response cancellation control"),
                result = tokio::time::timeout(Duration::from_secs(5), response_saved) => result.unwrap(),
            }
            // Drop while actual response evidence is durable and pacing is
            // still pending, independently of host scheduling latency.
        }
        assert!(call(&adapter, 2).await.is_err());
        assert_eq!(server.requests.lock().unwrap().len(), 1);
        assert!(std::fs::read_to_string(&path).unwrap().contains("aborted"));
        drop(sink);
        drop(adapter);
        drop(file);
        std::fs::remove_file(path).unwrap();
    }

    fn limits_value() -> Value {
        json!({"contract":"reason-cli-normal-v3-http-limits-v1", "max_request_utf8_json_bytes":12000,
            "input_overhead_tokens":512, "max_input_tokens_per_generation":12512,
            "max_output_tokens_per_generation":1024, "max_input_tokens_total":24000,
            "max_output_tokens_total":4096, "max_generations":3, "min_generation_start_interval_ms":20})
    }
    fn parsed_limits(value: &Value) -> HttpLimits {
        HttpLimits::parse(&value.to_string(), true).unwrap()
    }
    fn request_bytes() -> Vec<u8> {
        serde_json::to_vec(&serde_json::to_value(request()).unwrap()).unwrap()
    }
    async fn bounded_call(
        adapter: &MistralAdapter,
        attempt: u32,
    ) -> Result<reasoning_harness_core::ModelResponse, ModelError> {
        let raw = request_bytes();
        with_http_observation_context(
            HttpObservationContext {
                correlation_id: "control".into(),
                logical_attempt: attempt,
                request_sha256: sha256(&raw),
                budget_input: Some(
                    reasoning_harness_providers::http_observation::HttpBudgetInput {
                        request_utf8_json_bytes: raw.len() as u64,
                        max_output_tokens: request().max_tokens,
                    },
                ),
            },
            adapter.generate(request()),
        )
        .await
    }
    fn rows(path: &PathBuf) -> Vec<Value> {
        std::fs::read_to_string(path)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }

    #[test]
    fn native_limits_reject_unknown_duplicate_float_or_weakened_production_values() {
        let base = limits_value();
        assert!(HttpLimits::parse(&base.to_string(), false).is_err());
        let mut production = base.clone();
        production["min_generation_start_interval_ms"] = json!(10500);
        assert!(HttpLimits::parse(&production.to_string(), false).is_ok());
        for (key, value) in [
            ("unknown", json!(1)),
            ("max_request_utf8_json_bytes", json!(12001)),
            ("input_overhead_tokens", json!(511)),
            ("max_output_tokens_total", json!(4097)),
            ("max_generations", json!(25)),
            ("max_input_tokens_total", json!(24000.0)),
            ("min_generation_start_interval_ms", json!(-1)),
        ] {
            let mut changed = base.clone();
            changed[key] = value;
            assert!(HttpLimits::parse(&changed.to_string(), true).is_err());
        }
        let duplicate = base.to_string().replacen('{', "{\"max_generations\":1,", 1);
        assert!(HttpLimits::parse(&duplicate, true).is_err());
    }

    #[tokio::test]
    async fn request_bytes_input_estimate_or_output_reserve_denial_sends_zero_http() {
        let size = request_bytes().len() as u64;
        for (key, cap) in [
            ("max_request_utf8_json_bytes", size - 1),
            ("max_input_tokens_per_generation", size + 511),
            ("max_input_tokens_total", size + 511),
            ("max_output_tokens_per_generation", 9),
            ("max_output_tokens_total", 9),
        ] {
            let mut policy = limits_value();
            policy[key] = json!(cap);
            let server = Server::new(
                200,
                response(
                    json!({"prompt_tokens":2,"completion_tokens":3,"total_tokens":5}),
                    "fixture",
                ),
                "",
            );
            let (path, file, sink) = limited_fixture(Some(parsed_limits(&policy)));
            let mut adapter =
                MistralAdapter::with_base_url("fixture-key", "fixture", &server.url).unwrap();
            adapter.set_http_observer(sink.clone());
            assert!(bounded_call(&adapter, 1).await.is_err());
            assert!(bounded_call(&adapter, 2).await.is_err());
            assert!(server.requests.lock().unwrap().is_empty());
            assert!(rows(&path).iter().any(|r| r["event"]["kind"] == "blocked"));
            drop(adapter);
            drop(sink);
            drop(file);
            std::fs::remove_file(path).unwrap();
        }
    }

    #[tokio::test]
    async fn separate_input_and_output_totals_block_next_send_with_large_combined_reserve() {
        for (key, cap) in [
            ("max_input_tokens_total", request_bytes().len() as u64 + 513),
            ("max_output_tokens_total", 15),
        ] {
            let mut policy = limits_value();
            policy[key] = json!(cap);
            let server = Server::new(
                200,
                response(
                    json!({"prompt_tokens":8,"completion_tokens":6,"total_tokens":14}),
                    "fixture",
                ),
                "",
            );
            let (path, file, sink) = limited_fixture(Some(parsed_limits(&policy)));
            let mut adapter =
                MistralAdapter::with_base_url("fixture-key", "fixture", &server.url).unwrap();
            adapter.set_http_observer(sink.clone());
            bounded_call(&adapter, 1).await.unwrap();
            assert!(bounded_call(&adapter, 2).await.is_err());
            assert_eq!(server.requests.lock().unwrap().len(), 1);
            assert_eq!(
                rows(&path)
                    .iter()
                    .find(|r| r["event"]["kind"] == "response")
                    .unwrap()["event"]["usage"]["total_tokens"],
                14
            );
            drop(adapter);
            drop(sink);
            drop(file);
            std::fs::remove_file(path).unwrap();
        }
    }

    #[tokio::test]
    async fn actual_usage_outside_preflight_or_unknown_latches_without_refund() {
        for usage in [
            json!({"prompt_tokens":request_bytes().len()+513,"completion_tokens":3,"total_tokens":request_bytes().len()+516}),
            json!({"prompt_tokens":2,"completion_tokens":11,"total_tokens":13}),
            json!({"prompt_tokens":2,"total_tokens":5}),
        ] {
            let server = Server::new(200, response(usage.clone(), "fixture"), "");
            let (path, file, sink) = limited_fixture(Some(parsed_limits(&limits_value())));
            let mut adapter =
                MistralAdapter::with_base_url("fixture-key", "fixture", &server.url).unwrap();
            adapter.set_http_observer(sink.clone());
            assert!(bounded_call(&adapter, 1).await.is_err());
            assert!(bounded_call(&adapter, 2).await.is_err());
            assert_eq!(server.requests.lock().unwrap().len(), 1);
            assert!(rows(&path).iter().any(|r| r["event"]["kind"] == "response"));
            drop(adapter);
            drop(sink);
            drop(file);
            std::fs::remove_file(path).unwrap();
        }
    }

    #[tokio::test]
    async fn production_10500ms_start_spacing_is_observed_by_actual_http() {
        let mut policy = limits_value();
        policy["min_generation_start_interval_ms"] = json!(10500);
        let parsed = HttpLimits::parse(&policy.to_string(), false).unwrap();
        let server = Server::new(
            200,
            response(
                json!({"prompt_tokens":2,"completion_tokens":3,"total_tokens":5}),
                "fixture",
            ),
            "",
        );
        let (path, file, sink) = limited_fixture(Some(parsed));
        let mut adapter =
            MistralAdapter::with_base_url("fixture-key", "fixture", &server.url).unwrap();
        adapter.set_http_observer(sink.clone());
        bounded_call(&adapter, 1).await.unwrap();
        bounded_call(&adapter, 2).await.unwrap();
        assert_eq!(server.requests.lock().unwrap().len(), 2);
        let starts: Vec<_> = rows(&path)
            .iter()
            .filter(|r| r["event"]["kind"] == "request")
            .map(|r| r["event"]["start_elapsed_ns"].as_u64().unwrap())
            .collect();
        assert!(starts[1] - starts[0] >= 10_500_000_000);
        assert!(
            rows(&path)
                .iter()
                .all(|r| r["contract"] == "reason-cli-evaluation-http-journal-v2")
        );
        drop(adapter);
        drop(sink);
        drop(file);
        std::fs::remove_file(path).unwrap();
    }

    #[tokio::test]
    async fn cancelling_pre_send_pacing_burns_preflight_and_never_sends_second_request() {
        let mut policy = limits_value();
        policy["min_generation_start_interval_ms"] = json!(2000);
        let server = Server::new(
            200,
            response(
                json!({"prompt_tokens":2,"completion_tokens":3,"total_tokens":5}),
                "fixture",
            ),
            "",
        );
        let (path, file, sink) = limited_fixture(Some(parsed_limits(&policy)));
        let mut adapter =
            MistralAdapter::with_base_url("fixture-key", "fixture", &server.url).unwrap();
        adapter.set_http_observer(sink.clone());
        bounded_call(&adapter, 1).await.unwrap();
        assert!(
            tokio::time::timeout(Duration::from_millis(20), bounded_call(&adapter, 2))
                .await
                .is_err()
        );
        assert!(bounded_call(&adapter, 3).await.is_err());
        assert_eq!(server.requests.lock().unwrap().len(), 1);
        let events = rows(&path);
        assert_eq!(
            events
                .iter()
                .filter(|r| r["event"]["kind"] == "preflight")
                .count(),
            2
        );
        assert!(events.iter().any(|r| r["event"]["kind"] == "aborted"));
        drop(adapter);
        drop(sink);
        drop(file);
        std::fs::remove_file(path).unwrap();
    }

    #[tokio::test]
    async fn generation_ceiling_and_missing_request_measurement_fail_before_send() {
        for missing in [false, true] {
            let mut policy = limits_value();
            policy["max_generations"] = json!(1);
            let server = Server::new(
                200,
                response(
                    json!({"prompt_tokens":2,"completion_tokens":3,"total_tokens":5}),
                    "fixture",
                ),
                "",
            );
            let (path, file, sink) = limited_fixture(Some(parsed_limits(&policy)));
            let mut adapter =
                MistralAdapter::with_base_url("fixture-key", "fixture", &server.url).unwrap();
            adapter.set_http_observer(sink.clone());
            if missing {
                assert!(call(&adapter, 1).await.is_err());
            } else {
                bounded_call(&adapter, 1).await.unwrap();
                assert!(bounded_call(&adapter, 2).await.is_err());
            }
            assert_eq!(server.requests.lock().unwrap().len(), usize::from(!missing));
            drop(adapter);
            drop(sink);
            drop(file);
            std::fs::remove_file(path).unwrap();
        }
    }

    #[tokio::test]
    async fn concurrent_generation_during_reservation_stops_both_before_extra_http() {
        let mut policy = limits_value();
        policy["min_generation_start_interval_ms"] = json!(100);
        let server = Server::new(
            200,
            response(
                json!({"prompt_tokens":2,"completion_tokens":3,"total_tokens":5}),
                "fixture",
            ),
            "",
        );
        let (path, file, sink) = limited_fixture(Some(parsed_limits(&policy)));
        let mut adapter =
            MistralAdapter::with_base_url("fixture-key", "fixture", &server.url).unwrap();
        adapter.set_http_observer(sink.clone());
        bounded_call(&adapter, 1).await.unwrap();
        let (left, right) = tokio::join!(bounded_call(&adapter, 2), bounded_call(&adapter, 3));
        assert!(left.is_err() && right.is_err());
        assert_eq!(server.requests.lock().unwrap().len(), 1);
        drop(adapter);
        drop(sink);
        drop(file);
        std::fs::remove_file(path).unwrap();
    }

    #[tokio::test]
    async fn forged_context_measurements_do_not_match_actual_provider_request() {
        for wrong in ["hash", "bytes", "output"] {
            let server = Server::new(
                200,
                response(
                    json!({"prompt_tokens":2,"completion_tokens":3,"total_tokens":5}),
                    "fixture",
                ),
                "",
            );
            let (path, file, sink) = limited_fixture(Some(parsed_limits(&limits_value())));
            let mut adapter =
                MistralAdapter::with_base_url("fixture-key", "fixture", &server.url).unwrap();
            adapter.set_http_observer(sink.clone());
            let raw = request_bytes();
            let mut context = HttpObservationContext {
                correlation_id: "control".into(),
                logical_attempt: 1,
                request_sha256: sha256(&raw),
                budget_input: Some(
                    reasoning_harness_providers::http_observation::HttpBudgetInput {
                        request_utf8_json_bytes: raw.len() as u64,
                        max_output_tokens: request().max_tokens,
                    },
                ),
            };
            match wrong {
                "hash" => context.request_sha256 = "f".repeat(64),
                "bytes" => {
                    context
                        .budget_input
                        .as_mut()
                        .unwrap()
                        .request_utf8_json_bytes = 1
                }
                _ => context.budget_input.as_mut().unwrap().max_output_tokens = Some(1),
            }
            assert!(
                with_http_observation_context(context, adapter.generate(request()))
                    .await
                    .is_err()
            );
            assert!(bounded_call(&adapter, 2).await.is_err());
            assert!(server.requests.lock().unwrap().is_empty());
            assert!(rows(&path).iter().any(|r| r["event"]["kind"] == "blocked"));
            drop(adapter);
            drop(sink);
            drop(file);
            std::fs::remove_file(path).unwrap();
        }
    }
}
