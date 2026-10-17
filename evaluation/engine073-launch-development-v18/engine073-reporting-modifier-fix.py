from pathlib import Path
p=Path('crates/reasoning-harness-core/src/evidence_relevance.rs');s=p.read_text();old='''                        ["report", "reports", "record", "records", "state", "states"]
                            .into_iter()
                            .map(move |verb| format!("{actor} {verb} that "))''';new='''                        ["report", "reports", "record", "records", "state", "states"]
                            .into_iter()
                            .flat_map(move |verb| {
                                ["", "also ", "explicitly ", "formally ", "clearly "]
                                    .into_iter()
                                    .map(move |modifier| format!("{actor} {modifier}{verb} that "))
                            })''';assert s.count(old)==1;s=s.replace(old,new);p.write_text(s)
p=Path('crates/reasoning-harness-core/tests/launch_context_markers_520_521.rs');s=p.read_text();a=s.index('#[test]\nfn engine073_independent_v16_seen_replay_preserves_full_scope()');b=s.index('\n#[test]',a+10);s+='\n'+s[a:b].replace('v16','v18')
s+='''

#[test]
fn positive_reporting_modifiers_preserve_inner_nonexistence_and_source_denial() {
    let mut cases = Vec::new();
    for actor in ["we", "the editor", "the author"] {
        for verb in ["report", "records", "states"] {
            for modifier in ["", "also ", "explicitly ", "formally ", "clearly "] {
                for denial in ["Silver Lens does not exist", "Silver Lens was not launched", "the whole source is retracted"] {
                    cases.push(json!({"id":format!("{actor}:{modifier}{verb}:{denial}"),"signals":[{"kind":"excerpt","text":format!("Silver Lens was launched. Subject: the font of Silver Lens packing list. We deny that Subject although {actor} {modifier}{verb} that {denial}.")}],"expected_relevant":false}));
                }
            }
        }
    }
    cases.push(json!({"id":"nested-positive-shells","signals":[{"kind":"excerpt","text":"Silver Lens was launched. We also state that the editor clearly records that Silver Lens does not exist."}],"expected_relevant":false}));
    cases.push(json!({"id":"reporting-property-positive","signals":[{"kind":"excerpt","text":"Silver Lens was launched. We also state that the colour of Silver Lens cap is blue."}],"expected_relevant":true}));
    check(cases);
}
''';p.write_text(s)
