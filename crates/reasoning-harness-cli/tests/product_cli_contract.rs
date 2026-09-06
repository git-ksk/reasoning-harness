use std::{
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

use serde_json::Value;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root")
}

fn reason_command() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_reason"));
    command.current_dir(workspace_root());
    command
}

fn run_reason(args: &[&str], stdin: Option<&[u8]>) -> Output {
    let mut command = reason_command();
    command
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if stdin.is_some() {
        command.stdin(Stdio::piped());
    }
    let mut child = command.spawn().expect("spawn reason");
    if let Some(bytes) = stdin {
        child
            .stdin
            .as_mut()
            .expect("stdin pipe")
            .write_all(bytes)
            .expect("write stdin");
    }
    child.wait_with_output().expect("wait for reason")
}

fn json_stdout(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "stdout must be JSON: {error}; stdout={}; stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

#[test]
fn unknown_is_a_successful_versioned_product_outcome() {
    let output = run_reason(
        &[
            "run",
            "--input",
            "examples/input.json",
            "--candidate",
            "examples/candidate.json",
            "--no-config",
            "--format",
            "json",
        ],
        None,
    );
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());

    let value = json_stdout(&output);
    assert_eq!(value["schema_version"], "reason-cli-output-v1");
    assert_eq!(value["command"], "run");
    assert_eq!(value["contracts"]["artifact"], "reasoning-artifact-v1");
    assert_eq!(value["contracts"]["candidate"], "reasoning-candidate-v1");
    assert_eq!(value["contracts"]["config"], "reason-config-v1");
    assert_eq!(value["result"]["outcome"]["verdict"], "unknown");
}

#[test]
fn supported_stdin_paths_preserve_the_same_machine_contract() {
    let input = std::fs::read(workspace_root().join("examples/input.json")).expect("input fixture");
    let run = run_reason(
        &[
            "run",
            "--input",
            "-",
            "--candidate",
            "examples/candidate.json",
            "--no-config",
            "--format",
            "json",
        ],
        Some(&input),
    );
    assert_eq!(run.status.code(), Some(0));
    assert_eq!(json_stdout(&run)["command"], "run");

    let artifact =
        std::fs::read(workspace_root().join("examples/artifact.json")).expect("artifact fixture");
    let verify = run_reason(&["verify", "-", "--format", "json"], Some(&artifact));
    assert_eq!(verify.status.code(), Some(0));
    let value = json_stdout(&verify);
    assert_eq!(value["schema_version"], "reason-cli-output-v1");
    assert_eq!(value["command"], "verify");
    assert_eq!(value["result"]["valid"], true);
}

#[test]
fn json_operational_failure_is_exit_one_and_stays_machine_readable() {
    let output = run_reason(
        &[
            "run",
            "--input",
            "-",
            "--candidate",
            "examples/candidate.json",
            "--no-config",
            "--format",
            "json",
        ],
        Some(b"{}"),
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());

    let value = json_stdout(&output);
    assert_eq!(value["schema_version"], "reason-cli-output-v1");
    assert_eq!(value["command"], "run");
    assert_eq!(value["result"]["status"], "failed");
    assert_eq!(value["result"]["failure"]["failure_class"], "input");
}

#[test]
fn cli_usage_error_is_exit_two_and_not_an_epistemic_outcome() {
    let output = run_reason(&["run", "--not-a-real-option"], None);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unexpected argument") || stderr.contains("Usage:"));
}

#[test]
fn schema_command_exposes_all_compatibility_tracked_contract_ids() {
    for (kind, expected) in [
        ("artifact", "reasoning-artifact-v1"),
        ("candidate", "reasoning-candidate-v1"),
        ("config", "reason-config-v1"),
        ("semantic-check", "semantic-check-input-v1"),
    ] {
        let output = run_reason(&["schema", kind], None);
        assert_eq!(output.status.code(), Some(0), "schema {kind}");
        assert!(output.stderr.is_empty(), "schema {kind}");
        let value = json_stdout(&output);
        assert_eq!(value["schema_version"], "reason-cli-output-v1");
        assert_eq!(value["command"], "schema");
        assert_eq!(value["result"]["contract_id"], expected);
        assert!(value["result"]["schema"].is_object());
    }
}

