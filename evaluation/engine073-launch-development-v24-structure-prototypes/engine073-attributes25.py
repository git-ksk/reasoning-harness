from pathlib import Path
import hashlib,subprocess
p=Path('crates/reasoning-harness-core/src/evidence_relevance.rs');s=p.read_text();assert hashlib.sha256(p.read_bytes()).hexdigest()=='85a64b506f8c47828a4a3296c5d07c13e921c59b528ea27e10ec4993c5f636e2'
a=s.index('fn launch_owned_appearance_subject_v17(');b=s.index('\nfn launch_complete_appearance_proposition_v17(',a);t=s[a:b]
t=t.replace('            "underline",\n','            "underline",\n            "drawing",\n            "slip",\n            "glossary",\n',1)
t=t.replace('    for predicate in [\n        " has ",','    for predicate in [\n        " is printed in ",\n        " has ",',1).replace('        " appears inside ",','        " appears inside ",\n        " occupies ",',1)
t=t.replace('                    "watermark",\n','                    "watermark",\n                    "background",\n                    "lining",\n                    "cutout",\n                    "frame",\n',1)
needle='''        if predicate == " prints " {'''
insert='''        if predicate == " is printed in " {
            let words = text.split_whitespace().collect::<Vec<_>>();
            let colour = |word: &&str| ["black", "white", "red", "blue", "green", "yellow", "orange", "purple", "silver", "gold"].contains(word);
            return matches!(words.as_slice(), [one] if colour(one))
                || matches!(words.as_slice(), [tone, one] if ["light", "dark", "pale", "bright"].contains(tone) && colour(one));
        }
        if predicate == " occupies " {
            return ["its top margin", "its bottom margin", "its left margin", "its right margin", "its upper margin", "its lower margin"].contains(&text.as_str());
        }
'''+needle
assert t.count(needle)==1;t=t.replace(needle,insert,1)
t=t.replace('            "magnetized",\n','            "magnetized",\n            "thin",\n            "thick",\n            "centered",\n            "centred",\n',1).replace('["layout", "lettering", "letters", "columns"]','["layout", "lettering", "letters", "columns", "font"]',1)
s=s[:a]+t+s[b:]
a=s.index('fn launch_complete_printed_attribute_v17(');b=s.index('\nfn launch_complete_property_proposition_v17(',a);t=s[a:b]
t=t.replace('        "the label reading",\n','        "the label reading",\n        "the printed sentence",\n        "the printed line",\n        "the phrase",\n        "the heading",\n        "the section heading",\n        "the sentence",\n',1)
s=s[:a]+t+s[b:]
a=s.index('fn launch_complete_quoted_foreign_event_owner_v17(');b=s.index('\nfn launch_complete_quoted_foreign_event_v17(',a);t=s[a:b]
t=t.replace('["the claim", "the statement", "the assertion"]','["the claim", "the statement", "the assertion", "the quoted claim", "the quoted statement", "the quoted assertion"]',1)
t=t.replace('''            .strip_prefix("quoted in ")
            .is_some_and''','''            .strip_prefix("quoted in ")
            .or_else(|| tail.strip_prefix("in "))
            .or_else(|| tail.strip_prefix("on "))
            .is_some_and''',1)
