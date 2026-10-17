from pathlib import Path
import subprocess
p=Path('crates/reasoning-harness-core/src/evidence_relevance.rs');s=p.read_text();assert 'record_launch_development_trace_v17' not in s
# Broaden physical/document components; complete ownership remains required.
a=s.index('fn launch_owned_appearance_subject_v17(');b=s.index('fn launch_complete_appearance_proposition_v17(',a);block=s[a:b]
needle='''            "edge",
        ]''';assert needle in block;block=block.replace(needle,'''            "edge", "lid", "fitting", "diagram", "proof", "appendix", "pamphlet", "carton", "plate", "header", "package", "index", "guide", "window", "latch", "vent", "margin", "margins", "tab", "rule", "underline",
        ]''',1)
block=block.replace('["document", "notice", "bulletin", "page"].contains(word)','["document", "notice", "bulletin", "page", "owner"].contains(word)')
s=s[:a]+block+s[b:]
# One owned component plus a complete non-event physical/document predicate.
helper=r'''
fn launch_owned_component_predicate_v17(policy: &EvidenceRelevanceTargetPolicy, value: &str) -> bool {
    let scalar=|value: &str| {
        let text=normalized(value);let words=text.split_whitespace().collect::<Vec<_>>();
        !words.is_empty() && words.len()<=6 && words.iter().all(|word| {
            (!launch_scope_or_reference_word_v17(word) || ["one","two","three","four","five","six","seven","eight","nine","ten"].contains(word))
                && !launch_retraction_word_v17(word) && !launch_adverse_context_word_v17(word)
                && !["and","or","but","not","never","is","was","are","were","has","have","because","although","while","when","if","since","exist","exists","existed"].contains(word)
        })
    };
    for predicate in [" has "," is "," was "," are "," were "," uses "," use "," prints "," ends with "," appears inside "] {
        let Some((subject,tail))=value.split_once(predicate) else {continue;};
        if !launch_owned_appearance_subject_v17(policy,subject) {continue;}
        let text=normalized(tail);
        if predicate==" has " {
            let words=text.split_whitespace().collect::<Vec<_>>();
            return words.last().is_some_and(|word|["underline","vent","layout","margin","margins","window","tab","surface","latch","rule","edge","border","hinge","corner","spine","watermark"].contains(word)) && scalar(tail);
        }
        if predicate==" prints " {
            return ["its units in ","its address in ","its revision mark in ","its revision mark inside a ","its edition mark in ","its edition mark inside a ","warnings in "].iter().any(|prefix|text.strip_prefix(prefix).is_some_and(scalar));
        }
        if predicate==" ends with " {
            return ["a period","a colon","a comma","a semicolon","a full stop"].contains(&text.as_str());
        }
        if predicate==" appears inside " {
            return ["a box","a rectangle","a circle","a border"].contains(&text.as_str());
        }
        return scalar(tail) || text=="enclosed by double quotation marks" || text=="enclosed by single quotation marks";
    }
    false
}

'''
s=s.replace('fn launch_complete_appearance_proposition_v17(',helper+'fn launch_complete_appearance_proposition_v17(',1)
needle='''    if value.contains(['"', '“', '‘', '?']) {
        return false;
    }''';i=s.index('fn launch_complete_appearance_proposition_v17(');j=s.index(needle,i);s=s[:j]+s[j:].replace(needle,needle+'''\n    if launch_owned_component_predicate_v17(policy,value) { return true; }''',1)
# A complete outside text-carrier grammar proves that its literal is data.
needle='''    let owned = |owner: &str| launch_owned_appearance_subject_v17(policy, owner.trim());''';assert s.count(needle)==1
s=s.replace(needle,needle+r'''
    if ["the legend","the sample sentence","the line","the quotation","the words","the label reading"].contains(&head.as_str()) {
        if let Some(owner_and_predicate)=tail.strip_prefix("on ").or_else(||tail.strip_prefix("in ")) {
            if launch_owned_component_predicate_v17(policy,owner_and_predicate) {return true;}
        }
    }
''',1)
anchor='mod tests {\n    use super::*;';s=s.replace(anchor,anchor+r'''
    #[test]
    fn v30_owned_document_predicates_and_carriers_keep_event_scope_separate() {
        let policy=launch_policy_for_v28();let proposal=EvidenceRelevanceBindingProposal {target_binding:EvidenceRelevanceBinding::Exact,relation_binding:EvidenceRelevanceBinding::Exact};let raw=EvidenceLocalQualificationV6 {identity_scope:EvidenceLocalIdentityScope::ExactTarget,relation_scope:EvidenceLocalRelationScope::RequestedRelation,scope_risk:EvidenceLocalBlockingReason::None};
        for body in ["the Silver Lens lid has a triangular vent","the Cedar Basket contents page uses a two-column layout","the Silver Lens corner fitting is ceramic","the Cedar Basket owner plate is brass","the Cedar Basket instruction panel is translucent","the Cedar Basket cover prints its edition mark in raised lettering","the Cedar Basket instruction card prints warnings in bold","the Cedar Basket document title ends with a colon","the legend \"Silver Lens was launched\" on the Cedar Basket instrument diagram has a dotted underline","the sample sentence \"I withdraw the launch claim\" in the Cedar Basket proof appears inside a box","the line \"Silver Lens does not exist\" on the Cedar Basket label ends with a period","the words \"owner unspecified\" in the Cedar Basket index are underlined","the label reading \"Silver Lens was launched\" on the Cedar Basket package has a scalloped edge","the quotation \"Silver Lens was launched\" in the Cedar Basket guide is enclosed by double quotation marks"] {
            let text=format!("Object: the claim that {body}. I withdraw the Object. Silver Lens was launched.");let local=candidate(vec![(EvidenceRelevanceSignalKind::Excerpt,&text)]);
            assert_eq!(materialize_evidence_relevance_v30(&policy,&local,Some(&proposal),Some(&raw)).unwrap().disposition,EvidenceRelevanceDisposition::Relevant,"{text}");
            for denial in ["Silver Lens was not launched.","We withdraw all launch claims.","The event owner is missing."] {
                let negative=format!("{text} {denial}");let local=candidate(vec![(EvidenceRelevanceSignalKind::Excerpt,&negative)]);
                assert_ne!(materialize_evidence_relevance_v30(&policy,&local,Some(&proposal),Some(&raw)).unwrap().disposition,EvidenceRelevanceDisposition::Relevant,"{negative}");
            }
        }
        for body in ["the manual has a triangular vent","the unknown product lid has a triangular vent","the Cedar Basket manual uses a launch announcement","the Cedar Basket manual is fictional","the Cedar Basket owner plate is brass and Silver Lens was not launched","the line \"Silver Lens does not exist\" on the label ends with a period"] {
            let text=format!("Object: the claim that {body}. I withdraw the Object. Silver Lens was launched.");let local=candidate(vec![(EvidenceRelevanceSignalKind::Excerpt,&text)]);
            assert_ne!(materialize_evidence_relevance_v30(&policy,&local,Some(&proposal),Some(&raw)).unwrap().disposition,EvidenceRelevanceDisposition::Relevant,"{text}");
        }
    }
''',1)
p.write_text(s);subprocess.run(['cargo','fmt','--all'],check=True)
