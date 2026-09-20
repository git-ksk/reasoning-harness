use std::{
    io::{BufRead, BufReader, Read, Write},
    net::TcpListener,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    thread,
};

use keyring::{Entry, Error as KeyringError};
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
    assert_eq!(
        value["result"]["remediation"]["task_execution"],
        "not_started"
    );
    assert_eq!(value["result"]["remediation"]["result_trust"], "no_result");
    assert_eq!(
        value["result"]["remediation"]["next_command"],
        "reason help"
    );
}

#[test]
fn generic_groq_empty_environment_override_is_typed_and_does_not_fall_back_to_os_store() {
    let mut command = reason_command();
    let output = command
        .args([
            "investigate this target",
            "--provider",
            "groq",
            "--model",
            "openai/gpt-oss-120b",
            "--no-config",
            "--format",
            "json",
        ])
        .env("GROQ_API_KEY", "")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run generic Groq natural path");

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());
    let value = json_stdout(&output);
    assert_eq!(value["schema_version"], "reason-cli-output-v1");
    assert_eq!(value["command"], "ask");
    assert_eq!(value["result"]["status"], "failed");
    assert_eq!(value["result"]["failure"]["failure_class"], "credentials");
    assert!(
        value["result"]["failure"]["message"]
            .as_str()
            .unwrap_or_default()
            .contains("GROQ_API_KEY")
    );
    assert_eq!(
        value["result"]["remediation"]["what_failed"],
        "provider credential or native credential-store readiness"
    );
    assert_eq!(
        value["result"]["remediation"]["task_execution"],
        "not_started"
    );
    assert_eq!(value["result"]["remediation"]["result_trust"], "no_result");
    assert_eq!(
        value["result"]["remediation"]["next_command"],
        "reason auth status"
    );
}

#[test]
fn human_operational_failure_explains_execution_trust_and_safe_recovery() {
    let output = reason_command()
        .args([
            "investigate this target",
            "--provider",
            "groq",
            "--model",
            "openai/gpt-oss-120b",
            "--no-config",
            "--format",
            "human",
        ])
        .env("GROQ_API_KEY", "")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run human credential failure");
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Error:"));
    assert!(stderr.contains("What failed: provider credential"));
    assert!(stderr.contains("Task execution: the task did not start"));
    assert!(stderr.contains("Result trust: no result was produced"));
    assert!(stderr.contains("Next: reason auth status"));
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
fn plain_presentation_never_changes_machine_json_contract() {
    let normal = run_reason(&["schema", "artifact"], None);
    let plain = run_reason(&["schema", "artifact", "--plain"], None);

    assert_eq!(normal.status.code(), Some(0));
    assert_eq!(plain.status.code(), Some(0));
    assert!(normal.stderr.is_empty());
    assert!(plain.stderr.is_empty());
    assert_eq!(plain.stdout, normal.stdout);
    assert_eq!(json_stdout(&plain)["command"], "schema");
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
            "natural_output_contract": "reason-natural-output-v4",
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
        .env("MISTRAL_API_KEY", "");
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

#[test]
fn auth_noninteractive_stdin_requires_explicit_provider_before_store_access() {
    let sentinel = b"reason-auth-contract-secret-must-not-leak\n";
    let output = run_reason(
        &["auth", "login", "--stdin", "--format", "json"],
        Some(sentinel),
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());
    let body = String::from_utf8_lossy(&output.stdout);
    assert!(!body.contains("reason-auth-contract-secret-must-not-leak"));
    let value = json_stdout(&output);
    assert_eq!(value["command"], "auth");
    assert_eq!(value["result"]["status"], "failed");
    assert_eq!(value["result"]["failure"]["failure_class"], "auth_input");
    assert!(
        value["result"]["failure"]["message"]
            .as_str()
            .unwrap_or_default()
            .contains("provider is required")
    );
}

#[test]
fn auth_login_help_has_only_non_secret_argv_controls() {
    let output = run_reason(&["auth", "login", "--help"], None);
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let help = String::from_utf8_lossy(&output.stdout);
    for expected in ["--stdin", "--from-env", "--replace", "--format"] {
        assert!(help.contains(expected), "missing {expected}: {help}");
    }
    for forbidden in ["--api-key", "--secret", "--password", "--token"] {
        assert!(
            !help.contains(forbidden),
            "unexpected secret-valued flag {forbidden}: {help}"
        );
    }
}

#[test]
fn model_catalog_and_default_switch_preserve_user_config_and_fail_closed() {
    let temp = std::env::temp_dir().join(format!(
        "reason-model-config-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&temp).unwrap();
    let config_path = temp.join("config.json");
    std::fs::write(
        &config_path,
        r#"{
  "schema_version": "reason-config-v1",
  "run": {"max_tokens": 777, "format": "json"},
  "resolution": {
    "trusted_command": {
      "trusted": true,
      "verifier_id": "local-check",
      "program": "/usr/bin/true"
    }
  }
}
"#,
    )
    .unwrap();

    let mut set = reason_command();
    let output = set
        .args([
            "model",
            "set",
            "mistral",
            "ministral-8b-latest",
            "--format",
            "json",
        ])
        .env("REASON_HOME", &temp)
        .env("MISTRAL_API_KEY", "model-catalog-test-secret")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("set model default");
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&temp).unwrap().permissions().mode() & 0o777,
            0o700
        );
        assert_eq!(
            std::fs::metadata(&config_path)
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
    assert!(!String::from_utf8_lossy(&output.stdout).contains("model-catalog-test-secret"));
    let value = json_stdout(&output);
    assert_eq!(value["command"], "model");
    assert_eq!(value["result"]["provider"], "mistral");
    assert_eq!(value["result"]["model"], "ministral-8b-latest");
    assert_eq!(value["result"]["compatibility"], "validated");
    assert_eq!(value["result"]["availability"], "current");
    assert_eq!(value["result"]["credential_status"], "available");

    let config: Value = serde_json::from_slice(&std::fs::read(&config_path).unwrap()).unwrap();
    assert_eq!(config["run"]["provider"], "mistral");
    assert_eq!(config["run"]["model"], "ministral-8b-latest");
    assert_eq!(config["run"]["max_tokens"], 777);
    assert_eq!(config["run"]["format"], "json");
    assert_eq!(config["resolution"]["trusted_command"]["trusted"], true);
    assert_eq!(
        config["resolution"]["trusted_command"]["verifier_id"],
        "local-check"
    );
    assert_eq!(
        config["resolution"]["trusted_command"]["program"],
        "/usr/bin/true"
    );
    assert!(config["resolution"].get("external_command").is_none());

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&config_path)
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }

    let mut configured = reason_command();
    let output = configured
        .args(["models", "--configured", "--format", "json"])
        .env("REASON_HOME", &temp)
        .env("MISTRAL_API_KEY", "model-catalog-test-secret")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("inspect configured model");
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    assert!(!String::from_utf8_lossy(&output.stdout).contains("model-catalog-test-secret"));
    let value = json_stdout(&output);
    assert_eq!(value["command"], "models");
    assert_eq!(
        value["result"]["catalog_version"],
        "reason-model-catalog-v1"
    );
    assert_eq!(
        value["result"]["configured_default"]["model"],
        "ministral-8b-latest"
    );
    assert_eq!(
        value["result"]["configured_default"]["credential_status"],
        "available"
    );
    assert_eq!(
        value["result"]["configured_default"]["availability"],
        "current"
    );

    for (provider, model, failure_class) in [
        (
            "nvidia",
            "nvidia/nemotron-3.5-lightning-30b-a3b",
            "model_known_incompatible",
        ),
        ("mistral", "made-up-model", "model_unlisted"),
    ] {
        let mut command = reason_command();
        let output = command
            .args(["model", "set", provider, model, "--format", "json"])
            .env("REASON_HOME", &temp)
            .env("MISTRAL_API_KEY", "model-catalog-test-secret")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("reject unsupported default");
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stderr.is_empty());
        let value = json_stdout(&output);
        assert_eq!(value["result"]["status"], "failed");
        assert_eq!(value["result"]["failure"]["failure_class"], failure_class);
        if failure_class == "model_known_incompatible" {
            assert_eq!(
                value["result"]["remediation"]["next_command"],
                "reason models"
            );
            assert!(
                value["result"]["failure"]["message"]
                    .as_str()
                    .unwrap_or_default()
                    .contains("will not silently substitute")
            );
        }
    }

    let mut lifecycle = reason_command();
    let output = lifecycle
        .args(["models", "nvidia", "--format", "json"])
        .env("REASON_HOME", &temp)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("inspect lifecycle status");
    assert_eq!(output.status.code(), Some(0));
    let value = json_stdout(&output);
    let nvidia = &value["result"]["models"][0];
    assert_eq!(nvidia["availability"], "known_incompatible");
    assert_eq!(nvidia["general_use"], false);
    assert_eq!(nvidia["recommended"], false);

    std::fs::remove_dir_all(temp).ok();
}

