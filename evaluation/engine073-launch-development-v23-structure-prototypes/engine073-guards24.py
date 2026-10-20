from pathlib import Path
import subprocess
p=Path('crates/reasoning-harness-core/src/evidence_relevance.rs');s=p.read_text()
a=s.index('fn launch_target_nonexistence_v17(');b=s.index('// The event owner inside the quotation',a);v=s[a:b];needle='''    let text = normalized(value);
    target_identity_phrases(policy)''';assert needle in v
v=v.replace(needle,'''    let text = normalized(value);
    // A finite discourse adjunct does not turn an actual existence denial
    // into a correction of the preceding printed/physical property.
    let text = ["separately ","independently ","moreover ","however ","also ","additionally ","furthermore ","in fact "].iter().find_map(|prefix|text.strip_prefix(prefix)).unwrap_or(&text);
    target_identity_phrases(policy)''',1);s=s[:a]+v+s[b:]
helper=r'''
// Reply labels are syntactic wrappers, not trust authority. Every complete
// same-role owner declaration inside one participates in uniqueness checks.
fn launch_owner_reply_label_v17(label: &str) -> bool {
    let text=normalized(label);let words=text.split_whitespace().collect::<Vec<_>>();
    words==["reply"] || matches!(words.as_slice(), [kind,"reply"] if ["author","editor","publisher","first","second","third","fourth","fifth","additional","another","alternative","competing","further","subsequent"].contains(kind))
}

'''
s=s.replace('fn launch_typed_objects_v17(',helper+'fn launch_typed_objects_v17(',1)
needle='''if ["author reply", "editor reply", "publisher reply"]
                        .contains(&normalized(label).as_str())''';assert needle in s;s=s.replace(needle,'if launch_owner_reply_label_v17(label)',1)
anchor='mod tests {\n    use super::*;';s=s.replace(anchor,anchor+r'''
    #[test]
    fn v30_prefixed_existence_denial_and_competing_reply_cannot_borrow_property_scope() {
        let policy=launch_policy_for_v28();let proposal=EvidenceRelevanceBindingProposal {target_binding:EvidenceRelevanceBinding::Exact,relation_binding:EvidenceRelevanceBinding::Exact};let raw=EvidenceLocalQualificationV6 {identity_scope:EvidenceLocalIdentityScope::ExactTarget,relation_scope:EvidenceLocalRelationScope::RequestedRelation,scope_risk:EvidenceLocalBlockingReason::None};
        let base="Item: the claim that the line \"Silver Lens does not exist\" on the Cedar Basket label ends with a period. I retract the Item. Silver Lens was launched.";
        for prefix in ["Separately, ","Independently, ","Moreover, ","However, "] {
            let text=format!("{base} {prefix}Silver Lens does not exist; this is our assertion about the product.");let local=candidate(vec![(EvidenceRelevanceSignalKind::Excerpt,&text)]);
            let q=derive_effective_evidence_local_qualification_v17(&policy,&local,Some(&proposal),Some(&raw)).unwrap();assert_ne!(q.relation_scope,EvidenceLocalRelationScope::RequestedRelation,"{text}");assert_ne!(materialize_evidence_relevance_v30(&policy,&local,Some(&proposal),Some(&raw)).unwrap().disposition,EvidenceRelevanceDisposition::Relevant,"{text}");
        }
        for reply in ["Second reply","Another reply","Competing reply","Reply"] {
            let text=format!("Item: the launch claim. I withdraw the Item. Author reply: owner of the Item is Violet Bell. {reply}: owner of the Item is Silver Lens. Object: the claim that the Violet Bell carton has a woven pull tab. We deny the Object. Silver Lens was launched.");let local=candidate(vec![(EvidenceRelevanceSignalKind::Excerpt,&text)]);
            assert_ne!(derive_effective_evidence_local_qualification_v17(&policy,&local,Some(&proposal),Some(&raw)).unwrap().relation_scope,EvidenceLocalRelationScope::RequestedRelation,"{text}");assert_ne!(materialize_evidence_relevance_v30(&policy,&local,Some(&proposal),Some(&raw)).unwrap().disposition,EvidenceRelevanceDisposition::Relevant,"{text}");
        }
        let local=candidate(vec![(EvidenceRelevanceSignalKind::Excerpt,base)]);assert_eq!(materialize_evidence_relevance_v30(&policy,&local,Some(&proposal),Some(&raw)).unwrap().disposition,EvidenceRelevanceDisposition::Relevant);
    }
''',1);p.write_text(s);subprocess.run(['cargo','fmt','--all'],check=True)
