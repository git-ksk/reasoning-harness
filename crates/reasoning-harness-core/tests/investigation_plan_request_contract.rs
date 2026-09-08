use std::collections::BTreeSet;

use reasoning_harness_core::{
    InvestigationCapability, ModelOutputFormat, build_investigation_plan_request,
    investigation_action_schema, investigation_plan_schema, parse_investigation_plan,
};
use serde::Serialize;

#[derive(Serialize)]
struct LegacyVisibleCapability<'a> {
    id: &'a str,
    adapter: &'a str,
    read_only: bool,
    supported_fact_keys: &'a BTreeSet<String>,
}

fn capability(id: &str, keys: &[&str]) -> InvestigationCapability {
    InvestigationCapability {
        id: id.into(),
        adapter: "fixture_readonly".into(),
        read_only: true,
        selection_priority: None,
        supported_fact_keys: keys.iter().map(|key| (*key).to_string()).collect(),
    }
}

fn legacy_visible_capabilities(capabilities: &[InvestigationCapability]) -> String {
    let visible = capabilities
        .iter()
        .map(|capability| LegacyVisibleCapability {
            id: &capability.id,
            adapter: &capability.adapter,
            read_only: capability.read_only,
            supported_fact_keys: &capability.supported_fact_keys,
        })
        .collect::<Vec<_>>();
    serde_json::to_string_pretty(&visible).unwrap()
}

fn assert_legacy_request(task: &str, capabilities: &[InvestigationCapability]) {
    let request = build_investigation_plan_request(task, capabilities, Some(128), Some(7)).unwrap();
    let visible = legacy_visible_capabilities(capabilities);
    let expected_system = "You are an untrusted investigation planner inside a verification harness. Return only the requested structured plan. Propose bounded questions to investigate; do not answer them, invent evidence, claim authority, select write capabilities, or decide correctness. expected_fact_key is only a selector hint when the task clearly names the fact family; omit it when uncertain.";
    let expected_task = format!(
        "User task:\n{task}\n\nHarness-configured read-only capability descriptors:\n{visible}\n\nPropose concise investigation targets. Targets are untrusted planning objects, not hypotheses or verified facts. Do not include tool arguments, evidence, identity claims, authority classes, answers, or verdicts."
    );

    assert_eq!(request.system.as_deref(), Some(expected_system));
    assert_eq!(request.task, expected_task);
    assert_eq!(request.max_tokens, Some(128));
    assert_eq!(request.random_seed, Some(7));
    assert_eq!(request.reasoning_preference, None);
    match request.output_format {
        ModelOutputFormat::JsonSchema { name, schema } => {
            assert_eq!(name, "reason-investigation-plan-v1");
            assert_eq!(schema, investigation_plan_schema());
        }
        other => panic!("unexpected output format: {other:?}"),
    }
}

#[test]
fn single_read_only_capability_is_byte_for_byte_legacy_request() {
    assert_legacy_request(
        "is the rollout gate open?",
        &[capability("gate-read", &["rollout.gate_open"])],
    );
}

#[test]
fn non_shared_capability_families_keep_legacy_request() {
    let cases = vec![
        vec![capability("wildcard", &[])],
        vec![capability("multi", &["service.owner", "service.region"])],
        vec![
            capability("owner-read", &["service.owner"]),
            capability("region-read", &["service.region"]),
        ],
    ];

    for capabilities in cases {
        assert_legacy_request("inspect the service", &capabilities);
    }
}

#[test]
fn strict_shared_exact_family_keeps_mandatory_exact_key_contract() {
    let capabilities = vec![
        capability("cache", &["routing.owner"]),
        capability("registry", &["routing.owner"]),
    ];
    let request =
        build_investigation_plan_request("find the owner", &capabilities, Some(128), Some(7))
            .unwrap();
    let system = request.system.as_deref().unwrap();

    assert!(system.contains("expected_fact_key is required"));
    assert!(system.contains("`routing.owner`"));
    assert!(request.task.contains("expected_fact_key is required"));
    assert!(request.task.contains("`routing.owner`"));
    match request.output_format {
        ModelOutputFormat::JsonSchema { schema, .. } => {
            assert_ne!(schema, investigation_plan_schema());
            let encoded = serde_json::to_string(&schema).unwrap();
            assert!(encoded.contains("routing.owner"));
        }
        other => panic!("unexpected output format: {other:?}"),
    }
}

#[test]
fn selection_priority_is_model_invisible_on_both_request_paths() {
    let plain_non_strict = vec![capability("read", &["service.region"])];
    let mut prioritized_non_strict = plain_non_strict.clone();
    prioritized_non_strict[0].selection_priority = Some(20);

    let plain =
        build_investigation_plan_request("find region", &plain_non_strict, None, Some(9)).unwrap();
    let prioritized =
        build_investigation_plan_request("find region", &prioritized_non_strict, None, Some(9))
            .unwrap();
    assert_eq!(plain.system, prioritized.system);
    assert_eq!(plain.task, prioritized.task);
    assert_eq!(plain.output_format, prioritized.output_format);
    assert!(!prioritized.task.contains("selection_priority"));

    let plain_strict = vec![
        capability("cache", &["routing.owner"]),
        capability("registry", &["routing.owner"]),
    ];
    let mut prioritized_strict = plain_strict.clone();
    prioritized_strict[0].selection_priority = Some(10);
    prioritized_strict[1].selection_priority = Some(20);

    let plain =
        build_investigation_plan_request("find owner", &plain_strict, None, Some(9)).unwrap();
    let prioritized =
        build_investigation_plan_request("find owner", &prioritized_strict, None, Some(9)).unwrap();
    assert_eq!(plain.system, prioritized.system);
    assert_eq!(plain.task, prioritized.task);
    assert_eq!(plain.output_format, prioritized.output_format);
    assert!(!prioritized.task.contains("selection_priority"));
}

#[test]
fn runtime_parser_remains_broad_and_action_schema_is_unchanged_by_planning_path() {
    let parsed = parse_investigation_plan(
        r#"{"targets":[{"id":"region","question":"Which region serves the deployment?"}]}"#,
    )
    .unwrap();
    assert_eq!(parsed.targets[0].expected_fact_key, None);

    let before = investigation_action_schema();
    let _ = build_investigation_plan_request(
        "find owner",
        &[
            capability("cache", &["routing.owner"]),
            capability("registry", &["routing.owner"]),
        ],
        None,
        Some(11),
    )
    .unwrap();
    assert_eq!(investigation_action_schema(), before);
}
