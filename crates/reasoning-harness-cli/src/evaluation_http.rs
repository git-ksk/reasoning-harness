//! Private evaluation sink. The operator passes an already-created private FD;
//! no path is chosen from model output, receipts, user config or environment.
use std::{collections::BTreeSet, fs::File, io::Write, sync::Mutex};

use reasoning_harness_core::ModelError;
use reasoning_harness_providers::http_observation::{
    HttpObservation, HttpObservationSink, observation_error, sha256,
};
use serde_json::{Value, json};

pub(crate) struct EvaluationHttpJournal {
    binding: String,
    max_attempts: u32,
    reserve: u64,
    state: Mutex<State>,
}

struct State {
    file: File,
    sequence: u64,
    previous: String,
    attempts: u32,
    used: u64,
    pending: Option<String>,
    finalizing: Option<String>,
    seen: BTreeSet<String>,
    halted: bool,
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
        Self::new(file, binding, max_attempts, reserve)
    }

    #[cfg(not(unix))]
    pub fn from_fd(
        _fd: i32,
        _binding: &str,
        _max_attempts: u32,
        _reserve: u64,
    ) -> Result<Self, ModelError> {
        Err(observation_error())
    }

    fn new(file: File, binding: &str, max_attempts: u32, reserve: u64) -> Result<Self, ModelError> {
        let sink = Self {
            binding: binding.to_owned(),
            max_attempts,
            reserve,
            state: Mutex::new(State {
                file,
                sequence: 0,
                previous: "0".repeat(64),
                attempts: 0,
                used: 0,
                pending: None,
                finalizing: None,
                seen: BTreeSet::new(),
                halted: false,
            }),
        };
        {
            let mut state = sink.state.lock().map_err(|_| observation_error())?;
            sink.append(
                &mut state,
                json!({"kind":"binding", "max_attempts":max_attempts, "token_reserve":reserve}),
            )?;
        }
        Ok(sink)
    }

    fn append(&self, state: &mut State, event: Value) -> Result<(), ModelError> {
        let row = json!({"contract":"reason-cli-evaluation-http-journal-v1", "sequence":state.sequence,
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
}

impl HttpObservationSink for EvaluationHttpJournal {
    fn record(&self, event: HttpObservation) -> Result<(), ModelError> {
        let mut state = self.state.lock().map_err(|_| observation_error())?;
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
                    || !state.seen.insert(request_id.clone())
                {
                    state.halted = true;
                    return Err(observation_error());
                }
                state.attempts += 1;
                state.pending = Some(request_id.clone());
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
                state.halted |= !(200..300).contains(http_status)
                    || !is_sha256(response_sha256)
                    || failure_class.is_some()
                    || tokens.is_none_or(|total| total > self.reserve);
                if let Some(tokens) = tokens {
                    state.used = tokens;
                }
                state.pending = None;
                state.finalizing = Some(request_id.clone());
            }
            HttpObservation::Aborted { request_id } => {
                if state.pending.as_ref() != Some(request_id)
                    && state.finalizing.as_ref() != Some(request_id)
                {
                    state.halted = true;
                    return Err(observation_error());
                }
                state.halted = true;
                state.pending = None;
                state.finalizing = None;
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
        let value = serde_json::to_value(event).map_err(|_| observation_error())?;
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
            atomic::{AtomicBool, Ordering},
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
        let path = std::env::temp_dir().join(format!(
            "reason-http-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&path)
            .unwrap();
        let sink = Arc::new(
            EvaluationHttpJournal::from_fd(file.as_raw_fd(), &"a".repeat(64), 3, 20).unwrap(),
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
        assert!(
            tokio::time::timeout(Duration::from_millis(200), call(&adapter, 1))
                .await
                .is_err()
        );
        assert!(call(&adapter, 2).await.is_err());
        assert_eq!(server.requests.lock().unwrap().len(), 1);
        assert!(std::fs::read_to_string(&path).unwrap().contains("aborted"));
        drop(sink);
        drop(adapter);
        drop(file);
        std::fs::remove_file(path).unwrap();
    }
}
