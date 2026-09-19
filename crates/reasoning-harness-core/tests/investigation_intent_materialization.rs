use reasoning_harness_core::{
    InvestigationActionKind, InvestigationActionProposal, InvestigationCapability,
    InvestigationIntentKind, InvestigationIntentProposal, InvestigationIntentRejection,
    InvestigationObservationStatus, InvestigationPolicy, InvestigationState,
    InvestigationStopReason, InvestigationTarget, InvestigationTargetOrigin, ModelOutputFormat,
    build_investigation_intent_request,
};

fn target(id: &str, key: Option<&str>) -> InvestigationTarget {
    InvestigationTarget {
        id: id.to_string(),
        question: format!("question for {id}"),
        expected_fact_key: key.map(str::to_string),
        origin: InvestigationTargetOrigin::ModelProposedUntrusted,
    }
}

fn capability(id: &str, keys: &[&str]) -> InvestigationCapability {
    InvestigationCapability {
        id: id.to_string(),
        adapter: "fixture_readonly".into(),
        read_only: true,
        supported_fact_keys: keys.iter().map(|key| (*key).to_string()).collect(),
        selection_priority: None,
    }
}

fn priority_capability(id: &str, keys: &[&str], priority: u32) -> InvestigationCapability {
    let mut capability = capability(id, keys);
    capability.selection_priority = Some(priority);
    capability
}

#[test]
fn intent_schema_bounds_targets_and_never_exposes_capability_identity() {
    let state = InvestigationState::new(
        vec![
            target("owner-a", Some("routing.owner")),
            target("owner-b", Some("routing.owner")),
        ],
        vec![
            priority_capability("cache", &["routing.owner"], 20),
            priority_capability("registry", &["routing.owner"], 10),
        ],
        InvestigationPolicy::default(),
    )
    .unwrap();

    assert_eq!(
        state.materializable_target_ids(),
        vec!["owner-a".to_string(), "owner-b".to_string()]
    );
    let request = build_investigation_intent_request(
        "find owner",
        state.telemetry(),
        &state.materializable_target_ids(),
        Some(128),
        Some(7),
    )
    .unwrap();
    let schema = match request.output_format {
        ModelOutputFormat::JsonSchema { schema, .. } => schema,
        _ => panic!("expected JSON schema"),
    };
    let encoded = schema.to_string();
    assert!(encoded.contains("owner-a"));
    assert!(encoded.contains("owner-b"));
    assert!(!encoded.contains("capability_id"));
    assert!(!request.task.contains("selection_priority"));
    assert!(!request.task.contains("\"cache\""));
    assert!(!request.task.contains("\"registry\""));
}

#[test]
fn same_key_siblings_remain_distinct_and_harness_owns_capability_id() {
    let mut state = InvestigationState::new(
        vec![
            target("owner-a", Some("routing.owner")),
            target("owner-b", Some("routing.owner")),
        ],
        vec![
            priority_capability("cache", &["routing.owner"], 20),
            priority_capability("registry", &["routing.owner"], 10),
        ],
        InvestigationPolicy::default(),
    )
    .unwrap();

    assert_eq!(state.unique_precedence_action_proposal(), None);
    let action = state
        .materialize_intent(
            1,
            InvestigationIntentProposal {
                action: InvestigationIntentKind::Continue,
                target_id: Some("owner-b".into()),
            },
        )
        .unwrap();
    assert_eq!(action.target_id.as_deref(), Some("owner-b"));
    assert_eq!(action.capability_id.as_deref(), Some("cache"));
    assert_eq!(state.telemetry().harness_intent_materializations, 1);
}

