use std::{collections::BTreeSet, fs, path::PathBuf};

use reasoning_harness_core::{
    EvidenceLocalQualificationV6, EvidenceLocalRelationScope, EvidenceRelevanceBindingProposal,
    EvidenceRelevanceCandidate, EvidenceRelevanceDisposition, EvidenceRelevanceTargetPolicy,
    derive_effective_evidence_local_qualification_v11, materialize_evidence_relevance_v23,
};
use serde::Deserialize;
use serde_json::Value;

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
    family: String,
    authority_expectation: String,
    #[serde(rename = "task")]
    _task: String,
    policy: EvidenceRelevanceTargetPolicy,
    candidate: EvidenceRelevanceCandidate,
    expected_proposal: EvidenceRelevanceBindingProposal,
    expected_local_qualification: EvidenceLocalQualificationV6,
    expected_disposition: EvidenceRelevanceDisposition,
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

fn load() -> Manifest {
    serde_json::from_slice(
        &fs::read(
            root().join("fixtures/evidence-relevance-successor-v9-development/manifest.json"),
        )
        .expect("read successor v9 development manifest"),
    )
    .expect("parse successor v9 development manifest")
}

fn normalized_tokens(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(str::to_owned)
        .collect()
}

fn eight_grams(text: &str) -> BTreeSet<Vec<String>> {
    let tokens = normalized_tokens(text);
    tokens
        .windows(8)
        .map(|window| window.to_vec())
        .collect::<BTreeSet<_>>()
}

fn collect_case_strings(case: &Value) -> Vec<String> {
    let mut values = Vec::new();
    for key in ["id", "task"] {
        if let Some(value) = case.get(key).and_then(Value::as_str) {
            values.push(value.to_owned());
        }
    }
    if let Some(question) = case
        .get("policy")
        .and_then(|value| value.get("target_question"))
        .and_then(Value::as_str)
    {
        values.push(question.to_owned());
    }
    if let Some(entity) = case
        .get("policy")
        .and_then(|value| value.get("entity"))
        .and_then(Value::as_object)
    {
        if let Some(name) = entity.get("canonical_name").and_then(Value::as_str) {
            values.push(name.to_owned());
        }
        if let Some(aliases) = entity.get("aliases").and_then(Value::as_array) {
            values.extend(aliases.iter().filter_map(Value::as_str).map(str::to_owned));
        }
    }
    if let Some(signals) = case
        .get("candidate")
        .and_then(|value| value.get("signals"))
        .and_then(Value::as_array)
    {
        values.extend(
            signals
                .iter()
                .filter_map(|signal| signal.get("text"))
                .filter_map(Value::as_str)
                .map(str::to_owned),
        );
    }
    values
}

#[test]
fn successor_v9_development_metadata_and_authority_modes_are_fixed_before_observation() {
    let manifest = load();
    assert_eq!(
        manifest.suite_id,
        "evidence-relevance-successor-v9-development"
    );
    assert_eq!(manifest.issue, 468);
    assert_eq!(manifest.status, "fresh_independent_development");
    assert_eq!(
        manifest.annotation_protocol_id,
        "evidence-relevance-effective-qualification-v11-materialization-v23"
    );
    assert_eq!(
        manifest.fixed_core_id,
        "evidence-relevance-fixed-core-successor-v9-development-v1"
    );
    assert!(
        manifest
            .source_rule
            .contains("before_any_v11_provider_observation")
    );
    assert_eq!(manifest.cases.len(), 19);

    let require_requested = manifest
        .cases
        .iter()
        .filter(|case| case.authority_expectation == "require_requested")
        .count();
    let forbid_requested = manifest
        .cases
        .iter()
        .filter(|case| case.authority_expectation == "forbid_requested")
        .count();
    let preserve_risk = manifest
        .cases
        .iter()
        .filter(|case| case.authority_expectation == "preserve_risk")
        .count();
    assert_eq!(
        (require_requested, forbid_requested, preserve_risk),
        (8, 9, 2)
    );

    let families = manifest
        .cases
        .iter()
        .map(|case| case.family.as_str())
        .collect::<BTreeSet<_>>();
    for required in [
        "availability_frame",
        "pricing_frame",
        "limit_frame",
        "change_frame",
        "definition_frame",
        "benefit_frame",
        "limit_numeric_observation",
        "change_phrasal_non_frame",
        "benefit_documentation_non_frame",
        "availability_feature_support_non_frame",
        "pricing_number_non_frame",
        "availability_general_availability_event",
        "change_general_availability_frame",
        "truncated_relation",
        "prompt_injection_inert",
        "exact_target_lexical_history",
        "identity_mapping_risk",
        "definition_document_non_frame",
        "exact_target_model_only_limit",
    ] {
        assert!(families.contains(required), "missing family {required}");
    }
}

