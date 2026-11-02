"""Prospective v3 ordinary-CLI measurement contracts. No live/runtime admission."""
import hashlib
import os
import json
import re
import stat
from pathlib import Path

PLAN_ID = "reason-cli-normal-v3-capture-manifest-v1"
OBS_ID = "reason-cli-normal-v3-observation-v1"
PACKET_ID = "reason-cli-normal-v3-review-packet-v1"
REVIEW_ID = "reason-cli-normal-v3-independent-review-v1"
SCORE_ID = "reason-cli-normal-v3-score-v1"
PHASES = frozenset((
    "initial_candidate", "candidate_regeneration", "investigation_plan",
    "investigation_intent", "investigation_action", "final_render",
))
FILES = ("cli.json", "trace.json", "http.json", "capture.json", "sources.json")
HEX = re.compile(r"^[a-f0-9]{64}$")


def require(condition, reason):
    if not condition:
        raise ValueError(reason)


def digest(blob):
    return hashlib.sha256(blob).hexdigest()


def canonical(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()


def keyed_digest(value):
    return digest(canonical(value))


def sha_field(value, label):
    require(isinstance(value, str) and HEX.fullmatch(value) is not None, label)
    return value


def read_json(path):
    return json.loads(Path(path).read_bytes())


def _open_trusted_directory(path):
    """Open a 0700 same-user directory without following the final symlink."""
    path = Path(path)
    fd = os.open(path, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    try:
        metadata = os.fstat(fd)
        require(
            stat.S_ISDIR(metadata.st_mode)
            and metadata.st_uid == os.getuid()
            and stat.S_IMODE(metadata.st_mode) == 0o700,
            "private_directory_owner_or_mode_unsafe",
        )
        return fd
    except BaseException:
        os.close(fd)
        raise


def private_new_directory(path):
    """Create exclusive 0700 evidence directory, regardless of process umask."""
    path = Path(path)
    # Parent cannot be the final symlink; a parent writable by other users
    # must be a sticky directory owned by root (e.g. /private/tmp).
    parent_fd = os.open(path.parent, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    try:
        parent = os.fstat(parent_fd)
        mode = parent.st_mode
        require(
            stat.S_ISDIR(mode)
            and (
                (parent.st_uid == os.getuid() and mode & 0o022 == 0)
                or (parent.st_uid == 0 and mode & stat.S_ISVTX != 0)
            ),
            "private_output_parent_untrusted",
        )
        os.mkdir(path.name, mode=0o700, dir_fd=parent_fd)
        output_fd = os.open(path.name, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW,
                            dir_fd=parent_fd)
        try:
            os.fchmod(output_fd, 0o700)
            os.fsync(output_fd)
        finally:
            os.close(output_fd)
        os.fsync(parent_fd)
    finally:
        os.close(parent_fd)


def exclusive_json(path, value):
    """Create private 0600 JSON using parent dirfd, O_EXCL and NOFOLLOW."""
    path = Path(path)
    dirfd = _open_trusted_directory(path.parent)
    try:
        fd = os.open(path.name,
                     os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW,
                     0o600, dir_fd=dirfd)
        with os.fdopen(fd, "w", encoding="utf-8") as stream:
            os.fchmod(stream.fileno(), 0o600)
            json.dump(value, stream, ensure_ascii=False, indent=2, sort_keys=True)
            stream.write("\n")
            stream.flush()
            os.fsync(stream.fileno())
        os.fsync(dirfd)
    finally:
        os.close(dirfd)


def strict_keys(value, allowed, required, name):
    require(type(value) is dict, name + "_not_object")
    require(set(required) <= value.keys(), name + "_missing_keys")
    require(value.keys() <= set(allowed), name + "_unexpected_keys")
    return value


def safe_id(value, name):
    require(
        type(value) is str
        and 0 < len(value) <= 120
        and re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._-]*", value) is not None
        and value not in (".", ".."),
        name,
    )
    return value


def unsigned(value, name):
    require(type(value) is int and value >= 0, name)
    return value


def positive(value, name):
    require(type(value) is int and value > 0, name)
    return value


def exclusive_durable_claim(path, claim):
    """Durable receipt O_EXCL barrier in a trusted private ledger, no unlock."""
    path = Path(path)
    dirfd = _open_trusted_directory(path.parent)
    try:
        data = canonical(claim) + b"\n"
        fd = os.open(path.name,
                     os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW,
                     0o600, dir_fd=dirfd)
        with os.fdopen(fd, "wb") as stream:
            os.fchmod(stream.fileno(), 0o600)
            stream.write(data)
            stream.flush()
            os.fsync(stream.fileno())
        os.fsync(dirfd)
    finally:
        os.close(dirfd)
