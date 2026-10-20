"""Bind the CLI's sanitized diagnostic transcript to complete adapter I/O."""
import json
from scorer import require

def sanitize(value):
    if isinstance(value,dict):
        out={}
        for k,v in value.items():
            key=k.lower()
            hidden=any(x in key for x in ['chain_of_thought','chain-of-thought','internal_reasoning','hidden_reasoning','reasoning_content','thoughts','scratchpad']) or key in ['analysis','reasoning','thinking','thought']
            secret=any(x in key for x in ['authorization','api_key','apikey','access_key','secret','credential','private_key','password'])
            out[k]='[OMITTED_HIDDEN_REASONING]' if hidden else ('[REDACTED]' if secret else sanitize(v))
        return out
    if isinstance(value,list):return [sanitize(v) for v in value]
    if isinstance(value,str):
        # Rust str::lines removes a final newline, and splits LF/CRLF only.
        lines=value.split('\n')
        if lines and lines[-1]=='':lines.pop()
        needles=['authorization:','bearer ','api_key','api-key','apikey','access_key','private_key','password=','password:','credential','secret=','secret:']
        return '\n'.join('[REDACTED]' if any(n in line.lower() for n in needles) else line.removesuffix('\r') for line in lines)
    return value

def bind(trace,events,provider,model):
    require(trace.get('contract')=='reason-natural-diagnostic-trace-v1' and trace.get('provider')==provider and trace.get('model')==model,'trace_contract_or_identity')
    requests=[e for e in trace['events'] if e['kind']=='model_request'];responses=[e for e in trace['events'] if e['kind']=='model_response']
    fullq=[e for e in events if e['kind']=='model_request'];fullr=[e for e in events if e['kind']=='model_response']
    require(len(requests)==len(responses)==len(fullq)==len(fullr)>0,'trace_audit_generation_mismatch')
    for q,r,fq,fr in zip(requests,responses,fullq,fullr):
        require(all(e.get('provider')==provider and e.get('model')==model for e in [q,r]),'trace_model_identity')
        require((q['correlation_id'],q['attempt'],q.get('phase'),q.get('round'))==(r['correlation_id'],r['attempt'],r.get('phase'),r.get('round')),'trace_audit_correlation')
        require(q.get('request')==sanitize(fq['request']),'trace_request_not_actual_adapter_request')
        actual=fr['response'];diagnostic=r.get('response',{})
        require(diagnostic.get('model')==actual['model'] and diagnostic.get('usage')==sanitize(actual['usage']) and diagnostic.get('provider_attempts')==actual['provider_attempts'],'trace_response_usage_not_actual')
        require(diagnostic.get('bytes')==len(actual['text'].encode()),'trace_response_bytes_not_actual')
        if diagnostic.get('structured'):
            try:parsed,end=json.JSONDecoder().raw_decode(actual['text'].lstrip())
            except ValueError as e:raise ValueError('trace_structured_response_not_actual') from e
            require(diagnostic.get('value')==sanitize(parsed),'trace_structured_response_not_actual')
