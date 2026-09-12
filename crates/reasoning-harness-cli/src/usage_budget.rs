use reasoning_harness_core::{GroundedResolutionOutcome, ModelUsage};
use serde::{Deserialize, Serialize};

use super::{CliError, GenerationFailure, GenerationObservation, NaturalSafetyObservation};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub(super) struct UsageBudget {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_model_calls: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_total_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_cost_per_million: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_cost_per_million: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pricing_source: Option<String>,
}

impl UsageBudget {
    pub(super) fn validate(&self) -> Result<(), String> {
        for (config_name, flag_name, value) in [
            ("max_model_calls", "max-model-calls", self.max_model_calls),
            (
                "max_output_tokens",
                "max-output-tokens",
                self.max_output_tokens,
            ),
            (
                "max_total_tokens",
                "max-total-tokens",
                self.max_total_tokens,
            ),
        ] {
            if value == Some(0) {
                return Err(format!(
                    "run.{config_name} / --{flag_name} must be at least 1"
                ));
            }
        }
        let prices = (self.input_cost_per_million, self.output_cost_per_million);
        match prices {
            (None, None) => {
                if self.pricing_source.is_some() {
                    return Err("pricing_source requires both input/output cost rates".into());
                }
            }
            (Some(input), Some(output)) => {
                if !input.is_finite() || !output.is_finite() || input < 0.0 || output < 0.0 {
                    return Err(
                        "explicit model pricing rates must be finite and non-negative".into(),
                    );
                }
                if self
                    .pricing_source
                    .as_deref()
                    .is_none_or(|source| source.trim().is_empty())
                {
                    return Err("explicit model pricing requires a non-empty pricing_source".into());
                }
            }
            _ => return Err("model pricing requires both input and output cost rates".into()),
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct UsageTotals {
    pub tracked_from_session_start: bool,
    pub model_calls: u64,
    pub provider_attempts: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_tokens: Option<u64>,
    pub resolution_calls: u64,
    pub resolution_added_tokens: u64,
    pub resolution_elapsed_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution_cost_usd: Option<f64>,
}

impl Default for UsageTotals {
    fn default() -> Self {
        Self {
            tracked_from_session_start: true,
            model_calls: 0,
            provider_attempts: 0,
            input_tokens: Some(0),
            output_tokens: Some(0),
            total_tokens: Some(0),
            resolution_calls: 0,
            resolution_added_tokens: 0,
            resolution_elapsed_ms: 0,
            resolution_cost_usd: Some(0.0),
        }
    }
}

impl UsageTotals {
    pub(super) fn untracked_history() -> Self {
        Self {
            tracked_from_session_start: false,
            input_tokens: None,
            output_tokens: None,
            total_tokens: None,
            resolution_cost_usd: None,
            ..Self::default()
        }
    }

    pub(super) fn add(&self, other: &Self) -> Self {
        Self {
            tracked_from_session_start: self.tracked_from_session_start
                && other.tracked_from_session_start,
            model_calls: self.model_calls.saturating_add(other.model_calls),
            provider_attempts: self
                .provider_attempts
                .saturating_add(other.provider_attempts),
            input_tokens: add_optional(self.input_tokens, other.input_tokens),
            output_tokens: add_optional(self.output_tokens, other.output_tokens),
            total_tokens: add_optional(self.total_tokens, other.total_tokens),
            resolution_calls: self.resolution_calls.saturating_add(other.resolution_calls),
            resolution_added_tokens: self
                .resolution_added_tokens
                .saturating_add(other.resolution_added_tokens),
            resolution_elapsed_ms: self
                .resolution_elapsed_ms
                .saturating_add(other.resolution_elapsed_ms),
            resolution_cost_usd: add_optional_f64(
                self.resolution_cost_usd,
                other.resolution_cost_usd,
            ),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct UsageReport {
    pub run: UsageTotals,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<UsageTotals>,
    pub budget: UsageBudget,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_model_cost_usd: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pricing_source: Option<String>,
}

pub(super) struct UsageTracker {
    budget: UsageBudget,
    baseline: Option<UsageTotals>,
    run: UsageTotals,
}

impl UsageTracker {
    pub(super) fn new(
        budget: UsageBudget,
        baseline: Option<UsageTotals>,
    ) -> Result<Self, CliError> {
        budget
            .validate()
            .map_err(|message| CliError::new("configuration", message))?;
        let tracker = Self {
            budget,
            baseline,
            run: UsageTotals::default(),
        };
        tracker.check_current_limits()?;
        Ok(tracker)
    }

    fn combined(&self) -> UsageTotals {
        self.baseline
            .as_ref()
            .map(|baseline| baseline.add(&self.run))
            .unwrap_or_else(|| self.run.clone())
    }

    pub(super) fn cap_next_model_tokens(&self, requested: u32) -> Result<u32, CliError> {
        let combined = self.combined();
        if self.budget.max_model_calls.is_some() && !combined.tracked_from_session_start {
            return Err(budget_unmeasurable(
                "managed-session model-call history predates usage tracking; start a new session to enforce a cumulative model-call ceiling",
            ));
        }
        if let Some(max_calls) = self.budget.max_model_calls
            && combined.model_calls >= max_calls
        {
            return Err(budget_exceeded(format!(
                "model call budget exhausted ({}/{max_calls})",
                combined.model_calls
            )));
        }
        let Some(max_output) = self.budget.max_output_tokens else {
            return Ok(requested);
        };
        let Some(consumed) = combined.output_tokens else {
            return Err(budget_unmeasurable(
                "provider did not report output-token usage required by the configured cumulative output-token ceiling",
            ));
        };
        if consumed >= max_output {
            return Err(budget_exceeded(format!(
                "output-token budget exhausted ({consumed}/{max_output})"
            )));
        }
        let remaining = max_output - consumed;
        Ok(requested.min(u32::try_from(remaining).unwrap_or(u32::MAX)))
    }

    pub(super) fn record_generation(
        &mut self,
        observation: &GenerationObservation,
    ) -> Result<(), CliError> {
        self.run.model_calls = self.run.model_calls.saturating_add(1);
        self.run.provider_attempts = self
            .run
            .provider_attempts
            .saturating_add(u64::from(observation.provider_attempts));
        add_model_usage(&mut self.run, &observation.usage);
        self.check_current_limits()
    }

    pub(super) fn record_failure(&mut self, failure: &GenerationFailure) -> Result<(), CliError> {
        self.run.model_calls = self.run.model_calls.saturating_add(1);
        self.run.provider_attempts = self
            .run
            .provider_attempts
            .saturating_add(u64::from(failure.provider_attempts));
        if failure.provider_attempts > 0 {
            self.run.input_tokens = None;
            self.run.output_tokens = None;
            self.run.total_tokens = None;
        }
        self.check_current_limits()
    }

    pub(super) fn record_safety(
        &mut self,
        observation: &NaturalSafetyObservation,
    ) -> Result<(), CliError> {
        let Some(sufficiency) = observation.observation.sufficiency.as_ref() else {
            return Ok(());
        };
        self.run.model_calls = self.run.model_calls.saturating_add(1);
        self.run.provider_attempts = self
            .run
            .provider_attempts
            .saturating_add(u64::from(sufficiency.provider_attempts));
        add_model_usage(&mut self.run, &sufficiency.usage);
        self.check_current_limits()
    }

    pub(super) fn record_resolution(&mut self, round: &GroundedResolutionOutcome) {
        self.run.resolution_calls = self.run.resolution_calls.saturating_add(round.usage.calls);
        self.run.resolution_added_tokens = self
            .run
            .resolution_added_tokens
            .saturating_add(round.usage.added_tokens);
        self.run.resolution_elapsed_ms = self
            .run
            .resolution_elapsed_ms
            .saturating_add(round.usage.elapsed_ms);
        self.run.resolution_cost_usd =
            match (self.run.resolution_cost_usd, round.usage.cost_microusd) {
                (Some(current), Some(micro)) => Some(current + micro as f64 / 1_000_000.0),
                (_, None) => None,
                (None, Some(_)) => None,
            };
    }

    fn check_current_limits(&self) -> Result<(), CliError> {
        let combined = self.combined();
        if self.budget.max_model_calls.is_some() && !combined.tracked_from_session_start {
            return Err(budget_unmeasurable(
                "managed-session model-call history predates usage tracking; start a new session to enforce a cumulative model-call ceiling",
            ));
        }
        if let Some(max) = self.budget.max_model_calls
            && combined.model_calls > max
        {
            return Err(budget_exceeded(format!(
                "model call budget exceeded ({}/{max})",
                combined.model_calls
            )));
        }
        if let Some(max) = self.budget.max_output_tokens {
            let Some(actual) = combined.output_tokens else {
                return Err(budget_unmeasurable(
                    "provider did not report output-token usage required by the configured cumulative output-token ceiling",
                ));
            };
            if actual > max {
                return Err(budget_exceeded(format!(
                    "output-token budget exceeded ({actual}/{max})"
                )));
            }
        }
        if let Some(max) = self.budget.max_total_tokens {
            let Some(actual) = combined.total_tokens else {
                return Err(budget_unmeasurable(
                    "provider did not report total-token usage required by the configured total-token ceiling",
                ));
            };
            if actual > max {
                return Err(budget_exceeded(format!(
                    "total-token budget exceeded ({actual}/{max})"
                )));
            }
        }
        Ok(())
    }

    pub(super) fn report(&self) -> UsageReport {
        let session = self
            .baseline
            .as_ref()
            .map(|baseline| baseline.add(&self.run));
        let priced = session.as_ref().unwrap_or(&self.run);
        let estimated_model_cost_usd = match (
            self.budget.input_cost_per_million,
            self.budget.output_cost_per_million,
            priced.input_tokens,
            priced.output_tokens,
        ) {
            (Some(input_rate), Some(output_rate), Some(input), Some(output)) => Some(
                input as f64 * input_rate / 1_000_000.0 + output as f64 * output_rate / 1_000_000.0,
            ),
            _ => None,
        };
        UsageReport {
            run: self.run.clone(),
            session,
            budget: self.budget.clone(),
            estimated_model_cost_usd,
            pricing_source: self.budget.pricing_source.clone(),
        }
    }
}

fn add_model_usage(totals: &mut UsageTotals, usage: &ModelUsage) {
    totals.input_tokens = add_optional(totals.input_tokens, usage.input_tokens);
    totals.output_tokens = add_optional(totals.output_tokens, usage.output_tokens);
    totals.total_tokens = add_optional(totals.total_tokens, usage.total_tokens);
}

fn add_optional(left: Option<u64>, right: Option<u64>) -> Option<u64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.saturating_add(right)),
        _ => None,
    }
}

fn add_optional_f64(left: Option<f64>, right: Option<f64>) -> Option<f64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left + right),
        _ => None,
    }
}