#[test]
fn missing_priority_tie_and_write_only_are_not_materializable() {
    let missing = InvestigationState::new(
        vec![target("owner", Some("routing.owner"))],
        vec![
            capability("cache", &["routing.owner"]),
            capability("registry", &["routing.owner"]),
        ],
        InvestigationPolicy::default(),
    )
    .unwrap();
    assert!(missing.materializable_target_ids().is_empty());

    let tied = InvestigationState::new(
        vec![target("owner", Some("routing.owner"))],
        vec![
            priority_capability("cache", &["routing.owner"], 20),
            priority_capability("registry", &["routing.owner"], 20),
        ],
        InvestigationPolicy::default(),
    )
    .unwrap();
    assert!(tied.materializable_target_ids().is_empty());

    let mut write = capability("writer", &["routing.owner"]);
    write.read_only = false;
    let write_only = InvestigationState::new(
        vec![target("owner", Some("routing.owner"))],
        vec![write],
        InvestigationPolicy::default(),
    )
    .unwrap();
    assert!(write_only.materializable_target_ids().is_empty());
}

#[test]
fn attempted_pair_reduces_to_one_exact_capability_and_terminal_budget_blocks_it() {
    let mut state = InvestigationState::new(
        vec![target("owner", Some("routing.owner"))],
        vec![
            capability("cache", &["routing.owner"]),
            capability("registry", &["routing.owner"]),
        ],
        InvestigationPolicy::default(),
    )
    .unwrap();

    assert!(state.materializable_target_ids().is_empty());
    let round = state.begin_round().unwrap();
    let action = state
        .validate_action(InvestigationActionProposal {
            action: InvestigationActionKind::Acquire,
            target_id: Some("owner".into()),
            capability_id: Some("cache".into()),
        })
        .unwrap()
        .unwrap();
    state.record_observation(
        round,
        action,
        InvestigationObservationStatus::Ambiguous,
        0,
        false,
    );

    assert_eq!(state.materializable_target_ids(), vec!["owner".to_string()]);
    let next = state
        .materialize_intent(
            round,
            InvestigationIntentProposal {
                action: InvestigationIntentKind::Continue,
                target_id: Some("owner".into()),
            },
        )
        .unwrap();
    assert_eq!(next.capability_id.as_deref(), Some("registry"));

    state.stop(InvestigationStopReason::ActionBudget);
    assert!(state.materializable_target_ids().is_empty());
}

#[test]
fn intent_refusal_is_typed_and_never_creates_an_action() {
    let mut state = InvestigationState::new(
        vec![target("owner", Some("routing.owner"))],
        vec![capability("cache", &["routing.owner"])],
        InvestigationPolicy::default(),
    )
    .unwrap();

    let result = state.materialize_intent(
        1,
        InvestigationIntentProposal {
            action: InvestigationIntentKind::Continue,
            target_id: Some("missing".into()),
        },
    );
    assert_eq!(result, Err(InvestigationIntentRejection::UnknownTarget));
    assert_eq!(
        state.telemetry().intent_rejections[&InvestigationIntentRejection::UnknownTarget],
        1
    );
    assert!(state.telemetry().actions.is_empty());
    assert_eq!(state.telemetry().harness_intent_materializations, 0);
}

#[test]
fn no_result_followup_remains_existing_exact_target_fast_path() {
    let mut state = InvestigationState::new(
        vec![
            target("owner-a", Some("routing.owner")),
            target("owner-b", Some("routing.owner")),
        ],
        vec![
            capability("cache", &["routing.owner"]),
            capability("registry", &["routing.owner"]),
        ],
        InvestigationPolicy::default(),
    )
    .unwrap();

    let round = state.begin_round().unwrap();
    let first = state
        .validate_action(InvestigationActionProposal {
            action: InvestigationActionKind::Acquire,
            target_id: Some("owner-b".into()),
            capability_id: Some("cache".into()),
        })
        .unwrap()
        .unwrap();
    state.record_observation(
        round,
        first,
        InvestigationObservationStatus::NoResult,
        0,
        false,
    );

    let followup = state.unique_no_result_followup_proposal().unwrap();
    assert_eq!(followup.target_id.as_deref(), Some("owner-b"));
    assert_eq!(followup.capability_id.as_deref(), Some("registry"));
}

