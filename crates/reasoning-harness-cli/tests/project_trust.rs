use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

use serde_json::{Value, json};

struct TestDir(PathBuf);

impl TestDir {
    fn new(name: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "reason-project-trust-{name}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn reason_bin() -> &'static str {
    env!("CARGO_BIN_EXE_reason")
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn write_project_config(project: &Path, resolution: Value, run: Value) {
    let directory = project.join(".reason");
    fs::create_dir_all(&directory).unwrap();
    fs::write(
        directory.join("config.json"),
        serde_json::to_vec_pretty(&json!({
            "schema_version": "reason-config-v1",
            "run": run,
            "resolution": resolution,
        }))
        .unwrap(),
    )
    .unwrap();
}

fn acquisition_resolution(program: &str, marker: &str) -> Value {
    json!({
        "external_command": {
            "program": program,
            "args": [marker],
            "timeout_ms": 1000,
            "max_response_bytes": 4096
        }
    })
}

fn run_reason(home: &Path, cwd: &Path, args: &[&str]) -> Output {
    Command::new(reason_bin())
        .args(args)
        .current_dir(cwd)
        .env("REASON_HOME", home)
        .env_remove("XDG_CONFIG_HOME")
        .output()
        .unwrap()
}

fn trust_json(home: &Path, cwd: &Path, args: &[&str]) -> Value {
    let output = run_reason(home, cwd, args);
    assert!(
        output.status.success(),
        "command failed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn offline_run(home: &Path, cwd: &Path, extra: &[&str]) -> Output {
    let root = repo_root();
    let input = root.join("examples/input.json");
    let candidate = root.join("examples/candidate.json");
    let mut args = vec![
        "run",
        "--input",
        input.to_str().unwrap(),
        "--candidate",
        candidate.to_str().unwrap(),
        "--format",
        "json",
    ];
    args.extend_from_slice(extra);
    run_reason(home, cwd, &args)
}

#[test]
fn safe_project_defaults_do_not_require_trust() {
    let sandbox = TestDir::new("safe-defaults");
    let project = sandbox.path().join("project");
    let home = sandbox.path().join("home");
    fs::create_dir_all(&project).unwrap();
    fs::create_dir_all(&home).unwrap();
    write_project_config(&project, json!({}), json!({"max_tokens": 256}));

    let output = offline_run(&home, &project, &[]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let status = trust_json(&home, &project, &["trust", "status", "--format", "json"]);
    assert_eq!(status["result"]["state"], "not_required");
    assert_eq!(status["result"]["high_risk_config"], false);
}

#[test]
fn high_risk_project_config_is_fail_closed_until_explicitly_trusted() {
    let sandbox = TestDir::new("explicit-trust");
    let project = sandbox.path().join("project");
    let home = sandbox.path().join("home");
    fs::create_dir_all(&project).unwrap();
    fs::create_dir_all(&home).unwrap();
    write_project_config(
        &project,
        acquisition_resolution(reason_bin(), "marker-a"),
        json!({}),
    );

    let blocked = offline_run(&home, &project, &[]);
    assert!(!blocked.status.success());
    let stderr = String::from_utf8_lossy(&blocked.stderr);
    let stdout = String::from_utf8_lossy(&blocked.stdout);
    assert!(stderr.contains("not trusted") || stdout.contains("not trusted"));

    let add = trust_json(&home, &project, &["trust", "add", "--format", "json"]);
    assert_eq!(add["result"]["state"], "trusted");
    assert_eq!(add["result"]["trusted"], true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(&home).unwrap().permissions().mode() & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(home.join("project-trust.json"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
    assert_eq!(
        add["result"]["executable_programs"][0],
        fs::canonicalize(reason_bin()).unwrap().to_str().unwrap()
    );

    let allowed = offline_run(&home, &project, &[]);
    assert!(
        allowed.status.success(),
        "{}",
        String::from_utf8_lossy(&allowed.stderr)
    );

    let list = trust_json(&home, &project, &["trust", "list", "--format", "json"]);
    assert_eq!(list["result"]["projects"].as_array().unwrap().len(), 1);
}

#[test]
fn changed_project_executable_bytes_invalidate_existing_trust() {
    let sandbox = TestDir::new("executable-content-stale");
    let project = sandbox.path().join("project");
    let home = sandbox.path().join("home");
    fs::create_dir_all(&project).unwrap();
    fs::create_dir_all(&home).unwrap();

    let source = PathBuf::from(reason_bin());
    let name = if cfg!(windows) {
        "project-tool.exe"
    } else {
        "project-tool"
    };
    let executable = project.join(name);
    fs::copy(&source, &executable).unwrap();
    let configured = format!("./{name}");
    write_project_config(
        &project,
        acquisition_resolution(&configured, "marker-a"),
        json!({}),
    );

    let add = trust_json(&home, &project, &["trust", "add", "--format", "json"]);
    assert_eq!(add["result"]["state"], "trusted");
    let first_digest = add["result"]["executable_identities"][0]["sha256"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(first_digest.starts_with("sha256:"));

    let mut bytes = fs::read(&executable).unwrap();
    bytes.push(0);
    fs::write(&executable, bytes).unwrap();

    let status = trust_json(&home, &project, &["trust", "status", "--format", "json"]);
    assert_eq!(status["result"]["state"], "stale");
    assert_eq!(status["result"]["trusted"], false);
    assert_ne!(
        status["result"]["executable_identities"][0]["sha256"],
        first_digest
    );
    assert!(!offline_run(&home, &project, &[]).status.success());
}

#[test]
fn changed_high_risk_config_invalidates_existing_trust() {
    let sandbox = TestDir::new("stale");
    let project = sandbox.path().join("project");
    let home = sandbox.path().join("home");
    fs::create_dir_all(&project).unwrap();
    fs::create_dir_all(&home).unwrap();
    write_project_config(
        &project,
        acquisition_resolution(reason_bin(), "marker-a"),
        json!({}),
    );
    trust_json(&home, &project, &["trust", "add", "--format", "json"]);

    write_project_config(
        &project,
        acquisition_resolution(reason_bin(), "marker-b"),
        json!({}),
    );
    let status = trust_json(&home, &project, &["trust", "status", "--format", "json"]);
    assert_eq!(status["result"]["state"], "stale");
    assert_eq!(status["result"]["trusted"], false);

    let blocked = offline_run(&home, &project, &[]);
    assert!(!blocked.status.success());
    assert!(String::from_utf8_lossy(&blocked.stdout).contains("trust is stale"));
}

#[test]
fn revoke_and_no_config_are_fail_closed_and_hermetic() {
    let sandbox = TestDir::new("revoke");
    let project = sandbox.path().join("project");
    let home = sandbox.path().join("home");
    fs::create_dir_all(&project).unwrap();
    fs::create_dir_all(&home).unwrap();
    write_project_config(
        &project,
        acquisition_resolution(reason_bin(), "marker-a"),
        json!({}),
    );
    trust_json(&home, &project, &["trust", "add", "--format", "json"]);
    let revoked = trust_json(&home, &project, &["trust", "revoke", "--format", "json"]);
    assert_eq!(revoked["result"]["revoked"], 1);
    assert!(!offline_run(&home, &project, &[]).status.success());
    assert!(
        offline_run(&home, &project, &["--no-config"])
            .status
            .success()
    );
}

#[test]
fn moved_project_does_not_inherit_path_bound_trust() {
    let sandbox = TestDir::new("moved");
    let original = sandbox.path().join("project-a");
    let moved = sandbox.path().join("project-b");
    let home = sandbox.path().join("home");
    fs::create_dir_all(&original).unwrap();
    fs::create_dir_all(&home).unwrap();
    write_project_config(
        &original,
        acquisition_resolution(reason_bin(), "marker-a"),
        json!({}),
    );
    trust_json(&home, &original, &["trust", "add", "--format", "json"]);
    fs::rename(&original, &moved).unwrap();

    let status = trust_json(&home, &moved, &["trust", "status", "--format", "json"]);
    assert_eq!(status["result"]["state"], "untrusted");
    assert!(!offline_run(&home, &moved, &[]).status.success());
}

#[test]
fn trusted_command_is_never_authorized_by_project_trust() {
    let sandbox = TestDir::new("trusted-command");
    let project = sandbox.path().join("project");
    let home = sandbox.path().join("home");
    fs::create_dir_all(&project).unwrap();
    fs::create_dir_all(&home).unwrap();
    write_project_config(
        &project,
        json!({
            "trusted_command": {
                "trusted": true,
                "verifier_id": "project-oracle",
                "program": reason_bin(),
                "args": [],
                "timeout_ms": 1000,
                "max_response_bytes": 4096
            }
        }),
        json!({}),
    );

    let add = run_reason(&home, &project, &["trust", "add", "--format", "json"]);
    assert!(!add.status.success());
    assert!(String::from_utf8_lossy(&add.stdout).contains("cannot be trusted from project config"));
    assert!(!offline_run(&home, &project, &[]).status.success());
}

#[cfg(unix)]
#[test]
fn canonical_project_identity_treats_symlink_alias_as_same_project() {
    use std::os::unix::fs::symlink;

    let sandbox = TestDir::new("symlink");
    let real = sandbox.path().join("real-project");
    let alias = sandbox.path().join("alias-project");
    let home = sandbox.path().join("home");
    fs::create_dir_all(&real).unwrap();
    fs::create_dir_all(&home).unwrap();
    write_project_config(
        &real,
        acquisition_resolution(reason_bin(), "marker-a"),
        json!({}),
    );
    symlink(&real, &alias).unwrap();

    let add = trust_json(&home, &alias, &["trust", "add", "--format", "json"]);
    assert_eq!(add["result"]["state"], "trusted");
    let status = trust_json(&home, &real, &["trust", "status", "--format", "json"]);
    assert_eq!(status["result"]["state"], "trusted");
    assert_eq!(
        add["result"]["project_path"],
        status["result"]["project_path"]
    );
}

#[cfg(unix)]
#[test]
fn trust_store_is_user_only_on_unix() {
    use std::os::unix::fs::PermissionsExt;

    let sandbox = TestDir::new("trust-store-mode");
    let project = sandbox.path().join("project");
    let home = sandbox.path().join("home");
    fs::create_dir_all(&project).unwrap();
    fs::create_dir_all(&home).unwrap();
    write_project_config(
        &project,
        acquisition_resolution(reason_bin(), "marker-a"),
        json!({}),
    );
    trust_json(&home, &project, &["trust", "add", "--format", "json"]);

    let metadata = fs::metadata(home.join("project-trust.json")).unwrap();
    assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
}

#[cfg(unix)]
#[test]
fn project_relative_executable_symlink_escape_is_rejected() {
    use std::os::unix::fs::symlink;

    let sandbox = TestDir::new("program-symlink-escape");
    let project = sandbox.path().join("project");
    let home = sandbox.path().join("home");
    fs::create_dir_all(&project).unwrap();
    fs::create_dir_all(&home).unwrap();
    symlink(reason_bin(), project.join("tool")).unwrap();
    write_project_config(
        &project,
        acquisition_resolution("./tool", "marker-a"),
        json!({}),
    );

    let status = run_reason(&home, &project, &["trust", "status", "--format", "json"]);
    assert!(!status.status.success());
    let output = String::from_utf8_lossy(&status.stdout);
    assert!(
        output.contains("resolves outside the project root"),
        "{output}"
    );
}

#[cfg(unix)]
#[test]
fn project_config_symlink_escape_is_rejected() {
    use std::os::unix::fs::symlink;

    let sandbox = TestDir::new("config-symlink-escape");
    let project = sandbox.path().join("project");
    let outside = sandbox.path().join("outside-config.json");
    let home = sandbox.path().join("home");
    fs::create_dir_all(project.join(".reason")).unwrap();
    fs::create_dir_all(&home).unwrap();
    fs::write(
        &outside,
        serde_json::to_vec_pretty(&json!({
            "schema_version":"reason-config-v1",
            "resolution": acquisition_resolution(reason_bin(), "marker-a")
        }))
        .unwrap(),
    )
    .unwrap();
    symlink(&outside, project.join(".reason/config.json")).unwrap();

    let status = run_reason(&home, &project, &["trust", "status", "--format", "json"]);
    assert!(!status.status.success());
    assert!(String::from_utf8_lossy(&status.stdout).contains("resolves outside"));
}
