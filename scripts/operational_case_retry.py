"""Bounded case-level retry for provider-operational failures in paired evaluation.

This module deliberately does not know about evaluator scores or answer quality.  It
only inspects typed provider generation failures emitted by the product CLI, so an
already scorable semantic observation can never be re-sampled by this policy.
"""

from __future__ import annotations

from dataclasses import dataclass
import hashlib
import json
from typing import Any, Callable, Sequence

DEFAULT_MAX_CASE_ATTEMPTS = 2
_RETRYABLE_PROVIDER_CLASSES = frozenset({"transport", "provider_unavailable", "timeout"})
_GOOGLE_EMPTY_TEXT_PROTOCOL = "Gemini Interactions response contained no model text output"


@dataclass(frozen=True)
class CaseAttemptResult:
    returncode: int
    payload: Any


@dataclass(frozen=True)
class ProviderFailure:
    source: str
    failure_class: str
    message: str
    subtype: str


@dataclass(frozen=True)
class CaseRetryRecord:
    attempt: int
    command_fingerprint: str
    failure_source: str | None
    failure_class: str | None
    failure_subtype: str | None
    failure_message: str | None
    retryable_operational_failure: bool
    canonical: bool


@dataclass(frozen=True)
class CaseRetryOutcome:
    result: CaseAttemptResult
    records: tuple[CaseRetryRecord, ...]
    canonical_attempt: int
    retry_exhausted: bool


def command_fingerprint(command: Sequence[str]) -> str:
    encoded = json.dumps(list(command), ensure_ascii=False, separators=(",", ":")).encode()
    return hashlib.sha256(encoded).hexdigest()


def _mapping(value: Any) -> dict[str, Any] | None:
    return value if isinstance(value, dict) else None


def provider_failure(payload: Any) -> ProviderFailure | None:
    """Return only product-provider generation failures, never evaluator semantics.

    Natural-language commands expose provider generation failures in two compatible
    locations across the released v0.4.1 control and current candidate:
    * a top-level product failure envelope for generation that cannot start; or
    * ``result.investigation.generation_failure`` after initial generation succeeds.

    Tool/action operational failures and rendering fallbacks are intentionally not
    considered here.
    """

    envelope = _mapping(payload)
    result = _mapping(envelope.get("result")) if envelope else None
    if not result:
        return None

    investigation = _mapping(result.get("investigation"))
    generation = _mapping(investigation.get("generation_failure")) if investigation else None
    if generation:
        failure_class = generation.get("failure_class")
        message = generation.get("message")
        provider = generation.get("provider")
        if isinstance(failure_class, str) and isinstance(message, str) and isinstance(provider, str) and provider:
            subtype = (
                "google_empty_model_text"
                if failure_class == "protocol" and _GOOGLE_EMPTY_TEXT_PROTOCOL in message
                else "typed_provider_generation"
            )
            return ProviderFailure("investigation_generation", failure_class, message, subtype)

    failure = _mapping(result.get("failure"))
    if not failure:
        return None
    failure_class = failure.get("failure_class")
    message = failure.get("message")
    if not isinstance(failure_class, str) or not isinstance(message, str):
        return None

    # Released v0.4.1's top-level provider GenerationFailure is flattened into the
    # product failure message by format_generation_failure(). Requiring this prefix
    # prevents unrelated CLI/input/protocol failures from entering provider retry.
    provider_generation = message.startswith("provider=")
    empty_text = failure_class == "protocol" and _GOOGLE_EMPTY_TEXT_PROTOCOL in message
    if not provider_generation and not empty_text:
        return None
    subtype = "google_empty_model_text" if empty_text else "typed_provider_generation"
    return ProviderFailure("product_failure", failure_class, message, subtype)


def is_retryable_provider_failure(failure: ProviderFailure | None) -> bool:
    if failure is None:
        return False
    if failure.failure_class in _RETRYABLE_PROVIDER_CLASSES:
        return True
    return failure.failure_class == "protocol" and failure.subtype == "google_empty_model_text"


def run_case_with_operational_retry(
    command: Sequence[str],
    execute: Callable[[tuple[str, ...]], CaseAttemptResult],
    *,
    max_attempts: int = DEFAULT_MAX_CASE_ATTEMPTS,
) -> CaseRetryOutcome:
    """Execute one frozen case command with bounded operational-only retry.

    The exact same immutable command tuple is handed to ``execute`` on every
    attempt. The first non-retryable or operationally complete result is canonical.
    If all allowed attempts end in retryable provider failure, the final failure is
    canonical and ``retry_exhausted`` is true.
    """

    if max_attempts < 1:
        raise ValueError("max_attempts must be at least 1")
    frozen_command = tuple(str(part) for part in command)
    fingerprint = command_fingerprint(frozen_command)
    records: list[CaseRetryRecord] = []

    for attempt in range(1, max_attempts + 1):
        result = execute(frozen_command)
        if not isinstance(result, CaseAttemptResult):
            raise TypeError("execute must return CaseAttemptResult")
        failure = provider_failure(result.payload)
        retryable = is_retryable_provider_failure(failure)
        exhausted = retryable and attempt == max_attempts
        canonical = (not retryable) or exhausted
        records.append(
            CaseRetryRecord(
                attempt=attempt,
                command_fingerprint=fingerprint,
                failure_source=failure.source if failure else None,
                failure_class=failure.failure_class if failure else None,
                failure_subtype=failure.subtype if failure else None,
                failure_message=failure.message if failure else None,
                retryable_operational_failure=retryable,
                canonical=canonical,
            )
        )
        if canonical:
            return CaseRetryOutcome(
                result=result,
                records=tuple(records),
                canonical_attempt=attempt,
                retry_exhausted=exhausted,
            )

    raise AssertionError("bounded retry loop must return a canonical attempt")
