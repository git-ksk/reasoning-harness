
#[test]
fn complete_foreign_material_properties_keep_their_own_owner() {
    let mut cases=Vec::new();
    for property in ["the material of Maple Queue sleeve","the paper material of Cedar Court booklet","the material assertion about Azure Hall casing"] {
        cases.push(json!({"id":format!("property:{property}"),"signals":[{"kind":"excerpt","text":format!("Silver Lens was launched. Object: {property}. We deny this Object.")}],"expected_relevant":true}));
        for adverse in ["Subject: Silver Lens does not exist.","We withdraw the entire source.","We retract the remaining launch claim; its owner is unknown."] {
            cases.push(json!({"id":format!("separate:{property}:{adverse}"),"signals":[{"kind":"excerpt","text":format!("Silver Lens was launched. Object: {property}. We deny this Object. {adverse}")}],"expected_relevant":false}));
        }
    }
    for value in ["the material of an unidentified product sleeve","the material of Maple Queue launch claim","the material of Maple Queue entire source","the paper material of No Owner booklet"] {
        cases.push(json!({"id":format!("incomplete:{value}"),"signals":[{"kind":"excerpt","text":format!("Silver Lens was launched. Object: {value}. We deny this Object.")}],"expected_relevant":false}));
    }
    check(cases);
}

#[test]
fn property_owner_names_never_create_an_event_type() {
    let mut cases=Vec::new();
    for owner in ["Launch Claim","Release Statement","Launch Hall"] {
        cases.push(json!({"id":format!("property:{owner}"),"signals":[{"kind":"excerpt","text":format!("Silver Lens was launched. Object: the material of {owner} plate. We deny this Object.")}],"expected_relevant":true}));
        for reference in ["We deny that launch claim.","We withdraw the remaining event claim; its owner is unknown."] {
            cases.push(json!({"id":format!("no-event-transfer:{owner}:{reference}"),"signals":[{"kind":"excerpt","text":format!("Silver Lens was launched. Object: the material of {owner} plate. We deny this Object. {reference}")}],"expected_relevant":false}));
        }
    }
    check(cases);
}

#[test]
fn shared_property_owner_does_not_own_a_separate_launch_role() {
    let mut cases=Vec::new();
    for (event,expected) in [("the launch of Maple Queue",true),("the launch of Silver Lens",false),("an unowned launch claim",false)] {
        cases.push(json!({"id":format!("compound:{event}"),"signals":[{"kind":"source_title","text":"Publisher: Cedar Court"},{"kind":"excerpt","text":format!("Silver Lens was launched. Object: the colour and material of Cedar Court base. Item: {event}. We deny the Object and withdraw the Item.")}],"expected_relevant":expected}));
    }
    check(cases);
}

#[test]
fn foreign_printed_properties_keep_outside_ownership_and_raw_risks() {
    let mut cases=Vec::new();
    for quoted in ["Silver Lens was launched","no launch owner supplied","required source context is missing"] {
        for (tail,expected) in [("on Maple Court guide",true),("on an unidentified product guide",false),("on Maple Court launch claim",false)] {
            cases.push(json!({"id":format!("printed:{quoted}:{tail}"),"signals":[{"kind":"excerpt","text":format!("Silver Lens was launched. We deny the font assertion about the printed sentence \"{quoted}\" {tail}. Only the font of that sentence is the withdrawn property.")}],"expected_relevant":expected}));
        }
        for denial in ["Subject: Silver Lens does not exist.","We withdraw the entire source.","The product column is missing."] {
            cases.push(json!({"id":format!("outside:{quoted}:{denial}"),"signals":[{"kind":"excerpt","text":format!("Silver Lens was launched. We deny the font assertion about the printed sentence \"{quoted}\" on Maple Court guide. {denial}")}],"expected_relevant":false}));
        }
    }
    check(cases);
    let case=json!({"signals":[{"kind":"excerpt","text":"Silver Lens was launched. Object: the font of the literal printed words \"no launch owner supplied\" on Maple Court label. We deny the accuracy of this Object, only its font assertion."}]});
    let (policy,candidate,proposal,mut raw)=inputs(&case);
    for risk in ["ownership_scope","context_gap","identity_mapping","multiple"] {
        raw.scope_risk=serde_json::from_value(json!(risk)).unwrap();
        assert_ne!(materialize_evidence_relevance_v30(&policy,&candidate,Some(&proposal),Some(&raw)).unwrap().disposition,Disposition::Relevant);
    }
}

#[test]
fn explicit_field_owner_references_require_the_same_unique_role() {
    let mut cases=Vec::new();
    for suffix in ["whose owner is in the heading","whose owner is specified in the source title"] {
        for (owners,expected) in [(vec!["Owner of the Item: Maple Queue"],true),(vec!["Owner of the Object: Maple Queue"],false),(vec!["Owner of the Item: Silver Lens"],false),(vec!["Owner of the Item: Maple Queue","Owner of the Item: Cedar Court"],false)] {
            let mut signals=vec![json!({"kind":"excerpt","text":"Silver Lens was launched."}),json!({"kind":"source_title","text":"Publisher: Maple Queue"})];
            for owner in &owners { signals.push(json!({"kind":"heading","text":owner})); }
            signals.push(json!({"kind":"excerpt","text":format!("Object: the material of Maple Queue plate. We deny the Object. Item: an unowned launch claim. We retract the Item {suffix}.")}));
            cases.push(json!({"id":format!("{suffix}:{owners:?}"),"signals":signals,"expected_relevant":expected}));
        }
    }
    check(cases);
}

#[test]
fn complete_property_anaphors_and_foreign_adjuncts_preserve_actual_denials() {
    let mut cases=Vec::new();
    for reference in ["title-font","launch","entire-report"] {
        cases.push(json!({"id":format!("anaphor:{reference}"),"signals":[{"kind":"excerpt","text":format!("Silver Lens was launched. \"The title font of Maple Court guide is large.\" Author reply: I deny that complete {reference} claim.")}],"expected_relevant":reference=="title-font"}));
    }
    for owner in ["Maple Queue","Silver Lens","an unidentified product"] {
        for adjunct in ["as a further event correction","as a separate event claim","as an additional event correction"] {
            cases.push(json!({"id":format!("{owner}:{adjunct}"),"signals":[{"kind":"excerpt","text":format!("Silver Lens was launched. Object: the material of Cedar Court sleeve. We deny the Object. We retract the launch of {owner} {adjunct}.")}],"expected_relevant":owner=="Maple Queue"}));
        }
    }
    check(cases);
}
