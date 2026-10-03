use std::{collections::BTreeSet, fs, path::PathBuf};

use reasoning_harness_core::{
    EvidenceLocalBlockingReason, EvidenceLocalIdentityScope, EvidenceLocalQualificationV6,
    EvidenceLocalRelationScope, EvidenceRelevanceBinding, EvidenceRelevanceBindingProposal,
    EvidenceRelevanceCandidate, EvidenceRelevanceDisposition, EvidenceRelevanceTargetPolicy,
    derive_effective_evidence_local_qualification_v15, materialize_evidence_relevance_v28,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Manifest {
    suite_id: String,
    issue: u64,
    status: String,
    source_rule: String,
    annotation_protocol_id: String,
    fixed_core_id: String,
    cases: Vec<Case>,
}

#[derive(Debug, Deserialize)]
struct Case {
    id: String,
    task: String,
    authority_expectation: AuthorityExpectation,
    policy: EvidenceRelevanceTargetPolicy,
    candidate: EvidenceRelevanceCandidate,
    expected_proposal: EvidenceRelevanceBindingProposal,
    expected_local_qualification: EvidenceLocalQualificationV6,
    expected_disposition: EvidenceRelevanceDisposition,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum AuthorityExpectation {
    RequireDifferent,
    RequireRequested,
    ForbidDifferent,
    PreserveRisk,
}

#[derive(Debug, Deserialize)]
struct PriorManifest {
    cases: Vec<PriorCase>,
}

#[derive(Debug, Deserialize)]
struct PriorCase {
    id: String,
    #[serde(default)]
    task: String,
    policy: EvidenceRelevanceTargetPolicy,
    candidate: EvidenceRelevanceCandidate,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("workspace root")
        .to_path_buf()
}

fn load<T: serde::de::DeserializeOwned>(relative: &str) -> T {
    serde_json::from_slice(
        &fs::read(repo_root().join(relative)).expect("read evidence relevance fixture"),
    )
    .expect("parse evidence relevance fixture")
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
    tokens.windows(size).map(|w| w.join(" ")).collect()
}

#[test]
fn successor_v10_v4_manifest_is_fresh_and_precommitted() {
    let fresh: Manifest =
        load("fixtures/evidence-relevance-successor-v10-development-v4/manifest.json");
    assert_eq!(
        fresh.suite_id,
        "evidence-relevance-successor-v10-development-v4"
    );
    assert_eq!(fresh.issue, 468);
    assert_eq!(fresh.status, "fresh_independent_development");
    assert_eq!(
        fresh.annotation_protocol_id,
        "evidence-relevance-effective-qualification-v15-materialization-v28"
    );
    assert_eq!(
        fresh.fixed_core_id,
        "evidence-relevance-fixed-core-successor-v10-development-v4"
    );
    assert_eq!(
        fresh.source_rule,
        "independently_authored_after_holdout_v10_and_successor_v10_development_v1_v2_v3_observation_no_case_entity_task_signal_or_8token_surface_reuse"
    );
    assert_eq!(fresh.cases.len(), 24);

    let mut counts = [0usize; 4];
    for case in &fresh.cases {
        counts[match case.authority_expectation {
            AuthorityExpectation::RequireDifferent => 0,
            AuthorityExpectation::RequireRequested => 1,
            AuthorityExpectation::ForbidDifferent => 2,
            AuthorityExpectation::PreserveRisk => 3,
        }] += 1;
    }
    assert_eq!(counts, [11, 9, 3, 1]);

    let priors: [PriorManifest; 4] = [
        load("fixtures/evidence-relevance-holdout-v10/manifest.json"),
        load("fixtures/evidence-relevance-successor-v10-development/manifest.json"),
        load("fixtures/evidence-relevance-successor-v10-development-v2/manifest.json"),
        load("fixtures/evidence-relevance-successor-v10-development-v3/manifest.json"),
    ];
    let prior_cases = priors
        .iter()
        .flat_map(|m| m.cases.iter())
        .collect::<Vec<_>>();

    let old_ids = prior_cases
        .iter()
        .map(|c| c.id.as_str())
        .collect::<BTreeSet<_>>();
    let old_entities = prior_cases
        .iter()
        .filter_map(|c| c.policy.entity.as_ref())
        .map(|e| normalized(&e.canonical_name))
        .collect::<BTreeSet<_>>();
    let old_tasks = prior_cases
        .iter()
        .map(|c| {
            if c.task.is_empty() {
                normalized(&c.policy.target_question)
            } else {
                normalized(&c.task)
            }
        })
        .collect::<BTreeSet<_>>();
    let old_signals = prior_cases
        .iter()
        .flat_map(|c| c.candidate.signals.iter())
        .map(|s| normalized(&s.text))
        .collect::<BTreeSet<_>>();
    let old_windows = prior_cases
        .iter()
        .flat_map(|c| c.candidate.signals.iter())
        .flat_map(|s| windows(&s.text, 8))
        .collect::<BTreeSet<_>>();

    let mut ids = BTreeSet::new();
    for case in &fresh.cases {
        assert!(ids.insert(case.id.as_str()), "duplicate id: {}", case.id);
        assert!(
            !old_ids.contains(case.id.as_str()),
            "reused id: {}",
            case.id
        );
        let entity = case.policy.entity.as_ref().expect("fresh entity");
        assert!(
            !old_entities.contains(&normalized(&entity.canonical_name)),
            "reused entity: {}",
            case.id
        );
        assert!(
            !old_tasks.contains(&normalized(&case.task)),
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
fn successor_v10_v4_expected_path_matches_v15_v28_contract() {
    let fresh: Manifest =
        load("fixtures/evidence-relevance-successor-v10-development-v4/manifest.json");

    for case in fresh.cases {
        let effective = derive_effective_evidence_local_qualification_v15(
            &case.policy,
            &case.candidate,
            Some(&case.expected_proposal),
            Some(&case.expected_local_qualification),
        )
        .expect("derive v15");
        let assessment = materialize_evidence_relevance_v28(
            &case.policy,
            &case.candidate,
            Some(&case.expected_proposal),
            Some(&case.expected_local_qualification),
        )
        .expect("materialize v28");

        assert_eq!(
            assessment.disposition, case.expected_disposition,
            "expected disposition mismatch: {}",
            case.id
        );

        match case.authority_expectation {
            AuthorityExpectation::RequireDifferent => assert_eq!(
                effective.relation_scope,
                EvidenceLocalRelationScope::DifferentRelation,
                "different relation missing: {}",
                case.id
            ),
            AuthorityExpectation::RequireRequested => assert_eq!(
                effective.relation_scope,
                EvidenceLocalRelationScope::RequestedRelation,
                "requested relation missing: {}",
                case.id
            ),
            AuthorityExpectation::ForbidDifferent => assert_ne!(
                effective.relation_scope,
                EvidenceLocalRelationScope::DifferentRelation,
                "negative control gained different relation: {}",
                case.id
            ),
            AuthorityExpectation::PreserveRisk => assert_eq!(
                effective.scope_risk, case.expected_local_qualification.scope_risk,
                "risk changed: {}",
                case.id
            ),
        }
    }
}

#[test]
fn successor_v10_v4_harness_cues_recover_under_conservative_model_outputs() {
    let fresh: Manifest =
        load("fixtures/evidence-relevance-successor-v10-development-v4/manifest.json");
    let proposal = EvidenceRelevanceBindingProposal {
        target_binding: EvidenceRelevanceBinding::Exact,
        relation_binding: EvidenceRelevanceBinding::Unresolved,
    };
    let raw = EvidenceLocalQualificationV6 {
        identity_scope: EvidenceLocalIdentityScope::ExactTarget,
        relation_scope: EvidenceLocalRelationScope::RelationAbsent,
        scope_risk: EvidenceLocalBlockingReason::None,
    };

    for case in fresh.cases.iter().filter(|case| {
        matches!(
            case.authority_expectation,
            AuthorityExpectation::RequireDifferent
        )
    }) {
        let effective = derive_effective_evidence_local_qualification_v15(
            &case.policy,
            &case.candidate,
            Some(&proposal),
            Some(&raw),
        )
        .expect("derive conservative v15");
        let assessment = materialize_evidence_relevance_v28(
            &case.policy,
            &case.candidate,
            Some(&proposal),
            Some(&raw),
        )
        .expect("materialize conservative v28");
        assert_eq!(
            effective.relation_scope,
            EvidenceLocalRelationScope::DifferentRelation,
            "Harness cue failed under conservative model outputs: {}",
            case.id
        );
        assert_eq!(
            assessment.disposition,
            EvidenceRelevanceDisposition::Irrelevant,
            "Harness cue failed to reject: {}",
            case.id
        );
    }
}

#[test]
fn successor_v10_v4_instruction_controls_resist_adversarial_model_different_votes() {
    let fresh: Manifest =
        load("fixtures/evidence-relevance-successor-v10-development-v4/manifest.json");
    let proposal = EvidenceRelevanceBindingProposal {
        target_binding: EvidenceRelevanceBinding::Exact,
        relation_binding: EvidenceRelevanceBinding::Different,
    };
    let raw = EvidenceLocalQualificationV6 {
        identity_scope: EvidenceLocalIdentityScope::ExactTarget,
        relation_scope: EvidenceLocalRelationScope::Unresolved,
        scope_risk: EvidenceLocalBlockingReason::None,
    };

    for id in [
        "sv10v4_19_prompt_injection_numeric",
        "sv10v4_20_prompt_injection_definition",
    ] {
        let case = fresh
            .cases
            .iter()
            .find(|c| c.id == id)
            .expect("control case");
        let effective = derive_effective_evidence_local_qualification_v15(
            &case.policy,
            &case.candidate,
            Some(&proposal),
            Some(&raw),
        )
        .expect("derive adversarial v15");
        let assessment = materialize_evidence_relevance_v28(
            &case.policy,
            &case.candidate,
            Some(&proposal),
            Some(&raw),
        )
        .expect("materialize adversarial v28");

        assert_ne!(
            effective.relation_scope,
            EvidenceLocalRelationScope::DifferentRelation,
            "instruction text retained model-only negative authority: {id}"
        );
        assert_eq!(
            assessment.disposition,
            EvidenceRelevanceDisposition::Ambiguous,
            "instruction text became terminal reject: {id}"
        );
    }
}

#[test]
fn successor_v10_v4_context_gap_blocks_direct_definition_recovery() {
    let fresh: Manifest =
        load("fixtures/evidence-relevance-successor-v10-development-v4/manifest.json");
    let case = fresh
        .cases
        .iter()
        .find(|c| c.id == "sv10v4_22_context_gap_definition")
        .expect("context gap case");
    let proposal = EvidenceRelevanceBindingProposal {
        target_binding: EvidenceRelevanceBinding::Exact,
        relation_binding: EvidenceRelevanceBinding::Unresolved,
    };
    let raw = EvidenceLocalQualificationV6 {
        identity_scope: EvidenceLocalIdentityScope::ExactTarget,
        relation_scope: EvidenceLocalRelationScope::Unresolved,
        scope_risk: EvidenceLocalBlockingReason::ContextGap,
    };

    let effective = derive_effective_evidence_local_qualification_v15(
        &case.policy,
        &case.candidate,
        Some(&proposal),
        Some(&raw),
    )
    .expect("derive context gap v15");
    let assessment = materialize_evidence_relevance_v28(
        &case.policy,
        &case.candidate,
        Some(&proposal),
        Some(&raw),
    )
    .expect("materialize context gap v28");

    assert_eq!(
        effective.scope_risk,
        EvidenceLocalBlockingReason::ContextGap
    );
    assert_ne!(
        effective.relation_scope,
        EvidenceLocalRelationScope::DifferentRelation
    );
    assert_eq!(
        assessment.disposition,
        EvidenceRelevanceDisposition::Ambiguous
    );
}
