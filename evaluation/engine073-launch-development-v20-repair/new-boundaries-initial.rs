
#[test]
fn role_prefixed_nonexistence_precedes_attribute_words_in_proper_names() {
    let mut cases = Vec::new();
    for entity in ["Granite Shell", "Copper Font", "Ivory Material"] {
        for role in ["Object", "Subject", "Item"] {
            for denial in ["does not exist", "has never existed", "is nonexistent", "is not real"] {
                cases.push(json!({"id":format!("{entity}:{role}:{denial}"),"entity":entity,"signals":[{"kind":"excerpt","text":format!("{entity} was launched. Object: the material of {entity} rim. We deny this Object. {role}: {entity} {denial}. We maintain that {role} statement.")}],"expected_relevant":false}));
            }
        }
    }
    check(cases);
}

#[test]
fn distinct_unknown_event_owners_never_bind_to_the_previous_foreign_event() {
    let mut cases = Vec::new();
    for separate in ["remaining", "separate", "additional"] {
        for unavailable in ["its owner is unknown", "its product identity is unavailable", "its product identity is missing"] {
            cases.push(json!({"id":format!("{separate}:{unavailable}"),"signals":[{"kind":"excerpt","text":format!("Silver Lens was launched. We deny the launch of Maple Queue. We withdraw the {separate} launch claim; {unavailable} in this source. I deny the colour assertion about Silver Lens rim.")}],"expected_relevant":false}));
        }
    }
    cases.push(json!({"id":"self-contained-unknown-event","signals":[{"kind":"excerpt","text":"Silver Lens was launched. No actual launch of Maple Queue occurred. The separate event claim we withdraw has no recoverable product owner. We deny the width assertion about Silver Lens leg."}],"expected_relevant":false}));
    cases.push(json!({"id":"complete-foreign-plus-property","signals":[{"kind":"excerpt","text":"Silver Lens was launched. We deny the launch of Maple Queue. We also withdraw the remaining claim, the material assertion about Silver Lens shell."}],"expected_relevant":true}));
    check(cases);
}

#[test]
fn comma_predicates_require_each_exact_role_and_preserve_topic_owners() {
    let mut cases = Vec::new();
    for (subject, expected) in [("the launch of Maple Queue", true), ("the launch of Silver Lens", false), ("an unowned launch claim", false)] {
        for predicates in ["We deny the Object, withdraw the Subject and retract the Item.", "I retract the Item, withdraw the Subject and deny the Object."] {
            cases.push(json!({"id":format!("{subject}:{predicates}"),"signals":[{"kind":"source_title","text":"Object: the font of Silver Lens manual"},{"kind":"heading","text":format!("Subject: {subject}")},{"kind":"excerpt","text":format!("Silver Lens was launched. Item: the colour of Silver Lens cord. {predicates}")}],"expected_relevant":expected}));
        }
    }
    for owner in ["Maple Queue", "Silver Lens"] {
        cases.push(json!({"id":format!("topic:{owner}"),"signals":[{"kind":"excerpt","text":format!("Silver Lens was launched. On the topic of {owner}'s launch, I withdraw the previous statement. The withdrawn launch statement is about {owner}.")}],"expected_relevant":owner=="Maple Queue"}));
    }
    check(cases);
}

#[test]
fn exact_role_accuracy_and_field_owner_references_keep_conflicts_visible() {
    let mut cases = Vec::new();
    for (value, expected) in [("the colour of Silver Lens knob", true),("the launch of Silver Lens", false),("an unowned launch claim", false)] {
        cases.push(json!({"id":format!("accuracy:{value}"),"signals":[{"kind":"excerpt","text":format!("Silver Lens was launched. Object: {value}. Subject: the font of Silver Lens manual. We deny the accuracy of the Object and withdraw the Subject.")}],"expected_relevant":expected}));
    }
    for (owners, expected) in [(vec!["Owner of the Item: Maple Queue"],true),(vec!["Owner of the Subject: Maple Queue"],false),(vec!["Owner of the Item: Silver Lens"],false),(vec!["Owner of the Item: unknown"],false),(vec!["Owner of the Item: Maple Queue","Owner of the Item: Cedar Bolt"],false)] {
        let mut signals=vec![json!({"kind":"excerpt","text":"Silver Lens was launched."})];
        for owner in &owners { signals.push(json!({"kind":"source_title","text":owner})); }
        signals.push(json!({"kind":"excerpt","text":"We deny the launch of Cedar Bolt. Item: an unowned launch claim whose owner is specified in the source title. We retract the Item and deny the width claim about Silver Lens leg."}));
        cases.push(json!({"id":format!("owner:{owners:?}"),"signals":signals,"expected_relevant":expected}));
    }
    check(cases);
}

#[test]
fn complete_claim_appositions_do_not_retype_events_or_global_denials() {
    let mut cases = Vec::new();
    for (value, expected) in [("the material assertion about Silver Lens shell", true),("the page-width assertion about Silver Lens guide",true),("the launch of Silver Lens",false),("the event claim with an unknown owner",false),("the entire report",false)] {
        for correction in [format!("We also withdraw the remaining claim, {value}."),format!("The claim, {value}, is withdrawn by us.")] {
            cases.push(json!({"id":&correction,"signals":[{"kind":"source_title","text":"Publisher: Silver Lens"},{"kind":"excerpt","text":format!("Silver Lens was launched. We deny the launch of Maple Queue. {correction}")}],"expected_relevant":expected}));
        }
    }
    check(cases);
}

#[test]
fn carried_literal_text_needs_a_complete_external_property_and_safe_role() {
    let mut cases = Vec::new();
    for quote in ["Silver Lens does not exist", "we withdraw the entire source", "Silver Lens was never launched"] {
        let declaration=format!("Subject: the material of the Silver Lens label plate carrying the literal text \"{quote}\". We withdraw this Subject, only the plate-material assertion.");
        cases.push(json!({"id":format!("literal:{quote}"),"signals":[{"kind":"excerpt","text":format!("Silver Lens was launched. {declaration}")}],"expected_relevant":true}));
        for denial in ["Subject: Silver Lens does not exist.","We withdraw the entire source.","We withdraw the remaining launch claim; its product identity is unknown."] {
            cases.push(json!({"id":format!("actual:{quote}:{denial}"),"signals":[{"kind":"excerpt","text":format!("Silver Lens was launched. {declaration} {denial}")}],"expected_relevant":false}));
        }
    }
    cases.push(json!({"id":"material-cannot-retype-event","signals":[{"kind":"excerpt","text":"Silver Lens was launched. Subject: the launch of Silver Lens. We withdraw this Subject, only the launch-material assertion."}],"expected_relevant":false}));
    check(cases);
}
