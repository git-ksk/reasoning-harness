"""Operator-owned, create-only local authority store for one Mac/host.

No network, cloud SDK, implicit initialization or receipt-selected paths.
The OS account owning the store/profile is trusted; this is not a security
boundary against that same account or root, nor a distributed lock service.
"""
import json
import os
import secrets
import stat
from dataclasses import dataclass
from pathlib import Path

from authority import PREFIX
from contracts import (
    canonical, private_new_directory, require, safe_id, strict_keys,
    _open_trusted_directory, _reject_permissive_acl,
)

PROFILE = "reason-cli-normal-v3-local-authority-profile-v1"
IDENTITY = "reason-cli-normal-v3-local-authority-identity-v1"
IDENTITY_FILE = "authority-identity.json"
MAX_OBJECT_BYTES = 8 * 1024 * 1024
ROLES = frozenset(("issuer", "executor", "evaluator"))


def _directory(fd):
    meta = os.fstat(fd)
    require(stat.S_ISDIR(meta.st_mode) and meta.st_uid == os.getuid()
            and stat.S_IMODE(meta.st_mode) == 0o700, "local_authority_directory_unsafe")
    _reject_permissive_acl(fd)
    return meta


def _file(fd):
    meta = os.fstat(fd)
    require(stat.S_ISREG(meta.st_mode) and meta.st_uid == os.getuid()
            and stat.S_IMODE(meta.st_mode) == 0o600 and meta.st_nlink == 1
            and meta.st_size <= MAX_OBJECT_BYTES, "local_authority_file_unsafe")
    _reject_permissive_acl(fd)


def _read(fd):
    _file(fd)
    with os.fdopen(fd, "rb", closefd=False) as stream:
        data = stream.read(MAX_OBJECT_BYTES + 1)
    require(len(data) <= MAX_OBJECT_BYTES, "local_authority_object_too_large")
    return data


@dataclass(frozen=True)
class LocalStoreProfile:
    """Trusted operator configuration, injected outside plan/receipt/model data."""
    root: str
    identity_bytes: bytes

    def to_json(self):
        return {"contract": PROFILE, "root": self.root,
                "identity": json.loads(self.identity_bytes)}

    @classmethod
    def from_json(cls, value):
        fields = {"contract", "root", "identity"}
        strict_keys(value, fields, fields, "local_authority_profile")
        require(value["contract"] == PROFILE, "invalid_local_authority_profile")
        return cls(value["root"], canonical(value["identity"]))


def initialize_local_store(root):
    """Explicit enrollment only. Existing roots are never reset or overwritten.

    Save the returned profile in the operator's private configuration, outside
    captured receipts. Reopening a run never calls this function implicitly.
    """
    root = Path(root)
    require(root.is_absolute(), "absolute_local_authority_root_required")
    private_new_directory(root)
    root = root.resolve()
    fd = _open_trusted_directory(root)
    try:
        meta = _directory(fd)
        identity = canonical({"contract": IDENTITY, "scope": "single-host-local",
                              "nonce": secrets.token_hex(32),
                              "device": meta.st_dev, "inode": meta.st_ino})
        filefd = os.open(IDENTITY_FILE, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW,
                         0o600, dir_fd=fd)
        with os.fdopen(filefd, "wb") as stream:
            os.fchmod(stream.fileno(), 0o600)
            stream.write(identity)
            stream.flush()
            os.fsync(stream.fileno())
        os.fsync(fd)
    finally:
        os.close(fd)
    return LocalStoreProfile(str(root), identity)


