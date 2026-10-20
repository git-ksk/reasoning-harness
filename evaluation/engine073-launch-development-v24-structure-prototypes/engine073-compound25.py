from pathlib import Path
import subprocess,hashlib
p=Path('crates/reasoning-harness-core/src/evidence_relevance.rs');s=p.read_text();assert hashlib.sha256(p.read_bytes()).hexdigest()=='ea14a86d9a7fe88ca21d7ffd2ce1b0ec9efdf3b104d6ab4eb2d7fb5c7afe5fc7'
needle='fn launch_complete_appearance_proposition_v17('
helper='''// Resolve one possessive component only inside a fully proven two-property
// conjunction. The left owned component is the antecedent; no event/pronoun
// owner, truth status or self-certified scope can supply the right type.
fn launch_compound_owned_component_predicates_v17(
    policy: &EvidenceRelevanceTargetPolicy,
    value: &str,
) -> bool {
    let Some((left, right)) = value.split_once(" and ") else { return false; };
    if right.contains(" and ") || !launch_owned_component_predicate_v17(policy, left) { return false; }
    let predicates = [" is printed in ", " has ", " is ", " was ", " are ", " were ", " uses ", " use ", " prints ", " ends with ", " appears inside ", " occupies "];
    let Some(owner) = predicates.iter().find_map(|predicate| left.split_once(predicate).map(|(subject,_)| subject)) else { return false; };
    let Some(right) = right.strip_prefix("its ") else { return false; };
    predicates.iter().any(|predicate| right.split_once(predicate).is_some_and(|(part,tail)| {
        launch_owned_component_predicate_v17(policy,&format!("the {part} on {owner}{predicate}{tail}"))
    }))
}

'''
assert s.count(needle)==1;s=s.replace(needle,helper+needle,1)
a=s.index('fn launch_complete_appearance_proposition_v17(');b=s.index('// The quoted body is inert lettering',a);t=s[a:b]
t=t.replace('''    if launch_owned_component_predicate_v17(policy, value) {''','''    if launch_owned_component_predicate_v17(policy, value)
        || launch_compound_owned_component_predicates_v17(policy, value) {''',1);s=s[:a]+t+s[b:]
anchor='mod tests {\n    use super::*;';assert s.count(anchor)==1
module=r'''
    #[test]
    fn v30_possessive_property_conjunction_requires_complete_same_component_scope() {
        let policy=launch_policy_for_v28();
        let proposal=EvidenceRelevanceBindingProposal{target_binding:EvidenceRelevanceBinding::Exact,relation_binding:EvidenceRelevanceBinding::Exact};
        let raw=EvidenceLocalQualificationV6{identity_scope:EvidenceLocalIdentityScope::ExactTarget,relation_scope:EvidenceLocalRelationScope::RequestedRelation,scope_risk:EvidenceLocalBlockingReason::None};
        for (body,ok) in [
            ("the Silver Lens reference card is thin and its heading is blue",true),
            ("the Cedar Basket card is thick and its header has a red background",true),
            ("the Silver Lens card is thin and its heading is printed in green",true),
            ("the card is thin and its heading is blue",false),
            ("the unknown product card is thin and its heading is blue",false),
            ("the Silver Lens reference card is thin and its launch is blue",false),
            ("the Silver Lens reference card is thin and its heading is unverified",false),
            ("the Silver Lens reference card is thin and its heading is blue and Silver Lens was not launched",false),
            ("the Silver Lens reference card is thin and Silver Lens was not launched",false),
        ] {
            let text=format!("Object: the claim that {body}. We retract the Object. Silver Lens was launched.");
            let check=|text:&str,ok:bool|{
                let local=candidate(vec![(EvidenceRelevanceSignalKind::Excerpt,text)]);
                assert_eq!(derive_effective_evidence_local_qualification_v17(&policy,&local,Some(&proposal),Some(&raw)).unwrap().relation_scope==EvidenceLocalRelationScope::RequestedRelation,ok,"{text}");
                assert_eq!(materialize_evidence_relevance_v30(&policy,&local,Some(&proposal),Some(&raw)).unwrap().disposition==EvidenceRelevanceDisposition::Relevant,ok,"{text}");
            };
            check(&text,ok);
            if ok {for denial in ["Silver Lens was not launched.","We withdraw all launch claims.","The event owner is missing."]{check(&format!("{text} {denial}"),false);}}
        }
    }
'''
s=s.replace(anchor,anchor+module,1);p.write_text(s);subprocess.run(['cargo','fmt','--all'],check=True);print(hashlib.sha256(p.read_bytes()).hexdigest())
