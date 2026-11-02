"""Prospective v3 ordinary-CLI measurement contracts. No live/runtime admission."""
import hashlib
import os
import json
import re
import stat
import sys
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
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"), allow_nan=False).encode()


def keyed_digest(value):
    return digest(canonical(value))


def sha_field(value, label):
    require(isinstance(value, str) and HEX.fullmatch(value) is not None, label)
    return value


def frozen_request_digest(*, job_id, case_id, arm, question, request,
                          provider, model, receipt_sha256):
    """Freeze full first-request AND execution identity with one canonical encoder.

    This is local consistency only: an externally authenticated plan/receipt
    is still required to trust these claimed values (#580/#581).
    """
    safe_id(job_id, "invalid_job_id")
    safe_id(case_id, "bad_case")
    require(arm in ("baseline", "candidate"), "bad_arm")
    require(type(question) is str and bool(question.strip()), "missing_frozen_question")
    require(type(request) is dict and type(request.get("task")) is str,
            "invalid_first_model_request")
    require(type(provider) is str and bool(provider)
            and type(model) is str and bool(model), "missing_provider_identity")
    sha_field(receipt_sha256, "missing_receipt_hash")
    return keyed_digest({
        "job_id": job_id, "case_id": case_id, "arm": arm, "question": question,
        "request": request, "provider": provider, "model": model,
        "receipt_sha256": receipt_sha256,
    })


def read_json(path):
    return json.loads(Path(path).read_bytes())


def _reject_permissive_acl(fd):
    """Inspect the opened inode, never a second pathname, for extra ACL grants.

    macOS's 0700 mode does not describe inherited/explicit NFSv4 ACL grants.
    Restrictive deny entries (including the normal home-directory deny-delete)
    are allowed. Any allow ACE fails closed, as effective access calculation is
    platform- and group-membership-dependent.
    """
    if sys.platform == "darwin":
        import ctypes
        import errno
        libc = ctypes.CDLL("/usr/lib/libSystem.B.dylib", use_errno=True)
        libc.acl_get_fd.argtypes = [ctypes.c_int]
        libc.acl_get_fd.restype = ctypes.c_void_p
        libc.acl_to_text.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_ssize_t)]
        libc.acl_to_text.restype = ctypes.c_void_p
        libc.acl_free.argtypes = [ctypes.c_void_p]
        libc.acl_free.restype = ctypes.c_int
        ctypes.set_errno(0)
        acl = libc.acl_get_fd(fd)
        if not acl:
            require(ctypes.get_errno() == errno.ENOENT, "private_directory_acl_check_failed")
            return
        try:
            length = ctypes.c_ssize_t()
            ptr = libc.acl_to_text(acl, ctypes.byref(length))
            require(ptr is not None, "private_directory_acl_check_failed")
            try:
                text = ctypes.string_at(ptr, length.value).decode("utf-8")
                require(":allow:" not in text.lower(), "private_directory_acl_allow_unsafe")
            finally:
                libc.acl_free(ptr)
        finally:
            libc.acl_free(acl)
    elif hasattr(os, "listxattr"):
        # On Linux, mode bits encode the effective POSIX ACL mask, but a
        # default ACL can be inherited by new children. Reject extra ACLs.
        attrs = os.listxattr(fd)
        require(not any(x.startswith("system.posix_acl_") for x in attrs),
                "private_directory_acl_allow_unsafe")


def _safe_directory_metadata(fd):
    meta = os.fstat(fd)
    mode = meta.st_mode
    require(stat.S_ISDIR(mode) and meta.st_uid in (0, os.getuid())
            and (mode & 0o022 == 0 or
                 (meta.st_uid == 0 and mode & stat.S_ISVTX != 0)),
            "private_output_parent_untrusted")
    _reject_permissive_acl(fd)


def _open_safe_directory_path(path, *, private_leaf=False):
    """Walk every component with openat/NOFOLLOW, retaining its parent FD.

    A symlink hidden in a middle component must not redirect a ledger or
    output. macOS /var and /tmp are immutable root-managed system aliases;
    accept only these two aliases by spelling out their /private targets.
    """
    absolute = os.path.abspath(os.fspath(path))
    if sys.platform == "darwin":
        for alias in ("/var", "/tmp"):
            if absolute == alias or absolute.startswith(alias + "/"):
                absolute = "/private" + absolute
                break
    parts = Path(absolute).parts
    fd = os.open("/", os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    try:
        _safe_directory_metadata(fd)
        for index, part in enumerate(parts[1:]):
            child = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW,
                            dir_fd=fd)
            os.close(fd)
            fd = child
            if not private_leaf or index != len(parts) - 2:
                _safe_directory_metadata(fd)
        return fd
    except BaseException:
        os.close(fd)
        raise


def _open_trusted_directory(path):
    """Open a 0700 same-user private directory with safe ancestors and ACL."""
    fd = _open_safe_directory_path(path, private_leaf=True)
    try:
        meta = os.fstat(fd)
        require(stat.S_ISDIR(meta.st_mode) and meta.st_uid == os.getuid()
                and stat.S_IMODE(meta.st_mode) == 0o700,
                "private_directory_owner_or_mode_unsafe")
        _reject_permissive_acl(fd)
        return fd
    except BaseException:
        os.close(fd)
        raise


def private_new_directory(path):
    """Create private 0700 directory in a safely walked parent, exclusively."""
    path = Path(path)
    parent_fd = _open_safe_directory_path(path.parent)
    try:
        os.mkdir(path.name, mode=0o700, dir_fd=parent_fd)
        output_fd = os.open(path.name, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW,
                            dir_fd=parent_fd)
        try:
            os.fchmod(output_fd, 0o700)
            meta = os.fstat(output_fd)
            require(meta.st_uid == os.getuid()
                    and stat.S_IMODE(meta.st_mode) == 0o700,
                    "private_directory_owner_or_mode_unsafe")
            _reject_permissive_acl(output_fd)
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