class LocalAtomicStore:
    """Durable append-only objects; one fixed profile shared by local processes.

    role checks guard host integration mistakes, not a malicious same-UID
    operator. Independent review still requires the enrolled review signature.
    """
    def __init__(self, profile, *, role):
        require(type(profile) is LocalStoreProfile, "trusted_local_profile_required")
        require(role in ROLES, "invalid_local_authority_role")
        require(type(profile.root) is str and Path(profile.root).is_absolute()
                and str(Path(profile.root).resolve()) == profile.root,
                "canonical_local_authority_root_required")
        require(type(profile.identity_bytes) is bytes, "local_authority_identity_missing")
        identity = json.loads(profile.identity_bytes)
        fields = {"contract", "scope", "nonce", "device", "inode"}
        strict_keys(identity, fields, fields, "local_authority_identity")
        require(canonical(identity) == profile.identity_bytes
                and identity["contract"] == IDENTITY and identity["scope"] == "single-host-local",
                "invalid_local_authority_identity")
        require(type(identity["nonce"]) is str and len(identity["nonce"]) == 64
                and all(c in "0123456789abcdef" for c in identity["nonce"]),
                "invalid_local_authority_nonce")
        require(type(identity["device"]) is int and type(identity["inode"]) is int,
                "invalid_local_authority_inode")
        self.profile, self.role = profile, role
        self._identity = identity
        fd = self._root()
        os.close(fd)

    def _root(self):
        fd = _open_trusted_directory(self.profile.root)
        try:
            meta = _directory(fd)
            require((meta.st_dev, meta.st_ino) ==
                    (self._identity["device"], self._identity["inode"]),
                    "local_authority_root_replaced")
            identityfd = os.open(IDENTITY_FILE, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK, dir_fd=fd)
            try:
                require(_read(identityfd) == self.profile.identity_bytes,
                        "local_authority_identity_changed")
            finally:
                os.close(identityfd)
            return fd
        except BaseException:
            os.close(fd)
            raise

    def _parts(self, key, *, writing=False):
        require(type(key) is str and key.startswith(PREFIX + "/"), "local_authority_namespace_escape")
        parts = key.split("/")
        for part in parts:
            safe_id(part[:-5] if part.endswith(".json") else part, "local_authority_unsafe_key")
        if writing:
            relative = key[len(PREFIX) + 1:]
            receipt = relative.startswith("receipts/")
            score = relative.startswith("claims/") and relative.endswith("/first-score.json")
            execution = relative.startswith("cohorts/") or (relative.startswith("claims/") and not score)
            require((self.role == "issuer" and receipt)
                    or (self.role == "evaluator" and score)
                    or (self.role == "executor" and execution), "local_authority_role_denied")
        return parts

    def _parent(self, parts, *, create):
        fd = self._root()
        try:
            for part in parts[:-1]:
                if create:
                    try:
                        os.mkdir(part, mode=0o700, dir_fd=fd)
                    except FileExistsError:
                        pass
                    else:
                        os.fsync(fd)
                child = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=fd)
                os.close(fd)
                fd = child
                _directory(fd)
            return fd
        except BaseException:
            os.close(fd)
            raise

    def read(self, key):
        parts = self._parts(key)
        # Check the root separately: a missing/replaced authority is an error,
        # while a genuinely absent object below that authority returns None.
        rootfd = self._root()
        os.close(rootfd)
        try:
            parentfd = self._parent(parts, create=False)
        except FileNotFoundError:
            self._assert_root()
            return None
        try:
            try:
                fd = os.open(parts[-1], os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK, dir_fd=parentfd)
            except FileNotFoundError:
                self._assert_root()
                return None
            try:
                data = _read(fd)
            finally:
                os.close(fd)
            self._assert_root()
            return data
        finally:
            os.close(parentfd)

    def _assert_root(self):
        fd = self._root()
        os.close(fd)

    def create(self, key, data):
        parts = self._parts(key, writing=True)
        require(type(data) is bytes and len(data) <= MAX_OBJECT_BYTES, "local_authority_bytes_required")
        parentfd = self._parent(parts, create=True)
        try:
            fd = os.open(parts[-1], os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW,
                         0o600, dir_fd=parentfd)
            # Never unlink on an interrupted/ambiguous write: the exclusive
            # claim remains burned, including after SIGKILL or fsync failure.
            with os.fdopen(fd, "wb") as stream:
                os.fchmod(stream.fileno(), 0o600)
                stream.write(data)
                stream.flush()
                os.fsync(stream.fileno())
            os.fsync(parentfd)
            self._assert_root()
        finally:
            os.close(parentfd)
