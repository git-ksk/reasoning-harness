use std::{collections::BTreeMap, fs, path::PathBuf};

use reasoning_harness_core::{
    EvidenceLocalBlockingReason, EvidenceLocalQualificationV6, EvidenceRelevanceBindingProposal,
    EvidenceRelevanceCandidate, EvidenceRelevanceDisposition, EvidenceRelevanceTargetPolicy,
    derive_effective_evidence_local_qualification_v8,
    derive_effective_evidence_local_qualification_v9, materialize_evidence_relevance_v22,
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

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}
fn load_manifest(path: &str) -> Manifest {
    serde_json::from_str(&fs::read_to_string(root().join("fixtures").join(path)).unwrap()).unwrap()
}
fn load_replay(path: &str) -> Replay {
    serde_json::from_str(&fs::read_to_string(root().join("fixtures").join(path)).unwrap()).unwrap()
}
fn replay(manifest_path: &str, replay_path: &str, expected_cases: usize) {
    let manifest = load_manifest(manifest_path);
    let replay = load_replay(replay_path);
    let cases = manifest
        .cases
        .iter()
        .map(|c| (c.id.as_str(), c))
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
        let mut wrong_target_relevant = 0;
        for observed in &provider.observations {
            let case = cases.get(observed.id.as_str()).unwrap();
            let effective = derive_effective_evidence_local_qualification_v9(
                &case.policy,
                &case.candidate,
                observed.observed_proposal.as_ref(),
                observed.observed_local_qualification.as_ref(),
            )
            .unwrap();
            assert_eq!(
                effective.scope_risk, case.expected_local_qualification.scope_risk,
                "{} {} risk",
                provider.provider, observed.id
            );
            if case.expected_local_qualification.scope_risk == EvidenceLocalBlockingReason::None {
                assert_eq!(
                    effective.identity_scope, case.expected_local_qualification.identity_scope,
                    "{} {} identity",
                    provider.provider, observed.id
                );
                assert_eq!(
                    effective.relation_scope, case.expected_local_qualification.relation_scope,
                    "{} {} relation",
                    provider.provider, observed.id
                );
            }
            let assessment = materialize_evidence_relevance_v22(
                &case.policy,
                &case.candidate,
                observed.observed_proposal.as_ref(),
                observed.observed_local_qualification.as_ref(),
            )
            .unwrap();
            assert_eq!(
                assessment.disposition, case.expected_disposition,
                "{} {} successor-v7; historical {:?}",
                provider.provider, observed.id, observed.materialized_disposition
            );
            if case.expected_disposition != EvidenceRelevanceDisposition::Relevant
                && assessment.disposition == EvidenceRelevanceDisposition::Relevant
            {
                wrong_target_relevant += 1;
            }
        }
        assert_eq!(
            wrong_target_relevant, 0,
            "{} wrong-target Relevant",
            provider.provider
        );
    }
}

#[test]
fn successor_v7_replays_v23_and_observed_holdouts_through_v7_exactly() {
    replay(
        "evidence-relevance-calibration-v23/manifest.json",
        "evidence-relevance-holdout-successor-v2/v23-observation-replay.json",
        48,
    );
    replay(
        "evidence-relevance-holdout-v1/manifest.json",
        "evidence-relevance-holdout-successor-v2/holdout-v1-observation-replay.json",
        26,
    );
    replay(
        "evidence-relevance-holdout-v2/manifest.json",
        "evidence-relevance-holdout-successor-v3/holdout-v2-observation-replay.json",
        26,
    );
    replay(
        "evidence-relevance-holdout-v3/manifest.json",
        "evidence-relevance-holdout-successor-v4/holdout-v3a-observation-replay.json",
        26,
    );
    replay(
        "evidence-relevance-holdout-v4/manifest.json",
        "evidence-relevance-holdout-successor-v5/holdout-v4-observation-replay.json",
        26,
    );
    replay(
        "evidence-relevance-holdout-v5/manifest.json",
        "evidence-relevance-holdout-successor-v6/holdout-v5-observation-replay.json",
        26,
    );
    replay(
        "evidence-relevance-holdout-v7/manifest.json",
        "evidence-relevance-holdout-successor-v7/holdout-v7-observation-replay.json",
        26,
    );
}

