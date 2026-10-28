#[cfg(test)]
mod independent_redirect_review {
use super::*;
use reasoning_harness_core::{ModelAdapter,ModelOutputFormat,ModelRequest};
use std::{io::{Read,Write},net::TcpListener,thread,time::{Duration,Instant},sync::{Arc,atomic::{AtomicUsize,Ordering}}};
#[tokio::test]
async fn independently_probe_redirects_with_single_attempt() {
  let listener=TcpListener::bind("127.0.0.1:0").unwrap();let addr=listener.local_addr().unwrap();listener.set_nonblocking(true).unwrap();
  let count=Arc::new(AtomicUsize::new(0));let n=count.clone();
  let server=thread::spawn(move || {
    let deadline=Instant::now()+Duration::from_secs(5);
    while Instant::now()<deadline && n.load(Ordering::SeqCst)<2 {
      match listener.accept(){Ok((mut stream,_))=>{
       stream.set_read_timeout(Some(Duration::from_secs(2))).unwrap();let mut buf=[0u8;4096];let _=stream.read(&mut buf);
       let index=n.fetch_add(1,Ordering::SeqCst);
       if index==0 { write!(stream,"HTTP/1.1 307 Temporary Redirect\r\nLocation: http://{addr}/redirected\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap(); }
       else { let body=r#"{"model":"test-model","status":"completed","steps":[{"type":"model_output","content":[{"type":"text","text":"ok"}]}],"usage":{"total_input_tokens":2,"total_output_tokens":1,"total_tokens":3}}"#;
         write!(stream,"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).unwrap(); }
      },Err(e) if e.kind()==std::io::ErrorKind::WouldBlock=>thread::sleep(Duration::from_millis(10)),Err(e)=>panic!("{e}")}
    }
  });
  let mut adapter=GoogleAdapter::with_base_url("test-key","test-model",&format!("http://{addr}/v1/")).unwrap();adapter.set_single_attempt(true);
  let result=adapter.generate(ModelRequest{task:"fixture".into(),system:None,output_format:ModelOutputFormat::Text,max_tokens:Some(8),random_seed:None,reasoning_preference:None}).await;
  server.join().unwrap();println!("physical_http_requests={} logical_attempts={:?}",count.load(Ordering::SeqCst),result.as_ref().map(|r|r.provider_attempts));
  assert_eq!(count.load(Ordering::SeqCst),1);assert_eq!(result.unwrap_err().provider_attempts,1);
}
}