fn budget_exceeded(message: String) -> CliError {
    CliError::new("usage_budget_exceeded", message)
}

fn budget_unmeasurable(message: &'static str) -> CliError {
    CliError::new("usage_budget_unmeasurable", message)
}

pub(super) fn print_human(report: &UsageReport) {
    let totals = report.session.as_ref().unwrap_or(&report.run);
    print_totals_human(
        totals,
        report.estimated_model_cost_usd,
        report.pricing_source.as_deref(),
    );
}

pub(super) fn print_totals_human(
    totals: &UsageTotals,
    estimated_model_cost_usd: Option<f64>,
    pricing_source: Option<&str>,
) {
    println!("\nUsage");
    println!(
        "- model calls={} | provider attempts={} | input tokens={} | output tokens={} | total tokens={}",
        totals.model_calls,
        totals.provider_attempts,
        display_optional(totals.input_tokens),
        display_optional(totals.output_tokens),
        display_optional(totals.total_tokens),
    );
    if !totals.tracked_from_session_start {
        println!(
            "- note: persisted usage history is incomplete for turns created before usage tracking"
        );
    }
    if totals.resolution_calls > 0 {
        println!(
            "- resolver calls={} | added tokens={} | elapsed={}ms | reported external cost={}",
            totals.resolution_calls,
            totals.resolution_added_tokens,
            totals.resolution_elapsed_ms,
            totals
                .resolution_cost_usd
                .map(|cost| format!("${cost:.6}"))
                .unwrap_or_else(|| "unavailable".into())
        );
    }
    if let (Some(cost), Some(source)) = (estimated_model_cost_usd, pricing_source) {
        println!("- estimated model cost=${cost:.6} (pricing source: {source})");
    }
}