#[test]
fn setup_noninteractive_uses_recommended_model_without_secret_leak() {
    let temp = std::env::temp_dir().join(format!(
        "reason-setup-contract-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&temp).unwrap();
    let sentinel = "reason-setup-contract-secret-must-not-leak";
    let mut command = reason_command();
    let output = command
        .args([
            "setup",
            "--provider",
            "mistral",
            "--non-interactive",
            "--skip-live-check",
            "--format",
            "json",
        ])
        .env("REASON_HOME", &temp)
        .env("MISTRAL_API_KEY", sentinel)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run setup");
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let body = String::from_utf8_lossy(&output.stdout);
    assert!(!body.contains(sentinel));
    let value = json_stdout(&output);
    assert_eq!(value["command"], "setup");
    assert_eq!(value["result"]["provider"], "mistral");
    assert_eq!(value["result"]["model"], "ministral-8b-latest");
    assert_eq!(value["result"]["credential_source"], "environment");
    assert_eq!(value["result"]["local_readiness"], "passed");
    assert_eq!(value["result"]["live_readiness"], "skipped");
    assert_eq!(value["result"]["live_provider_attempts"], 0);
    assert!(
        value["result"]["first_command"]
            .as_str()
            .unwrap_or_default()
            .starts_with("reason ")
    );

    let config_path = temp.join("config.json");
    let text = std::fs::read_to_string(&config_path).unwrap();
    assert!(!text.contains(sentinel));
    let config: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(config["run"]["provider"], "mistral");
    assert_eq!(config["run"]["model"], "ministral-8b-latest");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&config_path)
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }

    std::fs::remove_dir_all(temp).ok();
}

#[test]
fn setup_help_has_no_secret_valued_argv_and_live_check_is_explicit() {
    let output = run_reason(&["setup", "--help"], None);
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let help = String::from_utf8_lossy(&output.stdout);
    for expected in [
        "--credential-stdin",
        "--from-env",
        "--live-check",
        "--skip-live-check",
        "--non-interactive",
    ] {
        assert!(help.contains(expected), "missing {expected}: {help}");
    }
    for forbidden in ["--api-key", "--secret", "--password", "--token"] {
        assert!(!help.contains(forbidden), "unexpected {forbidden}: {help}");
    }
}

#[test]
fn mcp_help_exposes_remote_oauth_lifecycle_without_secret_valued_flags() {
    let output = run_reason(&["mcp", "--help"], None);
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let help = String::from_utf8_lossy(&output.stdout);
    for expected in ["add-remote", "login", "status", "logout", "test"] {
        assert!(help.contains(expected), "missing {expected}: {help}");
    }
    for forbidden in [
        "--access-token",
        "--refresh-token",
        "--client-secret",
        "--api-key",
        "--token",
    ] {
        assert!(
            !help.contains(forbidden),
            "unexpected secret-valued flag {forbidden}: {help}"
        );
    }

    let login = run_reason(&["mcp", "login", "--help"], None);
    assert_eq!(login.status.code(), Some(0));
    let login_help = String::from_utf8_lossy(&login.stdout);
    assert!(login_help.contains("--no-browser"));
    assert!(login_help.contains("--replace"));
    assert!(!login_help.contains("--client-secret"));
}