#[test]
fn holdout_v5_provenance_and_historical_groq_miss_remain_immutable() {
    let replay =
        load_replay("evidence-relevance-holdout-successor-v6/holdout-v5-observation-replay.json");
    assert_eq!(replay.source.canonical_run, 36_694_957_246);
    assert_eq!(replay.source.attempt, 1);
    assert_eq!(
        replay.source.freeze_tag,
        "engine-0.6-evidence-relevance-holdout-v5-freeze"
    );
    assert_eq!(
        replay.source.freeze_commit,
        "270c1907103c8ef85fa72875b47ff883feec9a86"
    );
    let groq = replay
        .providers
        .iter()
        .find(|p| p.provider == "groq")
        .unwrap();
    let miss = groq
        .observations
        .iter()
        .find(|o| o.id == "v5h15_negative_prompt_injection_absence")
        .unwrap();
    assert_eq!(
        miss.materialized_disposition,
        Some(EvidenceRelevanceDisposition::Ambiguous)
    );

    let manifest = load_manifest("evidence-relevance-holdout-v5/manifest.json");
    let case = manifest.cases.iter().find(|c| c.id == miss.id).unwrap();
    let successor = materialize_evidence_relevance_v22(
        &case.policy,
        &case.candidate,
        miss.observed_proposal.as_ref(),
        miss.observed_local_qualification.as_ref(),
    )
    .unwrap();
    assert_eq!(
        successor.disposition,
        EvidenceRelevanceDisposition::Irrelevant
    );
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
    #[serde(default)]
    development_gate: Option<String>,
}

#[test]
fn successor_v7_replays_prior_successor_v6_development_observation() {
    let manifest = load_manifest("evidence-relevance-successor-v6-development/manifest.json");
    let raw = fs::read_to_string(root().join(
        "fixtures/evidence-relevance-successor-v6-development/observations-run-36728692499.json",
    ))
    .unwrap();
    let replay: DevelopmentReplay = serde_json::from_str(&raw).unwrap();
    assert_eq!(replay.source.run_id, 36_728_692_499);
    assert_eq!(
        replay.source.candidate_commit,
        "7a8ee5059079e709d82ccf962528f2e6af034853"
    );
    assert!(!replay.source.holdout_acceptance_evidence);
    assert_eq!(replay.providers.len(), 2);
    let cases = manifest
        .cases
        .iter()
        .map(|c| (c.id.as_str(), c))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(cases.len(), 16);
    for provider in &replay.providers {
        assert_eq!(provider.observations.len(), 16, "{}", provider.provider);
        for observed in &provider.observations {
            let case = cases.get(observed.id.as_str()).unwrap();
            let effective = derive_effective_evidence_local_qualification_v9(
                &case.policy,
                &case.candidate,
                observed.observed_proposal.as_ref(),
                observed.observed_local_qualification.as_ref(),
            )
            .unwrap();
            assert_eq!(
                effective, case.expected_local_qualification,
                "{} {} authority",
                provider.provider, observed.id
            );
            let assessment = materialize_evidence_relevance_v22(
                &case.policy,
                &case.candidate,
                observed.observed_proposal.as_ref(),
                observed.observed_local_qualification.as_ref(),
            )
            .unwrap();
            assert_eq!(
                assessment.disposition, case.expected_disposition,
                "{} {} disposition",
                provider.provider, observed.id
            );
        }
    }
}

