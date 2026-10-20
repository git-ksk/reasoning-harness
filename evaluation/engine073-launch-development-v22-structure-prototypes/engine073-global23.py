from pathlib import Path
import subprocess
p=Path('crates/reasoning-harness-core/src/evidence_relevance.rs');s=p.read_text()
anchor='fn launch_governing_context_blocks_v17(';assert s.count(anchor)==1
helper='''// A collective withdrawal of launch claims in this source is an event scope,
// even when a preceding typed role denotes a physical/printed property.
fn launch_collective_event_withdrawal_v17(statement: &str) -> bool {
    let text = normalized(statement);
    let words = text.split_whitespace().collect::<Vec<_>>();
    words.iter().enumerate().any(|(i, word)| {
        launch_retraction_word_v17(word) && !launch_retraction_negated_v17(&words[..i])
            && words[i + 1..].windows(3).any(|phrase| {
                matches!(phrase,
                    ["all" | "every" | "each", "launch" | "release",
                     "statement" | "statements" | "claim" | "claims" | "assertion" | "assertions"])
            })
    })
}

'''
s=s.replace(anchor,helper+anchor,1)
needle='''            if launch_unavailable_owner_note_v17(&segment.raw)''';assert s.count(needle)==1
s=s.replace(needle,'''            if launch_collective_event_withdrawal_v17(&segment.raw) {
                return true;
            }
'''+needle,1)
anchor='mod tests {\n    use super::*;'
test=r'''
    #[test]
    fn v30_printed_property_cannot_own_collective_event_withdrawal() {
        let policy=launch_policy_for_v28();
        let proposal=EvidenceRelevanceBindingProposal {target_binding:EvidenceRelevanceBinding::Exact,relation_binding:EvidenceRelevanceBinding::Exact};
        let raw=EvidenceLocalQualificationV6 {identity_scope:EvidenceLocalIdentityScope::ExactTarget,relation_scope:EvidenceLocalRelationScope::RequestedRelation,scope_risk:EvidenceLocalBlockingReason::None};
        let base="Object: the claim that the heading \"Launch status\" in the Cedar Basket catalog uses small capitals. We deny the Object. Silver Lens was launched.";
        for denial in ["We withdraw all launch statements in this material, including the Silver Lens statement.","I retract every launch claim.","We deny each release assertion."] {
            let text=format!("{base} {denial}");let local=candidate(vec![(EvidenceRelevanceSignalKind::Excerpt,&text)]);
            assert_ne!(materialize_evidence_relevance_v30(&policy,&local,Some(&proposal),Some(&raw)).unwrap().disposition,EvidenceRelevanceDisposition::Relevant,"{text}");
        }
        let text=format!("{base} We do not withdraw all launch statements.");let local=candidate(vec![(EvidenceRelevanceSignalKind::Excerpt,&text)]);
        assert!(!launch_collective_event_withdrawal_v17("We do not withdraw all launch statements."));
        assert_eq!(materialize_evidence_relevance_v30(&policy,&local,Some(&proposal),Some(&raw)).unwrap().disposition,EvidenceRelevanceDisposition::Relevant);
    }
'''
s=s.replace(anchor,anchor+'\n'+test,1);p.write_text(s);subprocess.run(['cargo','fmt','--all'],check=True);print('collective event withdrawal checked before local property references')
