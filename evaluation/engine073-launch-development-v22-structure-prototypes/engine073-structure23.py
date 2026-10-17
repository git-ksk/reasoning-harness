from pathlib import Path
import subprocess
p=Path('crates/reasoning-harness-core/src/evidence_relevance.rs');s=p.read_text();assert 'fn launch_complete_quoted_foreign_event_v17' not in s
helper=r'''// The event owner inside the quotation and the document owner outside it
// are separate roles. Both must be complete; a manual cannot lend event scope.
fn launch_complete_quoted_foreign_event_v17(policy: &EvidenceRelevanceTargetPolicy, value: &str) -> bool {
    let value=value.trim();
    let Some(start)=value.find(['"','“','‘']) else {return false;};
    if !["the claim","the statement","the assertion"].contains(&normalized(&value[..start]).as_str()) {return false;}
    let quote=value[start..].chars().next().unwrap();let closing=match quote {'“'=>'”','‘'=>'’',_=>'"'};
    let quoted=&value[start+quote.len_utf8()..];let Some(end)=quoted.find(closing) else {return false;};
    let body=&quoted[..end];let tail=quoted[end+closing.len_utf8()..].trim();
    if body.contains(['"','“','”','‘','’',',',';']) || tail.contains(['"','“','”','‘','’',',',';']) {return false;}
    let owner=[" was launched"," was released"," was not launched"," was not released"].iter().find_map(|suffix|body.strip_suffix(suffix));
    owner.is_some_and(|owner|launch_named_subject_v17(owner) && !target_identity_phrases(policy).iter().any(|target|normalized_phrase_matches(owner,target)))
        && tail.strip_prefix("quoted in ").is_some_and(|owner|launch_owned_appearance_subject_v17(policy,owner))
}

// An adjacent full-name explanation can confirm a syntactically owned foreign
// denial. It cannot assign an owner to a previously unresolved event.
fn launch_foreign_denial_name_confirmation_v17(policy: &EvidenceRelevanceTargetPolicy, statement: &str, previous: Option<&LaunchFactualSegmentV17>) -> bool {
    let Some(previous)=previous else {return false;};
    if !launch_retraction_other_object_v17(policy,&previous.raw) {return false;}
    ["launch","release"].iter().any(|event| {
        statement.strip_suffix(&format!(" is the full name of the product whose {event} is denied")).is_some_and(|owner| {
            launch_named_subject_v17(owner) && !target_identity_phrases(policy).iter().any(|target|normalized_phrase_matches(owner,target))
                && normalized(&previous.raw).ends_with(&format!("the {event} of {}",normalized(owner)))
        })
    })
}

'''
s=s.replace('fn launch_typed_object_scope_v17(',helper+'fn launch_typed_object_scope_v17(',1)
s=s.replace('''    if launch_owned_material_object_v17(value)
        || launch_complete_property_proposition_v17(policy, value)
        || launch_compound_attribute_object_v17(policy, value)
        || launch_printed_quoted_property_v17(policy, value)
    {''','''    if launch_owned_material_object_v17(value)
        || launch_complete_quoted_foreign_event_v17(policy, value)
        || launch_complete_property_proposition_v17(policy, value)
        || launch_compound_attribute_object_v17(policy, value)
        || launch_printed_quoted_property_v17(policy, value)
    {''',1)
s=s.replace('''                && (owner_bound
                    || (explicit_event''','''                && (owner_bound
                    || launch_complete_quoted_foreign_event_v17(policy, value)
                    || (explicit_event''',1)
old='''                let value = normalized(value);
                value.starts_with("namely ") || value.starts_with("specifically ")''';new='''                let text = normalized(value);
                text.starts_with("namely ") || text.starts_with("specifically ")
                    || launch_complete_property_proposition_v17(policy, value.trim())''';assert old in s;s=s.replace(old,new,1)
