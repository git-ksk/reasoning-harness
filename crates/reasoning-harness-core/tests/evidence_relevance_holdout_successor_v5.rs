use std::{collections::BTreeMap, fs, path::PathBuf};

use reasoning_harness_core::{
    EvidenceLocalQualificationV6, EvidenceRelevanceBindingProposal, EvidenceRelevanceCandidate,
    EvidenceRelevanceDisposition, EvidenceRelevanceTargetPolicy,
    derive_effective_evidence_local_qualification_v7, materialize_evidence_relevance_v20,
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
struct Replay {
    source: ReplaySource,
    providers: Vec<ProviderReplay>,
}

#[derive(Debug, Deserialize)]
struct ReplaySource {
    canonical_run: u64,
    attempt: u64,
    freeze_tag: String,
    freeze_commit: String,
}

#[derive(Debug, Deserialize)]
struct DevelopmentReplay {
    source: DevelopmentReplaySource,
    providers: Vec<ProviderReplay>,
}

#[derive(Debug, Deserialize)]
struct DevelopmentReplaySource {
    run_id: u64,
    candidate_commit: String,
    holdout_acceptance_evidence: bool,
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
    materialized_disposition: Option<EvidenceRelevanceDisposition>,
}

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root")
}

fn load_manifest_at(path: &str) -> Manifest {
    let raw = fs::read_to_string(repository_root().join("fixtures").join(path))
        .unwrap_or_else(|error| panic!("read manifest {path}: {error}"));
    serde_json::from_str(&raw).unwrap_or_else(|error| panic!("parse manifest {path}: {error}"))
}

fn load_replay_at(path: &str) -> Replay {
    let raw = fs::read_to_string(repository_root().join("fixtures").join(path))
        .unwrap_or_else(|error| panic!("read replay {path}: {error}"));
    serde_json::from_str(&raw).unwrap_or_else(|error| panic!("parse replay {path}: {error}"))
}

fn load_development_replay_at(path: &str) -> DevelopmentReplay {
    let raw = fs::read_to_string(repository_root().join("fixtures").join(path))
        .unwrap_or_else(|error| panic!("read development replay {path}: {error}"));
    serde_json::from_str(&raw)
        .unwrap_or_else(|error| panic!("parse development replay {path}: {error}"))
}

fn load_manifest() -> Manifest {
    load_manifest_at("evidence-relevance-holdout-v4/manifest.json")
}

fn load_replay() -> Replay {
    load_replay_at("evidence-relevance-holdout-successor-v5/holdout-v4-observation-replay.json")
}

