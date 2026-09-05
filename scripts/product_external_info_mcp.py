#!/usr/bin/env python3
"""Provider-neutral read-only MCP acquisition server for product evaluation.

This server is deliberately not a verifier. It performs only bounded HTTPS GETs to an
explicit public-host allowlist, extracts fixture-pinned JSON fields, and emits the
untrusted `reasoning_harness` acquisition envelope understood by `mcp_readonly_v1`.
"""
from __future__ import annotations

import json
import socket
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from typing import Any

PROTOCOL_VERSION = "2026-07-28"
TOOL_NAME = "fetch_json_fact"
USER_AGENT = "reasoning-harness-product-eval/1.0"
MAX_BODY_BYTES = 1_048_576
ALLOWED_HOSTS = frozenset({
    "api.github.com",
    "raw.githubusercontent.com",
    "pypi.org",
    "registry.npmjs.org",
    "crates.io",
})


def normalize_scalar(value: Any) -> str:
    if isinstance(value, bool):
        return "true" if value else "false"
    if value is None:
        return "null"
    if isinstance(value, (str, int, float)) and not isinstance(value, complex):
        return str(value)
    raise ValueError("selected JSON value must be a scalar")


def json_pointer(document: Any, pointer: str) -> Any:
    if pointer == "":
        return document
    if not pointer.startswith("/"):
        raise ValueError("JSON pointer must be empty or begin with '/'")
    current = document
    for raw in pointer.split("/")[1:]:
        token = raw.replace("~1", "/").replace("~0", "~")
        if isinstance(current, list):
            if not token.isdigit():
                raise KeyError(pointer)
            current = current[int(token)]
        elif isinstance(current, dict):
            current = current[token]
        else:
            raise KeyError(pointer)
    return current


def validate_url(url: str) -> None:
    parsed = urllib.parse.urlparse(url)
    if parsed.scheme != "https":
        raise ValueError("only https acquisition is permitted")
    if parsed.username or parsed.password or parsed.port not in (None, 443):
        raise ValueError("credentials and non-default ports are not permitted")
    host = (parsed.hostname or "").lower()
    if host not in ALLOWED_HOSTS:
        raise ValueError("host is not allowlisted")
    if not parsed.path.startswith("/"):
        raise ValueError("absolute HTTPS path is required")


def fetch_json(url: str, timeout_seconds: float = 8.0) -> tuple[Any, str]:
    validate_url(url)
    request = urllib.request.Request(
        url,
        headers={"Accept": "application/json", "User-Agent": USER_AGENT},
        method="GET",
    )
    with urllib.request.urlopen(request, timeout=timeout_seconds) as response:
        final_url = response.geturl()
        validate_url(final_url)
        body = response.read(MAX_BODY_BYTES + 1)
        if len(body) > MAX_BODY_BYTES:
            raise ValueError("response body exceeds acquisition bound")
        return json.loads(body), final_url


def identity_matches(document: Any, assertions: list[dict[str, Any]]) -> bool:
    for assertion in assertions:
        if set(assertion) != {"pointer", "equals"}:
            raise ValueError("identity assertion must contain exactly pointer and equals")
        try:
            actual = normalize_scalar(json_pointer(document, str(assertion["pointer"])))
        except (KeyError, IndexError, TypeError, ValueError):
            return False
        if actual != normalize_scalar(assertion["equals"]):
            return False
    return True


def observation_view(document: Any, pointers: list[str]) -> dict[str, Any]:
    view: dict[str, Any] = {}
    for pointer in pointers:
        try:
            view[pointer] = json_pointer(document, pointer)
        except (KeyError, IndexError, TypeError):
            view[pointer] = "<missing>"
    return view


def acquisition_envelope(
    *,
    observation: str,
    facts: dict[str, str],
    authority_class: str,
    observed_at: int,
    retrieved_at: int,
    scope: dict[str, Any] | None,
) -> dict[str, Any]:
    metadata: dict[str, Any] = {
        "observed_at_unix_seconds": observed_at,
        "retrieved_at_unix_seconds": retrieved_at,
        "claimed_authority_class": authority_class,
    }
    if scope is not None:
        metadata["scope"] = scope
    return {
        "reasoning_harness": {
            "observation": observation,
            "facts": facts,
            "acquisition_metadata": metadata,
        }
    }


