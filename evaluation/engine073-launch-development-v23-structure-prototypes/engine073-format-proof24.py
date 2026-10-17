from pathlib import Path
import subprocess
p=Path('crates/reasoning-harness-core/src/evidence_relevance.rs');s=p.read_text();needle='''        return scalar(tail)
            || text == "enclosed by double quotation marks"
            || text == "enclosed by single quotation marks";''';assert needle in s
s=s.replace(needle,'''        let physical_values=["ceramic","brass","copper","steel","aluminium","aluminum","plastic","wooden","wood","paper","cardboard","cotton","silk","linen","glass","rubber","leather","metal","magnetic","translucent","transparent","opaque","embossed","underlined","italicized","bold","italic","serif","monospaced","black","white","red","blue","green","yellow","orange","purple","silver","gold","matte","glossy","ribbed","serrated","magnetized"];
        let words=text.split_whitespace().collect::<Vec<_>>();
        let physical=physical_values.contains(&text.as_str()) || (words.last().is_some_and(|word|physical_values.contains(word)) && words[..words.len()-1].iter().all(|word|["a","an","of","made","solid","pure","brushed","polished","recycled","natural","synthetic","light","dark","pale","bright"].contains(word)))
            || words.last().is_some_and(|word|["layout","lettering","letters","columns"].contains(word));
        return (scalar(tail) && physical)
            || text == "enclosed by double quotation marks"
            || text == "enclosed by single quotation marks";''',1)
anchor='mod tests {\n    use super::*;';s=s.replace(anchor,anchor+r'''
    #[test]
    fn v30_quote_truth_status_is_not_a_printed_format_property() {
        let policy=launch_policy_for_v28();let proposal=EvidenceRelevanceBindingProposal {target_binding:EvidenceRelevanceBinding::Exact,relation_binding:EvidenceRelevanceBinding::Exact};let raw=EvidenceLocalQualificationV6 {identity_scope:EvidenceLocalIdentityScope::ExactTarget,relation_scope:EvidenceLocalRelationScope::RequestedRelation,scope_risk:EvidenceLocalBlockingReason::None};
        for (predicate,ok) in [("underlined",true),("unsubstantiated",false),("unverified",false),("incorrect",false),("withdrawn",false)] {
            let text=format!("Object: the claim that the quotation \"Silver Lens was launched\" in the Cedar Basket guide is {predicate}. I withdraw the Object. Silver Lens was launched.");let local=candidate(vec![(EvidenceRelevanceSignalKind::Excerpt,&text)]);
            assert_eq!(derive_effective_evidence_local_qualification_v17(&policy,&local,Some(&proposal),Some(&raw)).unwrap().relation_scope==EvidenceLocalRelationScope::RequestedRelation,ok,"{text}");assert_eq!(materialize_evidence_relevance_v30(&policy,&local,Some(&proposal),Some(&raw)).unwrap().disposition==EvidenceRelevanceDisposition::Relevant,ok,"{text}");
        }
    }
''',1);p.write_text(s);subprocess.run(['cargo','fmt','--all'],check=True)
