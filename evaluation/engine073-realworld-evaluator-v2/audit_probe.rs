//! Offline loopback-only integration probe for the exact audited provider libs.
use reasoning_harness_core::{ModelAdapter,ModelRequest,ModelOutputFormat};
use reasoning_harness_providers::{MistralAdapter,GoogleAdapter,GroqAdapter};
use serde_json::json;

#[tokio::main]
async fn main() {
    let args=std::env::args().collect::<Vec<_>>();let provider=&args[1];let base=&args[2];let mode=&args[3];
    assert!(base.starts_with("http://127.0.0.1:"));
    let adapter:Box<dyn ModelAdapter>=match provider.as_str() {
        "mistral"=>Box::new(MistralAdapter::with_base_url("inert-loopback-only", "fixture-model", base).unwrap()),
        "google"=>Box::new(GoogleAdapter::with_base_url("inert-loopback-only", "fixture-model", base).unwrap()),
        "groq"=>Box::new(GroqAdapter::evaluation_loopback("inert-loopback-only", "fixture-model", base).unwrap()),
        _=>panic!("invalid loopback fixture provider"),
    };
    let request=ModelRequest { task:if mode=="oversize" { "x".repeat(12001) } else { "Engineering guard control.".into() },system:None,
        output_format:ModelOutputFormat::Text,max_tokens:Some(32),random_seed:Some(1),reasoning_preference:None };
    let first=adapter.generate(request.clone()).await;
    let second=if mode=="missing-usage" { Some(adapter.generate(request).await) } else { None };
    println!("{}",json!({"first_ok":first.is_ok(),"first_error":first.err().map(|e|format!("{:?}",e.kind)),
        "second_ok":second.as_ref().map(|r|r.is_ok()),"second_error":second.and_then(|r|r.err()).map(|e|format!("{:?}",e.kind)),"loopback_only":true}));
}
