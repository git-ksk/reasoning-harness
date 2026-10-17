from pathlib import Path
import subprocess
p=Path('crates/reasoning-harness-core/src/evidence_relevance.rs');s=p.read_text()
i=s.index('fn launch_adverse_segments_v17(');j=s.index('fn launch_governing_context_blocks_v17(',i);block=s[i:j]
block=block.replace('''    let reporting = |text: &str| {''','''    // Keep a reported non-event as its own clause; it must retain its actual owner.
    let maintained = |text: &str| {
        let text=normalized(text);
        ["we maintain that ", "i maintain that ", "the author maintains that ", "the editor maintains that ", "the publisher maintains that "].iter().any(|prefix|text.strip_prefix(prefix).is_some_and(|body|body.contains(" was not launched") || body.contains(" was not released") || body.contains(" does not exist")))
    };
    let reporting = |text: &str| {''',1)
block=block.replace('''        normalized(text).split_whitespace().any(|word| {''','''        maintained(text) || normalized(text).split_whitespace().any(|word| {''',1)
assert '&& reporting(body)' in block;block=block.replace('&& reporting(body)','&& (reporting(head) || reporting(body))',1)
old='''                        ["report", "reports", "record", "records", "state", "states"]''';assert old in block;block=block.replace(old,'''                        ["report", "reports", "record", "records", "state", "states", "maintain", "maintains"]''',1)
s=s[:i]+block+s[j:]
anchor='mod tests {\n    use super::*;'
s=s.replace(anchor,anchor+r'''
    #[test]
    fn v30_separate_maintained_non_event_and_parenthesis_keep_each_scope() {
        let policy=launch_policy_for_v28();let proposal=EvidenceRelevanceBindingProposal {target_binding:EvidenceRelevanceBinding::Exact,relation_binding:EvidenceRelevanceBinding::Exact};let raw=EvidenceLocalQualificationV6 {identity_scope:EvidenceLocalIdentityScope::ExactTarget,relation_scope:EvidenceLocalRelationScope::RequestedRelation,scope_risk:EvidenceLocalBlockingReason::None};
        let property="Object: the claim that the Cedar Basket case has a recessed hinge.";
        for (body,ok) in [("I retract the Object (the correction concerns the depth of the hinge recess on Cedar Basket)",true),("I retract the Object (Silver Lens was not launched)",false),("I retract the Object (we retract the whole source)",false),("I retract the Object (the event owner is missing)",false),("We withdraw the Object. Subject: the launch of Violet Bell. We deny the Subject, and we maintain that Violet Bell was not launched",true),("We withdraw the Object. Subject: the launch of Violet Bell. We deny the Subject, and we maintain that Silver Lens was not launched",false),("We withdraw the Object. Subject: the launch of Violet Bell. We deny the Subject, and we maintain that it was not launched",false),("We withdraw the Object. Subject: the launch of Violet Bell. We deny the Subject, and we maintain that Silver Lens does not exist",false)] {
            let text=format!("{property} {body}. Silver Lens was launched.");let local=candidate(vec![(EvidenceRelevanceSignalKind::Excerpt,&text)]);
            assert_eq!(materialize_evidence_relevance_v30(&policy,&local,Some(&proposal),Some(&raw)).unwrap().disposition==EvidenceRelevanceDisposition::Relevant,ok,"{text}");
        }
    }
''',1)
p.write_text(s);subprocess.run(['cargo','fmt','--all'],check=True)
