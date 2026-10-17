from pathlib import Path
import subprocess
p=Path('crates/reasoning-harness-core/src/evidence_relevance.rs');s=p.read_text()
a=s.index('fn launch_complete_quoted_foreign_event_v17(');b=s.index('// An adjacent full-name explanation',a);v=s[a:b];needle='''    }) && tail
        .strip_prefix("quoted in ")
        .is_some_and(|owner| launch_owned_appearance_subject_v17(policy, owner))''';assert needle in v;v=v.replace(needle,'''    }) && (tail.is_empty() || tail
        .strip_prefix("quoted in ")
        .is_some_and(|owner| launch_owned_appearance_subject_v17(policy, owner)))''');s=s[:a]+v+s[b:]
a=s.index('fn launch_adverse_segments_v17(');b=s.index('fn launch_collective_event_withdrawal_v17(',a);v=s[a:b];start=v.index('    let maintained = |text: &str| {');end=v.index('    let reporting = |text: &str| {',start)
v=v[:start]+'''    let reported_non_event = |text: &str| {
        let text=normalized(text);
        ["we","i","the author","the editor","the publisher","the newsroom"].iter().any(|actor| {
            ["report","reports","record","records","state","states","explain","explains","maintain","maintains"].iter().any(|verb| {
                ["","also ","explicitly ","formally ","clearly "].iter().any(|modifier| {
                    text.strip_prefix(&format!("{actor} {modifier}{verb} that ")).is_some_and(|body|body.contains(" was not launched") || body.contains(" was not released") || body.contains(" does not exist"))
                })
            })
        })
    };
'''+v[end:];v=v.replace('maintained(text)','reported_non_event(text)');needle='''                            "report", "reports", "record", "records", "state", "states", "maintain",
                            "maintains",'''
if needle in v:v=v.replace(needle,needle+''' "explain", "explains",''',1)
else:
 # rustfmt may have placed the existing verbs on different lines.
 needle='''"maintains",
                        ]''';assert needle in v;v=v.replace(needle,'''"maintains", "explain", "explains",
                        ]''',1)
s=s[:a]+v+s[b:]
anchor='mod tests {\n    use super::*;';s=s.replace(anchor,anchor+r'''
    #[test]
    fn v30_complete_foreign_quoted_event_and_reply_owner_remain_role_scoped() {
        let policy=launch_policy_for_v28();let proposal=EvidenceRelevanceBindingProposal {target_binding:EvidenceRelevanceBinding::Exact,relation_binding:EvidenceRelevanceBinding::Exact};let raw=EvidenceLocalQualificationV6 {identity_scope:EvidenceLocalIdentityScope::ExactTarget,relation_scope:EvidenceLocalRelationScope::RequestedRelation,scope_risk:EvidenceLocalBlockingReason::None};
        for (context,ok) in [("Object: the claim \"Violet Bell was launched\". I withdraw the Object.",true),("Object: the claim \"Silver Lens was launched\". I withdraw the Object.",false),("Object: the claim \"it was launched\". I withdraw the Object.",false),("Object: the claim \"Violet Bell was launched and Silver Lens was launched\". I withdraw the Object.",false),("Subject: the launch claim. I retract the Subject. Reply: owner of the Subject is Violet Bell.",true),("Subject: the launch claim. I retract the Subject. Reply: owner of the Object is Violet Bell.",false),("Subject: the launch claim. I retract the Subject. Reply: owner of the Subject is Silver Lens.",false),("Subject: the launch claim. I retract the Subject. Reply: owner of the Subject is unknown.",false),("Subject: the launch claim. I retract the Subject. Reply: owner of the Subject is Violet Bell. Owner of the Subject: Silver Lens.",false)] {
            let text=format!("{context} Silver Lens was launched.");let local=candidate(vec![(EvidenceRelevanceSignalKind::Excerpt,&text)]);
            assert_eq!(materialize_evidence_relevance_v30(&policy,&local,Some(&proposal),Some(&raw)).unwrap().disposition==EvidenceRelevanceDisposition::Relevant,ok,"{text}");
        }
    }
    #[test]
    fn v30_foreign_role_and_reported_non_event_are_distinct_clauses() {
        let policy=launch_policy_for_v28();let proposal=EvidenceRelevanceBindingProposal {target_binding:EvidenceRelevanceBinding::Exact,relation_binding:EvidenceRelevanceBinding::Exact};let raw=EvidenceLocalQualificationV6 {identity_scope:EvidenceLocalIdentityScope::ExactTarget,relation_scope:EvidenceLocalRelationScope::RequestedRelation,scope_risk:EvidenceLocalBlockingReason::None};
        for (owner,ok) in [("Violet Bell",true),("Silver Lens",false),("an unknown product",false)] {
            let text=format!("Object: the launch of Violet Bell. I retract the Object, and the editor reports that {owner} was not launched. Subject: the claim that the Violet Bell instruction panel is translucent. I deny the Subject. Silver Lens was launched.");let local=candidate(vec![(EvidenceRelevanceSignalKind::Excerpt,&text)]);
            assert_eq!(materialize_evidence_relevance_v30(&policy,&local,Some(&proposal),Some(&raw)).unwrap().disposition==EvidenceRelevanceDisposition::Relevant,ok,"{text}");
        }
    }
''',1)
p.write_text(s);subprocess.run(['cargo','fmt','--all'],check=True)
