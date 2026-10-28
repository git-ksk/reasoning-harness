#[cfg(test)]
mod independent_budget_review {
    use super::*;
    use std::sync::{Arc,Mutex};
    use std::collections::VecDeque;
    struct Script { replies: Mutex<VecDeque<Result<reasoning_harness_core::ModelResponse, ModelError>>>, calls: Arc<Mutex<Vec<Option<u32>>>> }
    impl ModelAdapter for Script {
        fn generate<'a>(&'a self, req: ModelRequest) -> std::pin::Pin<Box<dyn std::future::Future<Output=Result<reasoning_harness_core::ModelResponse,ModelError>> + Send + 'a>> {
            self.calls.lock().unwrap().push(req.max_tokens);
            let result=self.replies.lock().unwrap().pop_front().expect("unexpected request");
            Box::pin(async move {result})
        }
    }
    fn response(s:&str, input: Option<u64>, output:Option<u64>, total:Option<u64>, attempts:u32)->Result<reasoning_harness_core::ModelResponse,ModelError>{
      Ok(reasoning_harness_core::ModelResponse{text:s.into(),model:"fixture".into(),usage:ModelUsage{input_tokens:input,output_tokens:output,total_tokens:total},provider_attempts:attempts,finish_reason:None})
    }
    fn generator(replies:Vec<Result<reasoning_harness_core::ModelResponse,ModelError>>)->(LiveGenerator,Arc<Mutex<Vec<Option<u32>>>>){
      let calls=Arc::new(Mutex::new(vec![]));let script=Script{replies:Mutex::new(replies.into()),calls:calls.clone()};
      (LiveGenerator::Test(DiagnosticTestAdapter(Box::new(script))),calls)
    }
    async fn run(g:&LiveGenerator, tracker:&usage_budget::UsageTracker, requested:u32)->Result<(ReasoningCandidate,GenerationObservation),GenerationFailure> {
      let max=tracker.cap_next_model_tokens(requested).unwrap();
      g.generate_traced(&HarnessInput::default(),max,None,"fixture",DiagnosticTraceCall{limits:tracker.generation_limits(),recorder:None,phase:DiagnosticPhase::new("independent",None)}).await
    }
    #[tokio::test] async fn independent_session_remaining_fallback_and_exhaustion(){
      let baseline=usage_budget::UsageTotals{output_tokens:Some(7),total_tokens:Some(10),..Default::default()};
      let mut tracker=usage_budget::UsageTracker::new(usage_budget::UsageBudget{max_output_tokens:Some(10),..Default::default()},Some(baseline)).unwrap();
      let (g,calls)=generator(vec![response("{}",Some(1),Some(2),Some(3),1),response(r#"{"claims":[]}"#,Some(1),Some(1),Some(2),1)]);
      let (_,observation)=run(&g,&tracker,10).await.unwrap();tracker.record_generation(&observation).unwrap();
      assert_eq!(*calls.lock().unwrap(),vec![Some(3),Some(1)]);assert_eq!(tracker.report().session.unwrap().output_tokens,Some(10));
      assert_eq!(tracker.cap_next_model_tokens(10).unwrap_err().failure_class,"usage_budget_exceeded");
    }
    #[tokio::test] async fn independent_unknown_internal_attempt_and_total_overrun_stop_fallback(){
      for (resp,budget,expected) in [
        (response("{}",Some(1),Some(2),Some(3),2),usage_budget::UsageBudget{max_output_tokens:Some(10),..Default::default()},"usage_budget_unmeasurable"),
        (response("{}",Some(9),Some(2),Some(11),1),usage_budget::UsageBudget{max_total_tokens:Some(10),..Default::default()},"usage_budget_exceeded"),
      ] {
        let (g,calls)=generator(vec![resp]);let mut t=usage_budget::UsageTracker::new(budget,None).unwrap();let error=run(&g,&t,10).await.unwrap_err();
        assert_eq!(error.failure_class,expected); assert_eq!(calls.lock().unwrap().len(),1);
        assert_eq!(t.record_failure(&error).unwrap_err().failure_class,expected);
      }
    }
    #[tokio::test] async fn independent_quota_records_unknown_and_does_not_retry(){
      let (g,calls)=generator(vec![Err(ModelError::new(ModelErrorKind::Quota,"quota"))]);
      let mut t=usage_budget::UsageTracker::new(usage_budget::UsageBudget{max_output_tokens:Some(10),..Default::default()},None).unwrap();
      let error=run(&g,&t,10).await.unwrap_err();assert_eq!(error.failure_class,"quota");
      assert_eq!(t.record_failure(&error).unwrap_err().failure_class,"quota");
      assert_eq!(t.cap_next_model_tokens(10).unwrap_err().failure_class,"usage_budget_unmeasurable");
      assert_eq!(calls.lock().unwrap().len(),1);
    }

    #[tokio::test] async fn independent_safety_unknown_usage_blocks_fallback_and_is_recorded(){
      let proposition=Proposition{key:"deployment.safe".into(),value:"true".into()};
      let artifact=ReasoningArtifact{task:"Decide whether deployment is safe.".into(),evidence:vec![reasoning_harness_core::Evidence{id:"e1".into(),source:"fixture".into(),observation:"deployment checks passed".into(),facts:std::collections::BTreeMap::from([("deployment.safe".into(),"true".into())]),metadata:Default::default()}],..Default::default()};
      let rendered=FinalAnswerCandidate{text:"safe".into(),factual_claims:vec![reasoning_harness_core::FinalAnswerClaim{proposition,mode:FinalClaimMode::Grounded}]};
      let baseline=FinalizationResult{status:FinalizationStatus::GroundedAnswer,text:Some("safe".into()),factual_claims:1,covered_claims:1,factual_claim_coverage:1.0,uncovered_propositions:vec![]};
      let (g,calls)=generator(vec![response("{",None,None,None,1)]);
      let mut t=usage_budget::UsageTracker::new(usage_budget::UsageBudget{max_output_tokens:Some(5),..Default::default()},None).unwrap();
      let error=apply_natural_answer_safety(NaturalAnswerSafetyCall{profile:AnswerSafetyProfile::D3SufficiencyV1,generator:&g,model:"fixture",artifact:&artifact,rendered:&rendered,baseline,max_tokens:5,seed:None,render_round:1,usage:&mut t}).await.unwrap_err();
      assert_eq!(error.failure_class,"usage_budget_unmeasurable");assert_eq!(calls.lock().unwrap().len(),1);
      assert_eq!(t.report().run.output_tokens,None);assert_eq!(t.report().run.provider_attempts,1);
    }
}
