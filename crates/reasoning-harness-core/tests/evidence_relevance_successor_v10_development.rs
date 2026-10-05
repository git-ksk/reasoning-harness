use std::{collections::BTreeSet, fs, path::PathBuf};

use reasoning_harness_core::{
    EvidenceLocalBlockingReason, EvidenceLocalIdentityScope, EvidenceLocalQualificationV6,
    EvidenceLocalRelationScope, EvidenceNegativeRelationConfirmation, EvidenceRelevanceBinding,
    EvidenceRelevanceBindingProposal, EvidenceRelevanceCandidate, EvidenceRelevanceDisposition,
    EvidenceRelevanceTargetPolicy, derive_effective_evidence_local_qualification_v13,
    materialize_evidence_relevance_v23, materialize_evidence_relevance_v25,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Manifest {
    suite_id: String,
    issue: u64,
    status: String,
    source_rule: String,
    cases: Vec<Case>,
}

#[derive(Debug, Deserialize)]
struct Case {
    id: String,
    policy: EvidenceRelevanceTargetPolicy,
    candidate: EvidenceRelevanceCandidate,
    #[serde(default)]
    expected_confirmation: Option<EvidenceNegativeRelationConfirmation>,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("workspace root")
        .to_path_buf()
}

fn load(relative: &str) -> Manifest {
    serde_json::from_slice(
        &fs::read(repo_root().join(relative)).expect("read evidence relevance manifest"),
    )
    .expect("parse evidence relevance manifest")
}

fn normalized(value: &str) -> String {
    value
        .chars()
        .flat_map(char::to_lowercase)
        .map(|character| {
            if character.is_alphanumeric() {
                character
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn windows(value: &str, size: usize) -> BTreeSet<String> {
    let normalized = normalized(value);
    let tokens = normalized.split_whitespace().collect::<Vec<_>>();
    if tokens.len() < size {
        return BTreeSet::new();
    }
    tokens
        .windows(size)
        .map(|window| window.join(" "))
        .collect()
}

#[test]
fn successor_v10_manifest_is_fresh_balanced_and_bound_to_issue_468() {
    let fresh = load("fixtures/evidence-relevance-successor-v10-development/manifest.json");
    let holdout_v10 = load("fixtures/evidence-relevance-holdout-v10/manifest.json");

    assert_eq!(
        fresh.suite_id,
        "evidence-relevance-successor-v10-development"
    );
    assert_eq!(fresh.issue, 468);
    assert_eq!(fresh.status, "fresh_independent_development");
    assert_eq!(
        fresh.source_rule,
        "independently_authored_after_holdout_v10_result_no_holdout_v10_case_entity_task_signal_or_8token_surface_reuse"
    );
    assert_eq!(fresh.cases.len(), 16);

    let confirmed = fresh
        .cases
        .iter()
        .filter(|case| {
            case.expected_confirmation
                == Some(EvidenceNegativeRelationConfirmation::ConfirmedDifferentRelation)
        })
        .count();
    let not_confirmed = fresh
        .cases
        .iter()
        .filter(|case| {
            case.expected_confirmation == Some(EvidenceNegativeRelationConfirmation::NotConfirmed)
        })
        .count();
    assert_eq!((confirmed, not_confirmed), (8, 8));

    let ids = fresh
        .cases
        .iter()
        .map(|case| case.id.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(ids.len(), fresh.cases.len());

    let old_ids = holdout_v10
        .cases
        .iter()
        .map(|case| case.id.as_str())
        .collect::<BTreeSet<_>>();
    let old_entities = holdout_v10
        .cases
        .iter()
        .filter_map(|case| case.policy.entity.as_ref())
        .map(|entity| normalized(&entity.canonical_name))
        .collect::<BTreeSet<_>>();
    let old_tasks = holdout_v10
        .cases
        .iter()
        .map(|case| normalized(&case.policy.target_question))
        .collect::<BTreeSet<_>>();
    let old_signals = holdout_v10
        .cases
        .iter()
        .flat_map(|case| case.candidate.signals.iter())
        .map(|signal| normalized(&signal.text))
        .collect::<BTreeSet<_>>();
    let old_windows = holdout_v10
        .cases
        .iter()
        .flat_map(|case| case.candidate.signals.iter())
        .flat_map(|signal| windows(&signal.text, 8))
        .collect::<BTreeSet<_>>();

    for case in &fresh.cases {
        assert!(
            !old_ids.contains(case.id.as_str()),
            "reused id: {}",
            case.id
        );
        let entity = case.policy.entity.as_ref().expect("fresh case entity");
        assert!(
            !old_entities.contains(&normalized(&entity.canonical_name)),
            "reused entity: {}",
            case.id
        );
        assert!(
            !old_tasks.contains(&normalized(&case.policy.target_question)),
            "reused task: {}",
            case.id
        );
        for signal in &case.candidate.signals {
            assert!(
                !old_signals.contains(&normalized(&signal.text)),
                "reused signal: {}",
                case.id
            );
            assert!(
                windows(&signal.text, 8).is_disjoint(&old_windows),
                "reused 8-token surface: {}",
                case.id
            );
        }
    }
}

#[test]
fn successor_v10_two_key_composition_is_monotone_on_fresh_surface() {
    let fresh = load("fixtures/evidence-relevance-successor-v10-development/manifest.json");

    let proposal = EvidenceRelevanceBindingProposal {
        target_binding: EvidenceRelevanceBinding::Exact,
        relation_binding: EvidenceRelevanceBinding::Unresolved,
    };
    let raw = EvidenceLocalQualificationV6 {
        identity_scope: EvidenceLocalIdentityScope::ExactTarget,
        relation_scope: EvidenceLocalRelationScope::RelationAbsent,
        scope_risk: EvidenceLocalBlockingReason::None,
    };

    for case in &fresh.cases {
        let baseline = materialize_evidence_relevance_v23(
            &case.policy,
            &case.candidate,
            Some(&proposal),
            Some(&raw),
        )
        .expect("v23 baseline materialization");

        let effective = derive_effective_evidence_local_qualification_v13(
            &case.policy,
            &case.candidate,
            Some(&proposal),
            Some(&raw),
            case.expected_confirmation.as_ref(),
        )
        .expect("v13 effective qualification");

        let successor = materialize_evidence_relevance_v25(
            &case.policy,
            &case.candidate,
            Some(&proposal),
            Some(&raw),
            case.expected_confirmation.as_ref(),
        )
        .expect("v25 successor materialization");

        match case
            .expected_confirmation
            .expect("fresh expected confirmation")
        {
            EvidenceNegativeRelationConfirmation::ConfirmedDifferentRelation => {
                assert_eq!(
                    baseline.disposition,
                    EvidenceRelevanceDisposition::Ambiguous,
                    "fresh positive control was already terminal under v23: {}",
                    case.id
                );
                assert_eq!(
                    effective.relation_scope,
                    EvidenceLocalRelationScope::DifferentRelation,
                    "two-key confirmation did not recover different relation: {}",
                    case.id
                );
                assert_eq!(
                    successor.disposition,
                    EvidenceRelevanceDisposition::Irrelevant,
                    "confirmed different relation did not reject: {}",
                    case.id
                );
            }
            EvidenceNegativeRelationConfirmation::NotConfirmed => {
                assert_ne!(
                    effective.relation_scope,
                    EvidenceLocalRelationScope::DifferentRelation,
                    "not-confirmed case gained negative relation authority: {}",
                    case.id
                );
                assert_eq!(
                    successor.disposition, baseline.disposition,
                    "not-confirmed case changed baseline materialization: {}",
                    case.id
                );
            }
        }
    }
}