def tool_result(arguments: dict[str, Any]) -> dict[str, Any]:
    allowed = {
        "url",
        "value_pointer",
        "fact_key",
        "authority_class",
        "identity_assertions",
        "observation_pointers",
        "scope",
        "mode",
        "sleep_ms",
        "operational_kind",
        "generic_text",
    }
    unknown = set(arguments) - allowed
    if unknown:
        raise ValueError(f"unknown tool arguments: {sorted(unknown)}")

    mode = arguments.get("mode", "live_json")
    if mode == "timeout":
        sleep_ms = int(arguments.get("sleep_ms", 60_000))
        if sleep_ms < 1 or sleep_ms > 120_000:
            raise ValueError("sleep_ms outside bounded range")
        time.sleep(sleep_ms / 1000)
        raise RuntimeError("timeout simulation returned unexpectedly")
    if mode == "tool_error":
        return {
            "content": [{"type": "text", "text": "synthetic read-only tool execution failure"}],
            "isError": True,
        }
    if mode == "generic_content":
        text = str(arguments.get("generic_text", "generic MCP content without Harness fact envelope"))
        if len(text.encode("utf-8")) > 16_384:
            raise ValueError("generic_text exceeds bounded observation size")
        return {
            "content": [{"type": "text", "text": text}],
        }
    if mode != "live_json":
        raise ValueError("unsupported acquisition mode")

    required = {"url", "value_pointer", "fact_key", "authority_class"}
    missing = required - set(arguments)
    if missing:
        raise ValueError(f"missing required tool arguments: {sorted(missing)}")

    url = str(arguments["url"])
    authority_class = str(arguments["authority_class"])
    if not authority_class or authority_class.strip() != authority_class:
        raise ValueError("authority_class must be a non-empty normalized token")

    retrieved_at = int(time.time())
    try:
        document, final_url = fetch_json(url)
    except urllib.error.HTTPError as error:
        if error.code == 404:
            return {
                "content": [{"type": "text", "text": "public source returned no matching resource"}],
                "structuredContent": acquisition_envelope(
                    observation=f"GET {url} returned HTTP 404; no fact candidate was produced.",
                    facts={},
                    authority_class=authority_class,
                    observed_at=retrieved_at,
                    retrieved_at=retrieved_at,
                    scope=arguments.get("scope"),
                ),
            }
        raise

    assertions = arguments.get("identity_assertions", [])
    if not isinstance(assertions, list):
        raise ValueError("identity_assertions must be an array")
    matched = identity_matches(document, assertions)
    pointers = arguments.get("observation_pointers", [])
    if not isinstance(pointers, list) or not all(isinstance(item, str) for item in pointers):
        raise ValueError("observation_pointers must be an array of strings")
    if not pointers:
        pointers = [str(arguments["value_pointer"])] + [str(item["pointer"]) for item in assertions]
    view = observation_view(document, pointers)

    facts: dict[str, str] = {}
    if matched:
        try:
            facts[str(arguments["fact_key"])] = normalize_scalar(
                json_pointer(document, str(arguments["value_pointer"]))
            )
        except (KeyError, IndexError, TypeError, ValueError):
            facts = {}

    observation = json.dumps(
        {
            "url": final_url,
            "identity_assertions_satisfied": matched,
            "selected": view,
        },
        ensure_ascii=False,
        sort_keys=True,
        separators=(",", ":"),
    )
    return {
        "content": [{"type": "text", "text": observation}],
        "structuredContent": acquisition_envelope(
            observation=observation,
            facts=facts,
            authority_class=authority_class,
            observed_at=retrieved_at,
            retrieved_at=retrieved_at,
            scope=arguments.get("scope"),
        ),
    }


def rpc_error(request_id: Any, code: int, message: str, operational_kind: str | None = None) -> dict[str, Any]:
    error: dict[str, Any] = {"code": code, "message": message}
    if operational_kind:
        error["data"] = {"reasoning_harness": {"operational_kind": operational_kind}}
    return {"jsonrpc": "2.0", "id": request_id, "error": error}


def handle_request(request: dict[str, Any]) -> dict[str, Any]:
    request_id = request.get("id")
    if request.get("jsonrpc") != "2.0":
        return rpc_error(request_id, -32600, "JSON-RPC 2.0 required")
    if request.get("method") != "tools/call":
        return rpc_error(request_id, -32601, "only stateless tools/call is supported")
    params = request.get("params")
    if not isinstance(params, dict) or params.get("name") != TOOL_NAME:
        return rpc_error(request_id, -32602, "unknown tool")
    meta = params.get("_meta", {})
    if meta.get("io.modelcontextprotocol/protocolVersion") != PROTOCOL_VERSION:
        return rpc_error(request_id, -32602, "MCP protocol revision mismatch")
    arguments = params.get("arguments", {})
    if not isinstance(arguments, dict):
        return rpc_error(request_id, -32602, "tool arguments must be an object")
    if arguments.get("mode") == "rpc_error":
        operational_kind = str(arguments.get("operational_kind", "transport"))
        return rpc_error(request_id, -32001, "synthetic typed operational failure", operational_kind)
    try:
        result = tool_result(arguments)
    except (ValueError, KeyError, TypeError) as error:
        return rpc_error(request_id, -32602, str(error), "policy_denied")
    except (urllib.error.URLError, socket.timeout, TimeoutError):
        return rpc_error(request_id, -32002, "read-only acquisition transport failure", "transport")
    except Exception:
        return rpc_error(request_id, -32003, "read-only acquisition tool failure", "tool_execution")
    return {"jsonrpc": "2.0", "id": request_id, "result": result}


def main() -> int:
    line = sys.stdin.buffer.readline(MAX_BODY_BYTES + 1)
    if not line or len(line) > MAX_BODY_BYTES:
        return 2
    try:
        request = json.loads(line)
    except json.JSONDecodeError:
        response = rpc_error(None, -32700, "invalid JSON")
    else:
        response = handle_request(request)
    sys.stdout.write(json.dumps(response, ensure_ascii=False, separators=(",", ":")) + "\n")
    sys.stdout.flush()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