#[test]
fn successor_v9_development_expected_inputs_are_exact_under_v11_v23() {
    for case in load().cases {
        let effective = derive_effective_evidence_local_qualification_v11(
            &case.policy,
            &case.candidate,
            Some(&case.expected_proposal),
            Some(&case.expected_local_qualification),
        )
        .unwrap_or_else(|error| panic!("{} effective: {error}", case.id));
        assert_eq!(
            effective, case.expected_local_qualification,
            "{} effective qualification",
            case.id
        );

        let assessment = materialize_evidence_relevance_v23(
            &case.policy,
            &case.candidate,
            Some(&case.expected_proposal),
            Some(&case.expected_local_qualification),
        )
        .unwrap_or_else(|error| panic!("{} materialization: {error}", case.id));
        assert_eq!(
            assessment.disposition, case.expected_disposition,
            "{} disposition",
            case.id
        );

        match case.authority_expectation.as_str() {
            "require_requested" => assert_eq!(
                effective.relation_scope,
                EvidenceLocalRelationScope::RequestedRelation,
                "{} requested authority",
                case.id
            ),
            "forbid_requested" => assert_ne!(
                effective.relation_scope,
                EvidenceLocalRelationScope::RequestedRelation,
                "{} forbidden requested authority",
                case.id
            ),
            "preserve_risk" => {}
            other => panic!("{} unknown authority expectation {other}", case.id),
        }
    }
}

#[test]
fn successor_v9_development_is_surface_independent_from_prior_holdouts_and_development() {
    let fresh: Value = serde_json::from_slice(
        &fs::read(
            root().join("fixtures/evidence-relevance-successor-v9-development/manifest.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let fresh_cases = fresh["cases"].as_array().unwrap();

    let mut prior_ids = BTreeSet::new();
    let mut prior_tasks = BTreeSet::new();
    let mut prior_entities = BTreeSet::new();
    let mut prior_grams = BTreeSet::new();

    for entry in fs::read_dir(root().join("fixtures")).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path().join("manifest.json");
        if !path.is_file()
            || path
                .to_string_lossy()
                .contains("evidence-relevance-successor-v9-development")
        {
            continue;
        }
        let text = path.to_string_lossy();
        if !text.contains("evidence-relevance-holdout")
            && !text.contains("evidence-relevance-successor")
        {
            continue;
        }
        let value: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        let Some(cases) = value.get("cases").and_then(Value::as_array) else {
            continue;
        };
        for case in cases {
            if let Some(id) = case.get("id").and_then(Value::as_str) {
                prior_ids.insert(id.to_owned());
            }
            if let Some(task) = case.get("task").and_then(Value::as_str) {
                prior_tasks.insert(task.to_owned());
            }
            if let Some(entity) = case
                .get("policy")
                .and_then(|value| value.get("entity"))
                .and_then(|value| value.get("canonical_name"))
                .and_then(Value::as_str)
            {
                prior_entities.insert(entity.to_owned());
            }
            for string in collect_case_strings(case) {
                prior_grams.extend(eight_grams(&string));
            }
        }
    }

    for case in fresh_cases {
        let id = case["id"].as_str().unwrap();
        let task = case["task"].as_str().unwrap();
        let entity = case["policy"]["entity"]["canonical_name"].as_str().unwrap();
        assert!(!prior_ids.contains(id), "reused id {id}");
        assert!(!prior_tasks.contains(task), "reused task {id}");
        assert!(
            !prior_entities.contains(entity),
            "reused entity {id}: {entity}"
        );
        for string in collect_case_strings(case) {
            for gram in eight_grams(&string) {
                assert!(
                    !prior_grams.contains(&gram),
                    "reused 8-token surface in {id}: {}",
                    gram.join(" ")
                );
            }
        }
    }
}
