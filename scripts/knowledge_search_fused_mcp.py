#!/usr/bin/env python3
import json
import sys
import time
import urllib.parse
import urllib.request

USER_AGENT = "reasoning-harness-mcp-search-probe/0.1 (https://github.com/git-ksk/reasoning-harness)"


def respond(request_id, result=None, error=None):
    payload = {"jsonrpc": "2.0", "id": request_id}
    if error is not None:
        payload["error"] = error
    else:
        payload["result"] = result
    sys.stdout.write(json.dumps(payload, ensure_ascii=False) + "\n")
    sys.stdout.flush()


def get_json(base, params, timeout=10):
    query = urllib.parse.urlencode(params)
    request = urllib.request.Request(f"{base}?{query}", headers={"User-Agent": USER_AGENT})
    last_error = None
    for attempt in range(2):
        try:
            with urllib.request.urlopen(request, timeout=timeout) as response:
                return json.load(response)
        except Exception as exc:
            last_error = exc
            if attempt == 0:
                time.sleep(0.15)
    raise last_error


def wikidata_search(query, language, limit=5):
    body = get_json(
        "https://www.wikidata.org/w/api.php",
        {
            "action": "wbsearchentities",
            "format": "json",
            "search": query,
            "language": language,
            "uselang": language,
            "type": "item",
            "limit": str(limit),
        },
    )
    return [
        {
            "id": item.get("id"),
            "label": item.get("label"),
            "description": item.get("description"),
        }
        for item in body.get("search", [])
        if item.get("id")
    ]


def wikipedia_search(query, language, limit=5):
    body = get_json(
        f"https://{language}.wikipedia.org/w/api.php",
        {
            "action": "query",
            "format": "json",
            "formatversion": "2",
            "generator": "search",
            "gsrsearch": query,
            "gsrnamespace": "0",
            "gsrlimit": str(limit),
            "prop": "pageprops",
            "ppprop": "wikibase_item|disambiguation",
        },
    )
    pages = ((body.get("query") or {}).get("pages") or [])
    pages.sort(key=lambda page: page.get("index", 1_000_000))
    return [
        {
            "title": page.get("title"),
            "wikibase_item": (page.get("pageprops") or {}).get("wikibase_item"),
            "disambiguation": "disambiguation" in (page.get("pageprops") or {}),
        }
        for page in pages
    ]


def wikidata_claim_values(entity_id, property_id, value_kind):
    body = get_json(
        "https://www.wikidata.org/w/api.php",
        {
            "action": "wbgetentities",
            "format": "json",
            "ids": entity_id,
            "props": "claims",
        },
    )
    entity = (body.get("entities") or {}).get(entity_id) or {}
    statements = (entity.get("claims") or {}).get(property_id) or []
    preferred = [statement for statement in statements if statement.get("rank") == "preferred"]
    selected = preferred or [statement for statement in statements if statement.get("rank") != "deprecated"]
    values = []
    for statement in selected:
        datavalue = (((statement.get("mainsnak") or {}).get("datavalue") or {}).get("value"))
        if datavalue is None:
            continue
        value = None
        if value_kind == "entity" and isinstance(datavalue, dict):
            numeric_id = datavalue.get("numeric-id")
            if isinstance(numeric_id, int):
                value = f"Q{numeric_id}"
        elif value_kind == "quantity" and isinstance(datavalue, dict):
            amount = datavalue.get("amount")
            if isinstance(amount, str):
                value = amount.lstrip("+")
        elif value_kind == "time" and isinstance(datavalue, dict):
            raw = datavalue.get("time")
            if isinstance(raw, str):
                value = raw
        elif value_kind == "string" and isinstance(datavalue, str):
            value = datavalue
        if value is not None and value not in values:
            values.append(value)
    return values


def result_payload(observation, facts, metadata_extra=None):
    now = int(time.time())
    metadata = {
        "observed_at_unix_seconds": now,
        "retrieved_at_unix_seconds": now,
        "claimed_authority_class": "public_cross_source",
    }
    if metadata_extra:
        metadata.update(metadata_extra)
    return {
        "content": [{"type": "text", "text": observation}],
        "structuredContent": {
            "reasoning_harness": {
                "observation": observation,
                "facts": facts,
                "acquisition_metadata": metadata,
            }
        },
        "isError": False,
    }


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
    if params.get("name") != "search_fact":
        respond(request_id, error={"code": -32602, "message": "unsupported tool"})
        return 0
    arguments = params.get("arguments") or {}
    query = arguments.get("query")
    property_id = arguments.get("property_id")
    value_kind = arguments.get("value_kind")
    fact_key = arguments.get("fact_key")
    language = arguments.get("language", "en")
    if not all(isinstance(value, str) and value for value in [query, property_id, value_kind, fact_key, language]):
        respond(request_id, error={"code": -32602, "message": "query/property_id/value_kind/fact_key/language are required"})
        return 0

    try:
        wd = wikidata_search(query, language)
        wp = wikipedia_search(query, language)
        if not wd or not wp:
            respond(request_id, result=result_payload(
                f"search unresolved: query={query!r}; wikidata_candidates={len(wd)}; wikipedia_candidates={len(wp)}",
                {},
            ))
            return 0

        wd_top = wd[0]
        wp_top = wp[0]
        if wp_top.get("disambiguation"):
            respond(request_id, result=result_payload(
                f"search ambiguous: Wikipedia top result {wp_top.get('title')!r} is a disambiguation page; wikidata_top={wd_top.get('id')}",
                {},
            ))
            return 0

        wd_id = wd_top.get("id")
        wp_id = wp_top.get("wikibase_item")
        if not wd_id or not wp_id or wd_id != wp_id:
            respond(request_id, result=result_payload(
                f"cross-source entity disagreement: query={query!r}; wikidata_top={wd_id}; wikipedia_top={wp_id}; wikipedia_title={wp_top.get('title')!r}",
                {},
            ))
            return 0

        values = wikidata_claim_values(wd_id, property_id, value_kind)
        if len(values) != 1:
            respond(request_id, result=result_payload(
                f"property unresolved or multi-valued after cross-source entity agreement: entity={wd_id}; property={property_id}; values={values}",
                {},
            ))
            return 0

        value = values[0]
        observation = (
            f"cross-source search resolved query={query!r} to {wd_id} via Wikidata + Wikipedia; "
            f"{fact_key}={value}; property={property_id}"
        )
        respond(request_id, result=result_payload(observation, {fact_key: value}))
        return 0
    except Exception as exc:
        respond(
            request_id,
            error={
                "code": -32000,
                "message": f"knowledge search failed: {exc}",
                "data": {"reasoning_harness": {"operational_kind": "transport"}},
            },
        )
        return 0


if __name__ == "__main__":
    raise SystemExit(main())
