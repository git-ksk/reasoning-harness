use reasoning_harness_core::{
    EvidenceLocalBlockingReason, EvidenceLocalIdentityScope, EvidenceLocalQualificationV6,
    EvidenceLocalRelationScope, EvidenceRelevanceAssessmentBudget, EvidenceRelevanceBinding,
    EvidenceRelevanceBindingProposal, EvidenceRelevanceCandidate, EvidenceRelevanceDisposition,
    EvidenceRelevanceIdentityRequirement, EvidenceRelevanceRelationKind, EvidenceRelevanceSignal,
    EvidenceRelevanceSignalKind, EvidenceRelevanceTargetPolicy, EvidenceTargetEntityIdentity,
    derive_effective_evidence_local_qualification_v8,
    derive_effective_evidence_local_qualification_v9, materialize_evidence_relevance_v21,
    materialize_evidence_relevance_v22,
};

fn availability_policy(name: &str) -> EvidenceRelevanceTargetPolicy {
    EvidenceRelevanceTargetPolicy {
        policy_id: format!("successor-v7-control-{name}"),
        target_id: format!("target-{name}"),
        target_question: format!("Where does {name} operate?"),
        entity: Some(EvidenceTargetEntityIdentity {
            canonical_id: format!("entity-{name}"),
            canonical_name: name.to_string(),
            aliases: vec![],
        }),
        relation: EvidenceRelevanceRelationKind::Availability,
        identity_requirement: EvidenceRelevanceIdentityRequirement::RequireHarnessAnchor,
        assessment_budget: EvidenceRelevanceAssessmentBudget {
            max_model_attempts: 2,
            max_tokens: 192,
            max_elapsed_ms: 60_000,
        },
    }
}

fn candidate(
    id: &str,
    signals: Vec<(EvidenceRelevanceSignalKind, &str)>,
) -> EvidenceRelevanceCandidate {
    EvidenceRelevanceCandidate {
        evidence_id: format!("evidence-{id}"),
        source_id: format!("source-{id}"),
        signals: signals
            .into_iter()
            .map(|(kind, text)| EvidenceRelevanceSignal {
                kind,
                text: text.to_string(),
            })
            .collect(),
    }
}

fn negative_proposal() -> EvidenceRelevanceBindingProposal {
    EvidenceRelevanceBindingProposal {
        target_binding: EvidenceRelevanceBinding::Different,
        relation_binding: EvidenceRelevanceBinding::Exact,
    }
}

fn distinct_requested_raw() -> EvidenceLocalQualificationV6 {
    EvidenceLocalQualificationV6 {
        identity_scope: EvidenceLocalIdentityScope::DistinctTarget,
        relation_scope: EvidenceLocalRelationScope::RequestedRelation,
        scope_risk: EvidenceLocalBlockingReason::None,
    }
}

#[test]
fn url_only_single_near_sibling_identity_floor_is_relation_orthogonal() {
    let policy = availability_policy("Silver Finch");
    let candidate = candidate(
        "url-paraphrase",
        vec![
            (
                EvidenceRelevanceSignalKind::CanonicalUrl,
                "https://docs.example.test/silver-finch/locations",
            ),
            (
                EvidenceRelevanceSignalKind::Excerpt,
                "Silver Hawk operates in Basin and Prairie areas.",
            ),
        ],
    );
    let proposal = negative_proposal();
    let raw = distinct_requested_raw();

    let historical = derive_effective_evidence_local_qualification_v8(
        &policy,
        &candidate,
        Some(&proposal),
        Some(&raw),
    )
    .unwrap();
    assert_eq!(
        historical.identity_scope,
        EvidenceLocalIdentityScope::DistinctTarget
    );
    assert_eq!(
        materialize_evidence_relevance_v21(&policy, &candidate, Some(&proposal), Some(&raw))
            .unwrap()
            .disposition,
        EvidenceRelevanceDisposition::Irrelevant
    );

    let successor = derive_effective_evidence_local_qualification_v9(
        &policy,
        &candidate,
        Some(&proposal),
        Some(&raw),
    )
    .unwrap();
    assert_eq!(
        successor.identity_scope,
        EvidenceLocalIdentityScope::Unresolved
    );
    assert_eq!(
        successor.relation_scope,
        EvidenceLocalRelationScope::RequestedRelation
    );
    assert_eq!(
        materialize_evidence_relevance_v22(&policy, &candidate, Some(&proposal), Some(&raw))
            .unwrap()
            .disposition,
        EvidenceRelevanceDisposition::Ambiguous
    );
}

#[test]
fn navigation_only_single_near_sibling_uses_the_same_identity_floor() {
    let policy = availability_policy("Copper Finch");
    let candidate = candidate(
        "navigation-paraphrase",
        vec![
            (
                EvidenceRelevanceSignalKind::NavigationOrFooter,
                "Copper Finch",
            ),
            (
                EvidenceRelevanceSignalKind::Excerpt,
                "Copper Hawk operates across Basin and Prairie areas.",
            ),
        ],
    );
    let proposal = negative_proposal();
    let raw = distinct_requested_raw();

    let successor = derive_effective_evidence_local_qualification_v9(
        &policy,
        &candidate,
        Some(&proposal),
        Some(&raw),
    )
    .unwrap();
    assert_eq!(
        successor.identity_scope,
        EvidenceLocalIdentityScope::Unresolved
    );
    assert_eq!(
        materialize_evidence_relevance_v22(&policy, &candidate, Some(&proposal), Some(&raw))
            .unwrap()
            .disposition,
        EvidenceRelevanceDisposition::Ambiguous
    );
}

