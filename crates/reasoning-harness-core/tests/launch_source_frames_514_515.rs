use reasoning_harness_core::{
    EvidenceLocalBlockingReason as Risk, EvidenceLocalIdentityScope as Identity,
    EvidenceLocalQualificationV6, EvidenceLocalRelationScope as Relation,
    EvidenceRelevanceBinding as Binding, EvidenceRelevanceBindingProposal,
    EvidenceRelevanceCandidate, EvidenceRelevanceDisposition as Disposition,
    EvidenceRelevanceSignalKind as Kind, EvidenceRelevanceTargetPolicy,
    derive_effective_evidence_local_qualification_v16,
    derive_effective_evidence_local_qualification_v17, materialize_evidence_relevance_v29,
    materialize_evidence_relevance_v30,
};
use serde_json::json;

fn policy(target: &str) -> EvidenceRelevanceTargetPolicy {
    serde_json::from_value(json!({
        "policy_id":"launch-source-frames-514-515", "target_id":"target",
        "target_question":format!("When was {target} launched?"),
        "entity":{"canonical_id":"target", "canonical_name":target, "aliases":[]},
        "relation":"change_or_launch", "identity_requirement":"require_harness_anchor",
        "assessment_budget":{"max_model_attempts":2,"max_tokens":192,"max_elapsed_ms":90000}
    }))
    .unwrap()
}

fn candidate(kind: Kind, text: &str) -> EvidenceRelevanceCandidate {
    serde_json::from_value(json!({
        "evidence_id":"evidence", "source_id":"source",
        "signals":[{"kind":kind, "text":text}]
    }))
    .unwrap()
}

fn proposal() -> EvidenceRelevanceBindingProposal {
    EvidenceRelevanceBindingProposal {
        target_binding: Binding::Exact,
        relation_binding: Binding::Exact,
    }
}

fn qualification() -> EvidenceLocalQualificationV6 {
    EvidenceLocalQualificationV6 {
        identity_scope: Identity::ExactTarget,
        relation_scope: Relation::RequestedRelation,
        scope_risk: Risk::None,
    }
}

