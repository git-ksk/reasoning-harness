use std::{collections::BTreeMap, fs, path::PathBuf};

use reasoning_harness_core::{
    EvidenceLocalQualificationV6, EvidenceRelevanceBindingProposal, EvidenceRelevanceCandidate,
    EvidenceRelevanceDisposition, EvidenceRelevanceTargetPolicy,
    materialize_evidence_relevance_v23, materialize_evidence_relevance_v26,
    materialize_evidence_relevance_v27, materialize_evidence_relevance_v28,
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
fn v28_preserves_all_precommitted_expected_contracts() {
    for relative in [
        "fixtures/evidence-relevance-successor-v9-development/manifest.json",
        "fixtures/evidence-relevance-holdout-v10/manifest.json",
        "fixtures/evidence-relevance-successor-v10-development-v2/manifest.json",
        "fixtures/evidence-relevance-successor-v10-development-v3/manifest.json",
    ] {
        let manifest: Manifest = load(relative);
        for case in manifest.cases {
            let assessment = materialize_evidence_relevance_v28(
                &case.policy,
                &case.candidate,
                Some(&case.expected_proposal),
                Some(&case.expected_local_qualification),
            )
            .expect("v28 expected-contract replay");
            assert_eq!(
                assessment.disposition, case.expected_disposition,
                "expected contract changed under v28: {}",
                case.id
            );
        }
    }
}

#[test]
fn v28_canonical_holdout_v10_replay_changes_only_known_groq_miss() {
    let cases = case_map("fixtures/evidence-relevance-holdout-v10/manifest.json");
    let mut baseline_misses = Vec::new();
    let mut successor_misses = Vec::new();
    let mut changed = Vec::new();

    for provider in ["mistral", "google", "groq"] {
        let result: ProviderResult = load(&format!(
            "fixtures/evidence-relevance-holdout-v10-result/{provider}-run-37085574010.json"
        ));
        for observation in result.observations {
            let case = cases.get(&observation.id).expect("v10 case");
            let proposal = observation
                .observed_proposal
                .as_ref()
                .expect("v10 observed proposal");
            let raw = observation
                .observed_local_qualification
                .as_ref()
                .expect("v10 observed local qualification");

            let baseline = materialize_evidence_relevance_v23(
                &case.policy,
                &case.candidate,
                Some(proposal),
                Some(raw),
            )
            .expect("v23 replay")
            .disposition;
            let successor = materialize_evidence_relevance_v28(
                &case.policy,
                &case.candidate,
                Some(proposal),
                Some(raw),
            )
            .expect("v28 replay")
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
        vec!["groq:v10h18_negative_exact_target_numeric_non_frame"]
    );
    assert!(
        successor_misses.is_empty(),
        "v28 canonical v10 misses: {successor_misses:?}"
    );
    assert_eq!(
        changed,
        vec!["groq:v10h18_negative_exact_target_numeric_non_frame:Ambiguous->Irrelevant"]
    );
}

#[test]
fn v28_immutable_v2_replay_keeps_only_the_v27_composition_fix() {
    let cases = case_map("fixtures/evidence-relevance-successor-v10-development-v2/manifest.json");
    let mut v26_misses = Vec::new();
    let mut v28_misses = Vec::new();
    let mut changed = Vec::new();

    for provider in ["mistral", "google"] {
        let result: ProviderResult = load(&format!(
            "fixtures/evidence-relevance-successor-v10-development-v2-result/{provider}-run-37100490212.json"
        ));
        for observation in result.observations {
            let case = cases.get(&observation.id).expect("v2 case");
            let proposal = observation
                .observed_proposal
                .as_ref()
                .expect("v2 observed proposal");
            let raw = observation
                .observed_local_qualification
                .as_ref()
                .expect("v2 observed local qualification");

            let baseline = materialize_evidence_relevance_v26(
                &case.policy,
                &case.candidate,
                Some(proposal),
                Some(raw),
            )
            .expect("v26 replay")
            .disposition;
            let successor = materialize_evidence_relevance_v28(
                &case.policy,
                &case.candidate,
                Some(proposal),
                Some(raw),
            )
            .expect("v28 replay")
            .disposition;

            if baseline != case.expected_disposition {
                v26_misses.push(format!("{provider}:{}", observation.id));
            }
            if successor != case.expected_disposition {
                v28_misses.push(format!("{provider}:{}", observation.id));
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
        v26_misses,
        vec!["mistral:sv10v2_18_requested_availability_with_feature"]
    );
    assert!(v28_misses.is_empty(), "v28 v2 misses: {v28_misses:?}");
    assert_eq!(
        changed,
        vec!["mistral:sv10v2_18_requested_availability_with_feature:Irrelevant->Relevant"]
    );
}

#[test]
fn v28_immutable_v3_mistral_replay_fixes_exactly_both_observed_misses() {
    let cases = case_map("fixtures/evidence-relevance-successor-v10-development-v3/manifest.json");
    let result: ProviderResult = load(
        "fixtures/evidence-relevance-successor-v10-development-v3-result/mistral-run-37105677785.json",
    );
    let mut v27_misses = Vec::new();
    let mut v28_misses = Vec::new();
    let mut changed = Vec::new();

    for observation in result.observations {
        let case = cases.get(&observation.id).expect("v3 case");
        let proposal = observation
            .observed_proposal
            .as_ref()
            .expect("v3 observed proposal");
        let raw = observation
            .observed_local_qualification
            .as_ref()
            .expect("v3 observed local qualification");

        let baseline = materialize_evidence_relevance_v27(
            &case.policy,
            &case.candidate,
            Some(proposal),
            Some(raw),
        )
        .expect("v27 replay")
        .disposition;
        let successor = materialize_evidence_relevance_v28(
            &case.policy,
            &case.candidate,
            Some(proposal),
            Some(raw),
        )
        .expect("v28 replay")
        .disposition;

        if baseline != case.expected_disposition {
            v27_misses.push(observation.id.clone());
        }
        if successor != case.expected_disposition {
            v28_misses.push(observation.id.clone());
        }
        if successor != baseline {
            changed.push(format!("{}:{baseline:?}->{successor:?}", observation.id));
        }
    }

    assert_eq!(
        v27_misses,
        vec!["sv10v3_08_launch_definition", "sv10v3_20_prompt_injection",]
    );
    assert!(v28_misses.is_empty(), "v28 v3 misses: {v28_misses:?}");
    assert_eq!(
        changed,
        vec![
            "sv10v3_08_launch_definition:Ambiguous->Irrelevant",
            "sv10v3_20_prompt_injection:Irrelevant->Ambiguous",
        ]
    );
}

#[test]
fn v28_v3_google_partial_replay_fixes_only_definition_gap() {
    let cases = case_map("fixtures/evidence-relevance-successor-v10-development-v3/manifest.json");
    let result: ProviderResult = load(
        "fixtures/evidence-relevance-successor-v10-development-v3-result/google-partial-checkpoint-run-37105677785.json",
    );
    let mut v27_misses = Vec::new();
    let mut v28_misses = Vec::new();
    let mut changed = Vec::new();

    for observation in result.observations {
        let case = cases.get(&observation.id).expect("v3 case");
        let proposal = observation
            .observed_proposal
            .as_ref()
            .expect("v3 google partial proposal");
        let raw = observation
            .observed_local_qualification
            .as_ref()
            .expect("v3 google partial local qualification");

        let baseline = materialize_evidence_relevance_v27(
            &case.policy,
            &case.candidate,
            Some(proposal),
            Some(raw),
        )
        .expect("v27 google partial replay")
        .disposition;
        let successor = materialize_evidence_relevance_v28(
            &case.policy,
            &case.candidate,
            Some(proposal),
            Some(raw),
        )
        .expect("v28 google partial replay")
        .disposition;

        if baseline != case.expected_disposition {
            v27_misses.push(observation.id.clone());
        }
        if successor != case.expected_disposition {
            v28_misses.push(observation.id.clone());
        }
        if successor != baseline {
            changed.push(format!("{}:{baseline:?}->{successor:?}", observation.id));
        }
    }

    assert_eq!(v27_misses, vec!["sv10v3_08_launch_definition"]);
    assert!(
        v28_misses.is_empty(),
        "v28 v3 google partial misses: {v28_misses:?}"
    );
    assert_eq!(
        changed,
        vec!["sv10v3_08_launch_definition:Ambiguous->Irrelevant"]
    );
}
