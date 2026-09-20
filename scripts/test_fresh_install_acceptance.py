#!/usr/bin/env python3
"""Pre-tag fresh-install acceptance for a packaged Reason CLI release candidate.

This intentionally exercises only the extracted native binary. It never invokes Cargo/Rust
and never performs a live provider call. Provider-success semantics and interactive/session
internals remain covered by the existing deterministic workspace contracts; this script is the
fresh-machine product/distribution lane that proves those contracts are reachable from the
packaged CLI without a Rust toolchain.
"""
from __future__ import annotations

import argparse
import json
import os
import platform
import shutil
import stat
import subprocess
import tarfile
import tempfile
import tomllib
import zipfile
from pathlib import Path

ENGINE_VERSION = "0.5.0"
SECRETS = (
    "reason-phase5-mistral-secret-7Fz3-not-real",
    "reason-phase5-google-secret-4Qm8-not-real",
)
PROVIDER_ENV = ("MISTRAL_API_KEY", "GEMINI_API_KEY", "GROQ_API_KEY", "NVIDIA_API_KEY")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--archive", required=True, type=Path)
    parser.add_argument("--asset", required=True)
    parser.add_argument("--repo-root", type=Path, default=Path(__file__).resolve().parents[1])
    return parser.parse_args()


def current_cli_version(repo_root: Path) -> str:
    manifest = repo_root / "crates" / "reasoning-harness-cli" / "Cargo.toml"
    with manifest.open("rb") as handle:
        document = tomllib.load(handle)
    version = document.get("package", {}).get("version")
    assert isinstance(version, str) and version, manifest
    return version


def safe_extract(archive: Path, destination: Path) -> None:
    destination = destination.resolve()
    if archive.suffix == ".zip":
        with zipfile.ZipFile(archive) as zf:
            for info in zf.infolist():
                target = (destination / info.filename).resolve()
                if destination not in target.parents and target != destination:
                    raise AssertionError(f"unsafe zip path: {info.filename}")
            zf.extractall(destination)
        return
    if archive.name.endswith(".tar.gz"):
        with tarfile.open(archive, "r:gz") as tf:
            for member in tf.getmembers():
                target = (destination / member.name).resolve()
                if destination not in target.parents and target != destination:
                    raise AssertionError(f"unsafe tar path: {member.name}")
            tf.extractall(destination, filter="data")
        return
    raise AssertionError(f"unsupported archive: {archive}")


def no_rust_path(bin_dir: Path) -> str:
    if os.name == "nt":
        system_root = Path(os.environ.get("SystemRoot", r"C:\Windows"))
        entries = [bin_dir, system_root / "System32", system_root, system_root / "System32" / "Wbem"]
        return os.pathsep.join(str(path) for path in entries)
    return os.pathsep.join([str(bin_dir), "/usr/bin", "/bin", "/usr/sbin", "/sbin"])


def base_env(home: Path, bin_dir: Path) -> dict[str, str]:
    env = os.environ.copy()
    for key in PROVIDER_ENV:
        env.pop(key, None)
    env["REASON_HOME"] = str(home)
    env["PATH"] = no_rust_path(bin_dir)
    # Keep diagnostics deterministic and prohibit accidental network routing through CI/user proxies.
    for key in (
        "HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY", "NO_PROXY",
        "http_proxy", "https_proxy", "all_proxy", "no_proxy", "REASON_CA_BUNDLE",
    ):
        env.pop(key, None)
    return env


def assert_secret_free(text: str, where: str) -> None:
    for secret in SECRETS:
        assert secret not in text, f"secret leaked in {where}"


def invoke(reason: Path, args: list[str], *, cwd: Path, env: dict[str, str], expected: int = 0) -> subprocess.CompletedProcess[str]:
    completed = subprocess.run(
        [str(reason), *args],
        cwd=cwd,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )
    assert_secret_free(completed.stdout, f"stdout for {args!r}")
    assert_secret_free(completed.stderr, f"stderr for {args!r}")
    assert completed.returncode == expected, (
        args,
        completed.returncode,
        completed.stdout,
        completed.stderr,
    )
    return completed