# Complete printed comma apposition must also be recognized by the quote-risk view.
anchor='''    let Some(start) = statement.find(['"', '“', '‘']) else {''';i=s.index('fn launch_complete_printed_property_correction_v17(');j=s.index(anchor,i)
s=s[:j]+'''    if statement.split_once(',').is_some_and(|(_, body)|
        launch_complete_printed_attribute_v17(policy, body.trim())
            && launch_property_apposition_v17(policy, statement)) { return true; }
'''+s[j:]
needle='''            let prior_foreign_denial = segments[..index].iter().any(|prior| {''';assert needle in s;s=s.replace(needle,'''            if launch_foreign_denial_name_confirmation_v17(policy, &segment.raw, index.checked_sub(1).and_then(|i|segments.get(i))) {continue;}
'''+needle,1)
anchor='mod tests {\n    use super::*;';s=s.replace(anchor,anchor+r'''
    #[test]
    fn v30_complete_foreign_event_quote_does_not_borrow_document_owner() {
        let policy=launch_policy_for_v28();let proposal=EvidenceRelevanceBindingProposal {target_binding:EvidenceRelevanceBinding::Exact,relation_binding:EvidenceRelevanceBinding::Exact};let raw=EvidenceLocalQualificationV6 {identity_scope:EvidenceLocalIdentityScope::ExactTarget,relation_scope:EvidenceLocalRelationScope::RequestedRelation,scope_risk:EvidenceLocalBlockingReason::None};
        for document in ["Cedar Basket","Silver Lens"] {
            let text=format!("Item: the claim \"Violet Bell was launched\" quoted in the {document} manual. I withdraw the Item. Silver Lens was launched.");let local=candidate(vec![(EvidenceRelevanceSignalKind::Excerpt,&text)]);
            assert_eq!(materialize_evidence_relevance_v30(&policy,&local,Some(&proposal),Some(&raw)).unwrap().disposition,EvidenceRelevanceDisposition::Relevant,"{text}");
        }
        for body in ["the claim \"Silver Lens was launched\" quoted in the Cedar Basket manual","the claim \"it was launched\" quoted in the Cedar Basket manual","the claim \"Violet Bell was launched\" quoted in the manual","the claim \"Violet Bell was launched and Silver Lens was launched\" quoted in the Cedar Basket manual"] {
            let text=format!("Item: {body}. I withdraw the Item. Silver Lens was launched.");let local=candidate(vec![(EvidenceRelevanceSignalKind::Excerpt,&text)]);
            assert_ne!(materialize_evidence_relevance_v30(&policy,&local,Some(&proposal),Some(&raw)).unwrap().disposition,EvidenceRelevanceDisposition::Relevant,"{text}");
        }
    }
    #[test]
    fn v30_complete_printed_apposition_retains_event_and_global_denials() {
        let policy=launch_policy_for_v28();let proposal=EvidenceRelevanceBindingProposal {target_binding:EvidenceRelevanceBinding::Exact,relation_binding:EvidenceRelevanceBinding::Exact};let raw=EvidenceLocalQualificationV6 {identity_scope:EvidenceLocalIdentityScope::ExactTarget,relation_scope:EvidenceLocalRelationScope::RequestedRelation,scope_risk:EvidenceLocalBlockingReason::None};
        for (correction,ok) in [("I withdraw this claim, the claim that the engraving \"Silver Lens was launched\" on the Cedar Basket badge is shallow",true),("I withdraw this claim, the claim that Silver Lens was launched",false),("I withdraw this claim, the claim that the engraving \"Silver Lens was launched\" on the badge is shallow",false),("I withdraw all launch statements, the claim that the engraving \"Silver Lens was launched\" on the Cedar Basket badge is shallow",false)] {
            let text=format!("{correction}. Silver Lens was launched.");let local=candidate(vec![(EvidenceRelevanceSignalKind::Excerpt,&text)]);
            assert_eq!(materialize_evidence_relevance_v30(&policy,&local,Some(&proposal),Some(&raw)).unwrap().disposition==EvidenceRelevanceDisposition::Relevant,ok,"{text}");
        }
    }
    #[test]
    fn v30_foreign_name_confirmation_requires_adjacent_same_owned_denial() {
        let policy=launch_policy_for_v28();
        for (prior,owner,ok) in [("We deny this claim, the launch of Violet Bell","Violet Bell",true),("We deny this claim, the launch of Cedar Basket","Violet Bell",false),("We deny this claim, the launch of Silver Lens","Silver Lens",false),("We deny this claim, a launch whose owner is unknown","Violet Bell",false)] {
            let previous=launch_factual_segments_v17(&policy,prior).pop().unwrap();let note=format!("{owner} is the full name of the product whose launch is denied");
            assert_eq!(launch_foreign_denial_name_confirmation_v17(&policy,&note,Some(&previous)),ok,"{prior}: {note}");
        }
    }
''',1)
p.write_text(s);subprocess.run(['cargo','fmt','--all'],check=True)
