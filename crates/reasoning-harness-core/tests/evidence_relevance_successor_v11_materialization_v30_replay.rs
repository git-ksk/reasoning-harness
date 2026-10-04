use std::{collections::BTreeMap, fs, path::PathBuf};

use reasoning_harness_core::{
    EvidenceLocalIdentityScope, EvidenceLocalQualificationV6, EvidenceRelevanceBindingProposal,
    EvidenceRelevanceCandidate, EvidenceRelevanceDisposition, EvidenceRelevanceTargetPolicy,
    derive_effective_evidence_local_qualification_v16,
    derive_effective_evidence_local_qualification_v17, materialize_evidence_relevance_v29,
    materialize_evidence_relevance_v30,
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
fn v30_development_v1_live_replay_repairs_only_explicit_ownership_identity_gap() {
    let cases = case_map("fixtures/evidence-relevance-successor-v11-development/manifest.json");
    let mut identity_changes = Vec::new();
    let mut terminal_changes = Vec::new();

    for provider in ["mistral", "google"] {
        let result: ProviderResult = load(&format!(
            "fixtures/evidence-relevance-successor-v11-development-v1-result/{provider}-run-37212349800.json"
        ));

        for observation in result.observations {
            let case = cases.get(&observation.id).expect("v1 case");
            let proposal = observation
                .observed_proposal
                .as_ref()
                .expect("v1 observed proposal");
            let raw = observation
                .observed_local_qualification
                .as_ref()
                .expect("v1 observed local qualification");

            let before = derive_effective_evidence_local_qualification_v16(
                &case.policy,
                &case.candidate,
                Some(proposal),
                Some(raw),
            )
            .expect("v16 replay");
            let after = derive_effective_evidence_local_qualification_v17(
                &case.policy,
                &case.candidate,
                Some(proposal),
                Some(raw),
            )
            .expect("v17 replay");

            if before.identity_scope != after.identity_scope {
                identity_changes.push(format!(
                    "{provider}:{}:{:?}->{:?}",
                    observation.id, before.identity_scope, after.identity_scope
                ));
            }
            assert_eq!(
                before.relation_scope, after.relation_scope,
                "v17 changed relation axis: {provider}:{}",
                observation.id
            );
            assert_eq!(
                before.scope_risk, after.scope_risk,
                "v17 changed scope risk: {provider}:{}",
                observation.id
            );

            let old_final = materialize_evidence_relevance_v29(
                &case.policy,
                &case.candidate,
                Some(proposal),
                Some(raw),
            )
            .expect("v29 replay")
            .disposition;
            let new_final = materialize_evidence_relevance_v30(
                &case.policy,
                &case.candidate,
                Some(proposal),
                Some(raw),
            )
            .expect("v30 replay")
            .disposition;
            if old_final != new_final {
                terminal_changes.push(format!(
                    "{provider}:{}:{old_final:?}->{new_final:?}",
                    observation.id
                ));
            }
            assert_eq!(
                new_final, case.expected_disposition,
                "v30 v1 terminal mismatch: {provider}:{}",
                observation.id
            );
        }
    }

    assert_eq!(
        identity_changes,
        vec![
            "mistral:sv11d_19_risk_clipped_column:ExactTarget->Unresolved",
            "google:sv11d_19_risk_clipped_column:ExactTarget->Unresolved",
        ]
    );
    assert!(
        terminal_changes.is_empty(),
        "v17/v30 must not rewrite immutable v1 terminal results: {terminal_changes:?}"
    );
}

#[test]
fn v17_does_not_downgrade_relation_only_context_gaps() {
    for (manifest_path, case_id) in [
        (
            "fixtures/evidence-relevance-successor-v10-development-v2/manifest.json",
            "sv10v2_15_context_gap_observation",
        ),
        (
            "fixtures/evidence-relevance-successor-v10-development-v3/manifest.json",
            "sv10v3_19_context_gap",
        ),
        (
            "fixtures/evidence-relevance-successor-v10-development-v4/manifest.json",
            "sv10v4_22_context_gap_definition",
        ),
    ] {
        let cases = case_map(manifest_path);
        let case = cases.get(case_id).expect("context-gap case");
        let effective = derive_effective_evidence_local_qualification_v17(
            &case.policy,
            &case.candidate,
            Some(&case.expected_proposal),
            Some(&case.expected_local_qualification),
        )
        .expect("v17 expected replay");

        assert_eq!(
            effective.identity_scope,
            EvidenceLocalIdentityScope::ExactTarget,
            "ordinary relation/context clipping lost visible target identity: {case_id}"
        );
        let assessment = materialize_evidence_relevance_v30(
            &case.policy,
            &case.candidate,
            Some(&case.expected_proposal),
            Some(&case.expected_local_qualification),
        )
        .expect("v30 expected replay");
        assert_eq!(assessment.disposition, case.expected_disposition);
    }
}

fn assert_v30_observations_match_manifest(manifest_path: &str, result_path: &str) {
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
        let successor = materialize_evidence_relevance_v30(
            &case.policy,
            &case.candidate,
            Some(proposal),
            Some(raw),
        )
        .expect("v30 replay")
        .disposition;
        assert_eq!(
            successor, case.expected_disposition,
            "v30 historical terminal miss: {result_path}:{}",
            observation.id
        );
    }
}

