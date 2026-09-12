use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

/// Product-layer cancellation signal for bounded subprocess-backed resolvers/verifiers.
///
/// This does not change Harness authority or epistemic semantics. Callers decide how a
/// cancellation is surfaced; subprocess adapters only use it to terminate owned children.
#[derive(Debug, Clone, Default)]
pub struct SubprocessCancellation {
    cancelled: Arc<AtomicBool>,
}

impl SubprocessCancellation {
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    pub fn reset(&self) {
        self.cancelled.store(false, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}