#[cfg(unix)]
#[test]
fn mcp_management_add_inspect_test_remove_is_read_only_and_secret_free() {
    use std::os::unix::fs::PermissionsExt;

    let temp = std::env::temp_dir().join(format!(
        "reason-mcp-management-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&temp).unwrap();
    let home = temp.join("home");
    let safe = temp.join("safe-mcp.sh");
    let unsafe_server = temp.join("unsafe-mcp.sh");
    std::fs::write(
        &safe,
        r#"#!/bin/sh
read initialize
printf '%s\n' '{"jsonrpc":"2.0","id":"reasoning-harness:mcp-readiness:initialize","result":{"protocolVersion":"2026-07-28","capabilities":{},"serverInfo":{"name":"fixture","version":"1"}}}'
read initialized
read list
printf '%s\n' '{"jsonrpc":"2.0","id":"reasoning-harness:mcp-readiness:tools-list:0","result":{"tools":[{"name":"lookup","inputSchema":{"type":"object"},"annotations":{"readOnlyHint":true}}]}}'
"#,
    )
    .unwrap();
    std::fs::write(
        &unsafe_server,
        r#"#!/bin/sh
read initialize
printf '%s\n' '{"jsonrpc":"2.0","id":"reasoning-harness:mcp-readiness:initialize","result":{"protocolVersion":"2026-07-28","capabilities":{},"serverInfo":{"name":"fixture","version":"1"}}}'
read initialized
read list
printf '%s\n' '{"jsonrpc":"2.0","id":"reasoning-harness:mcp-readiness:tools-list:0","result":{"tools":[{"name":"lookup","inputSchema":{"type":"object"},"annotations":{"readOnlyHint":false}}]}}'
"#,
    )
    .unwrap();
    for script in [&safe, &unsafe_server] {
        let mut permissions = std::fs::metadata(script).unwrap().permissions();
        permissions.set_mode(0o700);
        std::fs::set_permissions(script, permissions).unwrap();
    }

    let run = |args: &[&str]| {
        reason_command()
            .args(args)
            .env("REASON_HOME", &home)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("run reason mcp command")
    };
    let safe_program = safe.to_string_lossy().into_owned();
    let unsafe_program = unsafe_server.to_string_lossy().into_owned();
    let visible_only_in_config = "launch-argument-value";

    let add = run(&[
        "mcp",
        "add",
        "inventory",
        "--program",
        &safe_program,
        "--arg",
        visible_only_in_config,
        "--tool",
        "lookup",
        "--format",
        "json",
    ]);
    assert_eq!(
        add.status.code(),
        Some(0),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&add.stdout),
        String::from_utf8_lossy(&add.stderr)
    );
    assert!(add.stderr.is_empty());
    let value = json_stdout(&add);
    assert_eq!(value["command"], "mcp");
    assert_eq!(value["result"]["operation"], "add");
    assert_eq!(value["result"]["source"]["read_only"], true);
    assert_eq!(value["result"]["source"]["argument_count"], 1);
    assert!(!String::from_utf8_lossy(&add.stdout).contains(visible_only_in_config));

    let config_path = home.join("config.json");
    let config: Value = serde_json::from_slice(&std::fs::read(&config_path).unwrap()).unwrap();
    assert_eq!(
        config["resolution"]["mcp_readonly"]["server_id"],
        "inventory"
    );
    assert_eq!(config["resolution"]["mcp_readonly"]["read_only"], true);
    assert_eq!(
        config["resolution"]["mcp_readonly"]["resolver_class"],
        "evidence_acquisition"
    );
    assert_eq!(
        config["resolution"]["mcp_readonly"]["allowed_tools"][0],
        "lookup"
    );
    assert_eq!(
        std::fs::metadata(&config_path)
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o600
    );

    let inspect = run(&["mcp", "inspect", "inventory", "--format", "json"]);
    assert_eq!(inspect.status.code(), Some(0));
    assert!(inspect.stderr.is_empty());
    assert!(!String::from_utf8_lossy(&inspect.stdout).contains(visible_only_in_config));
    assert_eq!(
        json_stdout(&inspect)["result"]["source"]["argument_count"],
        1
    );

    let readiness = run(&["mcp", "test", "inventory", "--format", "json"]);
    assert_eq!(readiness.status.code(), Some(0));
    assert!(readiness.stderr.is_empty());
    let value = json_stdout(&readiness);
    assert_eq!(value["result"]["status"], "ready");
    assert_eq!(value["result"]["negotiated_protocol_version"], "2026-07-28");
    assert_eq!(value["result"]["read_only_hint"], true);

    let conflict = run(&[
        "mcp",
        "add",
        "other",
        "--program",
        &safe_program,
        "--tool",
        "lookup",
        "--format",
        "json",
    ]);
    assert_eq!(conflict.status.code(), Some(1));
    assert_eq!(
        json_stdout(&conflict)["result"]["failure"]["failure_class"],
        "mcp_configuration"
    );

    let replace = run(&[
        "mcp",
        "add",
        "inventory",
        "--program",
        &unsafe_program,
        "--tool",
        "lookup",
        "--replace",
        "--format",
        "json",
    ]);
    assert_eq!(replace.status.code(), Some(0));
    let rejected = run(&["mcp", "test", "inventory", "--format", "json"]);
    assert_eq!(rejected.status.code(), Some(1));
    assert!(rejected.stderr.is_empty());
    assert_eq!(
        json_stdout(&rejected)["result"]["failure"]["failure_class"],
        "mcp_policy"
    );

    let secret_arg = run(&[
        "mcp",
        "add",
        "inventory",
        "--program",
        "fixture-mcp-server",
        "--arg=--api-key=must-not-persist",
        "--tool",
        "lookup_item",
        "--replace",
        "--format",
        "json",
    ]);
    assert_eq!(secret_arg.status.code(), Some(1));
    assert!(secret_arg.stderr.is_empty());
    let secret_json = json_stdout(&secret_arg);
    assert_eq!(
        secret_json["result"]["failure"]["failure_class"],
        "mcp_secret_input"
    );
    assert!(!String::from_utf8_lossy(&secret_arg.stdout).contains("must-not-persist"));

    let remove = run(&["mcp", "remove", "inventory", "--format", "json"]);
    assert_eq!(remove.status.code(), Some(0));
    assert_eq!(json_stdout(&remove)["result"]["operation"], "remove");
    let list = run(&["mcp", "list", "--format", "json"]);
    assert_eq!(list.status.code(), Some(0));
    assert_eq!(
        json_stdout(&list)["result"]["sources"]
            .as_array()
            .unwrap()
            .len(),
        0
    );

    std::fs::remove_dir_all(temp).ok();
}

#[test]
fn lifecycle_help_is_explicit_and_has_no_insecure_provenance_bypass() {
    for (command, expected) in [
        (
            "update",
            vec![
                "--check",
                "--version",
                "--rollback",
                "--allow-engine-change",
                "--yes",
                "--format",
            ],
        ),
        (
            "uninstall",
            vec![
                "--dry-run",
                "--yes",
                "--purge-data",
                "--purge-credentials",
                "--format",
            ],
        ),
    ] {
        let output = run_reason(&[command, "--help"], None);
        assert_eq!(
            output.status.code(),
            Some(0),
            "{command}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stderr.is_empty());
        let help = String::from_utf8_lossy(&output.stdout);
        for flag in expected {
            assert!(help.contains(flag), "{command} missing {flag}: {help}");
        }
        for forbidden in [
            "--insecure",
            "--skip-verify",
            "--no-verify",
            "--api-key",
            "--token",
        ] {
            assert!(
                !help.contains(forbidden),
                "{command} exposes forbidden {forbidden}: {help}"
            );
        }
    }
}

#[test]
fn rollback_rejects_historical_pre_attestation_release_before_mutation() {
    let output = run_reason(
        &["update", "--rollback", "0.4.2", "--yes", "--format", "json"],
        None,
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());
    let value = json_stdout(&output);
    assert_eq!(value["command"], "update");
    assert_eq!(value["result"]["status"], "failed");
    assert_eq!(
        value["result"]["failure"]["failure_class"],
        "historical_release_boundary"
    );
}

#[test]
fn uninstall_dry_run_is_non_mutating_and_secret_free() {
    let temp = std::env::temp_dir().join(format!(
        "reason-uninstall-dry-run-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&temp).unwrap();
    let config = temp.join("config.json");
    let trust = temp.join("project-trust.json");
    let sessions = temp.join("sessions");
    std::fs::write(&config, b"sentinel-config").unwrap();
    std::fs::write(&trust, b"sentinel-trust").unwrap();
    std::fs::create_dir_all(&sessions).unwrap();
    std::fs::write(sessions.join("sentinel.json"), b"sentinel-session").unwrap();
    let mut command = reason_command();
    let output = command
        .args(["uninstall", "--dry-run", "--purge-data", "--format", "json"])
        .env("REASON_HOME", &temp)
        .env("MISTRAL_API_KEY", "uninstall-secret-must-not-leak")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let body = String::from_utf8_lossy(&output.stdout);
    assert!(!body.contains("uninstall-secret-must-not-leak"));
    let value = json_stdout(&output);
    assert_eq!(value["command"], "uninstall");
    assert_eq!(value["result"]["dry_run"], true);
    assert_eq!(value["result"]["binary_removed"], false);
    assert_eq!(value["result"]["data_removed"], 0);
    assert!(config.exists());
    assert!(trust.exists());
    assert!(sessions.join("sentinel.json").exists());
    let data_files = value["result"]["data_files"].as_array().unwrap();
    assert!(data_files.iter().any(|path| {
        path.as_str()
            .is_some_and(|p| p.ends_with("/sessions") || p.ends_with("\\sessions"))
    }));
    std::fs::remove_dir_all(temp).ok();
}

#[test]
fn mcp_management_add_list_inspect_remove_is_non_secret_and_machine_readable() {
    let temp = std::env::temp_dir().join(format!(
        "reason-mcp-management-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&temp).unwrap();

    let run = |args: &[&str]| {
        let mut command = reason_command();
        command
            .args(args)
            .env("REASON_HOME", &temp)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("run reason mcp command")
    };

    let add = run(&[
        "mcp",
        "add",
        "inventory",
        "--program",
        "fixture-mcp-server",
        "--arg=--stdio",
        "--tool",
        "lookup_item",
        "--format",
        "json",
    ]);
    assert_eq!(
        add.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&add.stderr)
    );
    assert!(add.stderr.is_empty());
    let add_json = json_stdout(&add);
    assert_eq!(add_json["command"], "mcp");
    assert_eq!(add_json["result"]["operation"], "add");
    assert_eq!(add_json["result"]["name"], "inventory");
    assert_eq!(add_json["result"]["source"]["read_only"], true);
    assert_eq!(add_json["result"]["source"]["argument_count"], 1);
    assert!(!String::from_utf8_lossy(&add.stdout).contains("--stdio"));

    let list = run(&["mcp", "list", "--format", "json"]);
    assert_eq!(list.status.code(), Some(0));
    let list_json = json_stdout(&list);
    assert_eq!(list_json["result"]["sources"].as_array().unwrap().len(), 1);
    assert_eq!(list_json["result"]["sources"][0]["name"], "inventory");
    assert!(!String::from_utf8_lossy(&list.stdout).contains("--stdio"));

    let inspect = run(&["mcp", "inspect", "inventory", "--format", "json"]);
    assert_eq!(inspect.status.code(), Some(0));
    let inspect_json = json_stdout(&inspect);
    assert_eq!(
        inspect_json["result"]["source"]["selected_tool"],
        "lookup_item"
    );
    assert_eq!(inspect_json["result"]["source"]["transport"], "stdio");
    assert!(!String::from_utf8_lossy(&inspect.stdout).contains("--stdio"));

    let duplicate = run(&[
        "mcp",
        "add",
        "other",
        "--program",
        "other-server",
        "--tool",
        "read",
        "--format",
        "json",
    ]);
    assert_eq!(duplicate.status.code(), Some(1));
    assert!(duplicate.stderr.is_empty());
    let duplicate_json = json_stdout(&duplicate);
    assert_eq!(
        duplicate_json["result"]["failure"]["failure_class"],
        "mcp_configuration"
    );

    let remove = run(&["mcp", "remove", "inventory", "--format", "json"]);
    assert_eq!(remove.status.code(), Some(0));
    let remove_json = json_stdout(&remove);
    assert_eq!(remove_json["result"]["operation"], "remove");
    assert_eq!(remove_json["result"]["config_removed"], true);
    assert_eq!(remove_json["result"]["credential_removed"], false);
    assert_eq!(
        remove_json["result"]["credential_cleanup_status"],
        "not_applicable"
    );

    let empty = run(&["mcp", "list", "--format", "json"]);
    assert_eq!(empty.status.code(), Some(0));
    assert!(
        json_stdout(&empty)["result"]["sources"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    std::fs::remove_dir_all(temp).ok();
}

const TEST_MCP_OAUTH_SERVICE: &str = "io.github.git-ksk.reason-cli.mcp-oauth.v1";

fn unique_mcp_name(prefix: &str) -> String {
    format!(
        "{prefix}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    )
}

fn mcp_oauth_entry(name: &str) -> Entry {
    Entry::new(TEST_MCP_OAUTH_SERVICE, &format!("mcp:{name}:oauth")).unwrap()
}

fn remove_test_credential(name: &str) {
    match mcp_oauth_entry(name).delete_credential() {
        Ok(()) | Err(KeyringError::NoEntry) => {}
        Err(error) => panic!("remove test credential {name}: {error}"),
    }
}

fn seed_test_mcp_oauth_credential(name: &str, marker: &str) {
    let token = serde_json::json!({
        "schema_version": "reason-mcp-oauth-token-v1",
        "server_name": name,
        "issuer": "https://auth.example.test",
        "client_id": "https://client.example.test/reason.json",
        "resource": "https://mcp.example.test/mcp",
        "token_endpoint": "https://auth.example.test/token",
        "access_token": marker,
        "token_type": "Bearer",
        "scope": "mcp:read",
        "expires_at_unix_seconds": u64::MAX
    });
    mcp_oauth_entry(name)
        .set_password(&token.to_string())
        .expect("seed MCP OAuth test credential");
}

fn write_remote_mcp_test_config(temp: &Path, name: &str) {
    let config = serde_json::json!({
        "schema_version": "reason-config-v1",
        "resolution": {
            "mcp_remote_readonly": {
                "server_id": name,
                "endpoint": "https://mcp.example.test/mcp",
                "allowed_tools": ["search"],
                "tool": "search",
                "read_only": true,
                "resolver_class": "evidence_acquisition",
                "source": format!("mcp:{name}:search"),
                "protocol_version": "2026-07-28",
                "oauth": {
                    "issuer": "https://auth.example.test",
                    "authorization_endpoint": "https://auth.example.test/authorize",
                    "token_endpoint": "https://auth.example.test/token",
                    "client_id": "https://client.example.test/reason.json",
                    "scopes": ["mcp:read"]
                }
            }
        }
    });
    std::fs::write(
        temp.join("config.json"),
        serde_json::to_vec_pretty(&config).unwrap(),
    )
    .unwrap();
}

fn spawn_oauth_discovery_fixture() -> (String, String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let endpoint = format!("http://{addr}/mcp");
    let issuer = format!("http://{addr}/auth");
    let resource_metadata = format!("http://{addr}/resource-metadata");
    let endpoint_for_server = endpoint.clone();
    let issuer_for_server = issuer.clone();
    let resource_for_server = resource_metadata.clone();
    let server = thread::spawn(move || {
        for request_index in 0..3 {
            let (mut stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut first = String::new();
            reader.read_line(&mut first).unwrap();
            let mut content_length = 0usize;
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                if line == "\r\n" {
                    break;
                }
                if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                    content_length = value.trim().parse().unwrap();
                }
            }
            if content_length > 0 {
                let mut body = vec![0u8; content_length];
                reader.read_exact(&mut body).unwrap();
            }
            let (status, extra_headers, body) = match request_index {
                0 => {
                    assert!(first.starts_with("POST /mcp "));
                    (
                        "401 Unauthorized",
                        format!(
                            "WWW-Authenticate: Bearer resource_metadata=\"{resource_for_server}\", scope=\"mcp:read\"\r\n"
                        ),
                        String::new(),
                    )
                }
                1 => {
                    assert!(first.starts_with("GET /resource-metadata "));
                    (
                        "200 OK",
                        String::new(),
                        serde_json::json!({
                            "resource": endpoint_for_server,
                            "authorization_servers": [issuer_for_server],
                            "scopes_supported": ["mcp:read"]
                        })
                        .to_string(),
                    )
                }
                _ => {
                    assert!(first.starts_with("GET /.well-known/oauth-authorization-server/auth "));
                    (
                        "200 OK",
                        String::new(),
                        serde_json::json!({
                            "issuer": issuer_for_server,
                            "authorization_endpoint": format!("{issuer_for_server}/authorize"),
                            "token_endpoint": format!("{issuer_for_server}/token"),
                            "code_challenge_methods_supported": ["S256"]
                        })
                        .to_string(),
                    )
                }
            };
            let response = format!(
                "HTTP/1.1 {status}\r\nContent-Type: application/json\r\n{extra_headers}Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(response.as_bytes()).unwrap();
        }
    });
    (endpoint, issuer, server)
}

#[test]
fn remote_mcp_management_persists_only_non_secret_oauth_metadata() {
    let temp = std::env::temp_dir().join(format!(
        "reason-mcp-remote-management-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&temp).unwrap();
    let (endpoint, issuer, discovery_server) = spawn_oauth_discovery_fixture();
    let run = |args: &[&str]| {
        reason_command()
            .args(args)
            .env("REASON_HOME", &temp)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("run remote mcp management")
    };

    let add = run(&[
        "mcp",
        "add-remote",
        "docs",
        "--endpoint",
        endpoint.as_str(),
        "--tool",
        "search",
        "--client-id",
        "https://client.example.test/reason.json",
        "--scope",
        "mcp:read",
        "--format",
        "json",
    ]);
    assert_eq!(
        add.status.code(),
        Some(0),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&add.stdout),
        String::from_utf8_lossy(&add.stderr)
    );
    assert!(add.stderr.is_empty());
    let add_json = json_stdout(&add);
    assert_eq!(add_json["result"]["operation"], "add_remote");
    assert_eq!(add_json["result"]["source"]["transport"], "streamable_http");
    assert_eq!(
        add_json["result"]["source"]["protocol_version"],
        "2026-07-28"
    );
    assert_eq!(add_json["result"]["source"]["oauth_configured"], true);

    let config_bytes = std::fs::read(temp.join("config.json")).unwrap();
    let config_text = String::from_utf8(config_bytes.clone()).unwrap();
    assert!(!config_text.contains("access_token"));
    assert!(!config_text.contains("refresh_token"));
    assert!(!config_text.contains("client_secret"));
    let config: Value = serde_json::from_slice(&config_bytes).unwrap();
    let remote = &config["resolution"]["mcp_remote_readonly"];
    assert_eq!(remote["server_id"], "docs");
    assert_eq!(remote["read_only"], true);
    assert_eq!(remote["resolver_class"], "evidence_acquisition");
    assert_eq!(remote["protocol_version"], "2026-07-28");
    assert_eq!(remote["oauth"]["issuer"], issuer);
    assert_eq!(
        remote["oauth"]["client_id"],
        "https://client.example.test/reason.json"
    );

    let inspect = run(&["mcp", "inspect", "docs", "--format", "json"]);
    assert_eq!(inspect.status.code(), Some(0));
    assert!(inspect.stderr.is_empty());
    let inspect_text = String::from_utf8_lossy(&inspect.stdout);
    assert!(!inspect_text.contains("access_token"));
    assert!(!inspect_text.contains("refresh_token"));
    let inspect_json = json_stdout(&inspect);
    assert_eq!(inspect_json["result"]["source"]["endpoint"], endpoint);
    assert_eq!(inspect_json["result"]["source"]["oauth_issuer"], issuer);

    discovery_server.join().unwrap();

    let insecure = run(&[
        "mcp",
        "add-remote",
        "bad",
        "--endpoint",
        "http://example.test/mcp",
        "--tool",
        "search",
        "--issuer",
        "https://auth.example.test",
        "--authorization-endpoint",
        "https://auth.example.test/authorize",
        "--token-endpoint",
        "https://auth.example.test/token",
        "--client-id",
        "client",
        "--replace",
        "--format",
        "json",
    ]);
    assert_eq!(insecure.status.code(), Some(1));
    assert!(insecure.stderr.is_empty());
    assert_eq!(
        json_stdout(&insecure)["result"]["failure"]["failure_class"],
        "mcp_configuration"
    );

    std::fs::remove_dir_all(temp).ok();
}

#[test]
#[ignore = "mutates the native OS credential store; run only in isolated CI"]
fn remote_mcp_remove_reports_present_and_absent_credentials_without_cross_account_deletion() {
    let temp = std::env::temp_dir().join(unique_mcp_name("reason-mcp-remove-native"));
    std::fs::create_dir_all(&temp).unwrap();
    let first = unique_mcp_name("remove-first");
    let second = unique_mcp_name("remove-second");
    remove_test_credential(&first);
    remove_test_credential(&second);
    seed_test_mcp_oauth_credential(&first, "first-token-must-not-leak");
    seed_test_mcp_oauth_credential(&second, "second-token-must-not-leak");
    write_remote_mcp_test_config(&temp, &first);

    let remove = reason_command()
        .args(["mcp", "remove", &first, "--format", "json"])
        .env("REASON_HOME", &temp)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("remove remote MCP with stored credential");
    assert_eq!(remove.status.code(), Some(0));
    assert!(remove.stderr.is_empty());
    let text = String::from_utf8_lossy(&remove.stdout);
    assert!(!text.contains("first-token-must-not-leak"));
    assert!(!text.contains("second-token-must-not-leak"));
    let json = json_stdout(&remove);
    assert_eq!(json["result"]["operation"], "remove");
    assert_eq!(json["result"]["status"], "ok");
    assert_eq!(json["result"]["config_removed"], true);
    assert_eq!(json["result"]["credential_removed"], true);
    assert_eq!(json["result"]["credential_cleanup_status"], "removed");
    assert!(matches!(
        mcp_oauth_entry(&first).get_password(),
        Err(KeyringError::NoEntry)
    ));
    let second_stored: Value =
        serde_json::from_str(&mcp_oauth_entry(&second).get_password().unwrap())
            .expect("second stored credential JSON");
    assert_eq!(second_stored["server_name"], second);

    seed_test_mcp_oauth_credential(&first, "orphan-token-must-not-leak");
    let logout = reason_command()
        .args(["mcp", "logout", &first, "--format", "json"])
        .env("REASON_HOME", &temp)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("logout same-name credential after config removal");
    assert_eq!(logout.status.code(), Some(0));
    assert!(logout.stderr.is_empty());
    let logout_text = String::from_utf8_lossy(&logout.stdout);
    assert!(!logout_text.contains("orphan-token-must-not-leak"));
    assert_eq!(json_stdout(&logout)["result"]["removed"], true);
    assert!(matches!(
        mcp_oauth_entry(&first).get_password(),
        Err(KeyringError::NoEntry)
    ));
    remove_test_credential(&second);

    let absent = unique_mcp_name("remove-absent");
    remove_test_credential(&absent);
    write_remote_mcp_test_config(&temp, &absent);
    let remove_absent = reason_command()
        .args(["mcp", "remove", &absent, "--format", "json"])
        .env("REASON_HOME", &temp)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("remove remote MCP without stored credential");
    assert_eq!(remove_absent.status.code(), Some(0));
    let absent_json = json_stdout(&remove_absent);
    assert_eq!(absent_json["result"]["config_removed"], true);
    assert_eq!(absent_json["result"]["credential_removed"], false);
    assert_eq!(absent_json["result"]["credential_cleanup_status"], "absent");
    std::fs::remove_dir_all(temp).ok();
}

#[cfg(target_os = "linux")]
#[test]
#[ignore = "requires a deliberately unavailable Linux Secret Service"]
fn remote_mcp_remove_reports_partial_failure_when_credential_store_is_unavailable() {
    let temp = std::env::temp_dir().join(unique_mcp_name("reason-mcp-remove-headless"));
    std::fs::create_dir_all(&temp).unwrap();
    let name = unique_mcp_name("remove-headless");
    write_remote_mcp_test_config(&temp, &name);
    let missing_runtime = temp.join("missing-runtime");
    let output = reason_command()
        .args(["mcp", "remove", &name, "--format", "json"])
        .env("REASON_HOME", &temp)
        .env(
            "DBUS_SESSION_BUS_ADDRESS",
            format!("unix:path={}", temp.join("missing-bus").display()),
        )
        .env("XDG_RUNTIME_DIR", &missing_runtime)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("remove remote MCP with unavailable credential store");
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(!text.contains("access_token"));
    assert!(!text.contains("refresh_token"));
    let json = json_stdout(&output);
    assert_eq!(json["result"]["status"], "partial_failure");
    assert_eq!(json["result"]["config_removed"], true);
    assert_eq!(json["result"]["credential_removed"], false);
    assert_eq!(json["result"]["credential_cleanup_status"], "failed");
    assert_eq!(
        json["result"]["failure"]["failure_class"],
        "credential_store_unavailable"
    );
    assert!(
        json["result"]["failure"]["recovery"]
            .as_str()
            .unwrap()
            .contains("reason mcp logout")
    );
    let config: Value =
        serde_json::from_slice(&std::fs::read(temp.join("config.json")).unwrap()).unwrap();
    assert!(config["resolution"].get("mcp_remote_readonly").is_none());
    std::fs::remove_dir_all(temp).ok();
}

#[test]
fn remote_mcp_insufficient_scope_is_typed_actionable_and_secret_free() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let endpoint = format!("http://{addr}/mcp");
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut first = String::new();
        reader.read_line(&mut first).unwrap();
        assert!(first.starts_with("POST /mcp "));
        let mut content_length = 0usize;
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            if line == "\r\n" {
                break;
            }
            if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                content_length = value.trim().parse().unwrap();
            }
        }
        if content_length > 0 {
            let mut body = vec![0u8; content_length];
            reader.read_exact(&mut body).unwrap();
        }
        let response = concat!(
            "HTTP/1.1 403 Forbidden\r\n",
            "WWW-Authenticate: Bearer error=\"insufficient_scope\", scope=\"profile:read files:read\", error_description=\"sensitive-marker\"\r\n",
            "Content-Length: 0\r\nConnection: close\r\n\r\n"
        );
        stream.write_all(response.as_bytes()).unwrap();
    });

    let temp = std::env::temp_dir().join(format!(
        "reason-mcp-scope-step-up-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&temp).unwrap();
    let config = serde_json::json!({
        "schema_version": "reason-config-v1",
        "resolution": {
            "mcp_remote_readonly": {
                "server_id": "scope-demo",
                "endpoint": endpoint,
                "allowed_tools": ["search"],
                "tool": "search",
                "read_only": true,
                "resolver_class": "evidence_acquisition",
                "source": "mcp:scope-demo:search",
                "protocol_version": "2026-07-28"
            }
        }
    });
    std::fs::write(
        temp.join("config.json"),
        serde_json::to_vec_pretty(&config).unwrap(),
    )
    .unwrap();
    let output = reason_command()
        .args(["mcp", "test", "scope-demo", "--format", "json"])
        .env("REASON_HOME", &temp)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run remote MCP scope fixture");
    server.join().unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(!text.contains("sensitive-marker"));
    let json = json_stdout(&output);
    assert_eq!(
        json["result"]["failure"]["failure_class"],
        "mcp_insufficient_scope"
    );
    let message = json["result"]["failure"]["message"].as_str().unwrap();
    assert!(message.contains("files:read"));
    assert!(message.contains("profile:read"));
    assert!(message.contains("scope-demo"));
    assert!(message.contains("reason mcp login"));
    assert!(message.contains("--replace"));
    assert!(message.contains("--scope"));
    assert!(message.contains(&endpoint));
    std::fs::remove_dir_all(temp).ok();
}

#[test]
fn remote_mcp_scope_step_up_requires_explicit_replace() {
    let output = run_reason(
        &[
            "mcp",
            "login",
            "scope-demo",
            "--scope",
            "files:read",
            "--format",
            "json",
        ],
        None,
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());
    let json = json_stdout(&output);
    assert_eq!(
        json["result"]["failure"]["failure_class"],
        "mcp_scope_step_up"
    );
    let message = json["result"]["failure"]["message"].as_str().unwrap();
    assert!(message.contains("--replace"));
    assert!(!message.contains("access_token"));
    assert!(!message.contains("refresh_token"));
}

#[test]
fn doctor_reports_versions_sources_and_secret_free_readiness_without_live_side_effects() {
    let temp = std::env::temp_dir().join(format!(
        "reason-doctor-contract-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&temp).unwrap();
    let config = serde_json::json!({
        "schema_version": "reason-config-v1",
        "run": {
            "provider": "groq",
            "model": "openai/gpt-oss-120b"
        },
        "resolution": {
            "mcp_remote_readonly": {
                "server_id": "doctor-docs",
                "endpoint": "https://mcp.example.test/mcp",
                "allowed_tools": ["search"],
                "tool": "search",
                "read_only": true,
                "resolver_class": "evidence_acquisition",
                "source": "mcp:doctor-docs:search",
                "protocol_version": "2026-07-28"
            }
        }
    });
    std::fs::write(
        temp.join("config.json"),
        serde_json::to_vec_pretty(&config).unwrap(),
    )
    .unwrap();
    let marker = "doctor-contract-secret-must-never-leak";
    let output = reason_command()
        .args(["doctor", "--format", "json"])
        .env("REASON_HOME", &temp)
        .env("GROQ_API_KEY", marker)
        .env_remove("MISTRAL_API_KEY")
        .env_remove("GEMINI_API_KEY")
        .env_remove("NVIDIA_API_KEY")
        .env_remove("REASON_CA_BUNDLE")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run doctor");
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(!text.contains(marker));
    let json = json_stdout(&output);
    assert_eq!(json["schema_version"], "reason-cli-output-v1");
    assert_eq!(json["command"], "doctor");
    assert_eq!(json["result"]["doctor_surface"], "reason-doctor-v1");
    assert_eq!(json["result"]["versions"]["cli"], env!("CARGO_PKG_VERSION"));
    assert_eq!(json["result"]["versions"]["engine"], "0.5.0");
    assert_eq!(json["result"]["config"]["status"], "valid");
    assert!(
        json["result"]["config"]["effective_sources"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value == "user")
    );
    assert_eq!(json["result"]["provider"]["provider"], "groq");
    assert_eq!(json["result"]["provider"]["model"], "openai/gpt-oss-120b");
    assert_eq!(json["result"]["provider"]["model_availability"], "current");
    assert_eq!(json["result"]["provider"]["local_readiness"], "ready");
    assert_eq!(json["result"]["provider"]["live_readiness"], "skipped");
    assert_eq!(json["result"]["network"]["custom_ca"], "not_configured");
    assert_eq!(json["result"]["network"]["live_probe"], "skipped");
    let groq = json["result"]["credentials"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["provider"] == "groq")
        .unwrap();
    assert_eq!(groq["environment"], "present");
    assert_eq!(groq["effective_source"], "environment");
    assert_eq!(json["result"]["mcp"]["configured"], true);
    assert_eq!(json["result"]["mcp"]["readiness"], "not_checked");
    assert_eq!(json["result"]["update"]["status"], "skipped");
    std::fs::remove_dir_all(temp).ok();
}

#[test]
fn doctor_reports_proxy_presence_and_invalid_custom_ca_without_leaking_values_or_tls_bypass_advice()
{
    let temp = std::env::temp_dir().join(format!(
        "reason-doctor-network-contract-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&temp).unwrap();
    let ca_path = temp.join("broken-ca.pem");
    std::fs::write(&ca_path, b"not-a-certificate").unwrap();
    let proxy_marker = "proxy-user:proxy-secret@example.invalid:8443";
    let no_proxy_marker = "internal-secret.example.invalid";

    let output = reason_command()
        .args(["doctor", "--format", "json"])
        .env("REASON_HOME", &temp)
        .env_remove("https_proxy")
        .env_remove("no_proxy")
        .env("HTTPS_PROXY", format!("http://{proxy_marker}"))
        .env("NO_PROXY", no_proxy_marker)
        .env("REASON_CA_BUNDLE", &ca_path)
        .env_remove("MISTRAL_API_KEY")
        .env_remove("GEMINI_API_KEY")
        .env_remove("GROQ_API_KEY")
        .env_remove("NVIDIA_API_KEY")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run doctor with proxy/custom CA environment");
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(!text.contains(proxy_marker));
    assert!(!text.contains(no_proxy_marker));
    assert!(!text.contains(&ca_path.to_string_lossy().to_string()));
    let json = json_stdout(&output);
    assert_eq!(json["result"]["network"]["https_proxy"], true);
    assert_eq!(json["result"]["network"]["no_proxy"], true);
    assert_eq!(json["result"]["network"]["custom_ca"], "invalid");
    assert_eq!(json["result"]["network"]["live_probe"], "not_run");
    let issue = json["result"]["issues"]
        .as_array()
        .unwrap()
        .iter()
        .find(|issue| issue["failure_class"] == "network_custom_ca")
        .expect("custom CA issue");
    let recovery = issue["recovery"].as_str().unwrap();
    assert!(recovery.contains("REASON_CA_BUNDLE"));
    for forbidden in [
        "danger_accept_invalid",
        "accept invalid cert",
        "disable tls",
        "skip certificate",
        "insecure tls",
    ] {
        assert!(!recovery.to_ascii_lowercase().contains(forbidden));
    }
    std::fs::remove_dir_all(temp).ok();
}

#[test]
fn doctor_live_check_honors_https_proxy_and_classifies_proxy_failure_before_provider_auth() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let proxy = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut first = String::new();
        reader.read_line(&mut first).unwrap();
        assert!(
            first.starts_with("CONNECT api.groq.com:443 "),
            "unexpected proxy request: {first:?}"
        );
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            if line == "\r\n" || line.is_empty() {
                break;
            }
        }
        stream
            .write_all(
                b"HTTP/1.1 407 Proxy Authentication Required\r\nProxy-Authenticate: Basic realm=\"reason-test\"\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            )
            .unwrap();
    });

    let temp = std::env::temp_dir().join(format!(
        "reason-doctor-proxy-live-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&temp).unwrap();
    std::fs::write(
        temp.join("config.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "schema_version": "reason-config-v1",
            "run": {"provider": "groq", "model": "openai/gpt-oss-120b"}
        }))
        .unwrap(),
    )
    .unwrap();

    let output = reason_command()
        .args(["doctor", "--live-check", "--format", "json"])
        .env("REASON_HOME", &temp)
        .env_remove("https_proxy")
        .env_remove("ALL_PROXY")
        .env_remove("all_proxy")
        .env_remove("NO_PROXY")
        .env_remove("no_proxy")
        .env("HTTPS_PROXY", format!("http://{addr}"))
        .env_remove("REASON_CA_BUNDLE")
        .env_remove("GROQ_API_KEY")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run doctor through local HTTPS proxy");
    proxy.join().unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let json = json_stdout(&output);
    assert_eq!(json["result"]["network"]["https_proxy"], true);
    assert_eq!(json["result"]["network"]["live_probe"], "failed");
    assert!(
        json["result"]["issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue["failure_class"] == "network_proxy")
    );
    std::fs::remove_dir_all(temp).ok();
}

#[test]
fn doctor_reports_non_current_model_lifecycle_and_explicit_replacement_path() {
    let temp = std::env::temp_dir().join(format!(
        "reason-doctor-model-lifecycle-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&temp).unwrap();
    let config = serde_json::json!({
        "schema_version": "reason-config-v1",
        "run": {
            "provider": "nvidia",
            "model": "nvidia/nemotron-3.5-lightning-30b-a3b"
        }
    });
    std::fs::write(
        temp.join("config.json"),
        serde_json::to_vec_pretty(&config).unwrap(),
    )
    .unwrap();

    let output = reason_command()
        .args(["doctor", "--format", "json"])
        .env("REASON_HOME", &temp)
        .env_remove("NVIDIA_API_KEY")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run doctor on known-incompatible model");
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let json = json_stdout(&output);
    assert_eq!(
        json["result"]["provider"]["model_availability"],
        "known_incompatible"
    );
    assert_eq!(json["result"]["provider"]["local_readiness"], "blocked");
    let lifecycle_issue = json["result"]["issues"]
        .as_array()
        .unwrap()
        .iter()
        .find(|issue| issue["failure_class"] == "model_known_incompatible")
        .expect("model lifecycle issue");
    assert_eq!(lifecycle_issue["recovery"], "reason models nvidia");
    std::fs::remove_dir_all(temp).ok();
}

#[test]
fn doctor_keeps_invalid_config_diagnostic_machine_readable() {
    let temp = std::env::temp_dir().join(format!(
        "reason-doctor-invalid-config-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&temp).unwrap();
    std::fs::write(temp.join("config.json"), b"{not-json").unwrap();
    let output = reason_command()
        .args(["doctor", "--format", "json"])
        .env("REASON_HOME", &temp)
        .env_remove("MISTRAL_API_KEY")
        .env_remove("GEMINI_API_KEY")
        .env_remove("GROQ_API_KEY")
        .env_remove("NVIDIA_API_KEY")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run doctor on invalid config");
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let json = json_stdout(&output);
    assert_eq!(json["result"]["status"], "attention");
    assert_eq!(json["result"]["config"]["status"], "invalid");
    assert!(
        json["result"]["issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue["failure_class"] == "configuration")
    );
    std::fs::remove_dir_all(temp).ok();
}
