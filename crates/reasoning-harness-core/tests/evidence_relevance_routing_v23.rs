use std::{fs, path::PathBuf};

use reasoning_harness_core::{
    EvidenceLocalQualificationV6, EvidenceRelevanceBindingProposal, EvidenceRelevanceCandidate,
    EvidenceRelevanceDisposition, EvidenceRelevanceTargetPolicy,
    derive_effective_evidence_local_qualification_v3, materialize_evidence_relevance_v16,
};
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
struct Manifest {
    suite_id: String,
    annotation_protocol_id: String,
    fixed_core_id: String,
    cases: Vec<Case>,
}

#[derive(Debug, Deserialize)]
struct Case {
    id: String,
    policy: EvidenceRelevanceTargetPolicy,
    candidate: EvidenceRelevanceCandidate,
    expected_proposal: EvidenceRelevanceBindingProposal,
    expected_local_qualification: EvidenceLocalQualificationV6,
    expected_disposition: EvidenceRelevanceDisposition,
}

#[derive(Debug, Deserialize)]
struct V22ReplayFixture {
    source: String,
    candidate_commit: String,
    providers: Vec<V22ProviderReplay>,
}

#[derive(Debug, Deserialize)]
struct V22ProviderReplay {
    provider: String,
    model: String,
    successful_case_ids: Vec<String>,
    overrides: Vec<V22ReplayOverride>,
}

#[derive(Debug, Deserialize)]
struct V22ReplayOverride {
    id: String,
    observed_proposal: Option<EvidenceRelevanceBindingProposal>,
    observed_local_qualification: Option<EvidenceLocalQualificationV6>,
}

fn fixture_path(version: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../fixtures/evidence-relevance-calibration-{version}/manifest.json"
    ))
}

fn v22_observation_replay() -> V22ReplayFixture {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/evidence-relevance-calibration-v23/v22-observation-replay.json");
    serde_json::from_slice(&fs::read(path).expect("read immutable v22 replay fixture"))
        .expect("parse immutable v22 replay fixture")
}

fn manifest() -> Manifest {
    serde_json::from_slice(&fs::read(fixture_path("v23")).expect("read v23 manifest"))
        .expect("parse v23 manifest")
}

#[test]
fn v23_keeps_the_frozen_v22_case_surface() {
    let v22: Value =
        serde_json::from_slice(&fs::read(fixture_path("v22")).expect("read frozen v22 manifest"))
            .expect("parse frozen v22 manifest");
    let v23: Value =
        serde_json::from_slice(&fs::read(fixture_path("v23")).expect("read v23 manifest"))
            .expect("parse v23 manifest");

    assert_eq!(v22["cases"], v23["cases"]);
    assert_eq!(v23["cases"].as_array().expect("v23 cases").len(), 48);
}

#[test]
fn v23_uses_effective_qualification_v3_and_fixed_core_v1() {
    let manifest = manifest();
    assert_eq!(manifest.suite_id, "evidence-relevance-calibration-v23");
    assert_eq!(
        manifest.annotation_protocol_id,
        "evidence-relevance-effective-qualification-v23"
    );
    assert_eq!(manifest.fixed_core_id, "evidence-relevance-fixed-core-v1");
    assert_eq!(manifest.cases.len(), 48);
}

#[test]
fn v23_expected_cases_remain_exact_under_v3_semantics() {
    for case in manifest().cases {
        let effective = derive_effective_evidence_local_qualification_v3(
            &case.policy,
            &case.candidate,
            Some(&case.expected_proposal),
            Some(&case.expected_local_qualification),
        )
        .unwrap_or_else(|error| panic!("{} qualification: {error}", case.id));
        assert_eq!(effective, case.expected_local_qualification, "{}", case.id);

        let assessment = materialize_evidence_relevance_v16(
            &case.policy,
            &case.candidate,
            Some(&case.expected_proposal),
            Some(&case.expected_local_qualification),
        )
        .unwrap_or_else(|error| panic!("{} materialization: {error}", case.id));
        assert_eq!(
            assessment.disposition, case.expected_disposition,
            "{}",
            case.id
        );
    }
}

#[test]
fn v23_replays_all_v22_provider_observations_to_required_exactness() {
    let replay = v22_observation_replay();
    assert_eq!(replay.source, "v22_immutable_canonical_run_36338187291");
    assert_eq!(
        replay.candidate_commit,
        "2e0268dc0fb565dab35d591e831aa5170ad75601"
    );

    let cases = manifest()
        .cases
        .into_iter()
        .map(|case| (case.id.clone(), case))
        .collect::<std::collections::BTreeMap<_, _>>();

    for provider in replay.providers {
        match provider.provider.as_str() {
            "mistral" => assert_eq!(provider.model, "ministral-8b-latest"),
            "groq" => assert_eq!(provider.model, "openai/gpt-oss-120b"),
            "google" => assert_eq!(provider.model, "gemini-3.5-flash-lite"),
            other => panic!("unexpected replay provider {other}"),
        }
        assert_eq!(provider.successful_case_ids.len(), 48);

        let overrides = provider
            .overrides
            .into_iter()
            .map(|entry| (entry.id.clone(), entry))
            .collect::<std::collections::BTreeMap<_, _>>();

        let mut authority_exact = 0usize;
        let mut materialized_exact = 0usize;
        for id in provider.successful_case_ids {
            let case = cases
                .get(&id)
                .unwrap_or_else(|| panic!("missing frozen case {id}"));
            let observed = overrides.get(&id);
            let proposal = observed
                .and_then(|entry| entry.observed_proposal.as_ref())
                .unwrap_or(&case.expected_proposal);
            let raw = observed
                .and_then(|entry| entry.observed_local_qualification.as_ref())
                .unwrap_or(&case.expected_local_qualification);

            let effective = derive_effective_evidence_local_qualification_v3(
                &case.policy,
                &case.candidate,
                Some(proposal),
                Some(raw),
            )
            .unwrap_or_else(|error| panic!("{} {id} qualification: {error}", provider.provider));

            assert_eq!(
                effective.scope_risk, case.expected_local_qualification.scope_risk,
                "{} {id} effective risk regressed",
                provider.provider
            );
            if case.expected_local_qualification.scope_risk
                == reasoning_harness_core::EvidenceLocalBlockingReason::None
            {
                assert_eq!(
                    effective.identity_scope, case.expected_local_qualification.identity_scope,
                    "{} {id} authority identity regressed",
                    provider.provider
                );
                assert_eq!(
                    effective.relation_scope, case.expected_local_qualification.relation_scope,
                    "{} {id} authority relation regressed",
                    provider.provider
                );
            }
            authority_exact += 1;

            let assessment = materialize_evidence_relevance_v16(
                &case.policy,
                &case.candidate,
                Some(proposal),
                Some(raw),
            )
            .unwrap_or_else(|error| panic!("{} {id} materialization: {error}", provider.provider));
            assert_eq!(
                assessment.disposition, case.expected_disposition,
                "{} {id} materialization regressed",
                provider.provider
            );
            materialized_exact += 1;
        }

        assert_eq!(authority_exact, 48, "{}", provider.provider);
        assert_eq!(materialized_exact, 48, "{}", provider.provider);
    }
}