fn assert_cases(cases: &[(&str, &str, bool)]) {
    let mut mismatches = Vec::new();
    for (target, text, positive) in cases {
        let policy = policy(target);
        let candidate = candidate(Kind::Excerpt, text);
        let effective = derive_effective_evidence_local_qualification_v17(
            &policy,
            &candidate,
            Some(&proposal()),
            Some(&qualification()),
        )
        .unwrap();
        let assessment = materialize_evidence_relevance_v30(
            &policy,
            &candidate,
            Some(&proposal()),
            Some(&qualification()),
        )
        .unwrap();
        if (assessment.disposition == Disposition::Relevant) != *positive
            || (effective.relation_scope == Relation::RequestedRelation) != *positive
        {
            mismatches.push(format!(
                "{text:?}: expected_relevant={positive}, relation={:?}, disposition={:?}",
                effective.relation_scope, assessment.disposition
            ));
        }
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

#[test]
fn issue514_affirmative_launch_predicates() {
    assert_cases(&[
        (
            "Silver Lens",
            "Silver Lens was launched in 2026 for customers.",
            true,
        ),
        (
            "Silver Lens",
            "Silver Lens has been launched for customers.",
            true,
        ),
        (
            "Silver Lens",
            "Silver Lens was introduced in 2026 for customers.",
            true,
        ),
        (
            "Silver Lens",
            "Silver Lens is now generally available to customers.",
            true,
        ),
    ]);
}

#[test]
fn issue514_launch_negatives() {
    assert_cases(&[
        ("Silver Lens", "Silver Lens was launched?", false),
        (
            "Silver Lens",
            "If Silver Lens was launched in 2026, customers could sign up.",
            false,
        ),
        (
            "Silver Lens",
            "Silver Lens was not launched in 2026.",
            false,
        ),
        (
            "Silver Lens",
            "The vendor denied that Silver Lens was launched in 2026.",
            false,
        ),
        (
            "Silver Lens",
            "Silver Lens was launched according to an unverified rumor.",
            false,
        ),
        (
            "Silver Lens",
            "Silver Lens was launched in a fictional scenario.",
            false,
        ),
        (
            "Silver Lens",
            "Silver Lens might have been launched in 2026.",
            false,
        ),
        (
            "Silver Lens",
            "Maple Queue was launched in 2026 while Silver Lens remains in preview.",
            false,
        ),
        (
            "Silver Lens",
            "Silver Lens documentation was launched in 2026.",
            false,
        ),
        (
            "Silver Lens",
            "Silver Lens rival was launched in 2026.",
            false,
        ),
        (
            "Silver Lens",
            "The documentation for Silver Lens was launched in 2026.",
            false,
        ),
        (
            "Silver Lens",
            "The documentation for Silver Lens is now generally available.",
            false,
        ),
        (
            "Silver Lens",
            "The guide about Silver Lens has been launched.",
            false,
        ),
        (
            "Silver Lens",
            "The rival of Silver Lens became generally available.",
            false,
        ),
        (
            "Silver Lens",
            "The vendor has made the documentation for Silver Lens generally available.",
            false,
        ),
        (
            "Silver Lens",
            "The latest version of Silver Lens was launched in 2026.",
            true,
        ),
        (
            "GLM 5.3",
            "Documentation for GLM 5.3 was launched in 2026.",
            false,
        ),
        (
            "GLM 5.3",
            "A competitor to GLM 5.3 was launched in 2026.",
            false,
        ),
        (
            "GLM 5.3",
            "It is false that GLM 5.3 was launched in 2026.",
            false,
        ),
        (
            "GLM 5.3",
            "The assertion that GLM 5.3 was launched in 2026 is false.",
            false,
        ),
        ("GLM 5.3", "GLM 5.3 was launched in 2026 is false.", false),
        (
            "GLM 5.3",
            "GLM 5.3 was launched in a fabricated announcement.",
            false,
        ),
        (
            "GLM 5.3",
            "The vendor rejected the assertion that it has made GLM 5.3 generally available.",
            false,
        ),
        (
            "GLM 5.3",
            "The vendor rejects the assertion that it has made GLM 5.3 generally available.",
            false,
        ),
        (
            "GLM 5.3",
            "The vendor doubts that it has made GLM 5.3 generally available.",
            false,
        ),
        (
            "Silver Lens",
            "In 2026, Silver Lens became generally available.",
            true,
        ),
        (
            "GLM 5.3",
            "Today, GLM 5.3 was launched for customers.",
            true,
        ),
        (
            "Silver Lens",
            "Silver Lens was introduced on May 1, 2026.",
            true,
        ),
        (
            "Silver Lens",
            "In documentation for Silver Lens, the rival was launched.",
            false,
        ),
        (
            "Silver Lens",
            "Silver Lens launched a new documentation portal.",
            false,
        ),
        (
            "Silver Lens",
            "Silver Lens launched on its website a new documentation portal.",
            false,
        ),
        (
            "Silver Lens",
            "Silver Lens launched, in 2026, a new documentation portal.",
            false,
        ),
        (
            "GLM 5.3",
            "The vendor may, 2 days from now, announce it has made GLM 5.3 generally available.",
            false,
        ),
        (
            "GLM 5.3",
            "The vendor doubts it has made GLM 5.3 generally available.",
            false,
        ),
        (
            "Silver Lens",
            "Silver Lens was launched only in private preview.",
            false,
        ),
        (
            "GLM 5.3",
            "GLM 5.3 was launched only in a simulation.",
            false,
        ),
        (
            "Silver Lens",
            "Silver Lens was launched in a simulated environment.",
            false,
        ),
        (
            "GLM 5.3",
            "The vendor has made GLM 5.3 generally available in a simulated exercise.",
            false,
        ),
        (
            "Silver Lens",
            "Silver Lens was launched after its preview ended, but that assertion is untrue.",
            false,
        ),
        (
            "Silver Lens",
            "Silver Lens may be launched on May 1, 2026.",
            false,
        ),
        (
            "Silver Lens",
            "Silver Lens: Maple Queue was launched in 2026.",
            false,
        ),
        (
            "Silver Lens",
            "Silver Lens, Maple Queue was launched in 2026.",
            false,
        ),
        (
            "Silver Lens",
            "Silver Lens. Maple Queue was launched in 2026.",
            false,
        ),
        ("Silver Lens", "Silver Lens.was launched in 2026.", false),
        ("Silver Lens", "Silver Lens was launched？", false),
        (
            "Silver Lens",
            "Silver Lens was launched, possibly in 2026.",
            false,
        ),
        (
            "Silver Lens",
            "Silver Lens was launched, supposedly in 2026.",
            false,
        ),
        ("Silver Lens", "Silver Lens; was launched in 2026.", false),
        ("Silver Lens", "Silver Lens\nwas launched in 2026.", false),
        ("Silver Lens", "Silver Lens。was launched in 2026.", false),
    ]);
}

#[test]
fn issue515_dotted_names_and_decimal_versions() {
    assert_cases(&[
        (
            "GLM 5.3",
            "Amazon Bedrock has made the latest version of GLM 5.3 from Z.ai generally available. This model is a powerful option.",
            true,
        ),
        (
            "GLM 5.3",
            "Amazon Bedrock has made GLM 5.3 generally available to customers.",
            true,
        ),
        (
            "GLM 5.3",
            "GLM 5.3 is now generally available to customers.",
            true,
        ),
        ("GLM 5.3", "GLM 5.3 was launched in 2026.", true),
        (
            "Cedar 2.4.1",
            "The vendor has made Cedar 2.4.1 generally available.",
            true,
        ),
        (
            "Z.ai Forge",
            "Z.ai Forge became generally available yesterday.",
            true,
        ),
        ("U.S. Forge", "U.S. Forge was launched in 2026.", true),
        (
            "Birch 4.2",
            "Orchid.Labs has made the latest version of Birch 4.2 from Orchid.Labs generally available.",
            true,
        ),
        (
            "Birch 4.2",
            "The vendor has made Birch 4.2 from Orchid.Labs generally available.",
            true,
        ),
        (
            "Birch 4.2",
            "Orchid Research Labs has made Birch 4.2 from Orchid Research Labs generally available.",
            true,
        ),
        (
            "Birch 4.2",
            "Orchid.Labs has made Birch 4.2 from Orchid.Labs. Maple Queue is generally available.",
            false,
        ),
        (
            "Birch 4.2",
            "Orchid.Labs has made Birch 4.2 from Orchid.Labs.Maple Queue is generally available.",
            false,
        ),
        (
            "GLM 5.3",
            "注記：Z.ai has made GLM 5.3 generally available.",
            true,
        ),
    ]);
}

#[test]
fn issue515_punctuation_must_not_borrow_predicates() {
    assert_cases(&[
        (
            "Gear 4.2",
            "The vendor has made Gear 4.2 from copper.Labs—today—offer generally available equipment.",
            false,
        ),
        (
            "Gear 4.2",
            "The vendor has made Gear 4.2 from copper.Labs—now—offer generally available equipment.",
            false,
        ),
        (
            "GLM 5.3",
            "The vendor has made GLM 5.3 from Ruzani.Labs—today—stay generally available.",
            false,
        ),
        (
            "Gear 4.2",
            "The vendor has made Gear 4.2 from copper.Labs-today-offer generally available equipment.",
            false,
        ),
        (
            "Gear 4.2",
            "The vendor has made Gear 4.2 from copper.Labs offer generally available equipment.",
            false,
        ),
        (
            "GLM 5.3",
            "The vendor has made GLM 5.3 from Ruzani.Labs stay generally available.",
            false,
        ),
        (
            "GLM 5.3",
            "Acme.Services has made tools. The vendor has made GLM 5.3 from Acme.Services stay generally available.",
            false,
        ),
        (
            "Gear 4.2",
            "The vendor has made Gear 4.2 from copper.Labs are generally available for rental.",
            false,
        ),
        (
            "GLM 5.3",
            "The vendor has made GLM 5.3 from Ruzani.Labs are generally available.",
            false,
        ),
        (
            "GLM 5.3",
            "The vendor has made GLM 5.3 from Ruzani.Labs were generally available.",
            false,
        ),
        (
            "GLM 5.3",
            "The vendor has made GLM 5.3 from Ruzani.Labs became generally available.",
            false,
        ),
        (
            "GLM 5.3",
            "The vendor has made GLM 5.3 from Ruzani.Labs have become generally available.",
            false,
        ),
        (
            "GLM 5.3",
            "Acme.Services has made tools. The vendor has made GLM 5.3 from Acme.Services are generally available.",
            false,
        ),
        (
            "GLM 5.3",
            "GLM 5.3 documentation was introduced in 2026.",
            false,
        ),
        ("GLM 5.3", "GLM 5.3 Maple Queue went live in 2026.", false),
        (
            "GLM 5.3",
            "The vendor has made GLM 5.3 documentation generally available.",
            false,
        ),
        (
            "GLM 5.3",
            "The vendor has made documentation for GLM 5.3 generally available.",
            false,
        ),
        (
            "GLM 5.3",
            "Amazon Bedrock has made GLM 5.4 from Z.ai generally available.",
            false,
        ),
        (
            "GLM 5.3",
            "Amazon Bedrock has not made GLM 5.3 from Z.ai generally available.",
            false,
        ),
        (
            "GLM 5.3",
            "Amazon Bedrock has made GLM 5.3 from Z.ai not generally available.",
            false,
        ),
        (
            "GLM 5.3",
            "If Amazon Bedrock has made GLM 5.3 from Z.ai generally available, customers could sign up.",
            false,
        ),
        (
            "GLM 5.3",
            "Amazon Bedrock has made GLM 5.3 from Z.ai generally available according to unconfirmed reports.",
            false,
        ),
        (
            "GLM 5.3",
            "Amazon Bedrock has made GLM 5.3 from Z.ai generally available?",
            false,
        ),
        (
            "GLM 5.3",
            "Amazon Bedrock has made GLM 5.3 from Z.ai documentation available and Maple Queue generally available.",
            false,
        ),
        (
            "GLM 5.3",
            "Amazon Bedrock has made GLM 5.3. Maple Queue is generally available.",
            false,
        ),
        (
            "GLM 5.3",
            "Amazon Bedrock has made GLM 5.3.generally available.",
            false,
        ),
        (
            "GLM 5.3",
            "GLM 5.3 from Z.ai. Maple Queue is now generally available.",
            false,
        ),
        (
            "GLM 5.3",
            "GLM 5.3 from Z.ai.Maple Queue is now generally available.",
            false,
        ),
        (
            "GLM 5.3",
            "GLM 5.3 from Z.ai。Maple Queue is now generally available.",
            false,
        ),
        (
            "GLM 5.3",
            "GLM 5.3; Maple Queue is now generally available.",
            false,
        ),
        (
            "GLM 5.3",
            "GLM 5.3\nMaple Queue is now generally available.",
            false,
        ),
        (
            "GLM 5.3",
            "GLM 5.3 from Z.ai, but Maple Queue is now generally available.",
            false,
        ),
        (
            "GLM 5.3",
            "The vendor has made GLM 5.3 from Z.ai Inc. Maple Queue is generally available.",
            false,
        ),
    ]);
}

#[test]
fn launch_recall_requires_factual_signal_identity_and_no_risk() {
    let policy = policy("GLM 5.3");
    let text = "The vendor has made GLM 5.3 from Z.ai generally available.";
    for kind in [
        Kind::SourceTitle,
        Kind::CanonicalUrl,
        Kind::NavigationOrFooter,
    ] {
        let candidate = candidate(kind, text);
        assert_ne!(
            materialize_evidence_relevance_v30(
                &policy,
                &candidate,
                Some(&proposal()),
                Some(&qualification())
            )
            .unwrap()
            .disposition,
            Disposition::Relevant,
            "{kind:?}"
        );
    }
    let candidate = candidate(Kind::Excerpt, text);
    assert_ne!(
        materialize_evidence_relevance_v30(&policy, &candidate, None, Some(&qualification()))
            .unwrap()
            .disposition,
        Disposition::Relevant
    );
    assert_ne!(
        materialize_evidence_relevance_v30(&policy, &candidate, Some(&proposal()), None)
            .unwrap()
            .disposition,
        Disposition::Relevant
    );
    for risk in [Risk::ContextGap, Risk::Multiple] {
        let raw = EvidenceLocalQualificationV6 {
            scope_risk: risk,
            ..qualification()
        };
        assert_ne!(
            materialize_evidence_relevance_v30(&policy, &candidate, Some(&proposal()), Some(&raw))
                .unwrap()
                .disposition,
            Disposition::Relevant
        );
    }
    for identity in [Identity::DistinctTarget, Identity::Unresolved] {
        let raw = EvidenceLocalQualificationV6 {
            identity_scope: identity,
            ..qualification()
        };
        assert_ne!(
            materialize_evidence_relevance_v30(&policy, &candidate, Some(&proposal()), Some(&raw))
                .unwrap()
                .disposition,
            Disposition::Relevant
        );
    }
}

#[test]
fn historical_v16_v29_recall_remains_unchanged() {
    for (target, text, relation, disposition) in [
        (
            "Silver Lens",
            "Silver Lens was launched in 2026 for customers.",
            Relation::RequestedRelation,
            Disposition::Relevant,
        ),
        (
            "GLM 5.3",
            "Amazon Bedrock has made the latest version of GLM 5.3 from Z.ai generally available.",
            Relation::Unresolved,
            Disposition::Ambiguous,
        ),
    ] {
        let policy = policy(target);
        let candidate = candidate(Kind::Excerpt, text);
        assert_eq!(
            derive_effective_evidence_local_qualification_v16(
                &policy,
                &candidate,
                Some(&proposal()),
                Some(&qualification())
            )
            .unwrap()
            .relation_scope,
            relation
        );
        assert_eq!(
            materialize_evidence_relevance_v29(
                &policy,
                &candidate,
                Some(&proposal()),
                Some(&qualification())
            )
            .unwrap()
            .disposition,
            disposition
        );
    }
}

// Additional deterministic development controls, separate from the public
// reproducers. These are not an independently authored/frozen provider holdout.
#[test]
fn additional_launch_control_corpus() {
    assert_cases(&[
        ("Harbor 4.8", "Harbor 4.8 was launched in June 2026.", true),
        (
            "Cobalt 1.2.6",
            "Cobalt 1.2.6 has been launched for customers.",
            true,
        ),
        ("Vale.ai", "Vale.ai was launched in 2026.", true),
        (
            "Maple U.S. Kit",
            "Maple U.S. Kit is now generally available.",
            true,
        ),
        (
            "Echo 3.1",
            "The vendor has made Echo 3.1 from R.io generally available.",
            true,
        ),
        (
            "Echo 3.1",
            "The vendor has made Echo 3.1 from R.io generally available. Registration is open.",
            true,
        ),
        (
            "星 2.4",
            "The vendor has made 星 2.4 generally available.",
            true,
        ),
        (
            "Echo 3.1",
            "The vendor has made Echo 3.1 from R.io generally available, allegedly.",
            false,
        ),
        (
            "Harbor 4.8",
            "Harbor 4.8 was launched in 2026, perhaps.",
            false,
        ),
        (
            "Cobalt 1.2.6",
            "Cobalt 1.2.6 has been launched in a hypothetical scenario.",
            false,
        ),
        (
            "Echo 3.1",
            "The vendor has made Echo 3.1 from R.io. Generally available is a phrase in the glossary.",
            false,
        ),
        (
            "Echo 3.1",
            "The vendor has made Echo 3.1 from R.io! Maple Queue is generally available.",
            false,
        ),
        (
            "Echo 3.1",
            "The vendor has made Echo 3.1 from R.io; Maple Queue is generally available.",
            false,
        ),
        (
            "Echo 3.1",
            "The vendor has made Echo 3.1 from R.io\nMaple Queue is generally available.",
            false,
        ),
        (
            "Echo 3.1",
            "The vendor has made Echo 3.1 from R.io, while Maple Queue is generally available.",
            false,
        ),
        (
            "Echo 3.1",
            "The vendor has made Echo 3.1 from R.io Inc. Generally available is a phrase in the glossary.",
            false,
        ),
        (
            "Echo 3.1",
            "Echo 3.1 was launched in an unverified rumor.",
            false,
        ),
        ("Echo 3.1", "Echo 3.10 was launched in 2026.", false),
    ]);
}

#[test]
fn recall_preserves_aliases_and_never_joins_signals() {
    let mut policy = policy("Cedar 2.4.1");
    policy.entity.as_mut().unwrap().aliases = vec!["Cedar 2.4 LTS".into()];
    let local = candidate(Kind::Excerpt, "Cedar 2.4 LTS was launched in 2026.");
    assert_eq!(
        materialize_evidence_relevance_v30(
            &policy,
            &local,
            Some(&proposal()),
            Some(&qualification())
        )
        .unwrap()
        .disposition,
        Disposition::Relevant
    );
    let local: EvidenceRelevanceCandidate = serde_json::from_value(json!({
        "evidence_id":"evidence", "source_id":"source",
        "signals":[{"kind":"excerpt", "text":"The vendor has made Cedar 2.4.1 from Z.ai."},
            {"kind":"fact", "text":"Maple Queue is generally available."}]
    }))
    .unwrap();
    assert_ne!(
        materialize_evidence_relevance_v30(
            &policy,
            &local,
            Some(&proposal()),
            Some(&qualification())
        )
        .unwrap()
        .disposition,
        Disposition::Relevant
    );
}
