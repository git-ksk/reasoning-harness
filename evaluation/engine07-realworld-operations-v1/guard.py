"""Provider-arm control for the next frozen runner. No network calls or retries."""
from dataclasses import dataclass, asdict
from enum import Enum

class FailureClass(str, Enum):
    QUOTA = 'quota'
    RATE_LIMIT = 'rate_limit'
    PROVIDER_UNAVAILABLE = 'provider_unavailable'
    TIMEOUT = 'timeout'
    WALL_TIMEOUT = 'wall_timeout'
    USAGE_BUDGET_EXCEEDED = 'usage_budget_exceeded'
    USAGE_BUDGET_UNMEASURABLE = 'usage_budget_unmeasurable'
    CANCELLED = 'cancelled'
    CREDENTIALS = 'credentials'
    TRANSPORT = 'transport'
    PROVIDER_ERROR = 'provider_error'
    PROTOCOL = 'protocol'
    UNSUPPORTED_CAPABILITY = 'unsupported_capability'
    UNKNOWN = 'unknown_failure'

CAPACITY_FAILURES = frozenset((FailureClass.RATE_LIMIT, FailureClass.PROVIDER_UNAVAILABLE,
                               FailureClass.TIMEOUT, FailureClass.WALL_TIMEOUT))

@dataclass(frozen=True)
class Outcome:
    failure_class: FailureClass | None
    tokens: int | None
    provider_attempts: int | None

@dataclass(frozen=True)
class Decision:
    allowed: bool
    reason: str | None

class ProviderArmGuard:
    def __init__(self, *, token_budget, known_quota_exhausted=False):
        if not isinstance(token_budget, int) or isinstance(token_budget, bool) or token_budget <= 0:
            raise ValueError('freeze a positive provider-arm self-budget')
        self.token_budget = token_budget
        self.observed_tokens = 0
        self.started_invocations = 0
        self.consecutive_capacity_failures = 0
        self.latch = 'known_daily_quota_exhausted' if known_quota_exhausted else None
        self.events = []
        self.inflight = False

    def before_invocation(self, token_reserve):
        if self.inflight:
            raise RuntimeError('serialize all normal/helper invocations in the same provider/model arm')
        if not isinstance(token_reserve, int) or isinstance(token_reserve, bool) or token_reserve <= 0:
            raise ValueError('a conservative next-invocation token reserve is required')
        if self.latch is None and self.observed_tokens + token_reserve > self.token_budget:
            self.latch = 'arm_token_budget_exhausted'
        decision = Decision(self.latch is None, self.latch)
        if decision.allowed:
            self.started_invocations += 1
            self.inflight = True
        self.events.append({'event': 'admission', 'decision': asdict(decision), 'started_invocations': self.started_invocations})
        return decision

    def observe(self, outcome):
        if not self.inflight:
            raise RuntimeError('no admitted invocation to account')
        self.inflight = False
        if outcome.failure_class is not None and (
            not isinstance(outcome.failure_class, FailureClass) or outcome.failure_class is FailureClass.UNKNOWN
        ):
            self.latch = 'unknown_failure_class'
        if outcome.failure_class is FailureClass.QUOTA:
            self.latch = 'typed_quota'
        elif outcome.failure_class is FailureClass.CANCELLED:
            self.latch = 'cancelled_operation'
        capacity = isinstance(outcome.failure_class, FailureClass) and outcome.failure_class in CAPACITY_FAILURES
        self.consecutive_capacity_failures = self.consecutive_capacity_failures + 1 if capacity else 0
        if self.consecutive_capacity_failures >= 2 and self.latch is None:
            self.latch = 'two_consecutive_capacity_failures'
        if outcome.tokens is None:
            # Unknown billed tokens are never zero and never refundable capacity.
            self.latch = self.latch or 'usage_unavailable'
        elif not isinstance(outcome.tokens, int) or isinstance(outcome.tokens, bool) or outcome.tokens < 0:
            self.latch = self.latch or 'invalid_usage'
        else:
            self.observed_tokens += outcome.tokens
            if self.observed_tokens > self.token_budget:
                self.latch = self.latch or 'arm_token_budget_exhausted'
        if outcome.provider_attempts is None:
            self.latch = self.latch or 'attempt_accounting_unavailable'
        elif not isinstance(outcome.provider_attempts, int) or isinstance(outcome.provider_attempts, bool) or outcome.provider_attempts < 0:
            self.latch = self.latch or 'invalid_attempt_accounting'
        self.events.append({'event': 'observation', 'outcome': asdict(outcome), 'observed_tokens': self.observed_tokens, 'latch': self.latch})

    def invoke(self, token_reserve, operation):
        decision = self.before_invocation(token_reserve)
        if not decision.allowed:
            return decision, None
        try:
            outcome, value = operation()
            if not isinstance(outcome, Outcome):
                raise TypeError('operation must preserve a typed Outcome')
            self.observe(outcome)
            return decision, value
        except BaseException:
            # A host exception can leave a billed request in flight. Never start
            # another request or claim that no tokens were consumed.
            self.inflight = False
            self.latch = self.latch or 'interrupted_accounting'
            self.events.append({'event': 'interrupted', 'latch': self.latch})
            raise

    def snapshot(self):
        return {'token_budget': self.token_budget, 'observed_tokens': self.observed_tokens,
                'started_invocations': self.started_invocations, 'latch': self.latch,
                'inflight': self.inflight, 'events': self.events}
