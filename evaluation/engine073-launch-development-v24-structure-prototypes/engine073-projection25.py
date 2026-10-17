from pathlib import Path
import hashlib,subprocess,ast
p=Path('crates/reasoning-harness-core/src/evidence_relevance.rs');s=p.read_text();assert hashlib.sha256(p.read_bytes()).hexdigest()=='ea14a86d9a7fe88ca21d7ffd2ce1b0ec9efdf3b104d6ab4eb2d7fb5c7afe5fc7'
a=s.index('fn launch_scope_annotation_view_v17(');b=s.index('\nfn launch_opaque_name_context_view_v17(',a);t=s[a:b]
t=t.replace('''            EvidenceRelevanceSignalKind::Fact | EvidenceRelevanceSignalKind::Excerpt
''','''            EvidenceRelevanceSignalKind::Fact | EvidenceRelevanceSignalKind::Excerpt
                | EvidenceRelevanceSignalKind::Heading | EvidenceRelevanceSignalKind::SourceTitle
''',1)
t=t.replace('''        let mut signal_changed = false;
''','''        // Heading/title projection is restricted to complete inert quoted
        // attributes; other source-wide notes retain their original authority.
        let printed_only = matches!(signal.kind, EvidenceRelevanceSignalKind::Heading | EvidenceRelevanceSignalKind::SourceTitle);
        let mut signal_changed = false;
''',1)
t=t.replace('''                let text = normalized(&segment.raw);
''','''                if printed_only { return segment.raw; }
                let text = normalized(&segment.raw);
''',1);s=s[:a]+t+s[b:]
needle='fn launch_governing_context_blocks_v17('
helper='''// A note denying that literal plate lettering supplies an event owner is
// not an unavailable-owner declaration. Both the printed object and the
// separately foreign event role must already have complete independent scope.
fn launch_literal_owner_supplement_note_v17(
    policy: &EvidenceRelevanceTargetPolicy,
    statement: &str,
    objects: &[LaunchTypedObjectV17],
) -> bool {
    let note = normalized(statement);
    let Some((carrier, description)) = note.strip_prefix("the ").and_then(|text| text.split_once(" describes ")) else { return false; };
    if !["line", "sentence", "phrase", "heading", "quotation", "legend", "words"].contains(&carrier) { return false; }
    for role in ["object", "subject", "item"] {
        let Some(printed) = description.strip_suffix(&format!(" not an owner supplement for the {role}")) else { continue; };
        let words = printed.split_whitespace().collect::<Vec<_>>();
        let ["printed", component, "text"] = words.as_slice() else { continue; };
        let selected = objects.iter().filter(|object|object.label == role).collect::<Vec<_>>();
        if selected.is_empty() || !selected.iter().all(|object| object.scope == LaunchTypedScopeV17::Excluded && object.foreign_event) { return false; }
        return objects.iter().any(|object| {
            if object.scope != LaunchTypedScopeV17::Excluded || object.foreign_event || !launch_complete_printed_attribute_v17(policy, &object.declared_value) { return false; }
            let value = launch_complete_assertion_body_v17(&object.declared_value);
            if !value.starts_with(&format!("the {carrier} ")) && !value.starts_with(&format!("the printed {carrier} ")) { return false; }
            let Some(start) = value.find(['"', '“', '‘']) else { return false; };
            let quote = value[start..].chars().next().unwrap();
            let closing = match quote { '“' => '”', '‘' => '’', _ => '"' };
            let body = &value[start + quote.len_utf8()..];
            let Some(end) = body.find(closing) else { return false; };
            normalized(&body[end + closing.len_utf8()..]).split_whitespace().any(|word|word == *component)
        });
    }
    false
}

'''
assert s.count(needle)==1;s=s.replace(needle,helper+needle,1)
a=s.index('fn launch_governing_context_blocks_v17(');b=s.index('\nfn target_owned_launch_fact_v17(',a);t=s[a:b];needle='''            if launch_unavailable_owner_note_v17(&segment.raw)''';assert t.count(needle)==1;t=t.replace(needle,'''            if launch_literal_owner_supplement_note_v17(policy, &segment.raw, &typed_objects) { continue; }
'''+needle,1);s=s[:a]+t+s[b:]
anchor='mod tests {\n    use super::*;';assert s.count(anchor)==1
# The current property parser already recognizes the two-attribute case once
# 'thin' is typed. Add only paired controls, not the unused broader helper.
script=ast.parse(Path('/private/tmp/engine073-compound25.py').read_text());compound=next(ast.literal_eval(n.value) for n in script.body if isinstance(n,ast.Assign) and any(isinstance(t,ast.Name) and t.id=='module' for t in n.targets))
module=r'''
    #[test]
    fn v30_quoted_owner_words_in_headings_do_not_hide_real_owner_gaps() {
        let policy=launch_policy_for_v28();
        let proposal=EvidenceRelevanceBindingProposal{target_binding:EvidenceRelevanceBinding::Exact,relation_binding:EvidenceRelevanceBinding::Exact};
        let raw=EvidenceLocalQualificationV6{identity_scope:EvidenceLocalIdentityScope::ExactTarget,relation_scope:EvidenceLocalRelationScope::RequestedRelation,scope_risk:EvidenceLocalBlockingReason::None};
        for kind in [EvidenceRelevanceSignalKind::Heading,EvidenceRelevanceSignalKind::SourceTitle,EvidenceRelevanceSignalKind::Excerpt] {
            let body="Object: the claim that the phrase \"event owner unknown\" in the Cedar Basket index uses a serif font";
            let text="We withdraw the Object. Item: the launch of Cedar Basket. We deny the Item. Silver Lens was launched.";
            let local=candidate(vec![(kind,body),(EvidenceRelevanceSignalKind::Excerpt,text)]);
            let q=derive_effective_evidence_local_qualification_v17(&policy,&local,Some(&proposal),Some(&raw)).unwrap();
            assert_eq!(q.identity_scope,EvidenceLocalIdentityScope::ExactTarget);
            assert_eq!(q.scope_risk,EvidenceLocalBlockingReason::None);
            assert_eq!(materialize_evidence_relevance_v30(&policy,&local,Some(&proposal),Some(&raw)).unwrap().disposition,EvidenceRelevanceDisposition::Relevant);
            let gap=EvidenceLocalQualificationV6{scope_risk:EvidenceLocalBlockingReason::ContextGap,..raw};
            assert_ne!(materialize_evidence_relevance_v30(&policy,&local,Some(&proposal),Some(&gap)).unwrap().disposition,EvidenceRelevanceDisposition::Relevant);
            for note in ["The event owner is missing.","Silver Lens was not launched.","We withdraw all launch claims."] {
                let extra=format!("{text} {note}");let local=candidate(vec![(kind,body),(EvidenceRelevanceSignalKind::Excerpt,&extra)]);
                assert_ne!(materialize_evidence_relevance_v30(&policy,&local,Some(&proposal),Some(&raw)).unwrap().disposition,EvidenceRelevanceDisposition::Relevant,"{note}");
            }
            let ambiguous="Object: the phrase \"event owner unknown\" in an unknown document uses a serif font";
            let local=candidate(vec![(kind,ambiguous),(EvidenceRelevanceSignalKind::Excerpt,text)]);
            assert_ne!(materialize_evidence_relevance_v30(&policy,&local,Some(&proposal),Some(&raw)).unwrap().disposition,EvidenceRelevanceDisposition::Relevant);
        }
        for (owner,ok) in [("Cedar Basket",true),("Silver Lens",false),("an unknown product",false)] {
            let text=format!("Item: the claim that the line \"Owner: Silver Lens\" on the Cedar Basket plate is centered. We deny the Item. Object: the launch of {owner}. We withdraw the Object. The line describes printed plate text, not an owner supplement for the Object. Silver Lens was launched.");
            let local=candidate(vec![(EvidenceRelevanceSignalKind::Excerpt,&text)]);
            assert_eq!(derive_effective_evidence_local_qualification_v17(&policy,&local,Some(&proposal),Some(&raw)).unwrap().relation_scope==EvidenceLocalRelationScope::RequestedRelation,ok,"{text}");
            assert_eq!(materialize_evidence_relevance_v30(&policy,&local,Some(&proposal),Some(&raw)).unwrap().disposition==EvidenceRelevanceDisposition::Relevant,ok,"{text}");
            for note in ["The event owner is missing.","Silver Lens was not launched."] {
                let text=format!("{text} {note}");let local=candidate(vec![(EvidenceRelevanceSignalKind::Excerpt,&text)]);
                assert_ne!(materialize_evidence_relevance_v30(&policy,&local,Some(&proposal),Some(&raw)).unwrap().disposition,EvidenceRelevanceDisposition::Relevant);
            }
        }
    }
'''
s=s.replace(anchor,anchor+compound+module,1);p.write_text(s);subprocess.run(['cargo','fmt','--all'],check=True);print(hashlib.sha256(p.read_bytes()).hexdigest())
