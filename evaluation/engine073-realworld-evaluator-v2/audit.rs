//! Evaluation-only pre-request bounds and complete model I/O capture.
//! Identical in both exported CLI hosts. No admission or semantic decisions.
use std::{fs::OpenOptions, io::Write, sync::Mutex, time::{Duration, Instant}};
use reasoning_harness_core::{ModelError, ModelErrorKind, ModelRequest, ModelResponse};
use serde_json::json;

struct State { generations:u64, last:Option<Instant>, input:u64, output:u64, inflight:bool, blocked:bool, reserved_input:u64, reserved_output:u64, wire_sent:bool, http_seen:bool, provider:String, model:String }
static STATE: Mutex<State> = Mutex::new(State { generations:0,last:None,input:0,output:0,inflight:false,blocked:false,reserved_input:0,reserved_output:0,wire_sent:false,http_seen:false,provider:String::new(),model:String::new() });

fn record(value:serde_json::Value) -> Result<(), ModelError> {
    let path=std::env::var("ENGINE073_MODEL_JOURNAL").map_err(|_|ModelError::new(ModelErrorKind::Protocol,"evaluation journal missing"))?;
    let mut file=OpenOptions::new().create(true).append(true).open(path).map_err(|_|ModelError::new(ModelErrorKind::Protocol,"evaluation journal unavailable"))?;
    writeln!(file,"{}",value).and_then(|_|file.sync_all()).map_err(|_|ModelError::new(ModelErrorKind::Protocol,"evaluation journal not durable"))
}

pub fn before(provider:&str,model:&str,request:&ModelRequest) -> Result<u64, ModelError> {
    let cap=std::env::var("ENGINE073_GENERATION_CAP").ok().and_then(|s|s.parse::<u64>().ok()).filter(|v|[1,24].contains(v)).ok_or_else(||ModelError::new(ModelErrorKind::Protocol,"evaluation generation cap missing"))?;
    let bytes=serde_json::to_vec(request).map_err(|_|ModelError::new(ModelErrorKind::Protocol,"request serialization failed"))?.len();
    let mut state=STATE.lock().unwrap();
    let input_cap=if cap==1 { 4096 } else { 24000 };
    let output_cap=if cap==1 { 384 } else { 4096 };
    let request_output=u64::from(request.max_tokens.unwrap_or(u32::MAX));
    if state.blocked || state.inflight || state.generations>=cap || bytes>12000 || !matches!(request.max_tokens,Some(1..=1024))
        || state.input + bytes as u64 + 512 > input_cap || state.output + request_output > output_cap {
        record(json!({"kind":"pre_request_refusal","provider":provider,"model":model,"request_bytes":bytes,"completed_or_reserved_generations":state.generations}))?;
        state.blocked=true;
        return Err(ModelError::new(ModelErrorKind::Protocol,"evaluation pre-request bound exceeded"));
    }
    if let Some(last)=state.last { std::thread::sleep(Duration::from_millis(10500).saturating_sub(last.elapsed())); }
    state.generations+=1;state.last=Some(Instant::now());state.inflight=true;state.reserved_input=bytes as u64+512;state.reserved_output=request_output;state.wire_sent=false;state.http_seen=false;state.provider=provider.to_owned();state.model=model.to_owned();
    record(json!({"kind":"model_request","generation":state.generations,"provider":provider,"model":model,"request":request,"request_bytes":bytes,
                  "reserved_input_token_upper_bound":bytes+512,"reserved_output_token_upper_bound":request_output,"reserved_attempts":1}))?;
    Ok(state.generations)
}

pub fn after(generation:u64,result:&Result<ModelResponse,ModelError>) -> Result<(), ModelError> {
    match result {
        Ok(response)=>record(json!({"kind":"model_response","generation":generation,"response":response})),
        Err(error)=>record(json!({"kind":"model_failure","generation":generation,"failure_class":format!("{:?}",error.kind),"provider_attempts":error.provider_attempts})),
    }?;
    let mut state=STATE.lock().unwrap();state.inflight=false;
    if let Ok(response)=result {
        if generation!=state.generations || !state.wire_sent || !state.http_seen || response.model!=state.model { state.blocked=true;return Err(ModelError::new(ModelErrorKind::Protocol,"evaluation response not bound to HTTP")); }
        if let (Some(input),Some(output),Some(total))=(response.usage.input_tokens,response.usage.output_tokens,response.usage.total_tokens) {
            if total==input+output && response.provider_attempts==1 && input<=state.reserved_input && output<=state.reserved_output {
                state.input+=input;state.output+=output;return Ok(());
            }
        }
    }
    state.blocked=true;
    if result.is_ok() { Err(ModelError::new(ModelErrorKind::Protocol,"evaluation usage incomplete")) } else { Ok(()) }
}

pub fn wire<T:serde::Serialize>(provider:&str,body:&T) -> Result<(),ModelError> {
    let value=serde_json::to_value(body).map_err(|_|ModelError::new(ModelErrorKind::Protocol,"wire serialization failed"))?;
    let bytes=serde_json::to_vec(&value).unwrap().len() as u64;
    let mut state=STATE.lock().unwrap();
    if state.blocked || !state.inflight || state.wire_sent || provider!=state.provider || value.get("model").and_then(|v|v.as_str())!=Some(state.model.as_str()) || bytes>state.reserved_input {
        state.blocked=true;
        return Err(ModelError::new(ModelErrorKind::Protocol,"unreserved or excessive evaluation HTTP request"));
    }
    record(json!({"kind":"wire_request","generation":state.generations,"provider":provider,"body":value,"wire_bytes":bytes}))?;
    state.wire_sent=true;Ok(())
}

pub fn wire_response(provider:&str,status:u16,headers:&reqwest::header::HeaderMap) -> Result<(),ModelError> {
    let allowed=["x-ratelimit-limit-requests","x-ratelimit-limit-tokens","x-ratelimit-remaining-requests","x-ratelimit-remaining-tokens","x-ratelimit-reset-requests","x-ratelimit-reset-tokens","retry-after"];
    let mut values=serde_json::Map::new();
    for key in allowed { if let Some(value)=headers.get(key).and_then(|v|v.to_str().ok()) { values.insert(key.into(),json!(value)); } }
    let mut state=STATE.lock().unwrap();
    if !state.inflight || !state.wire_sent || state.http_seen || provider!=state.provider { state.blocked=true;return Err(ModelError::new(ModelErrorKind::Protocol,"unbound evaluation HTTP response")); }
    state.http_seen=true;
    record(json!({"kind":"wire_http_response","generation":state.generations,"provider":provider,"status":status,"rate_headers":values}))
}

/// A credential-inert, numeric-loopback-only constructor hook for the SAME CLI
/// executable's offline controls. Live model identities cannot use this hook.
pub fn loopback_base(api_key:&str,model:&str) -> Result<Option<String>,ModelError> {
    let Ok(value)=std::env::var("ENGINE073_LOOPBACK_BASE_URL") else {return Ok(None)};
    let parsed=reqwest::Url::parse(&value).map_err(|_|ModelError::new(ModelErrorKind::Protocol,"invalid engineering loopback URL"))?;
    if api_key!="inert-loopback-only" || model!="fixture-model" || parsed.scheme()!="http"
        || parsed.host_str()!=Some("127.0.0.1") || parsed.port().is_none()
        || !parsed.username().is_empty() || parsed.password().is_some() || parsed.query().is_some() || parsed.fragment().is_some() {
        return Err(ModelError::new(ModelErrorKind::Protocol,"engineering loopback identity refused"));
    }
    Ok(Some(value))
}
