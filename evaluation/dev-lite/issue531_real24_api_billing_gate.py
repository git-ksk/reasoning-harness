#!/usr/bin/env python3
"""Current account-specific, model-free, fail-closed billing admission for #531.

API-key bearer is supplied only via GitHub Actions Secret. Never print API responses,
financial values, user identifiers, or the key. Does not change account settings.
"""
from __future__ import annotations
from decimal import Decimal, InvalidOperation
import json
import os
import urllib.error
import urllib.request

BASE='https://api.mistral.ai'
MODEL='ministral-8b-2512'
N_CASES=24
RUNS=2*N_CASES
TOTAL_TOKEN_CAP=RUNS*28096
MIN_INCLUDED_HEADROOM=Decimal('1.00')  # dollars/euros; both allowed

class BillingGateError(Exception):
    pass

def strict_decimal(value, field):
    if isinstance(value,bool) or value is None:
        raise BillingGateError('invalid_'+field)
    try:
        number=Decimal(str(value))
    except (InvalidOperation, TypeError, ValueError) as exc:
        raise BillingGateError('invalid_'+field) from exc
    if not number.is_finite():
        raise BillingGateError('nonfinite_'+field)
    return number

def check_account(subscription, budget, models):
    if not isinstance(subscription,dict) or not isinstance(budget,dict) or not isinstance(models,dict):
        raise BillingGateError('response_shape_invalid')
    if subscription.get('payg_enabled') is not False:
        raise BillingGateError('organization_api_payg_not_proven_disabled')
    pool=budget.get('api_budget')
    if not isinstance(pool,dict) or pool.get('payg_enabled') is not False:
        raise BillingGateError('api_budget_payg_not_proven_disabled')
    currency=pool.get('currency')
    if currency not in ('USD','EUR') or budget.get('currency')!=currency:
        raise BillingGateError('billing_currency_missing_or_unsupported')
    initial=strict_decimal(pool.get('initial_budget'),'initial_budget')
    percentage=strict_decimal(pool.get('usage_percentage'),'usage_percentage')
    if initial<=0 or not Decimal(0)<=percentage<=Decimal(100):
        raise BillingGateError('invalid_budget_amount_or_percent')
    remaining=initial*(Decimal(100)-percentage)/Decimal(100)
    if remaining<MIN_INCLUDED_HEADROOM:
        raise BillingGateError('insufficient_included_budget_headroom')
    rows=models.get('data')
    if not isinstance(rows,list) or not any(isinstance(r,dict) and r.get('id')==MODEL for r in rows):
        raise BillingGateError('fixed_model_not_available')
    return {'billing_gate':'PASS', 'account_scope':'authenticated_same_mistral_api_key',
            'api_payg_disabled':True, 'minimum_headroom_met':True,
            'headroom_floor_currency_units':str(MIN_INCLUDED_HEADROOM),
            'currency':currency, 'fixed_model_available':True,
            'max_model_generations_after_admission':RUNS*12,
            'total_token_upper_bound':TOTAL_TOKEN_CAP,
            'fresh_authenticated_account_status':True,
            'formal_quality_f1':None}

class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self,req,fp,code,msg,headers,newurl):
        raise BillingGateError('unexpected_redirect')

def get_json(opener,key,path):
    if path not in ('/v1/billing/subscription','/v1/billing/budget','/v1/models'):
        raise BillingGateError('unsupported_endpoint')
    request=urllib.request.Request(BASE+path,headers={'Authorization':'Bearer '+key,'Accept':'application/json'},method='GET')
    try:
        with opener.open(request,timeout=20) as response:
            if response.status!=200 or response.geturl()!=BASE+path:
                raise BillingGateError('unexpected_http_or_origin')
            raw=response.read(1000001)
            if len(raw)>1000000:
                raise BillingGateError('oversized_billing_response')
            return json.loads(raw)
    except (urllib.error.HTTPError,urllib.error.URLError,TimeoutError) as exc:
        raise BillingGateError('billing_request_failed_'+type(exc).__name__) from exc
    except (ValueError,UnicodeError) as exc:
        raise BillingGateError('bad_billing_json') from exc

def main():
    key=os.environ.get('MISTRAL_API_KEY','')
    if not key:raise BillingGateError('api_key_secret_missing')
    opener=urllib.request.build_opener(NoRedirect())
    subscription=get_json(opener,key,'/v1/billing/subscription')
    budget=get_json(opener,key,'/v1/billing/budget')
    models=get_json(opener,key,'/v1/models')
    result=check_account(subscription,budget,models)
    # Public CI logs: booleans and bounds only; not actual credit or identifiers.
    print(json.dumps(result,sort_keys=True))

if __name__=='__main__':
    try:main()
    except BillingGateError as exc:
        print('billing_gate=HOLD reason='+str(exc))
        raise SystemExit(2)
