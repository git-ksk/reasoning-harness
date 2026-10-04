use std::{collections::BTreeMap, fs, path::PathBuf};

use reasoning_harness_core::{
    EvidenceLocalQualificationV6, EvidenceRelevanceBindingProposal, EvidenceRelevanceCandidate,
    EvidenceRelevanceDisposition, EvidenceRelevanceTargetPolicy,
    derive_effective_evidence_local_qualification_v16, materialize_evidence_relevance_v28,
    materialize_evidence_relevance_v29,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Manifest {
    cases: Vec<Case>,
}

#[derive(Debug, Deserialize)]
struct Case {
    id: String,
    #[serde(default)]
    authority_expectation: String,
    policy: EvidenceRelevanceTargetPolicy,
    candidate: EvidenceRelevanceCandidate,
    expected_disposition: EvidenceRelevanceDisposition,
    expected_proposal: EvidenceRelevanceBindingProposal,
    expected_local_qualification: EvidenceLocalQualificationV6,
}

#[derive(Debug, Deserialize)]
struct ProviderResult {
    observations: Vec<Observation>,
}

#[derive(Debug, Deserialize)]
struct Observation {
    id: String,
    observed_proposal: Option<EvidenceRelevanceBindingProposal>,
    observed_local_qualification: Option<EvidenceLocalQualificationV6>,
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

fn case_map(relative: &str) -> BTreeMap<String, Case> {
    let manifest: Manifest = load(relative);
    manifest
        .cases
        .into_iter()
        .map(|case| (case.id.clone(), case))
        .collect()
}

#[test]
fn v29_preserves_precommitted_successor_contracts() {
    for relative in [
        "fixtures/evidence-relevance-successor-v9-development/manifest.json",
        "fixtures/evidence-relevance-holdout-v10/manifest.json",
        "fixtures/evidence-relevance-successor-v10-development-v2/manifest.json",
        "fixtures/evidence-relevance-successor-v10-development-v3/manifest.json",
        "fixtures/evidence-relevance-successor-v10-development-v4/manifest.json",
        "fixtures/evidence-relevance-holdout-v11/manifest.json",
    ] {
        let manifest: Manifest = load(relative);
        for case in manifest.cases {
            let assessment = materialize_evidence_relevance_v29(
                &case.policy,
                &case.candidate,
                Some(&case.expected_proposal),
                Some(&case.expected_local_qualification),
            )
            .expect("v29 expected-contract replay");
            assert_eq!(
                assessment.disposition, case.expected_disposition,
                "expected contract changed under v29: {}",
                case.id
            );
        }
    }
}

#[test]
fn v29_holdout_v11_replay_repairs_terminal_miss_without_new_terminal_changes() {
    let cases = case_map("fixtures/evidence-relevance-holdout-v11/manifest.json");
    let mut baseline_misses = Vec::new();
    let mut successor_misses = Vec::new();
    let mut changed = Vec::new();

    for provider in ["mistral", "google", "groq"] {
        let result: ProviderResult = load(&format!(
            "fixtures/evidence-relevance-holdout-v11-result/{provider}-run-37182677114.json"
        ));
        for observation in result.observations {
            let case = cases.get(&observation.id).expect("v11 case");
            let proposal = observation
                .observed_proposal
                .as_ref()
                .expect("v11 observed proposal");
            let raw = observation
                .observed_local_qualification
                .as_ref()
                .expect("v11 observed local qualification");

            let baseline = materialize_evidence_relevance_v28(
                &case.policy,
                &case.candidate,
                Some(proposal),
                Some(raw),
            )
            .expect("v28 replay")
            .disposition;
            let successor = materialize_evidence_relevance_v29(
                &case.policy,
                &case.candidate,
                Some(proposal),
                Some(raw),
            )
            .expect("v29 replay")
            .disposition;

            if baseline != case.expected_disposition {
                baseline_misses.push(format!("{provider}:{}", observation.id));
            }
            if successor != case.expected_disposition {
                successor_misses.push(format!("{provider}:{}", observation.id));
            }
            if successor != baseline {
                changed.push(format!(
                    "{provider}:{}:{baseline:?}->{successor:?}",
                    observation.id
                ));
            }
        }
    }

    assert_eq!(
        baseline_misses,
        vec!["groq:v11h21_ambiguous_generic_no_relation"]
    );
    assert!(
        successor_misses.is_empty(),
        "v29 holdout-v11 misses: {successor_misses:?}"
    );
    assert_eq!(
        changed,
        vec!["groq:v11h21_ambiguous_generic_no_relation:Irrelevant->Ambiguous"]
    );
}

#[derive(Debug, Deserialize)]
struct CombinedProviderResults {
    providers: Vec<NamedProviderResult>,
}

#[derive(Debug, Deserialize)]
struct NamedProviderResult {
    provider: String,
    observations: Vec<Observation>,
}

fn assert_v29_observations_match_manifest(manifest_path: &str, result_path: &str) {
    let cases = case_map(manifest_path);
    let result: ProviderResult = load(result_path);
    for observation in result.observations {
        let case = cases.get(&observation.id).expect("replay case");
        let proposal = observation
            .observed_proposal
            .as_ref()
            .expect("observed proposal");
        let raw = observation
            .observed_local_qualification
            .as_ref()
            .expect("observed local qualification");
        let successor = materialize_evidence_relevance_v29(
            &case.policy,
            &case.candidate,
            Some(proposal),
            Some(raw),
        )
        .expect("v29 replay")
        .disposition;
        assert_eq!(
            successor, case.expected_disposition,
            "v29 historical observation miss: {result_path}:{}",
            observation.id
        );
    }
}

#[test]
fn v29_replays_all_recent_immutable_provider_observations_to_frozen_terminal_contracts() {
    for result in [
        "fixtures/evidence-relevance-holdout-v10-result/mistral-run-37085574010.json",
        "fixtures/evidence-relevance-holdout-v10-result/google-run-37085574010.json",
        "fixtures/evidence-relevance-holdout-v10-result/groq-run-37085574010.json",
    ] {
        assert_v29_observations_match_manifest(
            "fixtures/evidence-relevance-holdout-v10/manifest.json",
            result,
        );
    }

    for result in [
        "fixtures/evidence-relevance-successor-v10-development-v2-result/mistral-run-37100490212.json",
        "fixtures/evidence-relevance-successor-v10-development-v2-result/google-run-37100490212.json",
    ] {
        assert_v29_observations_match_manifest(
            "fixtures/evidence-relevance-successor-v10-development-v2/manifest.json",
            result,
        );
    }

    for result in [
        "fixtures/evidence-relevance-successor-v10-development-v3-result/mistral-run-37105677785.json",
        "fixtures/evidence-relevance-successor-v10-development-v3-result/google-partial-checkpoint-run-37105677785.json",
    ] {
        assert_v29_observations_match_manifest(
            "fixtures/evidence-relevance-successor-v10-development-v3/manifest.json",
            result,
        );
    }

    for result in [
        "fixtures/evidence-relevance-successor-v10-development-v4-result/mistral-run-37111376390.json",
        "fixtures/evidence-relevance-successor-v10-development-v4-result/google-run-37111376390.json",
    ] {
        assert_v29_observations_match_manifest(
            "fixtures/evidence-relevance-successor-v10-development-v4/manifest.json",
            result,
        );
    }

    for result in [
        "fixtures/evidence-relevance-holdout-v11-result/mistral-run-37182677114.json",
        "fixtures/evidence-relevance-holdout-v11-result/google-run-37182677114.json",
        "fixtures/evidence-relevance-holdout-v11-result/groq-run-37182677114.json",
    ] {
        assert_v29_observations_match_manifest(
            "fixtures/evidence-relevance-holdout-v11/manifest.json",
            result,
        );
    }

    let cases = case_map("fixtures/evidence-relevance-successor-v9-development/manifest.json");
    let combined: CombinedProviderResults = load(
        "fixtures/evidence-relevance-successor-v9-development/observations-run-37015407859.json",
    );
    for provider in combined.providers {
        for observation in provider.observations {
            let case = cases.get(&observation.id).expect("v9 development case");
            let proposal = observation
                .observed_proposal
                .as_ref()
                .expect("v9 observed proposal");
            let raw = observation
                .observed_local_qualification
                .as_ref()
                .expect("v9 observed local qualification");
            let successor = materialize_evidence_relevance_v29(
                &case.policy,
                &case.candidate,
                Some(proposal),
                Some(raw),
            )
            .expect("v29 v9 replay")
            .disposition;
            assert_eq!(
                successor, case.expected_disposition,
                "v29 v9 historical observation miss: {}:{}",
                provider.provider, observation.id
            );
        }
    }
}

#[test]
fn v16_keeps_harness_owned_true_different_relation_cases_negative() {
    let manifest: Manifest = load("fixtures/evidence-relevance-holdout-v11/manifest.json");

    for case in manifest
        .cases
        .iter()
        .filter(|case| case.authority_expectation == "require_different")
    {
        let effective = derive_effective_evidence_local_qualification_v16(
            &case.policy,
            &case.candidate,
            Some(&case.expected_proposal),
            Some(&case.expected_local_qualification),
        )
        .expect("derive v16");
        assert_eq!(
            effective.relation_scope,
            reasoning_harness_core::EvidenceLocalRelationScope::DifferentRelation,
            "Harness-owned different relation lost: {}",
            case.id
        );
    }
}
