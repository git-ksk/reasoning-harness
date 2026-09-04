#!/usr/bin/env python3
import decimal
import json
import sys
import time
import urllib.parse
import urllib.request

SOURCE_AUTHORITY = "wikidata_public"
USER_AGENT = "reasoning-harness-mcp-knowledge-benchmark/0.1 (https://github.com/git-ksk/reasoning-harness)"


def respond(request_id, result=None, error=None):
    payload = {"jsonrpc": "2.0", "id": request_id}
    if error is not None:
        payload["error"] = error
    else:
        payload["result"] = result
    sys.stdout.write(json.dumps(payload, ensure_ascii=False) + "\n")
    sys.stdout.flush()


def canonical_quantity(raw):
    value = decimal.Decimal(str(raw))
    normalized = value.normalize()
    text = format(normalized, "f")
    if "." in text:
        text = text.rstrip("0").rstrip(".")
    return text or "0"


def extract_value(datavalue, value_kind):
    if not isinstance(datavalue, dict):
        return None
    value = datavalue.get("value")
    if value_kind == "entity":
        if isinstance(value, dict):
            entity_id = value.get("id")
            return entity_id if isinstance(entity_id, str) else None
        return None
    if value_kind == "quantity":
        if not isinstance(value, dict) or "amount" not in value:
            return None
        try:
            return canonical_quantity(value["amount"])
        except (decimal.InvalidOperation, ValueError):
            return None
    if value_kind == "string":
        return value if isinstance(value, str) else None
    if value_kind == "time":
        if isinstance(value, dict):
            raw = value.get("time")
            return raw if isinstance(raw, str) else None
        return None
    return None


def fetch_claim(entity_id, property_id, value_kind):
    query = urllib.parse.urlencode(
        {
            "action": "wbgetclaims",
            "format": "json",
            "entity": entity_id,
            "property": property_id,
        }
    )
    request = urllib.request.Request(
        f"https://www.wikidata.org/w/api.php?{query}",
        headers={"User-Agent": USER_AGENT, "Accept": "application/json"},
    )
    with urllib.request.urlopen(request, timeout=10) as response:
        payload = json.load(response)

    claims = ((payload.get("claims") or {}).get(property_id) or [])
    claims = [claim for claim in claims if claim.get("rank") != "deprecated"]
    claims.sort(key=lambda claim: 0 if claim.get("rank") == "preferred" else 1)
    for claim in claims:
        mainsnak = claim.get("mainsnak") or {}
        if mainsnak.get("snaktype") != "value":
            continue
        value = extract_value(mainsnak.get("datavalue"), value_kind)
        if value is not None:
            return value
    return None


def main():
    line = sys.stdin.readline()
    if not line:
        return 1
    try:
        request = json.loads(line)
    except json.JSONDecodeError as exc:
        respond(None, error={"code": -32700, "message": f"parse error: {exc}"})
        return 1

    request_id = request.get("id")
    if request.get("method") != "tools/call":
        respond(request_id, error={"code": -32601, "message": "method not found"})
        return 0

    params = request.get("params") or {}
    if params.get("name") != "typed_fact":
        respond(request_id, error={"code": -32602, "message": "unsupported tool"})
        return 0

    arguments = params.get("arguments") or {}
    entity_id = arguments.get("entity_id")
    property_id = arguments.get("property_id")
    fact_key = arguments.get("fact_key")
    value_kind = arguments.get("value_kind", "entity")
    if not isinstance(entity_id, str) or not entity_id.startswith("Q"):
        respond(request_id, error={"code": -32602, "message": "valid entity_id is required"})
        return 0
    if not isinstance(property_id, str) or not property_id.startswith("P"):
        respond(request_id, error={"code": -32602, "message": "valid property_id is required"})
        return 0
    if not isinstance(fact_key, str) or not fact_key:
        respond(request_id, error={"code": -32602, "message": "fact_key is required"})
        return 0
    if value_kind not in {"entity", "quantity", "string", "time"}:
        respond(request_id, error={"code": -32602, "message": "unsupported value_kind"})
        return 0

    try:
        value = fetch_claim(entity_id, property_id, value_kind)
    except Exception as exc:
        respond(
            request_id,
            error={
                "code": -32000,
                "message": f"Wikidata acquisition failed: {type(exc).__name__}",
                "data": {"reasoning_harness": {"operational_kind": "transport"}},
            },
        )
        return 0

    # Benchmark workaround for the current live-admission time boundary: observed time
    # reflects the acquisition, while the harness-side evaluation snapshot is configured
    # with a small bounded future window.
    now = int(time.time())
    facts = {} if value is None else {fact_key: value}
    observation = (
        f"{fact_key}=<no_value>; entity={entity_id}; property={property_id}"
        if value is None
        else f"{fact_key}={value}; entity={entity_id}; property={property_id}"
    )
    respond(
        request_id,
        result={
            "content": [{"type": "text", "text": observation}],
            "structuredContent": {
                "reasoning_harness": {
                    "observation": observation,
                    "facts": facts,
                    "acquisition_metadata": {
                        "observed_at_unix_seconds": now,
                        "retrieved_at_unix_seconds": now,
                        "claimed_authority_class": SOURCE_AUTHORITY,
                    },
                }
            },
            "isError": False,
        },
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
