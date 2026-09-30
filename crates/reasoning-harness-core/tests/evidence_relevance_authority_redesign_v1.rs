use std::{collections::BTreeMap, fs, path::PathBuf};

use reasoning_harness_core::{
    EvidenceLocalQualificationV6, EvidenceRelevanceBindingProposal, EvidenceRelevanceCandidate,
    EvidenceRelevanceDisposition, EvidenceRelevanceTargetPolicy,
    materialize_evidence_relevance_v20, resolve_evidence_local_authority_v1,
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
    expected_disposition: EvidenceRelevanceDisposition,
}

#[derive(Debug, Deserialize)]
struct ReplayFixture {
    providers: Vec<ProviderReplay>,
}

#[derive(Debug, Deserialize)]
struct ProviderReplay {
    provider: String,
    observations: Vec<ReplayObservation>,
}

#[derive(Debug, Deserialize)]
struct ReplayObservation {
    id: String,
    observed_proposal: Option<EvidenceRelevanceBindingProposal>,
    observed_local_qualification: Option<EvidenceLocalQualificationV6>,
}

fn fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(path)
}

fn manifest(path: &str) -> BTreeMap<String, Case> {
    let value: Manifest = serde_json::from_slice(&fs::read(fixture(path)).expect("read manifest"))
        .expect("parse manifest");
    value
        .cases
        .into_iter()
        .map(|case| (case.id.clone(), case))
        .collect()
}

fn replay(path: &str) -> ReplayFixture {
    serde_json::from_slice(&fs::read(fixture(path)).expect("read replay")).expect("parse replay")
}

fn assert_replay(manifest_path: &str, replay_path: &str, expected_cases: usize) {
    let cases = manifest(manifest_path);
    let replay = replay(replay_path);
    assert_eq!(cases.len(), expected_cases);

    for provider in replay.providers {
        assert_eq!(
            provider.observations.len(),
            expected_cases,
            "{}",
            provider.provider
        );
        let mut exact = 0usize;
        let mut wrong_target_relevant = 0usize;
        let mut false_relevance_rejections = 0usize;

        for observed in provider.observations {
            let case = cases
                .get(&observed.id)
                .unwrap_or_else(|| panic!("missing case {}", observed.id));
            let proposal = observed
                .observed_proposal
                .as_ref()
                .unwrap_or_else(|| panic!("{} {} proposal", provider.provider, observed.id));
            let raw = observed
                .observed_local_qualification
                .as_ref()
                .unwrap_or_else(|| panic!("{} {} raw", provider.provider, observed.id));

            let _authority = resolve_evidence_local_authority_v1(
                &case.policy,
                &case.candidate,
                Some(proposal),
                Some(raw),
            );
            let assessment = materialize_evidence_relevance_v20(
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
                exact += 1;
            }
            if case.expected_disposition == EvidenceRelevanceDisposition::Irrelevant
                && assessment.disposition == EvidenceRelevanceDisposition::Relevant
            {
                wrong_target_relevant += 1;
            }
            if case.expected_disposition == EvidenceRelevanceDisposition::Relevant
                && assessment.disposition == EvidenceRelevanceDisposition::Irrelevant
            {
                false_relevance_rejections += 1;
            }

            assert_eq!(
                assessment.disposition, case.expected_disposition,
                "{} {} authority-v1 mismatch",
                provider.provider, observed.id
            );
        }

        assert_eq!(exact, expected_cases, "{} exact", provider.provider);
        assert_eq!(
            wrong_target_relevant, 0,
            "{} wrong-target Relevant",
            provider.provider
        );
        assert_eq!(
            false_relevance_rejections, 0,
            "{} false relevance rejection",
            provider.provider
        );
    }
}

#[test]
fn authority_v1_replays_v23() {
    assert_replay(
        "evidence-relevance-calibration-v23/manifest.json",
        "evidence-relevance-holdout-successor-v2/v23-observation-replay.json",
        48,
    );
}

#[test]
fn authority_v1_replays_holdout_v1() {
    assert_replay(
        "evidence-relevance-holdout-v1/manifest.json",
        "evidence-relevance-holdout-successor-v2/holdout-v1-observation-replay.json",
        26,
    );
}

#[test]
fn authority_v1_replays_holdout_v2() {
    assert_replay(
        "evidence-relevance-holdout-v2/manifest.json",
        "evidence-relevance-holdout-successor-v3/holdout-v2-observation-replay.json",
        26,
    );
}

#[test]
fn authority_v1_replays_holdout_v3a() {
    assert_replay(
        "evidence-relevance-holdout-v3/manifest.json",
        "evidence-relevance-holdout-successor-v4/holdout-v3a-observation-replay.json",
        26,
    );
}
