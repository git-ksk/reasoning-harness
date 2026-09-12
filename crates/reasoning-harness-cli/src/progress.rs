use std::{
    future::Future,
    io::{self, IsTerminal},
    process,
    sync::{
        OnceLock,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use reasoning_harness_providers::SubprocessCancellation;

use super::{CliError, GenerationFailure, GenerationObservation, OutputFormat};

const PROVIDER_WAIT_FIRST_NOTICE: Duration = Duration::from_secs(3);
const PROVIDER_WAIT_REPEAT_NOTICE: Duration = Duration::from_secs(5);
const CANCEL_POLL_INTERVAL: Duration = Duration::from_millis(20);

static CANCELLATION_ACTIVE: AtomicBool = AtomicBool::new(false);
static CANCELLATION: OnceLock<SubprocessCancellation> = OnceLock::new();
static CTRL_C_HANDLER: OnceLock<Result<(), String>> = OnceLock::new();

fn cancellation_token() -> &'static SubprocessCancellation {
    CANCELLATION.get_or_init(SubprocessCancellation::default)
}

fn install_ctrl_c_handler() -> Result<(), CliError> {
    let installed = CTRL_C_HANDLER.get_or_init(|| {
        ctrlc::set_handler(|| {
            if CANCELLATION_ACTIVE.load(Ordering::SeqCst) {
                cancellation_token().cancel();
            } else {
                // At an idle prompt there is no in-flight product mutation to clean up. Preserve
                // ordinary terminal Ctrl+C behavior instead of leaving the user at a stuck prompt.
                process::exit(130);
            }
        })
        .map_err(|error| error.to_string())
    });
    match installed {
        Ok(()) => Ok(()),
        Err(message) => Err(CliError::new(
            "cancellation_setup",
            format!("cannot install Ctrl+C cancellation handler: {message}"),
        )),
    }
}

pub(super) struct CancellationRun {
    token: SubprocessCancellation,
    started: Instant,
}

impl CancellationRun {
    pub(super) fn begin() -> Result<Self, CliError> {
        install_ctrl_c_handler()?;
        if CANCELLATION_ACTIVE.swap(true, Ordering::SeqCst) {
            return Err(CliError::new(
                "cancellation_state",
                "another cancellable Reason operation is already active",
            ));
        }
        let token = cancellation_token().clone();
        token.reset();
        Ok(Self {
            token,
            started: Instant::now(),
        })
    }

    pub(super) fn subprocess_token(&self) -> SubprocessCancellation {
        self.token.clone()
    }

    pub(super) fn is_cancelled(&self) -> bool {
        self.token.is_cancelled()
    }

    pub(super) async fn wait_cancelled(&self) {
        while !self.is_cancelled() {
            tokio::time::sleep(CANCEL_POLL_INTERVAL).await;
        }
    }

    #[cfg(test)]
    pub(super) fn test_instance() -> Self {
        Self {
            token: SubprocessCancellation::default(),
            started: Instant::now(),
        }
    }

    #[cfg(test)]
    pub(super) fn cancel_for_test(&self) {
        self.token.cancel();
    }

    pub(super) fn error(&self) -> CliError {
        CliError::new(
            "cancelled",
            format!(
                "cancelled by Ctrl+C after {:.1}s; no incomplete turn or managed-session checkpoint was committed",
                self.started.elapsed().as_secs_f64()
            ),
        )
    }
}

impl Drop for CancellationRun {
    fn drop(&mut self) {
        CANCELLATION_ACTIVE.store(false, Ordering::SeqCst);
        self.token.reset();
    }
}

pub(super) struct ProgressReporter {
    enabled: bool,
    verbose: bool,
    started: Instant,
}

impl ProgressReporter {
    pub(super) fn new(format: OutputFormat, verbose: bool) -> Self {
        Self::with_terminals(
            format,
            verbose,
            io::stdin().is_terminal(),
            io::stdout().is_terminal(),
            io::stderr().is_terminal(),
        )
    }

    fn with_terminals(
        format: OutputFormat,
        verbose: bool,
        stdin: bool,
        stdout: bool,
        stderr: bool,
    ) -> Self {
        Self {
            enabled: format == OutputFormat::Human && stdin && stdout && stderr,
            verbose,
            started: Instant::now(),
        }
    }

    pub(super) fn phase(&self, phase: &str, detail: &str) {
        if self.enabled {
            eprintln!(
                "[{:.1}s] {phase}: {detail}",
                self.started.elapsed().as_secs_f64()
            );
        }
    }

    pub(super) async fn wait_for_provider<T, F>(&self, detail: &str, future: F) -> T
    where
        F: Future<Output = T>,
    {
        if !self.enabled {
            return future.await;
        }
        tokio::pin!(future);
        let notice = tokio::time::sleep(PROVIDER_WAIT_FIRST_NOTICE);
        tokio::pin!(notice);
        loop {
            tokio::select! {
                output = &mut future => return output,
                () = &mut notice => {
                    eprintln!(
                        "[{:.1}s] Waiting: {detail}; provider-side bounded retries may be active",
                        self.started.elapsed().as_secs_f64()
                    );
                    notice.as_mut().reset(tokio::time::Instant::now() + PROVIDER_WAIT_REPEAT_NOTICE);
                }
            }
        }
    }

    pub(super) fn provider(&self, observation: &GenerationObservation) {
        if !self.enabled {
            return;
        }
        if observation.provider_attempts > 1 {
            eprintln!(
                "[{:.1}s] Provider retry: {} completed after {} attempts",
                self.started.elapsed().as_secs_f64(),
                observation.provider,
                observation.provider_attempts
            );
        } else if self.verbose {
            eprintln!(
                "[{:.1}s] Provider: {} / {} attempt={} latency_ms={}",
                self.started.elapsed().as_secs_f64(),
                observation.provider,
                observation.model,
                observation.provider_attempts,
                observation.latency_ms
            );
        }
    }

    pub(super) fn provider_attempts(&self, provider: &str, attempts: u32) {
        if !self.enabled {
            return;
        }
        if attempts > 1 {
            eprintln!(
                "[{:.1}s] Provider retry: {provider} completed after {attempts} attempts",
                self.started.elapsed().as_secs_f64()
            );
        } else if self.verbose {
            eprintln!(
                "[{:.1}s] Provider: {provider} attempt={attempts}",
                self.started.elapsed().as_secs_f64()
            );
        }
    }

    pub(super) fn failure(&self, failure: &GenerationFailure) {
        if self.enabled {
            eprintln!(
                "[{:.1}s] Provider stopped: class={} attempts={}",
                self.started.elapsed().as_secs_f64(),
                failure.failure_class,
                failure.provider_attempts
            );
        }
    }

    pub(super) fn done(&self) {
        if self.enabled {
            eprintln!("[{:.1}s] Complete", self.started.elapsed().as_secs_f64());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_is_human_full_tty_only() {
        assert!(
            ProgressReporter::with_terminals(OutputFormat::Human, false, true, true, true).enabled
        );
        assert!(
            !ProgressReporter::with_terminals(OutputFormat::Json, true, true, true, true).enabled
        );
        assert!(
            !ProgressReporter::with_terminals(OutputFormat::Human, true, false, true, true).enabled
        );
        assert!(
            !ProgressReporter::with_terminals(OutputFormat::Human, true, true, false, true).enabled
        );
    }

    #[tokio::test]
    async fn provider_wait_wrapper_preserves_result_when_progress_is_disabled() {
        let reporter = ProgressReporter::with_terminals(OutputFormat::Json, true, true, true, true);
        assert_eq!(reporter.wait_for_provider("test", async { 42 }).await, 42);
    }
}
