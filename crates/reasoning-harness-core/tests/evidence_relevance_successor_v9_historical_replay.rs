use std::{collections::BTreeMap, fs, path::PathBuf};

use reasoning_harness_core::{
    EvidenceLocalQualificationV6, EvidenceLocalRelationScope, EvidenceRelevanceBindingProposal,
    EvidenceRelevanceCandidate, EvidenceRelevanceDisposition, EvidenceRelevanceTargetPolicy,
    derive_effective_evidence_local_qualification_v9,
    derive_effective_evidence_local_qualification_v11, materialize_evidence_relevance_v22,
};
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
struct Manifest {
    cases: Vec<Case>,
}

#[derive(Debug, Deserialize)]
struct Case {
    id: String,
    policy: EvidenceRelevanceTargetPolicy,
    candidate: EvidenceRelevanceCandidate,
    expected_local_qualification: EvidenceLocalQualificationV6,
    expected_disposition: EvidenceRelevanceDisposition,
}

#[derive(Debug, Deserialize)]
struct Replay {
    #[serde(default)]
    source: Option<ReplaySource>,
    providers: Vec<ProviderReplay>,
}

#[derive(Debug, Deserialize)]
struct ReplaySource {
    #[serde(default)]
    canonical_run: Option<u64>,
    #[serde(default)]
    attempt: Option<u64>,
    #[serde(default)]
    freeze_commit: Option<String>,
    #[serde(default)]
    freeze_tag: Option<String>,
    #[serde(default)]
    historical_result: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ProviderReplay {
    provider: String,
    observations: Vec<Observation>,
}

#[derive(Debug, Deserialize)]
struct Observation {
    id: String,
    observed_proposal: Option<EvidenceRelevanceBindingProposal>,
    observed_local_qualification: Option<EvidenceLocalQualificationV6>,
    #[serde(default)]
    materialized_disposition: Option<EvidenceRelevanceDisposition>,
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

fn load<T: for<'de> Deserialize<'de>>(path: &str) -> T {
    serde_json::from_slice(&fs::read(root().join(path)).expect("read replay input"))
        .expect("parse replay input")
}

#[test]
fn successor_v11_does_not_regress_prior_actual_relation_authority() {
    let pairs = [
        (
            "fixtures/evidence-relevance-holdout-v1/manifest.json",
            "fixtures/evidence-relevance-holdout-successor-v2/holdout-v1-observation-replay.json",
        ),
        (
            "fixtures/evidence-relevance-holdout-v2/manifest.json",
            "fixtures/evidence-relevance-holdout-successor-v3/holdout-v2-observation-replay.json",
        ),
        (
            "fixtures/evidence-relevance-holdout-v3/manifest.json",
            "fixtures/evidence-relevance-holdout-successor-v4/holdout-v3a-observation-replay.json",
        ),
        (
            "fixtures/evidence-relevance-holdout-v4/manifest.json",
            "fixtures/evidence-relevance-holdout-successor-v5/holdout-v4-observation-replay.json",
        ),
        (
            "fixtures/evidence-relevance-holdout-v5/manifest.json",
            "fixtures/evidence-relevance-holdout-successor-v6/holdout-v5-observation-replay.json",
        ),
        (
            "fixtures/evidence-relevance-holdout-v7/manifest.json",
            "fixtures/evidence-relevance-holdout-successor-v7/holdout-v7-observation-replay.json",
        ),
        (
            "fixtures/evidence-relevance-successor-v5-development/manifest.json",
            "fixtures/evidence-relevance-successor-v5-development/observations-run-36676852262.json",
        ),
        (
            "fixtures/evidence-relevance-successor-v6-development/manifest.json",
            "fixtures/evidence-relevance-successor-v6-development/observations-run-36728692499.json",
        ),
        (
            "fixtures/evidence-relevance-successor-v7-development/manifest.json",
            "fixtures/evidence-relevance-successor-v7-development/observations-run-36890934124.json",
        ),
        (
            "fixtures/evidence-relevance-successor-v7-development-v2/manifest.json",
            "fixtures/evidence-relevance-successor-v7-development-v2/observations-run-36937880336.json",
        ),
    ];

    let mut expected_requested_regressions = 0usize;
    let mut false_requested = 0usize;
    let mut changes_toward_expected = 0usize;

    for (manifest_path, replay_path) in pairs {
        let manifest: Manifest = load(manifest_path);
        let replay: Replay = load(replay_path);
        let cases = manifest
            .cases
            .iter()
            .map(|case| (case.id.as_str(), case))
            .collect::<BTreeMap<_, _>>();

        for provider in replay.providers {
            for observation in provider.observations {
                let case = cases
                    .get(observation.id.as_str())
                    .expect("known historical case");
                let v9 = derive_effective_evidence_local_qualification_v9(
                    &case.policy,
                    &case.candidate,
                    observation.observed_proposal.as_ref(),
                    observation.observed_local_qualification.as_ref(),
                )
                .unwrap();
                let v11 = derive_effective_evidence_local_qualification_v11(
                    &case.policy,
                    &case.candidate,
                    observation.observed_proposal.as_ref(),
                    observation.observed_local_qualification.as_ref(),
                )
                .unwrap();

                assert_eq!(
                    v11.identity_scope, v9.identity_scope,
                    "{replay_path} {} {} identity",
                    provider.provider, observation.id
                );
                assert_eq!(
                    v11.scope_risk, v9.scope_risk,
                    "{replay_path} {} {} scope risk",
                    provider.provider, observation.id
                );

                if case.expected_local_qualification.relation_scope
                    == EvidenceLocalRelationScope::RequestedRelation
                    && v9.relation_scope == EvidenceLocalRelationScope::RequestedRelation
                    && v11.relation_scope != EvidenceLocalRelationScope::RequestedRelation
                {
                    expected_requested_regressions += 1;
                }

                if case.expected_local_qualification.relation_scope
                    != EvidenceLocalRelationScope::RequestedRelation
                    && v11.relation_scope == EvidenceLocalRelationScope::RequestedRelation
                {
                    false_requested += 1;
                }

                if v9.relation_scope != case.expected_local_qualification.relation_scope
                    && v11.relation_scope == case.expected_local_qualification.relation_scope
                {
                    changes_toward_expected += 1;
                }
            }
        }
    }

    assert_eq!(expected_requested_regressions, 0);
    assert_eq!(false_requested, 0);
    assert_eq!(
        changes_toward_expected, 3,
        "bounded frame expansion should improve exactly three recorded v7 development observations"
    );
}

#[test]
fn successor_v11_replays_canonical_holdout_v8_without_rewriting_history() {
    let manifest: Manifest = load("fixtures/evidence-relevance-holdout-v8/manifest.json");
    let replay: Replay = load(
        "fixtures/evidence-relevance-successor-v9-replay/holdout-v8-observations-run-36946457925.json",
    );
    let source = replay.source.as_ref().expect("canonical source metadata");

    assert_eq!(source.canonical_run, Some(36_946_457_925));
    assert_eq!(source.attempt, Some(1));
    assert_eq!(
        source.freeze_commit.as_deref(),
        Some("867b7a90b4911c09c1397e2d15642b3dc9895022")
    );
    assert_eq!(
        source.freeze_tag.as_deref(),
        Some("engine-0.6-evidence-relevance-holdout-v8-freeze")
    );
    assert_eq!(source.historical_result.as_deref(), Some("failure"));

    let summary: Value = load(
        "fixtures/evidence-relevance-successor-v9-replay/holdout-v8-summary-run-36946457925.json",
    );
    assert_eq!(summary["required_qualification_gate_passed"], false);
    assert_eq!(
        summary["providers"]["mistral"]["effective_authority_relation_scope_misses"],
        1
    );
    assert_eq!(
        summary["providers"]["google"]["effective_authority_relation_scope_misses"],
        1
    );
    assert_eq!(
        summary["providers"]["groq"]["effective_authority_relation_scope_misses"],
        0
    );

    let cases = manifest
        .cases
        .iter()
        .map(|case| (case.id.as_str(), case))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(cases.len(), 26);
    assert_eq!(replay.providers.len(), 3);

    let mut v11_relation_misses = Vec::new();
    let mut wrong_target_relevant = 0usize;
    let mut v9_to_v11_changes = Vec::new();

    for provider in &replay.providers {
        assert_eq!(provider.observations.len(), 26, "{}", provider.provider);
        for observation in &provider.observations {
            let case = cases.get(observation.id.as_str()).expect("known v8 case");
            let v9 = derive_effective_evidence_local_qualification_v9(
                &case.policy,
                &case.candidate,
                observation.observed_proposal.as_ref(),
                observation.observed_local_qualification.as_ref(),
            )
            .unwrap();
            let v11 = derive_effective_evidence_local_qualification_v11(
                &case.policy,
                &case.candidate,
                observation.observed_proposal.as_ref(),
                observation.observed_local_qualification.as_ref(),
            )
            .unwrap();

            assert_eq!(
                v11.identity_scope, case.expected_local_qualification.identity_scope,
                "{} {} identity",
                provider.provider, observation.id
            );
            assert_eq!(
                v11.scope_risk, case.expected_local_qualification.scope_risk,
                "{} {} scope risk",
                provider.provider, observation.id
            );
            if v11.relation_scope != case.expected_local_qualification.relation_scope {
                v11_relation_misses.push((
                    provider.provider.as_str(),
                    observation.id.as_str(),
                    v9.relation_scope,
                    v11.relation_scope,
                    case.expected_local_qualification.relation_scope,
                ));
            }
            if v11.relation_scope != v9.relation_scope {
                v9_to_v11_changes.push((
                    provider.provider.as_str(),
                    observation.id.as_str(),
                    v9.relation_scope,
                    v11.relation_scope,
                ));
            }

            let assessment = materialize_evidence_relevance_v22(
                &case.policy,
                &case.candidate,
                observation.observed_proposal.as_ref(),
                observation.observed_local_qualification.as_ref(),
            )
            .unwrap();
            assert_eq!(
                assessment.disposition, case.expected_disposition,
                "{} {} frozen v22 materialization",
                provider.provider, observation.id
            );
            assert_eq!(
                observation.materialized_disposition,
                Some(case.expected_disposition),
                "{} {} captured disposition",
                provider.provider,
                observation.id
            );
            if case.expected_disposition != EvidenceRelevanceDisposition::Relevant
                && assessment.disposition == EvidenceRelevanceDisposition::Relevant
            {
                wrong_target_relevant += 1;
            }
        }
    }

    assert!(
        v11_relation_misses.is_empty(),
        "v11 relation misses: {v11_relation_misses:?}"
    );
    assert_eq!(wrong_target_relevant, 0);
    assert_eq!(
        v9_to_v11_changes,
        vec![
            (
                "mistral",
                "v8h15_negative_explicit_separate_service",
                EvidenceLocalRelationScope::DifferentRelation,
                EvidenceLocalRelationScope::RequestedRelation,
            ),
            (
                "google",
                "v8h15_negative_explicit_separate_service",
                EvidenceLocalRelationScope::RelationAbsent,
                EvidenceLocalRelationScope::RequestedRelation,
            ),
        ]
    );
}
