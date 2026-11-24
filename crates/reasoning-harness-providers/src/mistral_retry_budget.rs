//! Owner-managed development accounting; not the formal HTTP journal.
//! Charge rejected attempts conservatively; never report reserved usage as exact.
use reasoning_harness_core::ModelUsage;
use std::{cell::RefCell, future::Future};

#[derive(Clone, Copy)]
pub(crate) struct Limits {
    pub output: u64,
    pub total: u64,
}
struct Context {
    limits: Option<Limits>,
    result: Option<ModelUsage>,
    refusal: Option<&'static str>,
}
tokio::task_local! {
    static SCOPE: RefCell<Context>;
}

/// Scope one physical-adapter logical request, not a session or global mutable budget.
pub async fn with_limits<T>(
    output: Option<u64>,
    total: Option<u64>,
    future: impl Future<Output = T>,
) -> (T, Option<ModelUsage>, Option<&'static str>) {
    SCOPE
        .scope(
            RefCell::new(Context {
                limits: output
                    .zip(total)
                    .map(|(output, total)| Limits { output, total }),
                result: None,
                refusal: None,
            }),
            async move {
                let result = future.await;
                let (usage, refusal) = SCOPE.with(|c| {
                    let mut c = c.borrow_mut();
                    (c.result.take(), c.refusal.take())
                });
                (result, usage, refusal)
            },
        )
        .await
}

pub(crate) fn limits() -> Option<Limits> {
    SCOPE.try_with(|c| c.borrow().limits).ok().flatten()
}
pub(crate) fn publish(usage: ModelUsage) {
    let _ = SCOPE.try_with(|c| c.borrow_mut().result = Some(usage));
}
pub(crate) fn refuse(reason: &'static str) {
    let _ = SCOPE.try_with(|c| c.borrow_mut().refusal = Some(reason));
}

/// The existing native development admission bound uses UTF-8 JSON bytes + 512.
/// Use the larger of the logical request and actual wire JSON. This is an
/// operational reservation assumption for the pinned model, not a billing fact.
pub(crate) const INPUT_OVERHEAD: u64 = 512;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Tokens {
    pub input: u64,
    pub output: u64,
}
impl Tokens {
    pub fn total(self) -> u64 {
        self.input.saturating_add(self.output)
    }
    pub fn add(self, other: Self) -> Self {
        Self {
            input: self.input.saturating_add(other.input),
            output: self.output.saturating_add(other.output),
        }
    }
    pub fn fits(self, limits: Limits) -> bool {
        self.output <= limits.output && self.total() <= limits.total
    }
    pub fn within(self, reserve: Self) -> bool {
        self.input <= reserve.input && self.output <= reserve.output
    }
    pub fn from_usage(usage: &ModelUsage) -> Option<Self> {
        let t = Self {
            input: usage.input_tokens?,
            output: usage.output_tokens?,
        };
        (t.input.checked_add(t.output)? == usage.total_tokens?).then_some(t)
    }
    pub fn usage(self) -> ModelUsage {
        ModelUsage {
            input_tokens: Some(self.input),
            output_tokens: Some(self.output),
            total_tokens: Some(self.total()),
        }
    }
}
