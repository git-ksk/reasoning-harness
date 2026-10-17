    #[test]
    fn v30_printed_attribute_requires_outside_type_owner_and_balanced_literal() {
        let policy=launch_policy_for_v28();
        let proposal=EvidenceRelevanceBindingProposal {target_binding:EvidenceRelevanceBinding::Exact,relation_binding:EvidenceRelevanceBinding::Exact};
        let raw=EvidenceLocalQualificationV6 {identity_scope:EvidenceLocalIdentityScope::ExactTarget,relation_scope:EvidenceLocalRelationScope::RequestedRelation,scope_risk:EvidenceLocalBlockingReason::None};
        for property in [
            "the sentence \"Silver Lens was not launched\" is printed in green ink on the Marble Basket insert",
            "the section heading \"Launch status\" in the Amber Parcel catalog uses small capitals",
            "the heading \"Silver Lens was launched\" occupies three lines on the Indigo Gate instruction sheet",
            "the engraving \"Silver Lens was launched\" on the Birch Token badge is shallow",
            "the literal footer \"owner not supplied\" in the Quartz Button booklet is underlined",
        ] {
            let text=format!("Object: the claim that {property}. I retract the Object. Silver Lens was launched.");
            let local=candidate(vec![(EvidenceRelevanceSignalKind::Excerpt,&text)]);
            assert_eq!(materialize_evidence_relevance_v30(&policy,&local,Some(&proposal),Some(&raw)).unwrap().disposition,EvidenceRelevanceDisposition::Relevant,"{text}");
            for denial in ["Silver Lens was not launched.","We retract the whole source.","The event owner column is missing."] {
                let negative=format!("{text} {denial}");let local=candidate(vec![(EvidenceRelevanceSignalKind::Excerpt,&negative)]);
                assert_ne!(materialize_evidence_relevance_v30(&policy,&local,Some(&proposal),Some(&raw)).unwrap().disposition,EvidenceRelevanceDisposition::Relevant,"{negative}");
            }
        }
        for value in [
            "the sentence \"Silver Lens was not launched\" is our present assertion on the Marble Basket insert",
            "the claim \"Silver Lens was launched\" quoted in the Marble Basket manual",
            "the sentence \"Silver Lens was not launched\" is printed in green ink on an unknown product insert",
            "the sentence \"Silver Lens was not launched is printed in green ink on the Marble Basket insert",
            "the engraving \"Silver Lens was launched\" on the Birch Token badge is a false event claim",
        ] {
            let text=format!("Object: the claim that {value}. I retract the Object. Silver Lens was launched.");let local=candidate(vec![(EvidenceRelevanceSignalKind::Excerpt,&text)]);
            assert_ne!(materialize_evidence_relevance_v30(&policy,&local,Some(&proposal),Some(&raw)).unwrap().disposition,EvidenceRelevanceDisposition::Relevant,"{text}");
        }
    }