#[test]
fn v30_replays_recent_immutable_provider_observations_to_frozen_terminal_contracts() {
    for result in [
        "fixtures/evidence-relevance-holdout-v10-result/mistral-run-37085574010.json",
        "fixtures/evidence-relevance-holdout-v10-result/google-run-37085574010.json",
        "fixtures/evidence-relevance-holdout-v10-result/groq-run-37085574010.json",
    ] {
        assert_v30_observations_match_manifest(
            "fixtures/evidence-relevance-holdout-v10/manifest.json",
            result,
        );
    }

    for result in [
        "fixtures/evidence-relevance-successor-v10-development-v2-result/mistral-run-37100490212.json",
        "fixtures/evidence-relevance-successor-v10-development-v2-result/google-run-37100490212.json",
    ] {
        assert_v30_observations_match_manifest(
            "fixtures/evidence-relevance-successor-v10-development-v2/manifest.json",
            result,
        );
    }

    for result in [
        "fixtures/evidence-relevance-successor-v10-development-v3-result/mistral-run-37105677785.json",
        "fixtures/evidence-relevance-successor-v10-development-v3-result/google-partial-checkpoint-run-37105677785.json",
    ] {
        assert_v30_observations_match_manifest(
            "fixtures/evidence-relevance-successor-v10-development-v3/manifest.json",
            result,
        );
    }

    for result in [
        "fixtures/evidence-relevance-successor-v10-development-v4-result/mistral-run-37111376390.json",
        "fixtures/evidence-relevance-successor-v10-development-v4-result/google-run-37111376390.json",
    ] {
        assert_v30_observations_match_manifest(
            "fixtures/evidence-relevance-successor-v10-development-v4/manifest.json",
            result,
        );
    }

    for result in [
        "fixtures/evidence-relevance-holdout-v11-result/mistral-run-37182677114.json",
        "fixtures/evidence-relevance-holdout-v11-result/google-run-37182677114.json",
        "fixtures/evidence-relevance-holdout-v11-result/groq-run-37182677114.json",
    ] {
        assert_v30_observations_match_manifest(
            "fixtures/evidence-relevance-holdout-v11/manifest.json",
            result,
        );
    }

    for result in [
        "fixtures/evidence-relevance-successor-v11-development-v1-result/mistral-run-37212349800.json",
        "fixtures/evidence-relevance-successor-v11-development-v1-result/google-run-37212349800.json",
    ] {
        assert_v30_observations_match_manifest(
            "fixtures/evidence-relevance-successor-v11-development/manifest.json",
            result,
        );
    }
}
