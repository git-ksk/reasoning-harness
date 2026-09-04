#!/usr/bin/env python3
import json
import sys
import time
import urllib.parse
import urllib.request


def respond(request_id, result=None, error=None):
    payload = {"jsonrpc": "2.0", "id": request_id}
    if error is not None:
        payload["error"] = error
    else:
        payload["result"] = result
    sys.stdout.write(json.dumps(payload, ensure_ascii=False) + "\n")
    sys.stdout.flush()


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
    if params.get("name") != "page_exists":
        respond(request_id, error={"code": -32602, "message": "unsupported tool"})
        return 0

    arguments = params.get("arguments") or {}
    title = arguments.get("title")
    language = arguments.get("language", "ja")
    if not isinstance(title, str) or not title:
        respond(request_id, error={"code": -32602, "message": "title is required"})
        return 0
    if not isinstance(language, str) or not language.replace("-", "").isalnum():
        respond(request_id, error={"code": -32602, "message": "invalid language"})
        return 0

    query = urllib.parse.urlencode(
        {
            "action": "query",
            "format": "json",
            "redirects": "1",
            "titles": title,
        }
    )
    url = f"https://{language}.wikipedia.org/w/api.php?{query}"
    req = urllib.request.Request(
        url,
        headers={
            "User-Agent": "reasoning-harness-mcp-knowledge-probe/0.1 (https://github.com/git-ksk/reasoning-harness)"
        },
    )
    with urllib.request.urlopen(req, timeout=10) as response:
        body = json.load(response)

    pages = ((body.get("query") or {}).get("pages") or {})
    exists = any(str(page_id) != "-1" and "missing" not in page for page_id, page in pages.items())
    value = "true" if exists else "false"
    now = int(time.time())
    fact_key = "wikipedia.page_exists"

    respond(
        request_id,
        result={
            "content": [
                {
                    "type": "text",
                    "text": f"Wikipedia page existence for {language}:{title} = {value}",
                }
            ],
            "structuredContent": {
                "reasoning_harness": {
                    "observation": f"{fact_key}={value}",
                    "facts": {fact_key: value},
                    "acquisition_metadata": {
                        "observed_at_unix_seconds": now,
                        "retrieved_at_unix_seconds": now,
                        "claimed_authority_class": "primary",
                    },
                }
            },
            "isError": False,
        },
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
