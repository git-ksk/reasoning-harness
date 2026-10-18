from pathlib import Path
import subprocess,shutil,json,gzip,hashlib
r=Path.cwd();p=r/'crates/reasoning-harness-core/src/evidence_relevance.rs';s=p.read_text();assert hashlib.sha256(p.read_bytes()).hexdigest()=='5264d210fb25f1c6f6c3c2eec3e50ff42379947f99bd64242ae761e019bd4711'
d=r/'evaluation/engine073-launch-development-v25-role-prototypes/superseded-root-28cd088';d.mkdir()
shutil.move(str(r/'evaluation/engine073-launch-development-v25-role-frames'),str(d/'partial-development'))
for src,name in [('/private/tmp/engine073-source26-actual-final-measurement.log','measurement-failed.log'),('/private/tmp/engine073-source26-final2-validation-controller.log','validation-failed.log'),('/private/tmp/engine073-source26-actual-build.jsonl','cargo-build.jsonl')]:shutil.copyfile(src,d/name)
for n in ['clippy','fmt','workspace']:shutil.copyfile('/private/tmp/engine073-source26-final2-verified-'+n+'.log',d/(n+'.log'))
(d/'source.rs.gz').write_bytes(gzip.compress(p.read_bytes(),mtime=0));(d/'status.json').write_text(json.dumps({'source_commit':'28cd088192bcaa15e9d064e025b6a1a4c67d0ade','source_sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'status':'REJECTED_REGRESSION_NOT_ACCEPTANCE','known_regression_ids':['li24-025','li24-041'],'new_parent_test_error':'Foreign Violet Bell launch and physical colour are both excluded from Silver Lens launch; expected false was incorrect. Original independent labels untouched. Add target-owned event counterpart expected false.'},indent=2)+'\n')
a='''                let bounded_restatement = physical
'''
b='''                let bounded_body = body
                    .strip_prefix(&["only"])
                    .or_else(|| body.strip_prefix(&["solely"]))
                    .unwrap_or(body);
                let bounded_body = bounded_body
                    .strip_prefix(&["the"])
                    .or_else(|| bounded_body.strip_prefix(&["a"]))
                    .unwrap_or(bounded_body);
                let property_summary = matches!(bounded_body, ["typography"])
                    && prior.iter().all(|(_, prior_value)| {
                        let prior_text = normalized(prior_value);
                        prior_text.split_whitespace().any(|word| word == "printed")
                    })
                    || matches!(bounded_body, ["casing", "material", "claim"])
                        && prior.iter().all(|(_, prior_value)| {
                            let prior_words = launch_property_words_v17(policy, prior_value);
                            prior_words.iter().any(|word| word == "casing")
                                && prior_words.iter().any(|word| {
                                    ["steel", "plastic", "metal", "wood", "aluminium", "aluminum", "material"]
                                        .contains(&word.as_str())
                                })
                        });
                // A title reference may restate an already complete foreign
                // event of this role. It cannot supply a missing event owner.
                let title_event_restatement = matches!(
                    bounded_body,
                    ["event", "declared", "in", "the", "title"]
                ) && !prior.is_empty()
                    && prior.iter().all(|(_, prior_value)| {
                        launch_complete_foreign_role_owner_v17(policy, prior_value).is_some()
                            && candidate.signals.iter().any(|source| {
                                source.kind == EvidenceRelevanceSignalKind::SourceTitle
                                    && source.text.split_once(':').is_some_and(|(label, value)| {
                                        normalized(label) == role
                                            && normalized(value) == normalized(prior_value)
                                    })
                            })
                    });
                let bounded_restatement = (physical || property_summary)
'''
assert s.count(a)==1;s=s.replace(a,b,1)
s=s.replace('''                if bounded_restatement {
''','''                if bounded_restatement || title_event_restatement {
''',1)
a='''                "Object: the launch of Violet Bell. The Object is physical colour. We deny this Object.",
                false,'''
b='''                "Object: the launch of Violet Bell. The Object is physical colour. We deny this Object.",
                true,
            ),
            (
                "Object: the launch of Silver Lens. The Object is physical colour. We deny this Object.",
                false,'''
assert s.count(a)==1;s=s.replace(a,b,1)
# Explicit negatives for bounded summaries; original labels remain frozen.
anchor='''    #[test]
    fn v30_copular_role_owners_cannot_be_borrowed_or_conflicted()'''
new='''    #[test]
    fn v30_role_property_summaries_require_an_existing_complete_value() {
        for (context, expected) in [
            ("Subject: the claim that the Silver Lens chapter title is printed in bold. We deny the Subject. The Subject is only typography.", true),
            ("Subject: the claim that Silver Lens was launched. We deny the Subject. The Subject is only typography.", false),
            ("Subject: unknown. We deny the Subject. The Subject is only typography.", false),
            ("Object: the claim that the Silver Lens sensor casing is steel. I retract the Object. The Object is solely the casing-material claim.", true),
            ("Object: the claim that Silver Lens was launched. I retract the Object. The Object is solely the casing-material claim.", false),
            ("Object: unknown. I retract the Object. The Object is solely the casing-material claim.", false),
        ] {
            check_current_role_context(context, expected);
        }
    }

'''+anchor
assert s.count(anchor)==1;s=s.replace(anchor,new,1);p.write_text(s)
subprocess.run(['cargo','fmt','--all'],check=True)
print(hashlib.sha256(p.read_bytes()).hexdigest())
