use reasoning_harness_core::*;
use serde_json::json;
fn artifact(rows: &[(&str, &str)]) -> ReasoningArtifact {
    let mut a = ReasoningArtifact { task: "audit source-local statements".into(), ..Default::default() };
    for (i,(target,statement)) in rows.iter().enumerate() {
        if !a.source_attribution.targets.iter().any(|p|p.target_id==*target) {
            a.source_attribution.targets.push(SourceAttributionTargetPolicy {policy_id:format!("policy-{target}"),target_id:(*target).into(),target_question:"What does each source say?".into(),authority_ceiling:SourceAttributionAuthorityCeiling::SourceLocal,hard_verification_required:false});
        }
        let eid=format!("e{i}");let sid=format!("s{i}");let bid=format!("b{i}");
        a.evidence.push(Evidence {id:eid.clone(),source:sid.clone(),observation:(*statement).into(),facts:Default::default(),metadata:EvidenceMetadata::default()});
        a.source_attribution.bindings.push(SourceAttributionBinding {id:bid.clone(),target_id:(*target).into(),evidence_id:eid,source_id:sid,source_url:None,locator:None,retrieved_at_unix_seconds:Some(1800000000),source_version:Some("audit-r1".into()),span:SourceTextSpan {start_byte:0,end_byte:statement.len()}});
        let (_,claim)=materialize_source_attributed_claim(&a,format!("c{i}"),None,&SourceAttributionProposal {target_id:(*target).into(),binding_ids:vec![bid],transform_kind:SourceAttributionTransformKind::ExactQuote,transformed_statement:None,source_language:Some("en".into()),output_language:Some("en".into())},None).unwrap();
        append_source_attributed_claim(&mut a,None,claim).unwrap();
    }
    assert!(validate_source_attribution_state(&a).is_empty());a
}
fn check(id:&str,ok:bool){println!("{}",json!({"id":id,"pass":ok}));assert!(ok,"{id}");}
fn main(){
 let auth=TrustedSourceReviewAuthority::new("audit-host").unwrap();
 let a=artifact(&[("A","Lumen Cache supports streaming."),("A","Lumen Cache provides streaming."),("B","Maple Queue supports exporting."),("B","Maple Queue does not support exporting.")]);
 let before=serde_json::to_vec(&a).unwrap();let ids=vec!["A".into(),"B".into()];
 let r=record_trusted_source_equivalence(&a,&auth,"c0","c1").unwrap();
 let view=reconcile_source_attributed_targets(&a,&ids,Some(&auth),&[r.clone()]).unwrap();
 check("mixed-target-A-reviewed-B-conflict",view.target_answers[0].status==SourceReconciliationStatus::ReviewedCompatible && view.target_answers[1].status==SourceReconciliationStatus::Conflict && view.original.status==SourceAttributionFinalizationStatus::Conflict);
 check("original-artifact-byte-preserved",serde_json::to_vec(&a).unwrap()==before);
 check("no-hard-truth-claim-created",a.claims.is_empty() && a.verification_receipts.is_empty());
 check("review-without-authority-rejected",reconcile_source_attributed_targets(&a,&ids,None,&[r.clone()]).is_err());
 let other=TrustedSourceReviewAuthority::new("other-host").unwrap();
 check("wrong-host-rejected",reconcile_source_attributed_targets(&a,&ids,Some(&other),&[r.clone()]).is_err());
 check("negative-polarity-review-rejected",record_trusted_source_equivalence(&a,&auth,"c2","c3").is_err());
 check("cross-target-review-rejected",record_trusted_source_equivalence(&a,&auth,"c0","c2").is_err());
 let mut changed=a.clone();changed.evidence[0].observation.push_str(" Archive updated.");
 check("changed-unquoted-source-tail-invalidates-review",reconcile_source_attributed_targets(&changed,&ids,Some(&auth),&[r.clone()]).is_err());
 let mut changed=a.clone();changed.source_attribution.bindings[0].source_version=Some("audit-r2".into());
 check("changed-source-version-invalidates-review",reconcile_source_attributed_targets(&changed,&ids,Some(&auth),&[r.clone()]).is_err());
 let mut changed=a.clone();changed.source_attribution.targets[0].hard_verification_required=true;
 check("new-hard-verification-floor-invalidates-review",reconcile_source_attributed_targets(&changed,&ids,Some(&auth),&[r.clone()]).is_err());
 for (id,kind) in [("tampered-review-contract",0),("tampered-review-source",1),("tampered-review-target",2),("empty-review-evidence-no-panic",3),("tampered-review-binding",4)] {
  let mut rr=r.clone();match kind {0=>rr.contract_id="different-contract".into(),1=>rr.first.evidence[0].observation="forged".into(),2=>rr.first.target_policy.target_question="different question".into(),3=>rr.first.evidence.clear(),_=>rr.first.bindings[0].source_url=Some("https://forged.invalid".into())}
  check(id,reconcile_source_attributed_targets(&a,&ids,Some(&auth),&[rr]).is_err());
 }
 check("duplicate-target-rejected",reconcile_source_attributed_targets(&a,&["A".into(),"A".into()],Some(&auth),&[r.clone()]).is_err());
 check("duplicate-review-rejected",reconcile_source_attributed_targets(&a,&ids,Some(&auth),&[r.clone(),r.clone()]).is_err());
 check("unselected-target-review-rejected",reconcile_source_attributed_targets(&a,&["B".into()],Some(&auth),&[r.clone()]).is_err());
 check("too-many-reviews-rejected",reconcile_source_attributed_targets(&a,&ids,Some(&auth),&vec![r.clone();1025]).is_err());
 let restored:ReasoningArtifact=serde_json::from_slice(&before).unwrap();let rr:TrustedSourceCompatibilityReview=serde_json::from_slice(&serde_json::to_vec(&r).unwrap()).unwrap();
 let replay=reconcile_source_attributed_targets(&restored,&ids,Some(&auth),&[rr]).unwrap();
 check("serialized-replay-output-identical",replay==view && serde_json::to_vec(&restored).unwrap()==before);
 let tri=artifact(&[("A","Lumen Cache supports streaming."),("A","Lumen Cache provides streaming."),("A","Lumen Cache offers streaming.")]);
 let r01=record_trusted_source_equivalence(&tri,&auth,"c0","c1").unwrap();let r12=record_trusted_source_equivalence(&tri,&auth,"c1","c2").unwrap();
 let partial=reconcile_source_attributed_targets(&tri,&["A".into()],Some(&auth),&[r01.clone(),r12.clone()]).unwrap();
 check("no-transitive-review-closure",partial.target_answers[0].status==SourceReconciliationStatus::Conflict);
 let r02=record_trusted_source_equivalence(&tri,&auth,"c0","c2").unwrap();let complete=reconcile_source_attributed_targets(&tri,&["A".into()],Some(&auth),&[r01,r12,r02]).unwrap();
 check("complete-pair-review-keeps-original-conflict",complete.target_answers[0].status==SourceReconciliationStatus::ReviewedCompatible && complete.original.status==SourceAttributionFinalizationStatus::Conflict);
}
