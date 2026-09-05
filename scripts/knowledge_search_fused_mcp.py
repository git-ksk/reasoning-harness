#!/usr/bin/env python3
import json
import sys
import time
import urllib.parse
import urllib.request

USER_AGENT = "reasoning-harness-mcp-search-probe/0.4 (https://github.com/git-ksk/reasoning-harness)"
EMBED_SEARCH_STATE_IN_HARNESS = False
EXTERNAL_REQUESTS = 0


def respond(request_id, result=None, error=None):
    payload = {"jsonrpc": "2.0", "id": request_id}
    if error is not None:
        payload["error"] = error
    else:
        payload["result"] = result
    sys.stdout.write(json.dumps(payload, ensure_ascii=False) + "\n")
    sys.stdout.flush()


def get_json(base, params, timeout=10):
    global EXTERNAL_REQUESTS
    query = urllib.parse.urlencode(params)
    request = urllib.request.Request(f"{base}?{query}", headers={"User-Agent": USER_AGENT})
    last_error = None
    for attempt in range(2):
        try:
            EXTERNAL_REQUESTS += 1
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


def result_payload(observation, facts, search_state, metadata_extra=None):
    now = int(time.time())
    metadata = {
        "observed_at_unix_seconds": now,
        "retrieved_at_unix_seconds": now,
        "claimed_authority_class": "public_cross_source",
    }
    if metadata_extra:
        metadata.update(metadata_extra)
    harness_payload = {
        "observation": observation,
        "facts": facts,
        "acquisition_metadata": metadata,
    }
    # The standard mcp_readonly_v1 protocol intentionally rejects unknown fields.
    # Layer C calls this probe directly, so only that experimental path receives
    # the extra telemetry inside the harness payload expected by its raw parser.
    if EMBED_SEARCH_STATE_IN_HARNESS:
        harness_payload["search_state"] = search_state
    return {
        "content": [{"type": "text", "text": observation}],
        "structuredContent": {
            "reasoning_harness": harness_payload,
            "search_state": search_state,
        },
        "isError": False,
    }


def state(query, outcome_kind, wd=None, wp=None, **extra):
    wd = wd or []
    wp = wp or []
    extra.setdefault("external_requests", EXTERNAL_REQUESTS)
    payload = {
        "query": query,
        "outcome_kind": outcome_kind,
        "wikidata_candidate_ids": [item.get("id") for item in wd if item.get("id")],
        "wikipedia_candidates": [
            {
                "title": item.get("title"),
                "wikibase_item": item.get("wikibase_item"),
                "disambiguation": bool(item.get("disambiguation")),
            }
            for item in wp
        ],
    }
    payload.update(extra)
    return payload


def validate_agentic_query(query, property_id):
    """Validate the planner action before any external request.

    The tool already receives property_id separately. The query field is only an
    entity label/title search surface. Returning a typed observation instead of
    silently repairing the action lets the bounded planner loop learn from the
    tool contract while keeping the Harness in control of execution.
    """
    normalized = " ".join(query.split()).strip()
    if len(normalized) > 160:
        return False, "query_too_long", None

    lower = normalized.lower()
    if "http://" in lower or "https://" in lower or "site:" in lower:
        return False, "web_operator_or_url_not_allowed", None

    kept = []
    removed_property_token = False
    for token in normalized.split():
        stripped = token.strip(".,;:()[]{}<>'\"")
        upper = stripped.upper()
        if upper == property_id.upper() or (upper.startswith("P") and upper[1:].isdigit()):
            removed_property_token = True
            continue
        kept.append(token)

    if removed_property_token:
        suggested = " ".join(kept).strip() or None
        return False, "property_id_must_not_be_in_query", suggested

    if not normalized:
        return False, "empty_query", None

    return True, None, None


