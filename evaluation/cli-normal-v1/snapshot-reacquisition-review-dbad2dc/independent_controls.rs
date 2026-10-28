// Independently authored cfg(test) controls appended only to a private source export.
fn independent_nonempty_fixture(mixed:bool)->(HarnessInput,ReasoningCandidate,ExternalEvidenceAdmissionPolicy,HarnessOutcome) {
    let (mut input,_,_)=first_admission();
    let p=Proposition {key:"unit.value".into(),value:"4".into()};input.hypotheses=vec![p.clone()];
    let mut candidate=ReasoningCandidate {claims:vec![reasoning_harness_core::CandidateClaim {
        id:"existing-supported-claim".into(),statement:"unit.value=4".into(),proposed_state:reasoning_harness_core::EpistemicState::Assumed,
        proposition:Some(p),evidence_ids:vec!["snapshot-1".into()],
    }],..Default::default()};
    if mixed {candidate.claims.push(reasoning_harness_core::CandidateClaim {id:"existing-unresolved-claim".into(),statement:"A separate proposition remains unresolved".into(),proposed_state:reasoning_harness_core::EpistemicState::Unknown,proposition:None,evidence_ids:vec![]});}
    let admission=ExternalEvidenceAdmissionPolicy::new(config());let prepared=prepare_external_resolution_input(input.clone(),&admission).unwrap();
    let before=reasoning_harness_core::GroundingPipeline::run(&StandardGroundingPipeline,prepared,candidate.clone(),&[]).unwrap();
    assert!(!before.artifact.claims.is_empty());assert!(!before.artifact.verification_receipts.is_empty(),"fixture must contain actual verification receipts");
    (input,candidate,admission,before)
}
#[test]
fn independent_nonempty_claims_and_receipts_are_identical_after_noop_and_replay() {
    for mixed in [false,true] {
        let (input,candidate,admission,before)=independent_nonempty_fixture(mixed);
        let before_bytes=serde_json::to_vec(&before.artifact).unwrap();let resolver=Resolver::new(vec![raw()]);
        let first=run_investigation_resolution(input,candidate.clone(),&resolver,Some(&admission),request("different-planned-target")).unwrap();
        assert_eq!(resolver.calls.load(Ordering::SeqCst),1);assert_eq!(first.attempts.len(),1);assert_eq!(first.attempts[0].status,ResolutionAttemptStatus::NoResult);
        assert!(first.attempts[0].admitted_evidence_ids.is_empty());assert_eq!(first.attempts[0].verification_receipts,0);
        assert_eq!(first.initial_verdict,before.verdict);assert_eq!(first.final_verdict,before.verdict);assert_eq!(serde_json::to_vec(&first.final_artifact).unwrap(),before_bytes);
        let again=repeat(input_from_artifact(&first.final_artifact),candidate,vec![raw()],Some(&admission));
        assert_eq!(again.attempts[0].status,ResolutionAttemptStatus::NoResult);assert_eq!(serde_json::to_vec(&again.final_artifact).unwrap(),before_bytes);
        for outcome in [&first,&again] {assert_eq!(outcome.usage.calls,1);assert_eq!(outcome.usage.added_tokens,7);assert_eq!(outcome.attempts[0].cost.cost_microusd,Some(4));assert!(outcome.attempts[0].admitted_evidence_ids.is_empty());}
        println!("INDEPENDENT_SNAPSHOT_CONTROL {}",serde_json::json!({"case":"nonempty_actual_run_path","mixed_unresolved_claim":mixed,"existing_claims":before.artifact.claims.len(),"existing_receipts":before.artifact.verification_receipts.len(),"initial_verdict":before.verdict,"final_verdict":first.final_verdict,"artifact_bytes_unchanged":true,"replay_equal":true,"new_admissions":0,"new_receipts_by_attempt":0,"resolver_calls_each":1,"cost_microusd_each":4}));
    }
}
#[test]
fn independent_policy_revocation_and_full_metadata_difference_cannot_become_noop() {
    let (input,_,_)=first_admission();let existing=&input.evidence;
    let mut policies=Vec::new();let mut revoked=config();revoked.sources.clear();policies.push(("source_revoked",revoked));
    let mut shorter=config();shorter.sources.get_mut("source-1").unwrap().max_age_seconds=59;policies.push(("currently_fresh_but_different_effective_interval",shorter));
    let mut stronger=config();stronger.authority_policy.ranks.insert("stronger".into(),30);stronger.minimum_authority_class=Some("stronger".into());policies.push(("minimum_authority_raised",stronger));
    let mut scope=config();scope.required_scope=Some(BTreeMap::from([("region".into(),ScopeCoverage::Values {values:BTreeSet::from(["expected".into()])})]));policies.push(("new_required_scope",scope));
    for (name,cfg) in policies {let admission=ExternalEvidenceAdmissionPolicy::new(cfg);let resolver=Resolver::new(vec![raw()]);let wrapper=AcquisitionOnlyResolver {inner:&resolver,existing_evidence:existing,admission:Some(&admission)};let response=wrapper.resolve(&request("second"),0).unwrap();assert!(matches!(response.contribution,ResolutionResolverContribution::AcquiredEvidence {..}),"{name}");assert_eq!(response.cost.calls,1);assert_eq!(resolver.calls.load(Ordering::SeqCst),1);println!("INDEPENDENT_SNAPSHOT_CONTROL {}",serde_json::json!({"case":name,"not_suppressed":true,"cost_kept":true}));}
    let policy=ExternalEvidenceAdmissionPolicy::new(config());
    for changed in ["temporal","provenance"] {let mut altered=existing.clone();if changed=="temporal" {altered[0].metadata.temporal.as_mut().unwrap().effective_until_unix_seconds=Some(1061);}else {altered[0].metadata.provenance_class=Some("other".into());}let resolver=Resolver::new(vec![raw()]);let wrapper=AcquisitionOnlyResolver {inner:&resolver,existing_evidence:&altered,admission:Some(&policy)};assert!(matches!(wrapper.resolve(&request("second"),0).unwrap().contribution,ResolutionResolverContribution::AcquiredEvidence {..}));println!("INDEPENDENT_SNAPSHOT_CONTROL {}",serde_json::json!({"case":format!("existing_metadata_{changed}_different"),"not_suppressed":true}));}
    let mut future=raw();future.acquisition_metadata.retrieved_at_unix_seconds=Some(1011);let resolver=Resolver::new(vec![future]);let wrapper=AcquisitionOnlyResolver {inner:&resolver,existing_evidence:existing,admission:Some(&policy)};assert!(matches!(wrapper.resolve(&request("second"),0).unwrap().contribution,ResolutionResolverContribution::AcquiredEvidence {..}));
}
struct IndependentFailureResolver {mode:usize,calls:AtomicUsize}
impl ResolutionResolver for IndependentFailureResolver {
    fn name(&self)->&'static str {"fixture_resolver"}
    fn class(&self)->ResolverClass {ResolverClass::EvidenceAcquisition}
    fn config_id(&self)->Option<&str> {Some("failure-config")}
    fn resolve(&self,_:&ResolutionRequest,_:usize)->Result<ResolutionResolverOutput,ResolutionAdapterError> {
        self.calls.fetch_add(1,Ordering::SeqCst);let cost=ResolutionCost {calls:1,added_tokens:7,elapsed_ms:3,cost_microusd:Some(4)};
        if self.mode==0 {return Err(ResolutionAdapterError {kind:reasoning_harness_core::ResolutionAdapterErrorKind::Timeout,cost});}
        Ok(ResolutionResolverOutput {cost,contribution:if self.mode==1 {ResolutionResolverContribution::CandidateRevision {candidate:ReasoningCandidate::default()}}else {ResolutionResolverContribution::HumanReviewRequired}})
    }
}
#[test]
fn independent_cached_evidence_cannot_hide_inner_error_or_revision_and_cost_is_kept() {
    let (input,candidate,_)=first_admission();let admission=ExternalEvidenceAdmissionPolicy::new(config());
    for mode in 0..3 {let resolver=IndependentFailureResolver {mode,calls:AtomicUsize::new(0)};let wrapper=AcquisitionOnlyResolver {inner:&resolver,existing_evidence:&input.evidence,admission:Some(&admission)};let error=wrapper.resolve(&request("second"),0).unwrap_err();assert_eq!(resolver.calls.load(Ordering::SeqCst),1);assert_eq!(error.cost.calls,1);assert_eq!(error.cost.cost_microusd,Some(4));assert_eq!(error.kind,if mode==0 {reasoning_harness_core::ResolutionAdapterErrorKind::Timeout}else {reasoning_harness_core::ResolutionAdapterErrorKind::PolicyDenied});println!("INDEPENDENT_SNAPSHOT_CONTROL {}",serde_json::json!({"case":"inner_failure_or_disallowed_contribution","mode":mode,"error_kind":format!("{:?}",error.kind),"calls":1,"cost_microusd":4}));}
    let resolver=IndependentFailureResolver {mode:0,calls:AtomicUsize::new(0)};let result=run_investigation_resolution(input,candidate,&resolver,Some(&admission),request("second")).unwrap();assert_eq!(result.attempts[0].status,ResolutionAttemptStatus::TimedOut);assert_eq!(result.usage.calls,1);assert_eq!(result.usage.added_tokens,7);
}
