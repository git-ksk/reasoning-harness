"""Build the two historical cores with one immutable current CLI on this Mac.

No dependency download, model call, receipt, run claim or release. Historical
cores receive only the previously admitted common #552 acquisition patch.
"""
import argparse
import io
import os
from pathlib import Path
import re
import shutil
import subprocess
import tarfile

from contracts import canonical, digest, exclusive_json, private_new_directory, require
from native_bundle import SOURCE_DIRS, SOURCE_FILES, source_inventory

ARMS = {"baseline": "4abee90f0501f0d8fc09b453a0b45c942fbed69e",
        "candidate": "ab3c6322921c8c738ec35bd018345917b5afa538"}
PATCH_FROM = "2f6f88fd1a8b721ef11bcda4b7a1fb00e5691886"
PATCH_TO = "2c4fea29ac9ae7c0447766cd0d83200ae01ea9f1"


def git(repo, *args):
    return subprocess.check_output(["git", *args], cwd=repo)


def export(repo, commit, paths, destination):
    raw = git(repo, "archive", commit, *paths)
    with tarfile.open(fileobj=io.BytesIO(raw)) as archive:
        members = archive.getmembers()
        require(all(not m.name.startswith("/") and ".." not in Path(m.name).parts
                    and (m.isfile() or m.isdir()) for m in members), "unsafe_native_source_archive")
        for member in members:
            path = destination / member.name
            if member.isdir(): path.mkdir(mode=0o700, parents=True, exist_ok=True)
            else:
                path.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
                data = archive.extractfile(member).read()
                fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW,
                             0o700 if member.mode & 0o111 else 0o600)
                with os.fdopen(fd, "wb") as stream: stream.write(data)


def build(repo, commit, output):
    repo, output = Path(repo).resolve(), Path(output).resolve()
    require(re.fullmatch(r"[0-9a-f]{40}", commit) is not None, "immutable_cli_commit_required")
    require(git(repo, "rev-parse", commit + "^{commit}").decode().strip() == commit,
            "unknown_cli_commit")
    private_new_directory(output)
    private_new_directory(output / "bin")
    patch = git(repo, "diff", PATCH_FROM, PATCH_TO, "--", "crates/reasoning-harness-core/src/resolution.rs")
    proof = {"contract": "reason-cli-normal-v3-native-paired-build-v1", "cli_commit": commit,
             "rustc": subprocess.check_output(["rustc", "-vV"]).decode(),
             "build_script_sha256": digest(Path(__file__).read_bytes()),
             "common_resolution_patch_sha256": digest(patch), "arms": {},
             "candidate_target_version": "0.7.3-unpublished", "release_or_adoption": False}
    for arm, pin in ARMS.items():
        source = output / ("source-" + arm); private_new_directory(source)
        export(repo, commit, (*SOURCE_FILES, *SOURCE_DIRS), source)
        core = source / "crates/reasoning-harness-core"
        shutil.rmtree(core)
        export(repo, pin, ("crates/reasoning-harness-core",), source)
        before = {str(p.relative_to(core)): digest(p.read_bytes()) for p in core.rglob("*") if p.is_file()}
        subprocess.run(["patch", "-p1", "--batch", "--forward"], cwd=source, input=patch,
                       check=True, stdout=subprocess.DEVNULL)
        after = {str(p.relative_to(core)): digest(p.read_bytes()) for p in core.rglob("*") if p.is_file()}
        require(before.keys() == after.keys() and [n for n in before if before[n] != after[n]] == ["src/resolution.rs"],
                "unexpected_native_core_patch")
        version = re.search(r'^version = "([^"]+)"', (core / "Cargo.toml").read_text(), re.M).group(1)
        lock = source / "Cargo.lock"
        content, count = re.subn(r'(name = "reasoning-harness-core"\nversion = ")[^"]+("\n)',
                                lambda m: m[1] + version + m[2], lock.read_text())
        require(count == 1, "native_core_lock_inventory_changed"); lock.write_text(content)
        env = {**os.environ, "CARGO_TARGET_DIR": str(output / "target"), "CARGO_INCREMENTAL": "0",
               "CARGO_PROFILE_DEV_DEBUG": "0"}
        with (output / (arm + "-build.log")).open("xb") as log:
            subprocess.run(["cargo", "build", "--locked", "--offline", "-p", "reasoning-harness-cli",
                            "--bin", "reason", "--jobs", "2"], cwd=source, env=env,
                           stdout=log, stderr=subprocess.STDOUT, check=True)
        binary = output / "bin" / ("reason-" + arm)
        shutil.copyfile(output / "target/debug/reason", binary); binary.chmod(0o700)
        proof["arms"][arm] = {"engine_commit": pin, "core_package_version": version,
            "original_core_files": before, "patched_core_files": after,
            "source_files": {name: digest((source / name).read_bytes()) for name in source_inventory(source)},
            "binary_sha256": digest(binary.read_bytes()), "source_root": str(source), "binary": str(binary)}
        print("built " + arm, flush=True)
    # The host/provider bytes are identical; only core files and core lock version differ.
    first, second = (proof["arms"][a]["source_files"] for a in ARMS)
    require({k: v for k, v in first.items() if not k.startswith("crates/reasoning-harness-core/") and k != "Cargo.lock"}
            == {k: v for k, v in second.items() if not k.startswith("crates/reasoning-harness-core/") and k != "Cargo.lock"},
            "paired_native_host_not_identical")
    exclusive_json(output / "build.json", proof)
    return proof


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", type=Path, required=True)
    parser.add_argument("--cli-commit", required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    build(args.repo, args.cli_commit, args.output)
