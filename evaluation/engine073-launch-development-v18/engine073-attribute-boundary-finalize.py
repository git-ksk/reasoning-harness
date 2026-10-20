from pathlib import Path
p=Path('crates/reasoning-harness-core/src/evidence_relevance.rs');s=p.read_text();a=s.index('\n#[cfg(test)]\nmod development_scope_diagnostics18');s=s[:a]+'\n'
a=s.index('fn launch_coordinated_property_objects_v17');b=s.index('\nfn launch_explicit_property_context_v17',a);part=s[a:b];needle='''|| launch_retraction_other_object_v17(policy, &left_statement)''';assert part.count(needle)==1;part=part.replace(needle,'''|| (!target_identity_phrases(policy).iter().any(|target| normalized_phrase_matches(raw_left,target)) && launch_retraction_other_object_v17(policy, &left_statement))''');s=s[:a]+part+s[b:];p.write_text(s)
p=Path('crates/reasoning-harness-core/tests/launch_context_markers_520_521.rs');s=p.read_text();s+='''

#[test]
fn literal_field_types_never_retype_target_propositions_or_appended_denials() {
    let mut cases = Vec::new();
    for object in [
        "the font of the printed words \\\"Silver Lens was not launched\\\" on Silver Lens label",
        "\\\"Silver Lens lid is not copper\\\", the lid-material claim",
    ] {
        cases.push(json!({"id":format!("literal-property:{object}"),"signals":[{"kind":"excerpt","text":format!("Silver Lens was launched. Object: {object}. We deny this Object.")}],"expected_relevant":true}));
        for denial in ["We also state that Silver Lens does not exist.", "We withdraw the entire source.", "The owner of that other launch claim is unknown."] {
            cases.push(json!({"id":format!("literal-with-adverse:{object}:{denial}"),"signals":[{"kind":"excerpt","text":format!("Silver Lens was launched. Object: {object}. We deny this Object. We withdraw that other launch claim. {denial}")}],"expected_relevant":false}));
        }
    }
    for object in ["\\\"Silver Lens was launched\\\", the lid-material claim", "\\\"Silver Lens was not launched\\\", the font claim"] {
        cases.push(json!({"id":format!("event-cannot-be-retyped:{object}"),"signals":[{"kind":"excerpt","text":format!("Silver Lens was launched. Object: {object}. We deny this Object.")}],"expected_relevant":false}));
    }
    check(cases);
}

#[test]
fn every_coordinated_or_parenthetical_scope_requires_its_own_complete_type() {
    let mut cases = Vec::new();
    for (correction,expected) in [
        ("We withdraw the page-count claim about Silver Lens manual and the launch claim about Maple Queue.",true),
        ("We withdraw the launch claim about Maple Queue and the page-count claim about Silver Lens manual.",true),
        ("We withdraw the page-count claim about Silver Lens manual and the launch claim about Silver Lens.",false),
        ("We withdraw the page-count claim about Silver Lens manual and an unowned launch claim.",false),
        ("We withdraw the page-count claim about Silver Lens manual and the whole source.",false),
        ("Subject: the colour and font size of the printed title on Silver Lens guide. We retract the Subject, these two title-typography assertions.",true),
        ("Subject: the colour and launch of Silver Lens. We retract the Subject, these two title-typography assertions.",false),
        ("Subject: the colour and font size of the printed title on Silver Lens guide. We retract the Subject, these two launch assertions.",false),
        ("Object: the width of Silver Lens card. We deny that Object (the height of Silver Lens card is also withdrawn).",true),
        ("Object: the width of Silver Lens card. We deny that Object (we also state that Silver Lens does not exist).",false),
    ] {
        cases.push(json!({"id":correction,"signals":[{"kind":"excerpt","text":format!("Silver Lens was launched. {correction}")}],"expected_relevant":expected}));
    }
    cases.push(json!({"id":"canonical-both-word-cannot-be-stripped","entity":"Both Maple Queue","signals":[{"kind":"excerpt","text":"Both Maple Queue was launched. We withdraw both Maple Queue's launch claim and the page-count claim about Both Maple Queue manual."}],"expected_relevant":false}));
    check(cases);
}
''';p.write_text(s)
