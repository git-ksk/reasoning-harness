"""Prepare an explicit private CI source snapshot without inherited checkout ACL.

This does not relax admission or repair the checkout. Copy only bytes into new
0700/0600 (0700 executable) paths, then run normal native enrollment there.
Never used implicitly by a receipt/job or by the production NativeBundle.
"""
import argparse
import os
from pathlib import Path
import stat
import shutil
import tempfile

from contracts import require, _open_trusted_directory

IGNORE = frozenset((".git", "target", "__pycache__", "node_modules", ".venv", ".aws", ".ssh", ".gcloud", ".native-bin"))


def _copy(source, destination):
    fd = os.open(source, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    try:
        before = os.fstat(fd)
        require(stat.S_ISREG(before.st_mode), "source_snapshot_requires_regular_file")
        mode = 0o700 if before.st_mode & 0o111 else 0o600
        out = os.open(destination, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, mode)
        try:
            os.fchmod(out, mode)
            while True:
                data = os.read(fd, 1024 * 1024)
                if not data: break
                offset = 0
                while offset < len(data): offset += os.write(out, data[offset:])
            os.fsync(out)
        finally: os.close(out)
        after = os.fstat(fd)
        require((before.st_dev, before.st_ino, before.st_size, before.st_mtime_ns) ==
                (after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns), "source_changed_during_snapshot")
    finally: os.close(fd)


def prepare_snapshot(source, binary):
    source = Path(source).resolve(strict=True)
    root = Path(tempfile.mkdtemp(prefix="reason-v3-ci-source-")).resolve()
    private = _open_trusted_directory(root); os.close(private)
    try:
        _populate(source, binary, root)
        return root
    except BaseException:
        shutil.rmtree(root)
        raise


def _populate(source, binary, root):
    # os.walk does not follow symlinks. Do not use copytree/copy2: their
    # copystat also copies Linux ACL xattrs from the checkout.
    for parent, names, files in os.walk(source, followlinks=False):
        names[:] = [n for n in names if n not in IGNORE and not n.startswith(".local")]
        relative = Path(parent).relative_to(source)
        destination = root / relative
        for name in names:
            require(not (Path(parent) / name).is_symlink(), "source_snapshot_symlink")
            os.mkdir(destination / name, mode=0o700)
            os.chmod(destination / name, 0o700)
        for name in files:
            if name in IGNORE or name.startswith(".local"): continue
            _copy(Path(parent) / name, destination / name)
    binaries = root / ".native-bin"; binaries.mkdir(mode=0o700)
    _copy(Path(binary).resolve(strict=True), binaries / "reason")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--source", required=True, type=Path)
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--pointer", required=True, type=Path)
    args = parser.parse_args()
    root = prepare_snapshot(args.source, args.binary)
    fd = os.open(args.pointer, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
    with os.fdopen(fd, "w") as stream:
        stream.write(str(root) + "\n"); stream.flush(); os.fsync(stream.fileno())


if __name__ == "__main__": main()
