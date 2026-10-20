from pathlib import Path
p=Path.cwd()/'crates/reasoning-harness-core/src/evidence_relevance.rs';s=p.read_text();anchor='    #[test]\n    fn v30_copular_roles_preserve_property_and_target_denial_boundaries()'
new=r'''    #[test]
    fn v30_printed_title_literals_keep_outside_property_scope() {
        for quote in ["'", "\"", "‘"] {
            let close = if quote == "‘" { "’" } else { quote };
            check_current_role_context(&format!("The Object is the printed cover title {quote}Launch notice{close} on the Silver Lens booklet. We deny this Object."), true);
        }
        for context in [
            "The Object is the printed cover title 'Launch notice on the Silver Lens booklet. We deny this Object.",
            "The Object is the printed cover title 'Launch notice' on an unknown booklet. We deny this Object.",
            "The Object is the printed cover title 'Launch notice' on the Silver Lens booklet. We deny this Object and deny that Silver Lens was launched.",
        ] { check_current_role_context(context, false); }
    }

    #[test]
    fn v30_possessive_role_properties_require_the_same_proven_property() {
        for (context, expected) in [
            ("The Object is the paper weight of the Silver Lens quick guide. We retract this Object. We also deny the claim about that Object's paper weight.", true),
            ("The Object is unknown. We deny the claim about that Object's paper weight.", false),
            ("The Object is the paper weight of the Silver Lens quick guide. We deny the claim about that Subject's paper weight.", false),
            ("The Object is the paper weight of the Silver Lens quick guide. We deny the claim about that Object's launch.", false),
            ("The Object is the paper weight of the Silver Lens quick guide. We deny the claim about that Object's cover colour.", false),
        ] { check_current_role_context(context, expected); }
    }

    #[test]
    fn v30_local_role_appositions_and_denial_targets_need_complete_scope() {
        for (context, expected) in [
            ("We deny the Subject, the dotted line printed around the Silver Lens diagram, and withdraw the separate claim that Violet Bell was launched.", true),
            ("We deny the Subject, the claim that Silver Lens was launched.", false),
            ("We deny the Subject, the dotted line printed around an unknown diagram.", false),
            ("We deny the Object. Denial target: the raised lettering on the Silver Lens backplate. The dispute is about that physical lettering.", true),
            ("We deny the Object. Denial target: the claim that Silver Lens was launched.", false),
            ("We deny the Object. Denial target: unknown.", false),
            ("Object: unknown. We deny the Object. Denial target: the raised lettering on the Silver Lens backplate.", false),
            ("We deny the Object. Silver Lens was launched. Denial target: the raised lettering on the Silver Lens backplate.", false),
        ] { check_current_role_context(context, expected); }
    }

    #[test]
    fn v30_printed_illustration_is_a_property_only_with_its_named_carrier() {
        for (context, expected) in [
            ("The Subject is the opening-pattern illustration printed on the Silver Lens package. We retract that Subject.", true),
            ("The Subject is the opening-pattern illustration printed on an unknown package. We retract that Subject.", false),
            ("The Subject is the opening of Silver Lens. We retract that Subject.", false),
        ] { check_current_role_context(context, expected); }
    }

    #[test]
    fn v30_quoted_unowned_event_cannot_borrow_or_conflict_owners() {
        for (owners, expected) in [
            ("Object owner is Violet Bell", true),
            ("Object owner is Silver Lens", false),
            ("Object owner is unknown", false),
            ("Subject owner is Violet Bell", false),
            ("Object owner is Violet Bell. Second Object owner is Cedar Basket", false),
        ] {
            check_current_role_context(&format!("The Object is the quoted launch claim: 'The product was launched.' Immediate author reply: {owners}. We retract this Object."), expected);
        }
        check_current_role_context("The Object is the quoted launch claim: 'Silver Lens was launched.' Object owner: Violet Bell. We retract this Object.", false);
    }

    #[test]
    fn v30_complete_opaque_name_keeps_real_clipping_and_raw_risk_authoritative() {
        let mut policy = launch_policy_for_v28();
        policy.entity.as_mut().unwrap().canonical_name = "Rowan Estuary Clip".to_owned();
        policy.entity.as_mut().unwrap().aliases.clear();
        policy.target_question = "Was Rowan Estuary Clip launched?".to_owned();
        let proposal = EvidenceRelevanceBindingProposal { target_binding: EvidenceRelevanceBinding::Exact, relation_binding: EvidenceRelevanceBinding::Exact };
        for (note, raw_risk, expected) in [
            ("The full canonical product name here is Rowan Estuary Clip. The Subject is the diagonal grain on the Rowan Estuary Clip wooden insert. We withdraw the statement about this Subject.", EvidenceLocalBlockingReason::None, true),
            ("The full canonical product name here is Rowan Estuary Clip. The product row is clipped and its owner is not shown.", EvidenceLocalBlockingReason::None, false),
            ("The full canonical product name here is Rowan Estuary Clip.", EvidenceLocalBlockingReason::ContextGap, false),
            ("The full canonical product name here is Rowan Estuary Clip. We deny that Rowan Estuary Clip was launched.", EvidenceLocalBlockingReason::None, false),
        ] {
            let raw = EvidenceLocalQualificationV6 { identity_scope: EvidenceLocalIdentityScope::ExactTarget, relation_scope: EvidenceLocalRelationScope::RequestedRelation, scope_risk: raw_risk };
            let c = candidate(vec![(EvidenceRelevanceSignalKind::Excerpt,"Rowan Estuary Clip was launched."),(EvidenceRelevanceSignalKind::Excerpt,note)]);
            let actual=materialize_evidence_relevance_v30(&policy,&c,Some(&proposal),Some(&raw)).unwrap();
            assert_eq!(actual.disposition==EvidenceRelevanceDisposition::Relevant,expected,"{note}");
        }
    }

'''+anchor
assert s.count(anchor)==1;s=s.replace(anchor,new,1);p.write_text(s)
