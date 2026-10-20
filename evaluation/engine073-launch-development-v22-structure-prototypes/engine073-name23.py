from pathlib import Path
import subprocess
p=Path('crates/reasoning-harness-core/src/evidence_relevance.rs');s=p.read_text();i=s.index('fn launch_opaque_name_context_view_v17(');j=s.index('fn launch_foreign_context_view_v17(',i);block=s[i:j]
needle='''                    if complete_last_token_note || (words.contains(&"clip")''';assert needle in block
block=block.replace(needle,'''                    // Exact negated clipping note is metadata, not omitted context.
                    // This projection still requires the anchored target fact and a
                    // risk-free independently recomputed complete candidate below.
                    if note == "no excerpt has been clipped" {
                        signal_changed = true;
                        changed = true;
                        "the excerpt is complete".to_owned()
                    } else if complete_last_token_note || (words.contains(&"clip")''',1)
s=s[:i]+block+s[j:]
anchor='mod tests {\n    use super::*;';s=s.replace(anchor,anchor+r'''
    #[test]
    fn v30_complete_clip_name_note_cannot_erase_actual_context_gap() {
        let mut policy=launch_policy_for_v28();policy.entity.as_mut().unwrap().canonical_name="Leaf Clip".into();
        let proposal=EvidenceRelevanceBindingProposal {target_binding:EvidenceRelevanceBinding::Exact,relation_binding:EvidenceRelevanceBinding::Exact};let raw=EvidenceLocalQualificationV6 {identity_scope:EvidenceLocalIdentityScope::ExactTarget,relation_scope:EvidenceLocalRelationScope::RequestedRelation,scope_risk:EvidenceLocalBlockingReason::None};
        for (extra,ok) in [("",true),("The event owner column has been clipped.",false),("The event owner is missing.",false),("Leaf Clip was not launched.",false),("We withdraw the whole source.",false)] {
            let text=format!("Item: the claim that the Leaf Clip clasp is magnetic. I withdraw the Item. {extra}");
            let local=candidate(vec![(EvidenceRelevanceSignalKind::Heading,"Canonical product name: Leaf Clip. Clip is part of this complete name; no excerpt has been clipped."),(EvidenceRelevanceSignalKind::Excerpt,"Leaf Clip was launched."),(EvidenceRelevanceSignalKind::Excerpt,&text)]);
            assert_eq!(materialize_evidence_relevance_v30(&policy,&local,Some(&proposal),Some(&raw)).unwrap().disposition==EvidenceRelevanceDisposition::Relevant,ok,"{text}");
            let mut inherited=raw.clone();inherited.scope_risk=EvidenceLocalBlockingReason::ContextGap;
            assert!(launch_opaque_name_context_view_v17(&policy,&local,Some(&proposal),Some(&inherited)).unwrap().is_none());
        }
    }
''',1);p.write_text(s);subprocess.run(['cargo','fmt','--all'],check=True)
