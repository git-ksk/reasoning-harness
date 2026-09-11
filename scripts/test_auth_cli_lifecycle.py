#!/usr/bin/env python3
import json
import os
import platform
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REASON = ROOT / "target" / "debug" / ("reason.exe" if platform.system() == "Windows" else "reason")
SECRETS = [
    "reason-auth-ci-secret-one-7Qv9",
    "reason-auth-ci-secret-two-4Lm2",
    "reason-auth-ci-env-secret-8Px5",
]
PROVIDER_ENV = ["MISTRAL_API_KEY", "GEMINI_API_KEY", "GROQ_API_KEY", "NVIDIA_API_KEY"]


def clean_env(extra=None):
    env = os.environ.copy()
    for name in PROVIDER_ENV:
        env.pop(name, None)
    if extra:
        env.update(extra)
    return env


def invoke(args, *, secret=None, env=None, expected=0):
    completed = subprocess.run(
        [str(REASON), *args],
        input=(secret + "\n") if secret is not None else None,
        text=True,
        capture_output=True,
        env=env or clean_env(),
        check=False,
    )
    combined = completed.stdout + completed.stderr
    for value in SECRETS:
        assert value not in combined, f"secret leaked for {args!r}"
    assert completed.returncode == expected, (
        args,
        completed.returncode,
        completed.stdout,
        completed.stderr,
    )
    if "--format" in args and "json" in args:
        assert completed.stderr == "", (args, completed.stderr)
        return json.loads(completed.stdout)
    return None


def auth_result(args, **kwargs):
    doc = invoke(args, **kwargs)
    assert doc["schema_version"] == "reason-cli-output-v1"
    assert doc["command"] == "auth"
    return doc["result"]


def provider_row(result):
    rows = result["providers"]
    assert len(rows) == 1
    row = rows[0]
    assert row["provider"] == "mistral"
    assert row["account"] == "provider:mistral:account:default"
    assert row["environment_variable"] == "MISTRAL_API_KEY"
    return row


def main():
    assert REASON.is_file(), REASON
    # Ephemeral CI runners should start clean, but make cleanup idempotent and repeat it in finally.
    invoke(["auth", "logout", "mistral", "--format", "json"], env=clean_env())
    try:
        first = auth_result(
            ["auth", "login", "mistral", "--stdin", "--format", "json"],
            secret=SECRETS[0],
            env=clean_env(),
        )
        assert first["operation"] == "login"
        assert first["stored"] is True
        assert first["replaced"] is False
        assert first["effective_source"] == "os_store"

        row = provider_row(
            auth_result(["auth", "status", "mistral", "--format", "json"], env=clean_env())
        )
        assert row["environment"] == "missing"
        assert row["os_store"] == "present"
        assert row["effective_source"] == "os_store"

        duplicate = auth_result(
            ["auth", "login", "mistral", "--stdin", "--format", "json"],
            secret=SECRETS[1],
            env=clean_env(),
            expected=1,
        )
        assert duplicate["status"] == "failed"
        assert duplicate["failure"]["failure_class"] == "credential_exists"

        replaced = auth_result(
            ["auth", "login", "mistral", "--stdin", "--replace", "--format", "json"],
            secret=SECRETS[1],
            env=clean_env(),
        )
        assert replaced["stored"] is True
        assert replaced["replaced"] is True
        assert replaced["effective_source"] == "os_store"

        override_env = clean_env({"MISTRAL_API_KEY": SECRETS[2]})
        row = provider_row(
            auth_result(["auth", "status", "mistral", "--format", "json"], env=override_env)
        )
        assert row["environment"] == "present"
        assert row["os_store"] == "present"
        assert row["effective_source"] == "environment"

        logged_out = auth_result(
            ["auth", "logout", "mistral", "--format", "json"], env=override_env
        )
        assert logged_out["operation"] == "logout"
        assert logged_out["removed"] is True
        assert logged_out["effective_source"] == "environment"

        row = provider_row(
            auth_result(["auth", "status", "mistral", "--format", "json"], env=clean_env())
        )
        assert row["environment"] == "missing"
        assert row["os_store"] == "missing"
        assert row["effective_source"] == "missing"

        copied = auth_result(
            ["auth", "login", "mistral", "--from-env", "--format", "json"],
            env=override_env,
        )
        assert copied["stored"] is True
        assert copied["effective_source"] == "environment"

        listing = auth_result(["auth", "list", "--format", "json"], env=clean_env())
        assert len(listing["providers"]) == 4
        assert {row["provider"] for row in listing["providers"]} == {
            "mistral", "google", "groq", "nvidia"
        }
    finally:
        invoke(["auth", "logout", "mistral", "--format", "json"], env=clean_env())


if __name__ == "__main__":
    main()