fn synthetic_session_file(path: &Path) {
    use reasoning_harness_core::{
        AnswerSafetyProfile, FinalizationResult, FinalizationStatus, ReasoningArtifact,
        ReasoningCandidate, ReasoningThread, Verdict,
    };

    let mut thread = ReasoningThread::new("process-session-root").unwrap();
    thread
        .record_task("process-task-event", "process-task", "process session task")
        .unwrap();
    thread
        .record_candidate(
            "process-candidate-event",
            "process-candidate",
            None,
            ReasoningCandidate::default(),
        )
        .unwrap();
    thread
        .record_accepted_artifact(
            "process-artifact-event",
            ReasoningArtifact {
                task: "process session task".into(),
                ..Default::default()
            },
            Verdict::Unknown,
        )
        .unwrap();
    let checkpoint = thread
        .create_checkpoint("process-checkpoint-event", "process-checkpoint-1")
        .unwrap();
    thread
        .interrupt("process-interrupt-event", checkpoint.checkpoint_id.clone())
        .unwrap();

    let finalization = FinalizationResult {
        status: FinalizationStatus::Unresolved,
        text: None,
        factual_claims: 0,
        covered_claims: 0,
        factual_claim_coverage: 1.0,
        uncovered_propositions: vec![],
    };
    let safety = AnswerSafetyProfile::VerifiedTargetV1.identity();
    let value = serde_json::json!({
        "schema_version": "reason-session-v1",
        "runtime": {
            "natural_output_contract": "reason-natural-output-v3",
            "exposed_text_policy_id": "harness-canonical-exposed-text-v1",
            "reasoning_thread_schema_version": 1,
            "provider": "mistral",
            "model": "test-model",
            "max_tokens": 64,
            "safety_profile": "current",
            "safety_configuration_id": safety.configuration_id(),
            "continuation_policy_id": "session-replay-only-acquisition-v1",
            "config_sources": []
        },
        "thread": thread,
        "turns": [{
            "turn_index": 1,
            "checkpoint_id": "process-checkpoint-1",
            "finalization": finalization
        }]
    });
    std::fs::write(path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
}

#[test]
fn session_add_with_empty_nonterminal_stdin_is_not_rejected_as_piped_input() {
    let temp = std::env::temp_dir().join(format!(
        "reason-session-empty-stdin-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&temp).unwrap();
    let source = temp.join("source.json");
    synthetic_session_file(&source);

    let mut command = reason_command();
    command
        .args([
            "session",
            "add",
            "--store",
            source.to_str().unwrap(),
            "--fact",
            "feature.enabled=true",
            "--format",
            "json",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env_remove("MISTRAL_API_KEY");
    let output = command.output().expect("run session add");
    assert_eq!(output.status.code(), Some(1));
    let json = json_stdout(&output);
    assert_ne!(json["result"]["failure_class"], "input");
    assert!(!String::from_utf8_lossy(&output.stderr).contains("piped stdin"));

    std::fs::remove_dir_all(temp).ok();
}

#[test]
fn session_process_resume_and_fork_replay_no_external_side_effects() {
    let temp = std::env::temp_dir().join(format!(
        "reason-session-process-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&temp).unwrap();
    let source = temp.join("source.json");
    let forked = temp.join("forked.json");
    synthetic_session_file(&source);

    let source_arg = source.to_str().unwrap();
    let inspect = run_reason(
        &[
            "session", "inspect", "--store", source_arg, "--format", "json",
        ],
        None,
    );
    assert_eq!(inspect.status.code(), Some(0));
    let inspect_json = json_stdout(&inspect);
    assert_eq!(inspect_json["command"], "session");
    assert_eq!(
        inspect_json["result"]["session_contract"],
        "reason-session-v1"
    );
    assert_eq!(inspect_json["result"]["status"], "interrupted");
    assert_eq!(inspect_json["result"]["external_calls_replayed"], 0);

    let resume = run_reason(
        &[
            "session", "resume", "--store", source_arg, "--format", "json",
        ],
        None,
    );
    assert_eq!(resume.status.code(), Some(0));
    let resume_json = json_stdout(&resume);
    assert_eq!(resume_json["result"]["status"], "active");
    assert_eq!(resume_json["result"]["external_calls_replayed"], 0);

    let inspect_after = run_reason(
        &[
            "session", "inspect", "--store", source_arg, "--format", "json",
        ],
        None,
    );
    assert_eq!(inspect_after.status.code(), Some(0));
    assert_eq!(json_stdout(&inspect_after)["result"]["status"], "active");

    let before_fork = std::fs::read(&source).unwrap();
    let fork_arg = forked.to_str().unwrap();
    let fork = run_reason(
        &[
            "session",
            "fork",
            "--store",
            source_arg,
            "--checkpoint",
            "process-checkpoint-1",
            "--out",
            fork_arg,
            "--new-id",
            "process-session-fork",
            "--format",
            "json",
        ],
        None,
    );
    assert_eq!(fork.status.code(), Some(0));
    let fork_json = json_stdout(&fork);
    assert_eq!(fork_json["result"]["thread_id"], "process-session-fork");
    assert_eq!(
        fork_json["result"]["root_thread_id"],
        "process-session-root"
    );
    assert_eq!(
        fork_json["result"]["parent_thread_id"],
        "process-session-root"
    );
    assert_eq!(fork_json["result"]["external_calls_replayed"], 0);
    assert_eq!(std::fs::read(&source).unwrap(), before_fork);

    let fork_inspect = run_reason(
        &[
            "session", "inspect", "--store", fork_arg, "--format", "json",
        ],
        None,
    );
    assert_eq!(fork_inspect.status.code(), Some(0));
    let fork_inspect_json = json_stdout(&fork_inspect);
    assert_eq!(fork_inspect_json["result"]["status"], "active");
    assert_eq!(fork_inspect_json["result"]["external_calls_replayed"], 0);

    std::fs::remove_dir_all(temp).ok();
}
