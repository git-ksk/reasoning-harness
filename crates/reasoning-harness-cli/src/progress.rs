use std::{
    future::Future,
    sync::{
        OnceLock,
        atomic::{AtomicU8, Ordering},
    },
    time::{Duration, Instant},
};

use reasoning_harness_providers::SubprocessCancellation;

use super::{
    CliError, GenerationFailure, GenerationObservation, OutputFormat,
    terminal_presentation::TerminalPresentation,
};

const PROVIDER_WAIT_FIRST_NOTICE: Duration = Duration::from_secs(3);
const PROVIDER_WAIT_REPEAT_NOTICE: Duration = Duration::from_secs(5);
const CANCEL_POLL_INTERVAL: Duration = Duration::from_millis(20);

const CANCELLATION_IDLE: u8 = 0;
const CANCELLATION_ARMING: u8 = 1;
const CANCELLATION_ACTIVE: u8 = 2;
const CANCELLATION_PENDING: u8 = 3;

static CANCELLATION_STATE: AtomicU8 = AtomicU8::new(CANCELLATION_IDLE);
static CANCELLATION: OnceLock<SubprocessCancellation> = OnceLock::new();
static CTRL_C_HANDLER: OnceLock<Result<(), String>> = OnceLock::new();

fn cancellation_token() -> &'static SubprocessCancellation {
    CANCELLATION.get_or_init(SubprocessCancellation::default)
}

fn mark_ctrl_c_for_state(state: &AtomicU8, token: Option<&SubprocessCancellation>) -> bool {
    loop {
        match state.load(Ordering::SeqCst) {
            CANCELLATION_IDLE => return false,
            CANCELLATION_ACTIVE => {
                if let Some(token) = token {
                    token.cancel();
                }
                return true;
            }
            CANCELLATION_ARMING => {
                if state
                    .compare_exchange(
                        CANCELLATION_ARMING,
                        CANCELLATION_PENDING,
                        Ordering::SeqCst,
                        Ordering::SeqCst,
                    )
                    .is_ok()
                {
                    return true;
                }
            }
            CANCELLATION_PENDING => return true,
            _ => return false,
        }
    }
}

fn mark_product_ctrl_c() -> bool {
    mark_ctrl_c_for_state(&CANCELLATION_STATE, CANCELLATION.get())
}

#[cfg(not(windows))]
fn install_platform_ctrl_c_handler() -> Result<(), String> {
    ctrlc::set_handler(|| {
        if !mark_product_ctrl_c() {
            // At an idle prompt there is no in-flight state to clean up. Preserve ordinary
            // terminal Ctrl+C behavior instead of swallowing the signal.
            std::process::exit(130);
        }
    })
    .map_err(|error| error.to_string())
}

#[cfg(windows)]
fn install_platform_ctrl_c_handler() -> Result<(), String> {
    type Handler = Option<unsafe extern "system" fn(u32) -> i32>;

    #[link(name = "Kernel32")]
    unsafe extern "system" {
        fn SetConsoleCtrlHandler(handler: Handler, add: i32) -> i32;
    }

    unsafe extern "system" fn handler(control: u32) -> i32 {
        const CTRL_C_EVENT: u32 = 0;
        const CTRL_BREAK_EVENT: u32 = 1;
        if !matches!(control, CTRL_C_EVENT | CTRL_BREAK_EVENT) {
            return 0;
        }
        if mark_product_ctrl_c() {
            1
        } else {
            // Returning FALSE lets Windows continue to the default console handler, preserving
            // ordinary idle Ctrl+C termination without calling allocation-heavy Rust code here.
            0
        }
    }

    let installed = unsafe { SetConsoleCtrlHandler(Some(handler), 1) };
    if installed == 0 {
        Err(std::io::Error::last_os_error().to_string())
    } else {
        Ok(())
    }
}

