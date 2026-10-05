use std::{collections::BTreeMap, fs, path::PathBuf};

use reasoning_harness_core::{
    EvidenceLocalBlockingReason, EvidenceLocalQualificationV6, EvidenceRelevanceBindingProposal,
    EvidenceRelevanceCandidate, EvidenceRelevanceDisposition, EvidenceRelevanceTargetPolicy,
    derive_effective_evidence_local_qualification_v5, materialize_evidence_relevance_v18,
};
use serde::Deserialize;

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
struct ReplayFixture {
    schema_version: String,
    source: ReplaySource,
    providers: Vec<ProviderReplay>,
}

#[derive(Debug, Deserialize)]
struct ReplaySource {
    canonical_run: u64,
    attempt: u32,
    freeze_commit: String,
    freeze_tag: String,
}

#[derive(Debug, Deserialize)]
struct ProviderReplay {
    provider: String,
    model: String,
    observations: Vec<ReplayObservation>,
}

#[derive(Debug, Deserialize)]
struct ReplayObservation {
    id: String,
    observed_proposal: Option<EvidenceRelevanceBindingProposal>,
    observed_local_qualification: Option<EvidenceLocalQualificationV6>,
    effective_local_qualification: Option<EvidenceLocalQualificationV6>,
    materialized_disposition: Option<EvidenceRelevanceDisposition>,
}

fn repo_fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(path)
}

fn read_manifest(path: &str) -> Manifest {
    serde_json::from_slice(&fs::read(repo_fixture(path)).expect("read manifest"))
        .expect("parse manifest")
}

fn read_replay(path: &str) -> ReplayFixture {
    serde_json::from_slice(&fs::read(repo_fixture(path)).expect("read replay"))
        .expect("parse replay")
}

fn indexed_cases(manifest: Manifest) -> BTreeMap<String, Case> {
    manifest
        .cases
        .into_iter()
        .map(|case| (case.id.clone(), case))
        .collect()
}

fn assert_provider_identity(provider: &ProviderReplay) {
    match provider.provider.as_str() {
        "mistral" => assert_eq!(provider.model, "ministral-8b-latest"),
        "groq" => assert_eq!(provider.model, "openai/gpt-oss-120b"),
        "google" => assert_eq!(provider.model, "gemini-3.5-flash-lite"),
        other => panic!("unexpected replay provider {other}"),
    }
}

fn assert_full_successor_replay(manifest_path: &str, replay_path: &str, expected_cases: usize) {
    let cases = indexed_cases(read_manifest(manifest_path));
    let replay = read_replay(replay_path);
    assert_eq!(cases.len(), expected_cases);
    assert_eq!(replay.providers.len(), 3);

    for provider in replay.providers {
        assert_provider_identity(&provider);
        assert_eq!(
            provider.observations.len(),
            expected_cases,
            "{}",
            provider.provider
        );
        let mut authority_exact = 0usize;
        let mut materialized_exact = 0usize;
        let mut wrong_target_relevant = 0usize;
        let mut relevant_utility_misses = 0usize;

        for observed in provider.observations {
            let case = cases
                .get(&observed.id)
                .unwrap_or_else(|| panic!("missing case {}", observed.id));
            let proposal = observed.observed_proposal.as_ref().unwrap_or_else(|| {
                panic!("{} {} missing proposal", provider.provider, observed.id)
            });
            let raw = observed
                .observed_local_qualification
                .as_ref()
                .unwrap_or_else(|| {
                    panic!(
                        "{} {} missing raw qualification",
                        provider.provider, observed.id
                    )
                });
            assert!(observed.effective_local_qualification.is_some());
            assert!(observed.materialized_disposition.is_some());

            let effective = derive_effective_evidence_local_qualification_v5(
                &case.policy,
                &case.candidate,
                Some(proposal),
                Some(raw),
            )
            .unwrap_or_else(|error| {
                panic!(
                    "{} {} qualification: {error}",
                    provider.provider, observed.id
                )
            });

            assert_eq!(
                effective.scope_risk, case.expected_local_qualification.scope_risk,
                "{} {} effective risk regressed",
                provider.provider, observed.id
            );
            if case.expected_local_qualification.scope_risk == EvidenceLocalBlockingReason::None {
                assert_eq!(
                    effective.identity_scope, case.expected_local_qualification.identity_scope,
                    "{} {} authority identity regressed",
                    provider.provider, observed.id
                );
                assert_eq!(
                    effective.relation_scope, case.expected_local_qualification.relation_scope,
                    "{} {} authority relation regressed",
                    provider.provider, observed.id
                );
            }
            authority_exact += 1;

            let assessment = materialize_evidence_relevance_v18(
                &case.policy,
                &case.candidate,
                Some(proposal),
                Some(raw),
            )
            .unwrap_or_else(|error| {
                panic!(
                    "{} {} materialization: {error}",
                    provider.provider, observed.id
                )
            });
            if assessment.disposition == case.expected_disposition {
                materialized_exact += 1;
            }
            if case.expected_disposition == EvidenceRelevanceDisposition::Irrelevant
                && assessment.disposition == EvidenceRelevanceDisposition::Relevant
            {
                wrong_target_relevant += 1;
            }
            if case.expected_disposition == EvidenceRelevanceDisposition::Relevant
                && assessment.disposition != EvidenceRelevanceDisposition::Relevant
            {
                relevant_utility_misses += 1;
            }
            assert_eq!(
                assessment.disposition, case.expected_disposition,
                "{} {} materialization regressed",
                provider.provider, observed.id
            );
        }

        assert_eq!(authority_exact, expected_cases, "{}", provider.provider);
        assert_eq!(materialized_exact, expected_cases, "{}", provider.provider);
        assert_eq!(wrong_target_relevant, 0, "{}", provider.provider);
        assert_eq!(relevant_utility_misses, 0, "{}", provider.provider);
    }
}

