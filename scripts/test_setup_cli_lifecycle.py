#!/usr/bin/env python3
import json, os, platform, subprocess, tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REASON = ROOT / "target" / "debug" / ("reason.exe" if platform.system() == "Windows" else "reason")
SECRET = "reason-setup-ci-secret-3Jp8"
PROVIDER_ENV = ["MISTRAL_API_KEY", "GEMINI_API_KEY", "GROQ_API_KEY", "NVIDIA_API_KEY"]


def env_for(home):
    env = os.environ.copy()
    for name in PROVIDER_ENV:
        env.pop(name, None)
    env["REASON_HOME"] = str(home)
    return env


def invoke(args, home, secret=None, expected=0):
    completed = subprocess.run(
        [str(REASON), *args],
        input=(secret + "\n") if secret else None,
        text=True,
        capture_output=True,
        env=env_for(home),
        check=False,
    )
    combined = completed.stdout + completed.stderr
    assert SECRET not in combined, (args, "secret leaked")
    assert completed.returncode == expected, (args, completed.returncode, completed.stdout, completed.stderr)
    if "--format" in args and "json" in args:
        assert completed.stderr == "", (args, completed.stderr)
        return json.loads(completed.stdout)
    return None


def result(doc, command):
    assert doc["schema_version"] == "reason-cli-output-v1"
    assert doc["command"] == command
    return doc["result"]


def main():
    assert REASON.is_file(), REASON
    with tempfile.TemporaryDirectory(prefix="reason-setup-ci-") as tmp:
        home = Path(tmp)
        invoke(["auth", "logout", "mistral", "--format", "json"], home)
        try:
            first = result(invoke([
                "setup", "--provider", "mistral", "--non-interactive",
                "--credential-stdin", "--skip-live-check", "--format", "json"
            ], home, secret=SECRET), "setup")
            assert first["provider"] == "mistral"
            assert first["model"] == "ministral-8b-latest"
            assert first["credential_source"] == "os_store"
            assert first["local_readiness"] == "passed"
            assert first["live_readiness"] == "skipped"
            assert first["live_provider_attempts"] == 0
            assert first["first_command"].startswith("reason ")

            config_path = home / "config.json"
            config_text = config_path.read_text()
            assert SECRET not in config_text
            config = json.loads(config_text)
            assert config["run"]["provider"] == "mistral"
            assert config["run"]["model"] == "ministral-8b-latest"

            status = result(invoke(["auth", "status", "mistral", "--format", "json"], home), "auth")
            row = status["providers"][0]
            assert row["os_store"] == "present"
            assert row["effective_source"] == "os_store"

            second = result(invoke([
                "setup", "--provider", "mistral", "--non-interactive",
                "--skip-live-check", "--format", "json"
            ], home), "setup")
            assert second["model"] == "ministral-8b-latest"
            assert second["credential_source"] == "os_store"
            assert second["config_path"] == str(config_path)
        finally:
            invoke(["auth", "logout", "mistral", "--format", "json"], home)


if __name__ == "__main__":
    main()
