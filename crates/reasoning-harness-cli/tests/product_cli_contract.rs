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