fn display_optional(value: Option<u64>) -> String {
    value.map_or_else(|| "unreported".into(), |value| value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn observation(output: Option<u64>, total: Option<u64>) -> GenerationObservation {
        GenerationObservation {
            provider: "fixture",
            model: "fixture".into(),
            usage: ModelUsage {
                input_tokens: total.zip(output).map(|(total, output)| total - output),
                output_tokens: output,
                total_tokens: total,
            },
            latency_ms: 1,
            provider_attempts: 2,
            cost_usd: None,
        }
    }

    #[test]
    fn model_call_budget_is_checked_before_next_call() {
        let mut tracker = UsageTracker::new(
            UsageBudget {
                max_model_calls: Some(1),
                ..Default::default()
            },
            None,
        )
        .unwrap();
        tracker
            .record_generation(&observation(Some(2), Some(5)))
            .unwrap();
        assert_eq!(
            tracker
                .cap_next_model_tokens(100)
                .unwrap_err()
                .failure_class,
            "usage_budget_exceeded"
        );
    }

    #[test]
    fn output_budget_caps_next_request() {
        let mut tracker = UsageTracker::new(
            UsageBudget {
                max_output_tokens: Some(10),
                ..Default::default()
            },
            None,
        )
        .unwrap();
        tracker
            .record_generation(&observation(Some(6), Some(9)))
            .unwrap();
        assert_eq!(tracker.cap_next_model_tokens(100).unwrap(), 4);
    }

    #[test]
    fn strict_token_budget_fails_closed_when_provider_usage_is_missing() {
        let mut tracker = UsageTracker::new(
            UsageBudget {
                max_total_tokens: Some(100),
                ..Default::default()
            },
            None,
        )
        .unwrap();
        let error = tracker
            .record_generation(&observation(None, None))
            .unwrap_err();
        assert_eq!(error.failure_class, "usage_budget_unmeasurable");
    }

    #[test]
    fn failed_provider_attempt_makes_token_totals_unreported() {
        let mut tracker = UsageTracker::new(UsageBudget::default(), None).unwrap();
        tracker
            .record_generation(&observation(Some(20), Some(100)))
            .unwrap();
        tracker
            .record_failure(&GenerationFailure {
                provider: "fixture",
                model: "fixture".into(),
                latency_ms: 1,
                provider_attempts: 2,
                failure_class: "rate_limit",
                message: "fixture".into(),
            })
            .unwrap();
        let report = tracker.report();
        assert_eq!(report.run.input_tokens, None);
        assert_eq!(report.run.output_tokens, None);
        assert_eq!(report.run.total_tokens, None);
        assert_eq!(report.run.provider_attempts, 4);
    }

    #[test]
    fn cumulative_guard_rejects_untracked_pre_usage_session_history() {
        let error = match UsageTracker::new(
            UsageBudget {
                max_model_calls: Some(10),
                ..Default::default()
            },
            Some(UsageTotals::untracked_history()),
        ) {
            Ok(_) => panic!("untracked session history must not satisfy a cumulative guard"),
            Err(error) => error,
        };
        assert_eq!(error.failure_class, "usage_budget_unmeasurable");
    }

    #[test]
    fn usage_report_json_has_stable_provider_neutral_fields() {
        let report = UsageTracker::new(UsageBudget::default(), None)
            .unwrap()
            .report();
        let json = serde_json::to_value(report).unwrap();
        assert!(json["run"].get("model_calls").is_some());
        assert!(json["run"].get("provider_attempts").is_some());
        assert!(json["run"].get("resolution_calls").is_some());
        assert!(json.get("budget").is_some());
        assert!(json.get("estimated_model_cost_usd").is_none());
    }

    #[test]
    fn pricing_is_estimated_only_with_explicit_provenance() {
        let mut tracker = UsageTracker::new(
            UsageBudget {
                input_cost_per_million: Some(1.0),
                output_cost_per_million: Some(2.0),
                pricing_source: Some("operator-2026-09-12".into()),
                ..Default::default()
            },
            None,
        )
        .unwrap();
        tracker
            .record_generation(&observation(Some(20), Some(100)))
            .unwrap();
        let report = tracker.report();
        assert_eq!(
            report.pricing_source.as_deref(),
            Some("operator-2026-09-12")
        );
        assert!(report.estimated_model_cost_usd.unwrap() > 0.0);
    }
}