def json_invoke(reason: Path, args: list[str], *, cwd: Path, env: dict[str, str], expected: int = 0) -> dict:
    completed = invoke(reason, args, cwd=cwd, env=env, expected=expected)
    assert completed.stderr == "", (args, completed.stderr)
    document = json.loads(completed.stdout)
    assert document["schema_version"] == "reason-cli-output-v1", (args, document)
    return document


def scan_tree_for_secrets(root: Path) -> None:
    if not root.exists():
        return
    for path in root.rglob("*"):
        if path.is_file():
            data = path.read_bytes()
            for secret in SECRETS:
                assert secret.encode() not in data, f"secret persisted in {path}"


def assert_private_permissions(home: Path) -> None:
    if os.name != "posix":
        return
    home_mode = stat.S_IMODE(home.stat().st_mode)
    config_mode = stat.S_IMODE((home / "config.json").stat().st_mode)
    assert home_mode == 0o700, oct(home_mode)
    assert config_mode == 0o600, oct(config_mode)


def main() -> None:
    args = parse_args()
    archive = args.archive.resolve()
    repo_root = args.repo_root.resolve()
    cli_version = current_cli_version(repo_root)
    assert archive.is_file(), archive
    assert (repo_root / "examples" / "input.json").is_file(), repo_root

    with tempfile.TemporaryDirectory(prefix="reason-phase5-fresh-") as temp_raw:
        temp = Path(temp_raw)
        unpack = temp / "unpack"
        unpack.mkdir()
        safe_extract(archive, unpack)

        executable_name = "reason.exe" if os.name == "nt" else "reason"
        package_root = unpack / f"reason-v{cli_version}-{args.asset}"
        reason = package_root / executable_name
        assert reason.is_file(), reason
        if os.name != "nt":
            reason.chmod(reason.stat().st_mode | stat.S_IXUSR)

        home = temp / "home"
        workspace = temp / "workspace"
        home.mkdir(mode=0o700)
        workspace.mkdir()
        env = base_env(home, package_root)

        # The consumer process intentionally has no Cargo/Rust path. Hosted runner images may
        # contain Rust elsewhere, but the installed CLI cannot depend on it in this lane.
        assert shutil.which("cargo", path=env["PATH"]) is None, env["PATH"]
        assert shutil.which("rustc", path=env["PATH"]) is None, env["PATH"]

        version = invoke(reason, ["--version"], cwd=workspace, env=env).stdout.strip()
        assert version == f"reason {cli_version}", version

        # Empty-home setup uses an environment credential path (documented for headless/CI),
        # never a secret-valued argv. Live readiness is deliberately skipped to spend zero quota.
        setup_env = dict(env)
        setup_env["MISTRAL_API_KEY"] = SECRETS[0]
        setup = json_invoke(
            reason,
            ["setup", "--provider", "mistral", "--non-interactive", "--skip-live-check", "--format", "json"],
            cwd=workspace,
            env=setup_env,
        )
        result = setup["result"]
        assert setup["command"] == "setup"
        assert result["provider"] == "mistral"
        assert result["model"] == "ministral-8b-latest"
        assert result["credential_source"] == "environment"
        assert result["local_readiness"] == "passed"
        assert result["live_readiness"] == "skipped"
        assert result["live_provider_attempts"] == 0
        assert_private_permissions(home)
        scan_tree_for_secrets(home)

        # Explicit provider/model selection and config inspection. Switch away and back so the
        # acceptance catches silent-fallback/default-rewrite regressions without a provider call.
        switch_env = dict(setup_env)
        switch_env["GEMINI_API_KEY"] = SECRETS[1]
        selected = json_invoke(
            reason,
            ["model", "set", "google", "gemini-3.5-flash-lite", "--format", "json"],
            cwd=workspace,
            env=switch_env,
        )
        assert selected["command"] == "model"
        assert selected["result"]["provider"] == "google"
        assert selected["result"]["model"] == "gemini-3.5-flash-lite"
        assert selected["result"]["availability"] == "current"
        configured = json_invoke(
            reason, ["models", "--configured", "--format", "json"], cwd=workspace, env=switch_env
        )
        assert configured["result"]["configured_default"]["provider"] == "google"
        assert configured["result"]["configured_default"]["model"] == "gemini-3.5-flash-lite"
        json_invoke(reason, ["config", "list", "--format", "json"], cwd=workspace, env=switch_env)
        json_invoke(
            reason,
            ["model", "set", "mistral", "ministral-8b-latest", "--format", "json"],
            cwd=workspace,
            env=switch_env,
        )
        scan_tree_for_secrets(home)

        # Doctor is local-only by default and must report separate product/engine identities.
        doctor = json_invoke(reason, ["doctor", "--format", "json"], cwd=workspace, env=setup_env)
        assert doctor["command"] == "doctor"
        assert doctor["result"]["doctor_surface"] == "reason-doctor-v1"
        assert doctor["result"]["versions"]["cli"] == cli_version
        assert doctor["result"]["versions"]["engine"] == ENGINE_VERSION
        assert doctor["result"]["network"]["live_probe"] == "skipped"

        # One-shot natural-language dispatch through configured defaults, forced into the typed
        # missing-credential recovery path. Empty env is an explicit no-store-fallback sentinel,
        # so this cannot consume quota or accidentally read a developer credential.
        failure_env = dict(env)
        failure_env["MISTRAL_API_KEY"] = ""
        failed = json_invoke(
            reason, ["--format", "json", "Check this claim"], cwd=workspace, env=failure_env, expected=1
        )
        assert failed["result"]["status"] == "failed"
        assert failed["result"]["failure"]["failure_class"] == "credentials"
        remediation = failed["result"].get("remediation") or {}
        assert remediation.get("next_command"), failed

        # Existing non-interactive machine contracts must remain usable from the packaged binary.
        run_doc = json_invoke(
            reason,
            [
                "run", "--input", str(repo_root / "examples" / "input.json"),
                "--candidate", str(repo_root / "examples" / "candidate.json"),
                "--no-config", "--format", "json",
            ],
            cwd=workspace,
            env=env,
        )
        assert run_doc["command"] == "run"
        assert run_doc["result"]["outcome"]["verdict"] in {"accept", "reject", "unknown"}
        verify_doc = json_invoke(
            reason,
            ["verify", str(repo_root / "examples" / "artifact.json"), "--format", "json"],
            cwd=workspace,
            env=env,
        )
        assert verify_doc["command"] == "verify"
        assert verify_doc["result"]["valid"] is True

        # Lifecycle is exercised without publishing/mutating a release: historical rollback must
        # fail closed at the attestation boundary; uninstall dry-run must retain binary/data.
        rollback = json_invoke(
            reason,
            ["update", "--rollback", "0.4.2", "--yes", "--format", "json"],
            cwd=workspace,
            env=env,
            expected=1,
        )
        assert rollback["result"]["failure"]["failure_class"] == "historical_release_boundary"
        uninstall = json_invoke(
            reason,
            ["uninstall", "--dry-run", "--purge-data", "--format", "json"],
            cwd=workspace,
            env=env,
        )
        assert uninstall["command"] == "uninstall"
        assert uninstall["result"]["dry_run"] is True
        assert uninstall["result"]["binary_removed"] is False
        assert reason.is_file()

        scan_tree_for_secrets(home)
        print(json.dumps({
            "status": "passed",
            "asset": args.asset,
            "cli_version": cli_version,
            "engine_version": ENGINE_VERSION,
            "live_provider_calls": 0,
            "rust_toolchain_required": False,
            "secret_leakage": False,
        }, sort_keys=True))


if __name__ == "__main__":
    main()
