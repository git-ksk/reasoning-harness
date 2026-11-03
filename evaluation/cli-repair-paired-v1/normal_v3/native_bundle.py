"""Fixed operator-owned native execution identity. No cloud/image dependency.

Enrollment is explicit; receipts, jobs and captured output cannot enroll a new
root or select replacement files. The owner and OS remain trusted.
"""
import json
import os
from pathlib import Path
import platform
import stat
import sys
from dataclasses import dataclass

from contracts import canonical, digest, require, _open_safe_directory_path
from authority import AuthorityBinding

BUNDLE = "reason-cli-normal-v3-native-bundle-v1"
SOURCE_DIRS = ("crates", "fixtures", "examples", "scripts", "evaluation/cli-repair-paired-v1/normal_v3")
SOURCE_FILES = ("Cargo.toml", "Cargo.lock")
IGNORED_PARTS = frozenset(("__pycache__", ".git", "target"))


def _runtime_parent(path):
    # Homebrew's Cellar is commonly group-writable by local administrators.
    # This profile trusts OS/runtime administrators, but never world-writable
    # non-sticky paths. Private evidence/source paths retain the stricter rule.
    fd = os.open("/", os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    try:
        for part in Path(path).parts[1:]:
            child = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=fd)
            os.close(fd); fd = child
            meta = os.fstat(fd)
            require(meta.st_uid in (0, os.getuid()) and (not meta.st_mode & 0o002
                    or (meta.st_uid == 0 and meta.st_mode & stat.S_ISVTX)), "unsafe_native_runtime_parent")
        return fd
    except BaseException:
        os.close(fd); raise


def measured_file(path, *, runtime=False):
    path = Path(path)
    require(path.is_absolute(), "absolute_native_artifact_required")
    parent = _runtime_parent(path.parent) if runtime else _open_safe_directory_path(path.parent)
    try:
        fd = os.open(path.name, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK, dir_fd=parent)
        try:
            meta = os.fstat(fd)
            require(stat.S_ISREG(meta.st_mode) and meta.st_uid in (0, os.getuid())
                    and not meta.st_mode & 0o022, "unsafe_native_artifact")
            chunks = []
            while True:
                chunk = os.read(fd, 1024 * 1024)
                if not chunk: break
                chunks.append(chunk)
            after = os.fstat(fd)
            require((meta.st_dev, meta.st_ino, meta.st_size, meta.st_mtime_ns) ==
                    (after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns),
                    "native_artifact_changed_while_reading")
            return {"path": str(path), "device": meta.st_dev, "inode": meta.st_ino,
                    "size": meta.st_size, "sha256": digest(b"".join(chunks))}
        finally: os.close(fd)
    finally: os.close(parent)


def source_inventory(root):
    root = Path(root)
    require(root.is_absolute() and str(root.resolve()) == str(root), "canonical_native_source_required")
    names = set()
    for name in SOURCE_FILES:
        require((root / name).is_file(), "native_cargo_source_missing")
        names.add(name)
    for directory in SOURCE_DIRS:
        base = root / directory
        if not base.exists(): continue
        require(not base.is_symlink(), "native_source_symlink")
        for entry in base.rglob("*"):
            relative = entry.relative_to(root)
            if IGNORED_PARTS & set(relative.parts): continue
            require(not entry.is_symlink(), "native_source_symlink")
            if entry.is_file(): names.add(str(relative))
    return sorted(names)


def verifier_runtime_files():
    """Actual interpreter, harness modules and installed verifier artifacts.

    OS/kernel/system libraries are trusted; this is not an OS image hash.
    """
    import nacl
    import _cffi_backend
    files = {"python": Path(sys.executable).resolve(), "cffi-extension": Path(_cffi_backend.__file__).resolve()}
    here = Path(__file__).resolve().parent
    for file in here.glob("*.py"):
        if not file.name.startswith("test_"): files["harness/" + file.name] = file
    files["verifier-lock"] = here / "review-requirements.txt"
    for file in Path(nacl.__file__).parent.rglob("*"):
        if file.is_file() and file.suffix in (".py", ".so", ".dylib", ".pyd"):
            files["nacl/" + str(file.relative_to(Path(nacl.__file__).parent))] = file.resolve()
    venv = Path(sys.prefix) / "pyvenv.cfg"
    if venv.is_file(): files["python-environment"] = venv.resolve()
    return files


@dataclass(frozen=True)
class NativeBundle:
    manifest_bytes: bytes

    @classmethod
    def enroll(cls, *, source_roots, binaries, configurations, runtime_files=None):
        require(set(source_roots) == set(binaries) == {"baseline", "candidate"}, "native_arms_required")
        roots = {arm: str(Path(root).resolve(strict=True)) for arm, root in source_roots.items()}
        inventories = {arm: source_inventory(root) for arm, root in roots.items()}
        files = {}
        def enroll_file(label, path):
            path = Path(path).resolve(strict=True)
            files[label] = measured_file(path, runtime=label.startswith("runtime/"))
        for arm, root in roots.items():
            for name in inventories[arm]: enroll_file("source/" + arm + "/" + name, Path(root) / name)
        for arm, path in binaries.items(): enroll_file("binary/" + arm, path)
        for name, path in configurations.items(): enroll_file("config/" + name, path)
        for name, path in (verifier_runtime_files() if runtime_files is None else runtime_files).items():
            enroll_file("runtime/" + name, path)
        require("runtime/python" in files and "runtime/verifier-lock" in files, "native_runtime_identity_missing")
        data = {"contract": BUNDLE, "scope": "single-host-native-owner-trusted",
                "platform": platform.platform(), "python_version": platform.python_version(),
                "runtime_parent_policy": "owner-and-os-runtime-administrators-trusted",
                "source_roots": roots, "source_inventory": inventories, "files": files}
        return cls(canonical(data))

    @property
    def data(self):
        data = json.loads(self.manifest_bytes)
        require(canonical(data) == self.manifest_bytes and data.get("contract") == BUNDLE,
                "invalid_native_bundle")
        return data

    @property
    def sha256(self): return digest(self.manifest_bytes)

    def binding(self, executor_id, evaluator_id):
        files = self.data["files"]
        def group(prefix): return digest(canonical({k: v for k, v in files.items() if k.startswith(prefix)}))
        return AuthorityBinding(source_sha256=group("source/"), image_sha256=self.sha256,
                                cli_build_sha256=group("binary/"), runtime_sha256=group("runtime/"),
                                executor_id=executor_id, evaluator_id=evaluator_id)

    def verify(self):
        data = self.data
        require(data["platform"] == platform.platform() and data["python_version"] == platform.python_version(),
                "native_runtime_platform_changed")
        for arm, root in data["source_roots"].items():
            require(source_inventory(root) == data["source_inventory"][arm], "native_source_inventory_changed")
        # Deduplicate the same baseline/candidate files without trusting mtime
        # as a content hash. Every admitted byte is re-read for each job.
        observed = {}
        for label, record in data["files"].items():
            path = record["path"]
            key = (path, label.startswith("runtime/"))
            if key not in observed: observed[key] = measured_file(path, runtime=key[1])
            require(observed[key] == record, "native_bundle_artifact_changed")
        return self.sha256

    def binary(self, arm):
        require(arm in ("baseline", "candidate"), "unknown_native_arm")
        return self.data["files"]["binary/" + arm]["path"]
