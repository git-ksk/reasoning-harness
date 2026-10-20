
#[test]
fn property_anaphors_bind_owner_qualified_propositions_without_component_name_enumeration() {
    let mut cases = Vec::new();
    for component in ["housing", "casing", "enclosure"] {
        for actor in ["We", "I"] {
            for wrapper in ["", "the accuracy of "] {
                cases.push(json!({"id":format!("bound:{component}:{actor}:{wrapper}"),"signals":[{"kind":"excerpt","text":format!("Silver Lens was launched. The material of Silver Lens {component} is steel. {actor} deny {wrapper}that {component}-material assertion.")}],"expected_relevant":true}));
            }
        }
    }
    for reference in ["launch-material", "source-material", "entire-material"] {
        cases.push(json!({"id":format!("type-cannot-transfer:{reference}"),"signals":[{"kind":"excerpt","text":format!("Silver Lens was launched. The material of Silver Lens housing is steel. We deny that {reference} assertion.")}],"expected_relevant":false}));
    }
    cases.push(json!({"id":"unknown-launch-after-property-reference","signals":[{"kind":"excerpt","text":"Silver Lens was launched. The material of Silver Lens housing is steel. We deny that housing-material assertion. We withdraw that other launch claim; its owner is unknown."}],"expected_relevant":false}));
    check(cases);
}

#[test]
fn coordinated_predicates_preserve_all_typed_roles_and_owner_flips() {
    let mut cases = Vec::new();
    for item in ["the launch of Maple Queue", "the launch of Silver Lens", "an unowned launch claim", "the entire source"] {
        for correction in [
            "We deny the Object and Subject and withdraw the Item.",
            "I deny the Subject and Object and withdraw the Item.",
            "We withdraw the Item and deny the Object and Subject.",
        ] {
            cases.push(json!({"id":format!("{item}:{correction}"),"signals":[{"kind":"excerpt","text":format!("Silver Lens was launched. Object: the material of Silver Lens lid. Subject: the colour of Silver Lens seal. Item: {item}. {correction}")}],"expected_relevant":item=="the launch of Maple Queue"}));
        }
    }
    check(cases);
}

#[test]
fn fully_named_foreign_references_cannot_retarget_explicit_roles() {
    let mut cases = Vec::new();
    for foreign in ["Maple Queue", "Cedar Bolt"] {
        cases.push(json!({"id":format!("named-event:{foreign}"),"signals":[{"kind":"excerpt","text":format!("Silver Lens was launched. Object: the material of Silver Lens base. We deny this Object. \"{foreign} was launched.\" I withdraw that launch claim for {foreign}.")}],"expected_relevant":true}));
        cases.push(json!({"id":format!("named-adjunct:{foreign}"),"signals":[{"kind":"excerpt","text":format!("Silver Lens was launched. We withdraw the launch of {foreign} as a separate event claim.")}],"expected_relevant":true}));
        cases.push(json!({"id":format!("role-cannot-retarget:{foreign}"),"signals":[{"kind":"excerpt","text":format!("Silver Lens was launched. Object: the launch of Silver Lens. We withdraw this Object, the event claim for {foreign}.")}],"expected_relevant":false}));
    }
    for owner in ["Silver Lens", "an unidentified product"] {
        cases.push(json!({"id":format!("unsafe-event-reference:{owner}"),"signals":[{"kind":"excerpt","text":format!("Silver Lens was launched. Object: the material of Silver Lens base. We deny this Object. I withdraw that launch claim for {owner}.")}],"expected_relevant":false}));
    }
    check(cases);
}

#[test]
fn literal_field_operators_keep_actual_source_and_target_denials_separate() {
    let mut cases = Vec::new();
    for field in ["font", "typography", "colour"] {
        let declaration=format!("Subject: the {field} of the literal printed words \"we withdraw this entire source\" on Silver Lens card. We deny that Subject.");
        cases.push(json!({"id":format!("literal:{field}"),"signals":[{"kind":"excerpt","text":format!("Silver Lens was launched. {declaration}")}],"expected_relevant":true}));
        for denial in ["We withdraw the entire source.", "We also state that Silver Lens does not exist."] {
            cases.push(json!({"id":format!("actual-denial:{field}:{denial}"),"signals":[{"kind":"excerpt","text":format!("Silver Lens was launched. {declaration} {denial}")}],"expected_relevant":false}));
        }
    }
    check(cases);
}

#[test]
fn property_self_definitions_and_passive_references_require_a_non_event_value() {
    let mut cases = Vec::new();
    for noun in ["assertion", "claim", "statement"] {
        for (value, expected) in [
            ("the printed page-count claim about Silver Lens manual", true),
            ("the launch claim about Silver Lens", false),
            ("an unidentified event claim", false),
            ("the entire source", false),
        ] {
            cases.push(json!({"id":format!("self-definition:{noun}:{value}"),"signals":[{"kind":"excerpt","text":format!("Silver Lens was launched. The {noun} we deny is this, {value}. That complete document-property assertion is withdrawn by us.")}],"expected_relevant":expected}));
        }
    }
    check(cases);
}