#[test]
fn successor_v3_replays_v23_without_regression() {
    let replay = read_replay("evidence-relevance-holdout-successor-v2/v23-observation-replay.json");
    assert_eq!(
        replay.schema_version,
        "engine-0.6-evidence-relevance-calibration-v23-replay-v1"
    );
    assert_eq!(replay.source.canonical_run, 36_400_085_595);
    assert_eq!(replay.source.attempt, 1);
    assert_eq!(
        replay.source.freeze_commit,
        "0871169303d20c464b0454dbe0a150d2fef0ec43"
    );
    assert_eq!(
        replay.source.freeze_tag,
        "engine-0.6-evidence-relevance-calibration-v23-freeze"
    );

    assert_full_successor_replay(
        "evidence-relevance-calibration-v23/manifest.json",
        "evidence-relevance-holdout-successor-v2/v23-observation-replay.json",
        48,
    );
}

#[test]
fn successor_v3_replays_holdout_v1_without_safety_regression() {
    let replay =
        read_replay("evidence-relevance-holdout-successor-v2/holdout-v1-observation-replay.json");
    assert_eq!(
        replay.schema_version,
        "engine-0.6-evidence-relevance-holdout-v1-replay-v1"
    );
    assert_eq!(replay.source.canonical_run, 36_495_389_012);
    assert_eq!(replay.source.attempt, 1);
    assert_eq!(
        replay.source.freeze_commit,
        "5fda8675e642ee98de57e2c4d69e34a5622fa485"
    );
    assert_eq!(
        replay.source.freeze_tag,
        "engine-0.6-evidence-relevance-holdout-v1-freeze"
    );

    let cases = indexed_cases(read_manifest("evidence-relevance-holdout-v1/manifest.json"));
    let mut checked_h14 = false;
    for provider in &replay.providers {
        if provider.provider != "google" {
            continue;
        }
        let observed = provider
            .observations
            .iter()
            .find(|value| value.id == "h14_negative_comparison_only_mention")
            .expect("google h14 replay");
        assert_eq!(
            observed.materialized_disposition,
            Some(EvidenceRelevanceDisposition::Relevant),
            "immutable v1 observation remains historical FAIL"
        );
        let case = cases.get(&observed.id).expect("h14 manifest case");
        let assessment = materialize_evidence_relevance_v18(
            &case.policy,
            &case.candidate,
            observed.observed_proposal.as_ref(),
            observed.observed_local_qualification.as_ref(),
        )
        .expect("successor h14 materialization");
        assert_eq!(
            assessment.disposition,
            EvidenceRelevanceDisposition::Irrelevant
        );
        checked_h14 = true;
    }
    assert!(checked_h14);

    assert_full_successor_replay(
        "evidence-relevance-holdout-v1/manifest.json",
        "evidence-relevance-holdout-successor-v2/holdout-v1-observation-replay.json",
        26,
    );
}

#[test]
fn successor_v3_replays_holdout_v2_to_full_exact_without_rewriting_it() {
    let replay =
        read_replay("evidence-relevance-holdout-successor-v3/holdout-v2-observation-replay.json");
    assert_eq!(
        replay.schema_version,
        "engine-0.6-evidence-relevance-holdout-v2-replay-v1"
    );
    assert_eq!(replay.source.canonical_run, 36_533_340_582);
    assert_eq!(replay.source.attempt, 1);
    assert_eq!(
        replay.source.freeze_commit,
        "c38b5f7dbfcf8c01dcaa6f6cd7341e43d3b87325"
    );
    assert_eq!(
        replay.source.freeze_tag,
        "engine-0.6-evidence-relevance-holdout-v2-freeze"
    );

    let cases = indexed_cases(read_manifest("evidence-relevance-holdout-v2/manifest.json"));
    for provider in &replay.providers {
        if provider.provider == "groq" {
            continue;
        }
        let observed = provider
            .observations
            .iter()
            .find(|value| value.id == "v2h13_negative_unrelated_launch")
            .unwrap_or_else(|| panic!("{} v2h13 replay", provider.provider));
        assert_eq!(
            observed.materialized_disposition,
            Some(EvidenceRelevanceDisposition::Ambiguous),
            "immutable v2 observation remains historical FAIL for {}",
            provider.provider
        );
        let case = cases.get(&observed.id).expect("v2h13 manifest case");
        let assessment = materialize_evidence_relevance_v18(
            &case.policy,
            &case.candidate,
            observed.observed_proposal.as_ref(),
            observed.observed_local_qualification.as_ref(),
        )
        .expect("successor v2h13 materialization");
        assert_eq!(
            assessment.disposition,
            EvidenceRelevanceDisposition::Irrelevant,
            "{} v2h13 successor",
            provider.provider
        );
    }

    assert_full_successor_replay(
        "evidence-relevance-holdout-v2/manifest.json",
        "evidence-relevance-holdout-successor-v3/holdout-v2-observation-replay.json",
        26,
    );
}
