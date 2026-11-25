"""One bounded synthetic probe. Never enables billing, retries, or uses case data."""
import json, os, sys, urllib.request, urllib.error
from decimal import Decimal
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/"dev-lite"))
from issue531_real24_api_billing_gate import (
    BASE, MODEL, BillingGateError, NoRedirect, get_json, strict_decimal,
)
def need(ok, reason):
    if not ok: raise BillingGateError(reason)

def inspect_account(sub,budget,models):
    need(isinstance(sub,dict) and isinstance(budget,dict) and isinstance(models,dict),"invalid_response")
    pool=budget.get("api_budget")
    need(sub.get("payg_enabled") is False and isinstance(pool,dict) and pool.get("payg_enabled") is False,"payg_not_proven_off")
    need(pool.get("currency") in ("USD","EUR") and budget.get("currency")==pool.get("currency"),"invalid_currency")
    initial=strict_decimal(pool.get("initial_budget"),"initial_budget")
    used=strict_decimal(pool.get("usage_percentage"),"usage_percentage")
    rows=models.get("data")
    need(isinstance(rows,list) and any(isinstance(x,dict) and x.get("id")==MODEL for x in rows),"fixed_model_unavailable")
    print("same_key_payg_disabled=true model_available=true",flush=True)
    print("api_initial_budget_class="+("zero" if initial==0 else "positive" if initial>0 else "negative"),flush=True)
    print("api_usage_percentage_class="+("zero" if used==0 else "within_range" if 0<used<=100 else "out_of_range"),flush=True)
    # Do not interpret an exhausted positive allowance, negative amounts, or
    # out-of-range percentages as entitlement to free requests.
    if initial==0:
        need(used==0,"zero_budget_with_nonzero_usage")
        return "zero_monetary_pool"
    need(initial>0 and 0<=used<=100,"invalid_monetary_pool")
    need(initial*(100-used)/100>=Decimal("1"),"insufficient_monetary_headroom")
    return "positive_monetary_pool"

def main():
    key=os.environ.pop("MISTRAL_API_KEY","")
    need(bool(key),"missing_key")
    opener=urllib.request.build_opener(NoRedirect())
    mode=inspect_account(*(get_json(opener,key,p) for p in ("/v1/billing/subscription","/v1/billing/budget","/v1/models")))
    print("billing_mode="+mode,flush=True)
    req=urllib.request.Request(BASE+"/v1/chat/completions",data=json.dumps({
        "model":MODEL,"messages":[{"role":"user","content":"Reply OK."}],
        "max_tokens":8,"temperature":0,"stream":False,
    }).encode(),headers={"Authorization":"Bearer "+key,"Content-Type":"application/json","Accept":"application/json"},method="POST")
    print("synthetic_probe_attempts=1 max_output_tokens=8",flush=True)
    try:
        with opener.open(req,timeout=30) as res:
            need(res.status==200 and res.geturl()==BASE+"/v1/chat/completions","unexpected_origin_or_status")
            raw=res.read(100001)
            need(len(raw)<=100000,"oversized_response")
            data=json.loads(raw)
    except urllib.error.HTTPError as exc:
        print("probe_http_status="+str(exc.code),flush=True)
        raise BillingGateError("probe_http_refused") from None
    need(data.get("model")==MODEL and bool(data.get("choices")),"model_or_response_mismatch")
    u=data.get("usage",{})
    need(all(type(u.get(k)) is int and u[k]>=0 for k in ("prompt_tokens","completion_tokens","total_tokens")) and u["completion_tokens"]<=8 and u["total_tokens"]==u["prompt_tokens"]+u["completion_tokens"],"probe_usage_invalid")
    print("probe_http_status=200 usage_verified=true total_tokens="+str(u["total_tokens"]),flush=True)
    post=inspect_account(*(get_json(opener,key,p) for p in ("/v1/billing/subscription","/v1/billing/budget","/v1/models")))
    need(post==mode,"billing_mode_changed")
    print("probe_pass=true no_billing_settings_changed=true",flush=True)

if __name__=="__main__":
    try: main()
    except BillingGateError as e:
        print("probe_hold="+str(e),flush=True)
        sys.exit(2)
    except Exception as e:
        print("probe_hold="+type(e).__name__,flush=True)
        sys.exit(2)