#[test]
fn successor_v7_replays_converged_two_provider_development_observation() {
    let manifest = load_manifest("evidence-relevance-successor-v7-development/manifest.json");
    let raw = fs::read_to_string(root().join(
        "fixtures/evidence-relevance-successor-v7-development/observations-run-36890934124.json",
    ))
    .unwrap();
    let replay: DevelopmentReplay = serde_json::from_str(&raw).unwrap();
    assert_eq!(replay.source.run_id, 36_890_934_124);
    assert_eq!(
        replay.source.candidate_commit,
        "41d853080a991b3a9f3a976b758d8b338ce2c784"
    );
    assert!(!replay.source.holdout_acceptance_evidence);
    assert_eq!(
        replay.source.development_gate.as_deref(),
        Some("successor_owned_identity_scope_v2")
    );
    assert_eq!(replay.providers.len(), 2);

    let summary_raw =
        fs::read_to_string(root().join(
            "fixtures/evidence-relevance-successor-v7-development/summary-run-36890934124.json",
        ))
        .unwrap();
    let summary: serde_json::Value = serde_json::from_str(&summary_raw).unwrap();
    assert_eq!(
        summary["schema_version"],
        "engine-0.6-evidence-relevance-successor-v7-development-summary-v2"
    );
    assert_eq!(summary["holdout_acceptance_evidence"], false);
    assert_eq!(summary["development_gate_passed"], true);
    for provider in ["mistral", "google"] {
        let metrics = &summary["providers"][provider];
        assert_eq!(metrics["passed"], true, "{provider}");
        assert_eq!(
            metrics["effective_authority_identity_scope_misses"], 0,
            "{provider}"
        );
        assert_eq!(
            metrics["effective_qualification_scope_risk_misses"], 0,
            "{provider}"
        );
        assert_eq!(
            metrics["effective_qualification_spurious_scope_risks"], 0,
            "{provider}"
        );
        assert_eq!(metrics["materialized_exact_matches"], 12, "{provider}");
        assert_eq!(metrics["wrong_target_relevance_retention"], 0, "{provider}");
        assert_eq!(metrics["utility_misses"], 0, "{provider}");
    }

    let cases = manifest
        .cases
        .iter()
        .map(|c| (c.id.as_str(), c))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(cases.len(), 12);

    for provider in &replay.providers {
        assert_eq!(provider.observations.len(), 12, "{}", provider.provider);
        let mut wrong_target_relevant = 0;
        for observed in &provider.observations {
            let case = cases.get(observed.id.as_str()).unwrap();
            let historical = derive_effective_evidence_local_qualification_v8(
                &case.policy,
                &case.candidate,
                observed.observed_proposal.as_ref(),
                observed.observed_local_qualification.as_ref(),
            )
            .unwrap();
            let successor = derive_effective_evidence_local_qualification_v9(
                &case.policy,
                &case.candidate,
                observed.observed_proposal.as_ref(),
                observed.observed_local_qualification.as_ref(),
            )
            .unwrap();

            assert_eq!(
                successor.relation_scope, historical.relation_scope,
                "{} {} relation preservation",
                provider.provider, observed.id
            );
            assert_eq!(
                successor.scope_risk, historical.scope_risk,
                "{} {} risk preservation",
                provider.provider, observed.id
            );
            assert_eq!(
                successor.identity_scope, case.expected_local_qualification.identity_scope,
                "{} {} successor-owned identity",
                provider.provider, observed.id
            );
            assert_eq!(
                successor.scope_risk, case.expected_local_qualification.scope_risk,
                "{} {} successor-owned risk",
                provider.provider, observed.id
            );

            assert_eq!(
                observed.materialized_disposition,
                Some(case.expected_disposition),
                "{} {} captured disposition",
                provider.provider,
                observed.id
            );
            let assessment = materialize_evidence_relevance_v22(
                &case.policy,
                &case.candidate,
                observed.observed_proposal.as_ref(),
                observed.observed_local_qualification.as_ref(),
            )
            .unwrap();
            assert_eq!(
                assessment.disposition, case.expected_disposition,
                "{} {} disposition",
                provider.provider, observed.id
            );
            if case.expected_disposition != EvidenceRelevanceDisposition::Relevant
                && assessment.disposition == EvidenceRelevanceDisposition::Relevant
            {
                wrong_target_relevant += 1;
            }
        }
        assert_eq!(
            wrong_target_relevant, 0,
            "{} wrong-target Relevant",
            provider.provider
        );
    }
}

#[test]
fn holdout_v7_provenance_and_historical_groq_miss_remain_immutable() {
    let replay =
        load_replay("evidence-relevance-holdout-successor-v7/holdout-v7-observation-replay.json");
    assert_eq!(replay.source.canonical_run, 36_800_959_088);
    assert_eq!(replay.source.attempt, 1);
    assert_eq!(
        replay.source.freeze_tag,
        "engine-0.6-evidence-relevance-holdout-v7-freeze"
    );
    assert_eq!(
        replay.source.freeze_commit,
        "962709535a09de80246e2f9e7e626f508bf18230"
    );

    let groq = replay
        .providers
        .iter()
        .find(|p| p.provider == "groq")
        .unwrap();
    let miss = groq
        .observations
        .iter()
        .find(|o| o.id == "v7h25_ambiguous_url_single_sibling")
        .unwrap();
    assert_eq!(
        miss.materialized_disposition,
        Some(EvidenceRelevanceDisposition::Irrelevant)
    );

    let manifest = load_manifest("evidence-relevance-holdout-v7/manifest.json");
    let case = manifest.cases.iter().find(|c| c.id == miss.id).unwrap();
    let effective = derive_effective_evidence_local_qualification_v9(
        &case.policy,
        &case.candidate,
        miss.observed_proposal.as_ref(),
        miss.observed_local_qualification.as_ref(),
    )
    .unwrap();
    assert_eq!(effective, case.expected_local_qualification);
    let successor = materialize_evidence_relevance_v22(
        &case.policy,
        &case.candidate,
        miss.observed_proposal.as_ref(),
        miss.observed_local_qualification.as_ref(),
    )
    .unwrap();
    assert_eq!(
        successor.disposition,
        EvidenceRelevanceDisposition::Ambiguous
    );
}