#[test]
fn one_exact_untried_capability_materializes_without_priority() {
    let mut state = InvestigationState::new(
        vec![target("owner", Some("routing.owner"))],
        vec![capability("registry", &["routing.owner"])],
        InvestigationPolicy::default(),
    )
    .unwrap();

    assert_eq!(state.materializable_target_ids(), vec!["owner".to_string()]);
    let action = state
        .materialize_intent(
            1,
            InvestigationIntentProposal {
                action: InvestigationIntentKind::Continue,
                target_id: Some("owner".into()),
            },
        )
        .unwrap();
    assert_eq!(action.capability_id.as_deref(), Some("registry"));
    assert_eq!(state.telemetry().harness_intent_materializations, 1);
}

#[test]
fn keyless_and_wildcard_compatibility_never_materialize() {
    let keyless = InvestigationState::new(
        vec![target("owner", None)],
        vec![capability("registry", &["routing.owner"])],
        InvestigationPolicy::default(),
    )
    .unwrap();
    assert!(keyless.materializable_target_ids().is_empty());

    let wildcard = InvestigationState::new(
        vec![target("owner", Some("routing.owner"))],
        vec![capability("registry", &[])],
        InvestigationPolicy::default(),
    )
    .unwrap();
    assert!(wildcard.materializable_target_ids().is_empty());
}

#[test]
fn stop_intent_is_shape_checked_and_never_counts_as_materialization() {
    let mut state = InvestigationState::new(
        vec![target("owner", Some("routing.owner"))],
        vec![capability("registry", &["routing.owner"])],
        InvestigationPolicy::default(),
    )
    .unwrap();

    let invalid = state.materialize_intent(
        1,
        InvestigationIntentProposal {
            action: InvestigationIntentKind::Stop,
            target_id: Some("owner".into()),
        },
    );
    assert_eq!(invalid, Err(InvestigationIntentRejection::InvalidShape));

    let stop = state
        .materialize_intent(
            1,
            InvestigationIntentProposal {
                action: InvestigationIntentKind::Stop,
                target_id: None,
            },
        )
        .unwrap();
    assert_eq!(stop.action, InvestigationActionKind::Stop);
    assert_eq!(state.telemetry().harness_intent_materializations, 0);
}

#[test]
fn intent_and_legacy_action_model_calls_remain_separate_telemetry_axes() {
    let mut state = InvestigationState::new(
        vec![target("owner", Some("routing.owner"))],
        vec![capability("registry", &["routing.owner"])],
        InvestigationPolicy::default(),
    )
    .unwrap();

    state.note_planner_intent_call();
    assert_eq!(state.telemetry().planner_intent_calls, 1);
    assert_eq!(state.telemetry().planner_calls, 0);

    state.note_planner_call();
    assert_eq!(state.telemetry().planner_intent_calls, 1);
    assert_eq!(state.telemetry().planner_calls, 1);
}

#[test]
fn telemetry_wire_keeps_legacy_fields_and_adds_intent_identity_only() {
    let state = InvestigationState::new(
        vec![target("owner", Some("routing.owner"))],
        vec![capability("registry", &["routing.owner"])],
        InvestigationPolicy::default(),
    )
    .unwrap();
    let value = serde_json::to_value(state.telemetry()).unwrap();

    assert_eq!(value["runtime_id"], "bounded-investigation-v1");
    assert_eq!(value["plan_contract"], "reason-investigation-plan-v1");
    assert_eq!(value["action_contract"], "reason-investigation-action-v1");
    assert_eq!(value["intent_contract"], "reason-investigation-intent-v1");
    assert_eq!(
        value["materialization_policy"],
        "target-intent-materialization-v1"
    );
    assert_eq!(value["planner_calls"], 0);
    assert_eq!(value["planner_intent_calls"], 0);
    assert_eq!(value["harness_intent_materializations"], 0);
}

#[test]
fn legacy_action_contract_identity_remains_v1() {
    assert_eq!(
        reasoning_harness_core::INVESTIGATION_ACTION_CONTRACT_ID,
        "reason-investigation-action-v1"
    );
    assert_eq!(
        reasoning_harness_core::INVESTIGATION_INTENT_CONTRACT_ID,
        "reason-investigation-intent-v1"
    );
    assert_eq!(
        reasoning_harness_core::INVESTIGATION_MATERIALIZATION_POLICY_ID,
        "target-intent-materialization-v1"
    );
}
