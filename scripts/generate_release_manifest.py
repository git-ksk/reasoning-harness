#!/usr/bin/env python3
from __future__ import annotations

import argparse, hashlib, json
from pathlib import Path

SCHEMA = "reason-release-manifest-v1"
REPOSITORY = "git-ksk/reasoning-harness"
SIGNER_WORKFLOW = "git-ksk/reasoning-harness/.github/workflows/release-cli.yml"


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def build_manifest(dist: Path, tag: str, cli_version: str, engine_version: str, commit: str) -> dict:
    expected_prefix = f"reason-v{cli_version}-"
    artifacts = []
    for path in sorted(p for p in dist.iterdir() if p.is_file() and p.name != "release-manifest.json"):
        if path.name.startswith(expected_prefix) or path.name in {"SHA256SUMS", "install.sh", "install.ps1"}:
            artifacts.append({"name": path.name, "sha256": sha256(path), "size_bytes": path.stat().st_size})
    archives = [a for a in artifacts if a["name"].startswith(expected_prefix) and a["name"].endswith((".tar.gz", ".zip"))]
    if len(archives) != 4:
        raise SystemExit(f"expected 4 native archives, found {len(archives)}")
    return {
        "schema_version": SCHEMA,
        "repository": REPOSITORY,
        "signer_workflow": SIGNER_WORKFLOW,
        "tag": tag,
        "cli_version": cli_version,
        "engine_version": engine_version,
        "git_commit": commit,
        "artifacts": artifacts,
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--dist", type=Path, required=True)
    parser.add_argument("--tag", required=True)
    parser.add_argument("--cli-version", required=True)
    parser.add_argument("--engine-version", required=True)
    parser.add_argument("--commit", required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.tag != f"reason-v{args.cli_version}":
        raise SystemExit("release manifest tag must exactly match reason-v<cli-version>")
    manifest = build_manifest(args.dist, args.tag, args.cli_version, args.engine_version, args.commit)
    args.output.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
