from pathlib import Path
import hashlib,subprocess,difflib
p=Path('crates/reasoning-harness-core/src/evidence_relevance.rs');s=p.read_text();assert hashlib.sha256(p.read_bytes()).hexdigest()=='b7fec3e0f7c6b76dfbff1b801dd085a22661df9b22457431653eb2073a4e08f0';base=s
a=s.index('fn launch_complete_property_proposition_v17(');b=s.index('\nfn launch_printed_quoted_property_v17(',a);t=s[a:b]
old='    let property = launch_property_words_v17(policy, value);'
new='''    // A complete assertion wrapper does not change the property proposition.
    // Strip one finite wrapper only; the full body must still independently
    // pass the existing property/predicate/risk checks below.
    let value = value.trim();
    let folded = value.to_ascii_lowercase();
    let value = ["the claim that ", "the assertion that ", "the statement that "]
        .iter()
        .find(|prefix| folded.starts_with(**prefix))
        .map_or(value, |prefix| value[prefix.len()..].trim());
    let property = launch_property_words_v17(policy, value);'''
assert t.count(old)==1;t=t.replace(old,new);s=s[:a]+t+s[b:]
old='''.strip_prefix(&["an"])
                .or_else(|| words.strip_prefix(&["a"]))''';new='''.strip_prefix(&["an"])
                .or_else(|| words.strip_prefix(&["a"]))
                .or_else(|| words.strip_prefix(&["the"]))''';assert s.count(old)==1;s=s.replace(old,new)
old='''.strip_prefix(&["expressly"])
                .or_else(|| words.strip_prefix(&["explicitly"]))''';new='''.strip_prefix(&["expressly"])
                .or_else(|| words.strip_prefix(&["explicitly"]))
                .or_else(|| words.strip_prefix(&["otherwise"]))''';assert s.count(old)==1;s=s.replace(old,new)
old='''                    words,
                    [
                        "unowned",''';new='''                    words,
                    ["launch" | "release", "claim" | "statement" | "assertion"]
                    | [
                        "unowned",''';assert s.count(old)==1;s=s.replace(old,new)
anchor='mod tests {\n    use super::*;';assert s.count(anchor)==1
tests=r'''

    #[test]
    fn v30_complete_claim_wrapper_preserves_only_proven_property_scope() {
        let policy = launch_policy_for_v28();
        let proposal = EvidenceRelevanceBindingProposal { target_binding: EvidenceRelevanceBinding::Exact, relation_binding: EvidenceRelevanceBinding::Exact };
        let raw = EvidenceLocalQualificationV6 { identity_scope: EvidenceLocalIdentityScope::ExactTarget, relation_scope: EvidenceLocalRelationScope::RequestedRelation, scope_risk: EvidenceLocalBlockingReason::None };
        for wrapper in ["the claim that", "the assertion that", "the statement that"] {
            let text = format!("Object: {wrapper} the font of the Silver Lens card is blue. I retract the Object. Silver Lens was launched.");
            let local = candidate(vec![(EvidenceRelevanceSignalKind::Excerpt, &text)]);
            assert_eq!(materialize_evidence_relevance_v30(&policy,&local,Some(&proposal),Some(&raw)).unwrap().disposition,EvidenceRelevanceDisposition::Relevant,"{text}");
            for body in ["Silver Lens was launched", "the launch of Silver Lens is real", "the complete source is accurate", "the font of the card is unknown", "the statement that the font of the Silver Lens card is blue"] {
                let text = format!("Object: {wrapper} {body}. I retract the Object. Silver Lens was launched.");
                let local = candidate(vec![(EvidenceRelevanceSignalKind::Excerpt, &text)]);
                assert_ne!(materialize_evidence_relevance_v30(&policy,&local,Some(&proposal),Some(&raw)).unwrap().disposition,EvidenceRelevanceDisposition::Relevant,"{text}");
            }
        }
        for tail in ["We retract the whole source.","Silver Lens was not launched.","The product column naming the event owner is missing."] {
            let text=format!("Object: the claim that the font of the Silver Lens card is blue. I retract the Object. Silver Lens was launched. {tail}");
            let local=candidate(vec![(EvidenceRelevanceSignalKind::Excerpt,&text)]);
            assert_ne!(materialize_evidence_relevance_v30(&policy,&local,Some(&proposal),Some(&raw)).unwrap().disposition,EvidenceRelevanceDisposition::Relevant,"{text}");
        }
    }

    #[test]
    fn v30_unowned_reference_determiners_require_unique_same_role_owner() {
        let policy = launch_policy_for_v28();
        let proposal = EvidenceRelevanceBindingProposal { target_binding: EvidenceRelevanceBinding::Exact, relation_binding: EvidenceRelevanceBinding::Exact };
        let raw = EvidenceLocalQualificationV6 { identity_scope: EvidenceLocalIdentityScope::ExactTarget, relation_scope: EvidenceLocalRelationScope::RequestedRelation, scope_risk: EvidenceLocalBlockingReason::None };
        for object in ["the launch claim","the release assertion","an otherwise unowned launch claim","the explicitly unowned release statement"] {
            let text=format!("Object: {object}. Owner of the Object: Maple Queue. I retract the Object. Silver Lens was launched.");
            let local=candidate(vec![(EvidenceRelevanceSignalKind::Excerpt,&text)]);
            assert_eq!(materialize_evidence_relevance_v30(&policy,&local,Some(&proposal),Some(&raw)).unwrap().disposition,EvidenceRelevanceDisposition::Relevant,"{text}");
            for owners in ["", "Owner of the Subject: Maple Queue.", "Owner of the Object: Silver Lens.", "Owner of the Object: Maple Queue. Owner of the Object: Silver Lens.", "Owner of the Object: an unknown product."] {
                let text=format!("Object: {object}. {owners} I retract the Object. Silver Lens was launched.");
                let local=candidate(vec![(EvidenceRelevanceSignalKind::Excerpt,&text)]);
                assert_ne!(materialize_evidence_relevance_v30(&policy,&local,Some(&proposal),Some(&raw)).unwrap().disposition,EvidenceRelevanceDisposition::Relevant,"{text}");
            }
        }
        let local=candidate(vec![(EvidenceRelevanceSignalKind::Excerpt,"Object: the launch claim of Silver Lens. Owner of the Object: Maple Queue. I retract the Object. Silver Lens was launched.")]);
        assert_ne!(materialize_evidence_relevance_v30(&policy,&local,Some(&proposal),Some(&raw)).unwrap().disposition,EvidenceRelevanceDisposition::Relevant);
    }
'''
s=s.replace(anchor,anchor+tests);p.write_text(s);subprocess.run(['cargo','fmt','--all'],check=True)
Path('/private/tmp/engine073-foundation22.patch').write_text(''.join(difflib.unified_diff(base.splitlines(keepends=True),p.read_text().splitlines(keepends=True),fromfile='a/crates/reasoning-harness-core/src/evidence_relevance.rs',tofile='b/crates/reasoning-harness-core/src/evidence_relevance.rs')))
print('bounded wrapper/determiner foundation repair applied; full inner proof retained')