fn assert_successor_v5_replay(manifest_path: &str, replay_path: &str, expected_cases: usize) {
    use reasoning_harness_core::EvidenceLocalBlockingReason;

    let manifest = load_manifest_at(manifest_path);
    let replay = load_replay_at(replay_path);
    let cases = manifest
        .cases
        .iter()
        .map(|case| (case.id.as_str(), case))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(cases.len(), expected_cases);
    assert_eq!(replay.providers.len(), 3);

    for provider in &replay.providers {
        assert_eq!(
            provider.observations.len(),
            expected_cases,
            "{}",
            provider.provider
        );
        let mut authority_exact = 0usize;
        let mut materialized_exact = 0usize;
        let mut wrong_target_relevant = 0usize;

        for observed in &provider.observations {
            let case = cases
                .get(observed.id.as_str())
                .unwrap_or_else(|| panic!("{} missing {}", provider.provider, observed.id));
            let effective = derive_effective_evidence_local_qualification_v7(
                &case.policy,
                &case.candidate,
                observed.observed_proposal.as_ref(),
                observed.observed_local_qualification.as_ref(),
            )
            .unwrap_or_else(|error| {
                panic!(
                    "{} {} successor v5 qualification: {error}",
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

            let assessment = materialize_evidence_relevance_v20(
                &case.policy,
                &case.candidate,
                observed.observed_proposal.as_ref(),
                observed.observed_local_qualification.as_ref(),
            )
            .unwrap_or_else(|error| {
                panic!(
                    "{} {} successor v5 materialization: {error}",
                    provider.provider, observed.id
                )
            });
            assert_eq!(
                assessment.disposition, case.expected_disposition,
                "{} {} materialization regressed",
                provider.provider, observed.id
            );
            materialized_exact += 1;

            if case.expected_disposition != EvidenceRelevanceDisposition::Relevant
                && assessment.disposition == EvidenceRelevanceDisposition::Relevant
            {
                wrong_target_relevant += 1;
            }
        }

        assert_eq!(authority_exact, expected_cases, "{}", provider.provider);
        assert_eq!(materialized_exact, expected_cases, "{}", provider.provider);
        assert_eq!(wrong_target_relevant, 0, "{}", provider.provider);
    }
}

#[test]
fn successor_v5_replays_immutable_holdout_v4_observations_exactly_under_new_semantics() {
    let manifest = load_manifest();
    let replay = load_replay();

    assert_eq!(replay.source.canonical_run, 36_650_257_492);
    assert_eq!(replay.source.attempt, 1);
    assert_eq!(
        replay.source.freeze_tag,
        "engine-0.6-evidence-relevance-holdout-v4-freeze"
    );
    assert_eq!(
        replay.source.freeze_commit,
        "4e60db418175be36f1e910f52be88c4cb3b34830"
    );

    let cases = manifest
        .cases
        .iter()
        .map(|case| (case.id.as_str(), case))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(cases.len(), 26);
    assert_eq!(replay.providers.len(), 3);

    for provider in &replay.providers {
        assert_eq!(provider.observations.len(), 26, "{}", provider.provider);
        let mut qualification_exact = 0usize;
        let mut materialized_exact = 0usize;
        let mut wrong_target_relevant = 0usize;

        for observed in &provider.observations {
            let case = cases
                .get(observed.id.as_str())
                .unwrap_or_else(|| panic!("{} missing {}", provider.provider, observed.id));

            let effective = derive_effective_evidence_local_qualification_v7(
                &case.policy,
                &case.candidate,
                observed.observed_proposal.as_ref(),
                observed.observed_local_qualification.as_ref(),
            )
            .unwrap_or_else(|error| {
                panic!(
                    "{} {} successor v5 qualification: {error}",
                    provider.provider, observed.id
                )
            });
            if effective == case.expected_local_qualification {
                qualification_exact += 1;
            } else {
                panic!(
                    "{} {} successor v5 qualification mismatch: expected {:?}, got {:?}",
                    provider.provider, observed.id, case.expected_local_qualification, effective
                );
            }

            let assessment = materialize_evidence_relevance_v20(
                &case.policy,
                &case.candidate,
                observed.observed_proposal.as_ref(),
                observed.observed_local_qualification.as_ref(),
            )
            .unwrap_or_else(|error| {
                panic!(
                    "{} {} successor v5 materialization: {error}",
                    provider.provider, observed.id
                )
            });
            if assessment.disposition == case.expected_disposition {
                materialized_exact += 1;
            } else {
                panic!(
                    "{} {} successor v5 disposition mismatch: expected {:?}, got {:?}; historical {:?}",
                    provider.provider,
                    observed.id,
                    case.expected_disposition,
                    assessment.disposition,
                    observed.materialized_disposition
                );
            }

            if case.expected_disposition != EvidenceRelevanceDisposition::Relevant
                && assessment.disposition == EvidenceRelevanceDisposition::Relevant
            {
                wrong_target_relevant += 1;
            }
        }

        assert_eq!(
            qualification_exact, 26,
            "{} successor v5 authority qualification",
            provider.provider
        );
        assert_eq!(
            materialized_exact, 26,
            "{} successor v5 materialization",
            provider.provider
        );
        assert_eq!(
            wrong_target_relevant, 0,
            "{} successor v5 wrong-target Relevant",
            provider.provider
        );
    }
}

#[test]
fn successor_v5_preserves_all_pre_v4_immutable_replay_surfaces() {
    assert_successor_v5_replay(
        "evidence-relevance-calibration-v23/manifest.json",
        "evidence-relevance-holdout-successor-v2/v23-observation-replay.json",
        48,
    );
    assert_successor_v5_replay(
        "evidence-relevance-holdout-v1/manifest.json",
        "evidence-relevance-holdout-successor-v2/holdout-v1-observation-replay.json",
        26,
    );
    assert_successor_v5_replay(
        "evidence-relevance-holdout-v2/manifest.json",
        "evidence-relevance-holdout-successor-v3/holdout-v2-observation-replay.json",
        26,
    );
    assert_successor_v5_replay(
        "evidence-relevance-holdout-v3/manifest.json",
        "evidence-relevance-holdout-successor-v4/holdout-v3a-observation-replay.json",
        26,
    );
}

#[test]
fn successor_v5_recovers_the_failed_two_provider_development_observation() {
    let manifest = load_manifest_at("evidence-relevance-successor-v5-development/manifest.json");
    let replay = load_development_replay_at(
        "evidence-relevance-successor-v5-development/observations-run-36666145098.json",
    );
    assert_eq!(replay.source.run_id, 36_666_145_098);
    assert_eq!(
        replay.source.candidate_commit,
        "dabb7fba665e1f939f452753c587c8811288d3ef"
    );
    assert!(!replay.source.holdout_acceptance_evidence);
    assert_eq!(replay.providers.len(), 2);

    let cases = manifest
        .cases
        .iter()
        .map(|case| (case.id.as_str(), case))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(cases.len(), 16);

    for provider in &replay.providers {
        assert_eq!(provider.observations.len(), 16, "{}", provider.provider);
        for observed in &provider.observations {
            let case = cases
                .get(observed.id.as_str())
                .unwrap_or_else(|| panic!("{} missing {}", provider.provider, observed.id));
            let effective = derive_effective_evidence_local_qualification_v7(
                &case.policy,
                &case.candidate,
                observed.observed_proposal.as_ref(),
                observed.observed_local_qualification.as_ref(),
            )
            .unwrap_or_else(|error| {
                panic!(
                    "{} {} development qualification: {error}",
                    provider.provider, observed.id
                )
            });
            assert_eq!(
                effective, case.expected_local_qualification,
                "{} {} development authority mismatch",
                provider.provider, observed.id
            );

            let assessment = materialize_evidence_relevance_v20(
                &case.policy,
                &case.candidate,
                observed.observed_proposal.as_ref(),
                observed.observed_local_qualification.as_ref(),
            )
            .unwrap_or_else(|error| {
                panic!(
                    "{} {} development materialization: {error}",
                    provider.provider, observed.id
                )
            });
            assert_eq!(
                assessment.disposition, case.expected_disposition,
                "{} {} development disposition mismatch; historical {:?}",
                provider.provider, observed.id, observed.materialized_disposition
            );
        }
    }
}

#[test]
fn immutable_holdout_v4_failures_remain_recorded_in_the_replay() {
    let replay = load_replay();
    let historical_misses = replay
        .providers
        .iter()
        .flat_map(|provider| {
            provider
                .observations
                .iter()
                .map(move |observation| (provider.provider.as_str(), observation))
        })
        .filter(|(provider, observation)| {
            matches!(
                (
                    *provider,
                    observation.id.as_str(),
                    observation.materialized_disposition
                ),
                (
                    "mistral",
                    "v4h11_negative_navigation_target_other_subject",
                    Some(EvidenceRelevanceDisposition::Ambiguous)
                ) | (
                    "mistral",
                    "v4h12_negative_generic_catalog_absence",
                    Some(EvidenceRelevanceDisposition::Ambiguous)
                ) | (
                    "groq",
                    "v4h15_negative_prompt_injection_absence",
                    Some(EvidenceRelevanceDisposition::Ambiguous)
                ) | (
                    "google",
                    "v4h14_negative_explicit_separate_service",
                    Some(EvidenceRelevanceDisposition::Ambiguous)
                )
            )
        })
        .count();

    assert_eq!(historical_misses, 4);
}
