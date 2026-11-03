"""Frozen native request limits. No credential, enrollment or run permission."""
from contracts import require

HTTP_LIMITS = "reason-cli-normal-v3-http-limits-v1"
FROZEN_HTTP_LIMITS = {
    "contract": HTTP_LIMITS,
    "max_request_utf8_json_bytes": 12000,
    "input_overhead_tokens": 512,
    "max_input_tokens_per_generation": 12512,
    "max_output_tokens_per_generation": 1024,
    "max_input_tokens_total": 24000,
    "max_output_tokens_total": 4096,
    "max_generations": 24,
    "min_generation_start_interval_ms": 10500,
}


def validate_http_limits(value, *, control=False, budget=None):
    require(type(value) is dict and set(value) == set(FROZEN_HTTP_LIMITS)
            and value["contract"] == HTTP_LIMITS, "invalid_frozen_http_limits")
    for name, ceiling in FROZEN_HTTP_LIMITS.items():
        if name in ("contract", "min_generation_start_interval_ms", "input_overhead_tokens"): continue
        require(type(value[name]) is int and 0 < value[name] <= ceiling, "weakened_frozen_http_limit:" + name)
    require(type(value["input_overhead_tokens"]) is int and value["input_overhead_tokens"] == 512,
            "invalid_frozen_input_estimate")
    interval = value["min_generation_start_interval_ms"]
    require(type(interval) is int and (0 if control else 10500) <= interval <= 60000,
            "weakened_frozen_http_pacing")
    if budget is not None:
        require(budget["max_attempts_per_generation"] == 1
                and value["max_generations"] == budget["max_generations"]
                and value["max_input_tokens_per_generation"] <= budget["max_input_tokens_per_generation"]
                and value["max_output_tokens_per_generation"] <= budget["max_output_tokens_per_generation"],
                "http_limits_not_bound_to_reservation")
    return value