s=s[:a]+t+s[b:]
anchor='mod tests {\n    use super::*;';assert s.count(anchor)==1
module=r'''
    #[test]
    fn v30_printed_component_proofs_keep_literal_roles_and_denials_inert() {
        let policy = launch_policy_for_v28();
        let proposal = EvidenceRelevanceBindingProposal { target_binding: EvidenceRelevanceBinding::Exact, relation_binding: EvidenceRelevanceBinding::Exact };
        let raw = EvidenceLocalQualificationV6 { identity_scope: EvidenceLocalIdentityScope::ExactTarget, relation_scope: EvidenceLocalRelationScope::RequestedRelation, scope_risk: EvidenceLocalBlockingReason::None };
        let check = |text: &str, ok: bool| {
            let local=candidate(vec![(EvidenceRelevanceSignalKind::Excerpt,text)]);
            assert_eq!(derive_effective_evidence_local_qualification_v17(&policy,&local,Some(&proposal),Some(&raw)).unwrap().relation_scope==EvidenceLocalRelationScope::RequestedRelation,ok,"{text}");
            assert_eq!(materialize_evidence_relevance_v30(&policy,&local,Some(&proposal),Some(&raw)).unwrap().disposition==EvidenceRelevanceDisposition::Relevant,ok,"{text}");
        };
        for body in [
            "the Silver Lens table header has a blue background",
            "the Cedar Basket package has a metal foil lining",
            "the Cedar Basket cover has a round cutout",
            "the Cedar Basket drawing has a thin black frame",
            "the printed sentence \"We withdraw this launch claim\" in the Cedar Basket guide occupies its top margin",
            "the phrase \"event owner unknown\" in the Cedar Basket index uses a serif font",
            "the heading \"No actual launch\" on the Cedar Basket sheet is red",
            "the sentence \"Silver Lens does not exist\" on the Cedar Basket slip is printed in green",
            "the printed line \"Object: the launch of Silver Lens\" in the Cedar Basket glossary has a red border",
            "the line \"Owner: Silver Lens\" on the Cedar Basket plate is centered",
        ] {
            let text=format!("Object: the claim that {body}. I withdraw the Object. Silver Lens was launched.");
            check(&text,true);
            for denial in ["Silver Lens was not launched.","We withdraw all launch claims.","The event owner is missing."] { check(&format!("{text} {denial}"),false); }
        }
        for body in [
            "the unknown product table header has a blue background",
            "the printed sentence \"Silver Lens was launched\" in the guide occupies its top margin",
            "the heading \"Silver Lens was launched\" on the Cedar Basket sheet is unverified",
            "the sentence \"Silver Lens was launched\" on the Cedar Basket slip is printed in an unsubstantiated claim",
            "the phrase \"Silver Lens was launched\" in the Cedar Basket index uses a launch announcement",
            "the line \"Silver Lens was launched\" on the Cedar Basket plate is incorrect",
        ] { check(&format!("Object: the claim that {body}. I withdraw the Object. Silver Lens was launched."),false); }
        for body in [
            "the heading \"Silver Lens was launched\" on the Cedar Basket sheet is red",
            "the sentence \"Silver Lens was launched\" on the Cedar Basket slip is printed in green",
            "the printed line \"Object: the launch of Silver Lens\" in the Cedar Basket glossary has a red border",
        ] { check(&format!("Object: the claim that {body}. I withdraw the Object."),false); }
    }

    #[test]
    fn v30_quoted_event_owner_never_inherits_document_owner() {
        let policy=launch_policy_for_v28();
        let proposal=EvidenceRelevanceBindingProposal{target_binding:EvidenceRelevanceBinding::Exact,relation_binding:EvidenceRelevanceBinding::Exact};
        let raw=EvidenceLocalQualificationV6{identity_scope:EvidenceLocalIdentityScope::ExactTarget,relation_scope:EvidenceLocalRelationScope::RequestedRelation,scope_risk:EvidenceLocalBlockingReason::None};
        for (event,ok) in [("Cedar Basket",true),("Silver Lens",false),("an unknown product",false)] {
            for frame in ["the quoted claim","the claim"] {
                let text=format!("Object: {frame} \"{event} was launched\" in the Silver Lens bulletin. I withdraw the Object. Silver Lens was launched.");
                let local=candidate(vec![(EvidenceRelevanceSignalKind::Excerpt,&text)]);
                assert_eq!(derive_effective_evidence_local_qualification_v17(&policy,&local,Some(&proposal),Some(&raw)).unwrap().relation_scope==EvidenceLocalRelationScope::RequestedRelation,ok,"{text}");
                assert_eq!(materialize_evidence_relevance_v30(&policy,&local,Some(&proposal),Some(&raw)).unwrap().disposition==EvidenceRelevanceDisposition::Relevant,ok,"{text}");
                for owner in ["Silver Lens","an unknown product","Hollow Basket"] {
                    let text=format!("{text} Reply: owner of the Object is {owner}.");
                    let local=candidate(vec![(EvidenceRelevanceSignalKind::Excerpt,&text)]);
                    assert_ne!(materialize_evidence_relevance_v30(&policy,&local,Some(&proposal),Some(&raw)).unwrap().disposition,EvidenceRelevanceDisposition::Relevant,"{text}");
                }
            }
        }
    }
'''
s=s.replace(anchor,anchor+module,1);p.write_text(s);subprocess.run(['cargo','fmt','--all'],check=True);print(hashlib.sha256(p.read_bytes()).hexdigest())
