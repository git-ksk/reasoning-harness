    #[test]
    fn v30_owned_appearance_predicates_require_component_and_non_event_value() {
        let policy=launch_policy_for_v28();
        let proposal=EvidenceRelevanceBindingProposal {target_binding:EvidenceRelevanceBinding::Exact,relation_binding:EvidenceRelevanceBinding::Exact};
        let raw=EvidenceLocalQualificationV6 {identity_scope:EvidenceLocalIdentityScope::ExactTarget,relation_scope:EvidenceLocalRelationScope::RequestedRelation,scope_risk:EvidenceLocalBlockingReason::None};
        for property in [
            "the Cedar Basket manual has a bent corner",
            "the certificate supplied with Violet Bell has a blue watermark",
            "the Silver Lens label has a ribbed edge",
            "the battery table in the Silver Lens booklet prints its units in amperes",
            "the Jade Parcel sleeve prints its address in bold",
            "the Opal Button cover prints its revision mark inside a triangle",
        ] {
            let text=format!("Object: the claim that {property}. I retract the Object. Silver Lens was launched.");
            let local=candidate(vec![(EvidenceRelevanceSignalKind::Excerpt,&text)]);
            assert_eq!(materialize_evidence_relevance_v30(&policy,&local,Some(&proposal),Some(&raw)).unwrap().disposition,EvidenceRelevanceDisposition::Relevant,"{text}");
            for denial in ["Silver Lens was not launched.","We retract the whole source.","The event owner column is missing."] {
                let negative=format!("{text} {denial}");
                let local=candidate(vec![(EvidenceRelevanceSignalKind::Excerpt,&negative)]);
                assert_ne!(materialize_evidence_relevance_v30(&policy,&local,Some(&proposal),Some(&raw)).unwrap().disposition,EvidenceRelevanceDisposition::Relevant,"{negative}");
            }
        }
        for body in [
            "the unknown owner's manual has a bent corner",
            "the manual has a bent corner",
            "the Cedar Basket manual has a launch announcement",
            "the Cedar Basket manual prints its address in a denied launch",
            "the Silver Lens service has a launch date",
            "the Cedar Basket manual has a bent corner and Silver Lens was not launched",
        ] {
            let text=format!("Object: the claim that {body}. I retract the Object. Silver Lens was launched.");
            let local=candidate(vec![(EvidenceRelevanceSignalKind::Excerpt,&text)]);
            assert_ne!(materialize_evidence_relevance_v30(&policy,&local,Some(&proposal),Some(&raw)).unwrap().disposition,EvidenceRelevanceDisposition::Relevant,"{text}");
        }
        let local=candidate(vec![(EvidenceRelevanceSignalKind::Excerpt,"Subject: the claim that the Cedar Basket manual has a bent corner. Object: an unowned launch claim. I withdraw the Subject and retract the Object. Silver Lens was launched.")]);
        assert_ne!(materialize_evidence_relevance_v30(&policy,&local,Some(&proposal),Some(&raw)).unwrap().disposition,EvidenceRelevanceDisposition::Relevant);
    }
