"""Provider-free paired build using the original frozen Core patches."""
import argparse
import importlib.util
import json
from pathlib import Path
import subprocess

CLI = "7cf783d2169cd7882832fd780070ab7280ad63c0"
OLD = "b85a4a0b2ab03397b361a8620d7a97dd26b2eb36"
CHANGED = {
    "crates/reasoning-harness-cli/src/generation_usage.rs",
    "crates/reasoning-harness-cli/src/main.rs",
    "crates/reasoning-harness-cli/src/usage_budget.rs",
    "crates/reasoning-harness-providers/src/lib.rs",
    "crates/reasoning-harness-providers/src/mistral.rs",
    "crates/reasoning-harness-providers/src/mistral_retry_budget.rs",
}
HERE = Path(__file__).resolve().parent


def build(repo, output, compile_binaries):
    spec = importlib.util.spec_from_file_location(
        "frozen_pair_builder",
        HERE.parent / "engine073-531-independent24-20261116/build_completeness_pair.py",
    )
    original = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(original)
    diff = subprocess.check_output(
        ["git", "diff", "--name-only", OLD, CLI, "--", "Cargo.toml", "Cargo.lock", "crates"],
        cwd=repo,
    ).decode().splitlines()
    if set(diff) != CHANGED:
        raise ValueError("shared_cli_source_scope")
    original.CLI_COMMIT = CLI
    return original.prepare(repo, output, compile_binaries)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--mode", choices=("validate-only", "build"), default="validate-only")
    args = parser.parse_args()
    print(json.dumps(build(args.repo, args.output, args.mode == "build"), sort_keys=True))
