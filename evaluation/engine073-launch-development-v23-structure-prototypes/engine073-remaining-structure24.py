from pathlib import Path
import subprocess
p=Path('crates/reasoning-harness-core/src/evidence_relevance.rs');s=p.read_text()
a=s.index('fn launch_owned_appearance_subject_v17(');b=s.index('fn launch_owned_component_predicate_v17(',a);v=s[a:b];v=v.replace('"page", "owner"','"page", "owner", "reference"');s=s[:a]+v+s[b:]
# Retain the quoted event owner, allowing consistency checks against role frames.
a=s.index('fn launch_complete_quoted_foreign_event_v17(');b=s.index('// An adjacent full-name explanation',a);v=s[a:b];v=v.replace('fn launch_complete_quoted_foreign_event_v17(','fn launch_complete_quoted_foreign_event_owner_v17(',1).replace(') -> bool {',') -> Option<String> {',1).replace('return false;','return None;')
i=v.index('    owner.is_some_and(');v=v[:i]+v[i:].replace('    owner.is_some_and(','    if owner.is_some_and(',1);needle='''                .is_some_and(|owner| launch_owned_appearance_subject_v17(policy, owner)))
}'''
if needle not in v:
 print(v[-900:]);raise SystemExit('quote owner expected end not found; no source written')
v=v.replace(needle,'''                .is_some_and(|owner| launch_owned_appearance_subject_v17(policy, owner))) { owner.map(normalized) } else { None }
}''',1)
v+='''fn launch_complete_quoted_foreign_event_v17(policy: &EvidenceRelevanceTargetPolicy, value: &str) -> bool {
    launch_complete_quoted_foreign_event_owner_v17(policy,value).is_some()
}

''';s=s[:a]+v+s[b:]
needle='''            let mut scope = launch_typed_object_scope_v17(policy, value);
            let text = normalized(value);''';assert needle in s;s=s.replace(needle,'''            let mut scope = launch_typed_object_scope_v17(policy, value);
            if let Some(event_owner)=launch_complete_quoted_foreign_event_owner_v17(policy,value) {
                if owners.iter().filter(|(role,_)|role.is_none() || *role==Some(label.as_str())).any(|(_,owner)|owner.as_ref().is_none_or(|(name,scope)|*scope!=Scope::Excluded || *name!=event_owner)) {
                    scope=Scope::Unresolved;
                }
            }
            let text = normalized(value);''',1)
# A source's annotation alone is never a proof: require the adjacent named role,
# independently complete physical type and a matching local property identifier.
needle='''    let note = normalized(statement);
    if let [''';a=s.index('fn launch_bound_property_reference_v17(');j=s.index(needle,a)
s=s[:j]+s[j:].replace(needle,'''    let note = normalized(statement);
    if let Some(property)=note.strip_prefix("no event is the object of this ").and_then(|body|body.strip_suffix(" correction")) {
        let property=property.split_whitespace().collect::<Vec<_>>();
        let prior=normalized(&previous.raw);let prior=prior.split_whitespace().collect::<Vec<_>>();
        if let ["we"|"i",verb,"the",role @ ("object"|"subject"|"item")]=prior.as_slice() {
            if launch_retraction_word_v17(verb) && (2..=4).contains(&property.len()) && property.iter().all(|word|!launch_scope_or_reference_word_v17(word))
                && objects.iter().any(|object|object.label==*role && object.scope==LaunchTypedScopeV17::Excluded && !object.foreign_event && property.iter().all(|word|normalized(&object.declared_value).split_whitespace().any(|part|part==*word))) { return true; }
        }
    }
    if let [''',1)
# Positive/negative role-owner consistency for the newly admitted bare quote.
needle='''            (
                "Object: the claim \\"Violet Bell was launched\\". I withdraw the Object.",'''
anchor='mod tests {\n    use super::*;';s=s.replace(anchor,anchor+r'''
    #[test]
    fn v30_property_explanation_and_quoted_owner_need_independent_matching_role() {
        let policy=launch_policy_for_v28();let proposal=EvidenceRelevanceBindingProposal {target_binding:EvidenceRelevanceBinding::Exact,relation_binding:EvidenceRelevanceBinding::Exact};let raw=EvidenceLocalQualificationV6 {identity_scope:EvidenceLocalIdentityScope::ExactTarget,relation_scope:EvidenceLocalRelationScope::RequestedRelation,scope_risk:EvidenceLocalBlockingReason::None};
        for (context,ok) in [("Item: the claim that the Violet Bell pamphlet has a cut-out window. I retract the Item. No event is the object of this pamphlet-window correction.",true),("Item: the claim that the Violet Bell reference sheet has a punched top edge. I retract the Item.",true),("Item: the launch claim. I retract the Item. No event is the object of this pamphlet-window correction.",false),("Item: the claim that Silver Lens was launched. I retract the Item. No event is the object of this pamphlet-window correction.",false),("Object: the claim \"Violet Bell was launched\". Owner of the Object: Silver Lens. I retract the Object.",false),("Object: the claim \"Violet Bell was launched\". Owner of the Object: Cedar Basket. I retract the Object.",false),("Object: the claim \"Violet Bell was launched\". Owner of the Object: Violet Bell. I retract the Object.",true),("Object: the claim \"Violet Bell was launched\". Owner of the Subject: Cedar Basket. I retract the Object.",true)] {
            let text=format!("{context} Silver Lens was launched.");let local=candidate(vec![(EvidenceRelevanceSignalKind::Excerpt,&text)]);
            let q=derive_effective_evidence_local_qualification_v17(&policy,&local,Some(&proposal),Some(&raw)).unwrap();assert_eq!(q.relation_scope==EvidenceLocalRelationScope::RequestedRelation,ok,"{text}");
            assert_eq!(materialize_evidence_relevance_v30(&policy,&local,Some(&proposal),Some(&raw)).unwrap().disposition==EvidenceRelevanceDisposition::Relevant,ok,"{text}");
        }
    }
''',1)
p.write_text(s);subprocess.run(['cargo','fmt','--all'],check=True)