fn install_ctrl_c_handler() -> Result<(), CliError> {
    let installed = CTRL_C_HANDLER.get_or_init(install_platform_ctrl_c_handler);
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
        let token = cancellation_token().clone();
        install_ctrl_c_handler()?;
        if CANCELLATION_STATE
            .compare_exchange(
                CANCELLATION_IDLE,
                CANCELLATION_ARMING,
                Ordering::SeqCst,
                Ordering::SeqCst,
            )
            .is_err()
        {
            return Err(CliError::new(
                "cancellation_state",
                "another cancellable Reason operation is already active",
            ));
        }
        token.reset();
        match CANCELLATION_STATE.compare_exchange(
            CANCELLATION_ARMING,
            CANCELLATION_ACTIVE,
            Ordering::SeqCst,
            Ordering::SeqCst,
        ) {
            Ok(_) => {}
            Err(CANCELLATION_PENDING) => {
                token.cancel();
                CANCELLATION_STATE.store(CANCELLATION_ACTIVE, Ordering::SeqCst);
            }
            Err(_) => {
                CANCELLATION_STATE.store(CANCELLATION_IDLE, Ordering::SeqCst);
                return Err(CliError::new(
                    "cancellation_state",
                    "cancellation state changed unexpectedly while arming",
                ));
            }
        }
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
        CANCELLATION_STATE.store(CANCELLATION_IDLE, Ordering::SeqCst);
        self.token.reset();
    }
}

pub(super) struct ProgressReporter {
    enabled: bool,
    verbose: bool,
    started: Instant,
}

impl ProgressReporter {
    pub(super) fn new(format: OutputFormat, verbose: bool, explicit_plain: bool) -> Self {
        Self::with_presentation(
            verbose,
            TerminalPresentation::detect(format, explicit_plain),
        )
    }

    fn with_presentation(verbose: bool, presentation: TerminalPresentation) -> Self {
        Self {
            enabled: presentation.progress_allowed(),
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
    fn ctrl_c_during_arming_is_preserved_after_token_reset() {
        let state = AtomicU8::new(CANCELLATION_ARMING);
        let token = SubprocessCancellation::default();
        assert!(mark_ctrl_c_for_state(&state, Some(&token)));
        assert_eq!(state.load(Ordering::SeqCst), CANCELLATION_PENDING);
        assert!(!token.is_cancelled());

        // begin() resets the reusable token before promoting the pending signal to active.
        token.reset();
        assert_eq!(
            state.compare_exchange(
                CANCELLATION_ARMING,
                CANCELLATION_ACTIVE,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ),
            Err(CANCELLATION_PENDING)
        );
        token.cancel();
        state.store(CANCELLATION_ACTIVE, Ordering::SeqCst);
        assert!(token.is_cancelled());
    }

    #[test]
    fn progress_is_human_full_tty_and_non_plain_only() {
        let normal = TerminalPresentation::from_inputs(
            OutputFormat::Human,
            false,
            true,
            true,
            true,
            false,
            Some("xterm-256color"),
        );
        assert!(ProgressReporter::with_presentation(false, normal).enabled);

        for policy in [
            TerminalPresentation::from_inputs(
                OutputFormat::Json,
                false,
                true,
                true,
                true,
                false,
                Some("xterm-256color"),
            ),
            TerminalPresentation::from_inputs(
                OutputFormat::Human,
                true,
                true,
                true,
                true,
                false,
                Some("xterm-256color"),
            ),
            TerminalPresentation::from_inputs(
                OutputFormat::Human,
                false,
                true,
                true,
                true,
                true,
                Some("xterm-256color"),
            ),
            TerminalPresentation::from_inputs(
                OutputFormat::Human,
                false,
                true,
                true,
                true,
                false,
                Some("dumb"),
            ),
            TerminalPresentation::from_inputs(
                OutputFormat::Human,
                false,
                false,
                true,
                true,
                false,
                Some("xterm-256color"),
            ),
        ] {
            assert!(!ProgressReporter::with_presentation(true, policy).enabled);
        }
    }

    #[test]
    fn plain_mode_preserves_ctrl_c_cancellation_state_machine() {
        let policy = TerminalPresentation::from_inputs(
            OutputFormat::Human,
            true,
            true,
            true,
            true,
            false,
            Some("xterm-256color"),
        );
        assert!(policy.is_plain());
        let state = AtomicU8::new(CANCELLATION_ACTIVE);
        let token = SubprocessCancellation::default();
        assert!(mark_ctrl_c_for_state(&state, Some(&token)));
        assert!(token.is_cancelled());
    }

    #[tokio::test]
    async fn provider_wait_wrapper_preserves_result_when_progress_is_disabled() {
        let policy = TerminalPresentation::from_inputs(
            OutputFormat::Json,
            false,
            true,
            true,
            true,
            false,
            Some("xterm-256color"),
        );
        let reporter = ProgressReporter::with_presentation(true, policy);
        assert_eq!(reporter.wait_for_provider("test", async { 42 }).await, 42);
    }
}
