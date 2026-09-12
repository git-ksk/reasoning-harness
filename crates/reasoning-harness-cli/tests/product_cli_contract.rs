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
    assert!(!String::from_utf8_lossy(&output.stdout).contains("model-catalog-test-secret"));
    let value = json_stdout(&output);
    assert_eq!(value["command"], "model");
    assert_eq!(value["result"]["provider"], "mistral");
    assert_eq!(value["result"]["model"], "ministral-8b-latest");
    assert_eq!(value["result"]["compatibility"], "validated");
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

    for (provider, model, failure_class) in [
        (
            "nvidia",
            "nvidia/nemotron-3.5-lightning-30b-a3b",
            "model_not_general_use",
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
    }

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
    std::fs::write(&config, b"sentinel-config").unwrap();
    std::fs::write(&trust, b"sentinel-trust").unwrap();
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
    std::fs::remove_dir_all(temp).ok();
}
