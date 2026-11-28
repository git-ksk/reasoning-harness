"""CI-only bounded Gemini Interactions smoke test; no benchmark data."""
import datetime
import hashlib
import hmac
import json
import os
import re
import time
import urllib.error
import urllib.request

CHECKED = "2026-11-28T13:10:02.905496+00:00"
EXPIRES = "2026-11-29T13:10:02.905496+00:00"
MODEL = "gemini-3.5-flash-lite"
BASE = "https://generativelanguage.googleapis.com/v1beta/"
TAG = "rh-620-gemini-ci-smoke-20261128-v1"

class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, *args, **kwargs):
        return None

def require(ok, reason):
    if not ok:
        raise ValueError(reason)

def check_key(key, fingerprint, now):
    require(bool(key) and re.fullmatch("[0-9a-f]{64}", fingerprint) is not None
            and hmac.compare_digest(hashlib.sha256(key.encode()).hexdigest(), fingerprint), "key_binding")
    require(datetime.datetime.fromisoformat(CHECKED) <= now < datetime.datetime.fromisoformat(EXPIRES), "billing_attestation_expired")

def error_summary(raw, code, retry_after):
    try:
        d = json.loads(raw)
        err = d.get("error", {}) if isinstance(d, dict) else {}
        if not isinstance(err, dict): err = {}
    except (ValueError, UnicodeError):
        d, err = None, {}
    allowed = {"UNAVAILABLE","RESOURCE_EXHAUSTED","PERMISSION_DENIED","UNAUTHENTICATED","INVALID_ARGUMENT","INTERNAL","DEADLINE_EXCEEDED","FAILED_PRECONDITION"}
    text = raw.decode("utf-8", errors="replace").lower()
    flags = [x for x in ("quota","billing","credit","resource_exhausted","permission_denied","unauthenticated","api_key_invalid","invalid api key","payment","daily limit","forbidden") if x in text]
    types = [x.get("@type","").rsplit(".",1)[-1] for x in err.get("details",[]) if isinstance(x,dict)] if isinstance(err.get("details"),list) else []
    return {"http":code,"status":err.get("status") if err.get("status") in allowed else "absent_or_other",
            "json_object":isinstance(d,dict),"error_code_matches":err.get("code")==code,
            "conflict_markers":flags,"detail_types":[x for x in types if x in {"RetryInfo","QuotaFailure","ErrorInfo"}],
            "usage_present":isinstance(d,dict) and "usage" in d,
            "retry_after_seconds":int(retry_after) if retry_after and re.fullmatch("[0-9]{1,5}",retry_after) else None}

def call(opener, key, path, body=None):
    url=BASE+path
    headers={"x-goog-api-key":key,"Accept":"application/json"}
    data=None
    if body is not None:
        headers["Content-Type"]="application/json"
        data=json.dumps(body).encode()
    try:
        with opener.open(urllib.request.Request(url,data=data,headers=headers),timeout=35) as r:
            require(r.status==200 and r.geturl()==url,"origin")
            raw=r.read(1000001)
            require(len(raw)<=1000000,"response_size")
            return json.loads(raw)
    except urllib.error.HTTPError as exc:
        print(json.dumps(error_summary(exc.read(65536),exc.code,exc.headers.get("Retry-After"))),flush=True)
        raise ValueError("provider_http_"+str(exc.code)) from None

def run():
    require(os.getenv("GITHUB_ACTIONS")=="true" and os.getenv("GITHUB_RUN_ATTEMPT")=="1" and os.getenv("GITHUB_REF")=="refs/tags/"+TAG and os.getenv("GITHUB_REPOSITORY")=="git-ksk/reasoning-harness","ci_only_one_shot")
    key=os.environ.pop("GEMINI_API_KEY","")
    fingerprint=os.environ.pop("ENGINE620_GEMINI_KEY_SHA256","")
    check_key(key,fingerprint,datetime.datetime.now(datetime.timezone.utc))
    opener=urllib.request.build_opener(NoRedirect())
    for model in (MODEL,"gemini-3.8-flash"):
        d=call(opener,key,"models/"+model)
        require(d.get("name")=="models/"+model,"model_pin")
        print("metadata PASS "+model,flush=True)
    for i in range(2):
        check_key(key,fingerprint,datetime.datetime.now(datetime.timezone.utc))
        body={"model":MODEL,"input":"Return the JSON object with answer equal to 4.","system_instruction":"Return only valid JSON.",
              "response_format":{"type":"text","mime_type":"application/json","schema":{"type":"object","properties":{"answer":{"type":"integer"}},"required":["answer"]}},
              "generation_config":{"max_output_tokens":128,"seed":7,"temperature":1.0},"store":False}
        d=call(opener,key,"interactions",body)
        u=d.get("usage") or {}
        vals=[u.get(k) for k in ("total_input_tokens","total_output_tokens","total_tokens")]
        require(all(type(x) is int and x>=0 for x in vals) and vals[0]+vals[1]==vals[2] and vals[1]<=128,"unknown_or_invalid_usage")
        require(d.get("model",MODEL)==MODEL,"response_model_pin")
        txt="".join(c.get("text","") for step in d.get("steps",[]) if step.get("type")=="model_output" for c in step.get("content",[]) if c.get("type")=="text")
        require(json.loads(txt)=={"answer":4},"response_content")
        print(json.dumps({"call":i+1,"http":200,"usage":vals,"structured_answer":True}),flush=True)
        if i==0: time.sleep(30)

if __name__=="__main__":
    try:
        run()
    except BaseException as exc:
        print("HOLD: "+(str(exc) if type(exc) is ValueError else type(exc).__name__),flush=True)
        raise SystemExit(2)