#[test]
fn missing_target_single_near_sibling_uses_the_same_identity_floor() {
    let policy = availability_policy("Quartz Relay");
    let candidate = candidate(
        "no-target-paraphrase",
        vec![(
            EvidenceRelevanceSignalKind::Excerpt,
            "Quartz Link operates across Delta and Harbor areas.",
        )],
    );
    let proposal = negative_proposal();
    let raw = distinct_requested_raw();

    let successor = derive_effective_evidence_local_qualification_v9(
        &policy,
        &candidate,
        Some(&proposal),
        Some(&raw),
    )
    .unwrap();
    assert_eq!(
        successor.identity_scope,
        EvidenceLocalIdentityScope::Unresolved
    );
    assert_eq!(
        materialize_evidence_relevance_v22(&policy, &candidate, Some(&proposal), Some(&raw))
            .unwrap()
            .disposition,
        EvidenceRelevanceDisposition::Ambiguous
    );
}

#[test]
fn explicit_distinct_identity_evidence_is_not_weakened() {
    let policy = availability_policy("Birch Relay");
    let candidate = candidate(
        "explicit-distinct",
        vec![(
            EvidenceRelevanceSignalKind::Excerpt,
            "Cobalt Relay is a separate service from Birch Relay and is available in North.",
        )],
    );
    let proposal = negative_proposal();
    let raw = distinct_requested_raw();

    let successor = derive_effective_evidence_local_qualification_v9(
        &policy,
        &candidate,
        Some(&proposal),
        Some(&raw),
    )
    .unwrap();
    assert_eq!(
        successor.identity_scope,
        EvidenceLocalIdentityScope::DistinctTarget
    );
}

#[test]
fn content_bearing_target_anchor_is_not_downgraded() {
    let policy = availability_policy("Amber Relay");
    let candidate = candidate(
        "content-anchor",
        vec![
            (EvidenceRelevanceSignalKind::Heading, "Amber Relay"),
            (
                EvidenceRelevanceSignalKind::Excerpt,
                "Amber Link operates in Basin while Amber Relay remains the documented target.",
            ),
        ],
    );
    let proposal = EvidenceRelevanceBindingProposal {
        target_binding: EvidenceRelevanceBinding::Exact,
        relation_binding: EvidenceRelevanceBinding::Exact,
    };
    let raw = EvidenceLocalQualificationV6 {
        identity_scope: EvidenceLocalIdentityScope::ExactTarget,
        relation_scope: EvidenceLocalRelationScope::RequestedRelation,
        scope_risk: EvidenceLocalBlockingReason::None,
    };

    let successor = derive_effective_evidence_local_qualification_v9(
        &policy,
        &candidate,
        Some(&proposal),
        Some(&raw),
    )
    .unwrap();
    assert_eq!(
        successor.identity_scope,
        EvidenceLocalIdentityScope::ExactTarget
    );
}

#[test]
fn identity_floor_does_not_manufacture_relation_scope() {
    let policy = availability_policy("Indigo Mesh");
    let candidate = candidate(
        "relation-orthogonality",
        vec![
            (
                EvidenceRelevanceSignalKind::CanonicalUrl,
                "https://docs.example.test/indigo-mesh/locations",
            ),
            (
                EvidenceRelevanceSignalKind::Excerpt,
                "Indigo Link has a separate operational note.",
            ),
        ],
    );

    for (proposal_relation, raw_relation) in [
        (
            EvidenceRelevanceBinding::Different,
            EvidenceLocalRelationScope::DifferentRelation,
        ),
        (
            EvidenceRelevanceBinding::Unresolved,
            EvidenceLocalRelationScope::Unresolved,
        ),
    ] {
        let proposal = EvidenceRelevanceBindingProposal {
            target_binding: EvidenceRelevanceBinding::Different,
            relation_binding: proposal_relation,
        };
        let raw = EvidenceLocalQualificationV6 {
            identity_scope: EvidenceLocalIdentityScope::DistinctTarget,
            relation_scope: raw_relation,
            scope_risk: EvidenceLocalBlockingReason::None,
        };
        let successor = derive_effective_evidence_local_qualification_v9(
            &policy,
            &candidate,
            Some(&proposal),
            Some(&raw),
        )
        .unwrap();
        assert_eq!(
            successor.identity_scope,
            EvidenceLocalIdentityScope::Unresolved
        );
        assert_eq!(successor.relation_scope, raw_relation);
        assert_eq!(
            materialize_evidence_relevance_v22(&policy, &candidate, Some(&proposal), Some(&raw))
                .unwrap()
                .disposition,
            EvidenceRelevanceDisposition::Ambiguous
        );
    }
}