def main():
    global EMBED_SEARCH_STATE_IN_HARNESS

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
    allow_title_retry = arguments.get("allow_title_retry", True)
    if not all(isinstance(value, str) and value for value in [query, property_id, value_kind, fact_key, language]):
        respond(request_id, error={"code": -32602, "message": "query/property_id/value_kind/fact_key/language are required"})
        return 0
    if not isinstance(allow_title_retry, bool):
        respond(request_id, error={"code": -32602, "message": "allow_title_retry must be boolean"})
        return 0

    EMBED_SEARCH_STATE_IN_HARNESS = not allow_title_retry

    # Layer C is intentionally agentic. Reject malformed planner actions before
    # they can spend external-request budget. Layer B keeps its fixed-policy
    # behavior unchanged for an apples-to-apples non-agentic baseline.
    if not allow_title_retry:
        valid, validation_reason, suggested_query = validate_agentic_query(query, property_id)
        if not valid:
            observation = (
                f"planner search action rejected before external request: reason={validation_reason}; "
                f"invalid_query={query!r}; property={property_id} is already fixed by the tool; "
                f"query must be only an entity label/title, with no property IDs, URLs, or web operators; "
                f"suggested_query={suggested_query!r}; external_requests=0"
            )
            respond(request_id, result=result_payload(
                observation,
                {},
                state(
                    query,
                    "invalid_query",
                    validation_reason=validation_reason,
                    suggested_query=suggested_query,
                    query_constraint="entity_label_or_title_only",
                    external_requests=0,
                    suggested_action="search" if suggested_query else "stop",
                ),
            ))
            return 0

    try:
        wd = wikidata_search(query, language)
        wp = wikipedia_search(query, language)
        if not wd or not wp:
            observation = (
                f"search unresolved: query={query!r}; wikidata_candidates={len(wd)}; "
                f"wikipedia_candidates={len(wp)}; tool_query_semantics=entity_label_or_title_only_no_urls_no_site_operator_no_property_id"
            )
            respond(request_id, result=result_payload(
                observation,
                {},
                state(query, "search_unresolved", wd, wp),
            ))
            return 0

        wd_top = wd[0]
        wp_top = wp[0]
        if wp_top.get("disambiguation"):
            observation = (
                f"search ambiguous: Wikipedia top result {wp_top.get('title')!r} is a disambiguation page; "
                f"wikidata_top={wd_top.get('id')}; ambiguity_requires_user_or_evidence_context=true; "
                f"do_not_invent_disambiguation=true; suggested_action=stop"
            )
            respond(request_id, result=result_payload(
                observation,
                {},
                state(query, "ambiguous", wd, wp, disambiguation=True, suggested_action="stop"),
            ))
            return 0

        wp_id = wp_top.get("wikibase_item")
        wp_title = wp_top.get("title")
        wd_ids = [item.get("id") for item in wd if item.get("id")]
        corroboration = "original_query"
        corroboration_rank = None

        if wp_id and wp_id in wd_ids:
            corroboration_rank = wd_ids.index(wp_id) + 1
        elif wp_id and isinstance(wp_title, str) and wp_title and allow_title_retry:
            title_wd = wikidata_search(wp_title, language)
            title_wd_ids = [item.get("id") for item in title_wd if item.get("id")]
            if wp_id in title_wd_ids:
                corroboration = "wikipedia_title_retry"
                corroboration_rank = title_wd_ids.index(wp_id) + 1
            else:
                observation = (
                    f"cross-source entity disagreement after title retry: query={query!r}; original_wikidata_candidates={wd_ids}; "
                    f"wikipedia_top={wp_id}; wikipedia_title={wp_title!r}; title_retry_candidates={title_wd_ids}"
                )
                respond(request_id, result=result_payload(
                    observation,
                    {},
                    state(
                        query,
                        "entity_disagreement",
                        wd,
                        wp,
                        wikipedia_top_entity=wp_id,
                        wikipedia_top_title=wp_title,
                        title_retry_candidate_ids=title_wd_ids,
                    ),
                ))
                return 0
        elif wp_id and isinstance(wp_title, str) and wp_title:
            observation = (
                f"cross-source entity disagreement: query={query!r}; wikidata_candidates={wd_ids}; "
                f"wikipedia_top={wp_id}; wikipedia_title={wp_title!r}; fixed_title_retry_disabled=true; "
                f"suggested_query={wp_title!r}; tool_query_semantics=entity_label_or_title_only_no_urls_no_site_operator_no_property_id"
            )
            respond(request_id, result=result_payload(
                observation,
                {},
                state(
                    query,
                    "entity_disagreement",
                    wd,
                    wp,
                    wikipedia_top_entity=wp_id,
                    wikipedia_top_title=wp_title,
                    suggested_query=wp_title,
                    query_constraint="entity_label_or_title_only",
                ),
            ))
            return 0
        else:
            observation = (
                f"cross-source entity unresolved: query={query!r}; wikidata_candidates={wd_ids}; "
                f"wikipedia_top={wp_id}; wikipedia_title={wp_title!r}; "
                f"tool_query_semantics=entity_label_or_title_only_no_urls_no_site_operator_no_property_id"
            )
            respond(request_id, result=result_payload(
                observation,
                {},
                state(
                    query,
                    "entity_unresolved",
                    wd,
                    wp,
                    wikipedia_top_entity=wp_id,
                    wikipedia_top_title=wp_title,
                ),
            ))
            return 0

        values = wikidata_claim_values(wp_id, property_id, value_kind)
        if len(values) != 1:
            observation = f"property unresolved or multi-valued after cross-source entity agreement: entity={wp_id}; property={property_id}; values={values}"
            respond(request_id, result=result_payload(
                observation,
                {},
                state(
                    query,
                    "property_unresolved",
                    wd,
                    wp,
                    resolved_entity=wp_id,
                    corroboration_mode=corroboration,
                    corroboration_rank=corroboration_rank,
                    property_id=property_id,
                    property_values=values,
                ),
            ))
            return 0

        value = values[0]
        observation = (
            f"cross-source search resolved query={query!r} to {wp_id} via Wikipedia top result + Wikidata corroboration "
            f"(mode={corroboration}, rank={corroboration_rank}); {fact_key}={value}; property={property_id}"
        )
        respond(request_id, result=result_payload(
            observation,
            {fact_key: value},
            state(
                query,
                "fact_resolved",
                wd,
                wp,
                resolved_entity=wp_id,
                corroboration_mode=corroboration,
                corroboration_rank=corroboration_rank,
                property_id=property_id,
                property_values=values,
            ),
        ))
        return 0
    except Exception as exc:
        respond(
            request_id,
            error={
                "code": -32000,
                "message": f"knowledge search failed: {exc}",
                "data": {"reasoning_harness": {"operational_kind": "transport", "external_requests": EXTERNAL_REQUESTS}},
            },
        )
        return 0


if __name__ == "__main__":
    raise SystemExit(main())
