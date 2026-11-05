"""Encrypted, bounded GitHub Issue mailbox for an independently signed review.

The public repository carries ciphertext and routing metadata only. Encryption
protects the packet, while the existing Ed25519 verifier authenticates decisions.
"""
import base64
import binascii
from datetime import datetime, timedelta, timezone
import json
import lzma
import re
import time
import urllib.error
import urllib.request

from nacl.exceptions import CryptoError
from nacl.public import PrivateKey, PublicKey, SealedBox

from contracts import canonical, digest, require
from review_auth import validate_policy, ReviewAuthentication

REPOSITORY = "git-ksk/reasoning-harness"
ISSUE = 574
MARKER = "<!-- reason-v3-encrypted-exchange-v1 -->\n"
CONTRACT = "reason-cli-normal-v3-ci-exchange-v1"
APPROVAL = "reason-cli-normal-v3-ci-approval-v1"
MAX_PLAIN = 32 * 1024 * 1024
MAX_ARCHIVE = 256 * 1024 * 1024
CHUNK_BYTES = 48000
MAX_CHUNKS = 64
COMPRESSION_TAG = b"reason-cli-v3-xz-v1\0"


def seal(raw, public_key, *, limit=MAX_PLAIN):
    require(type(raw) is bytes and len(raw) <= limit, "ci_plaintext_limit")
    return SealedBox(PublicKey(public_key)).encrypt(COMPRESSION_TAG + lzma.compress(raw, preset=6))


def unseal(raw, private_key, *, limit=MAX_PLAIN):
    require(type(raw) is bytes and len(raw) <= limit + 65536, "ci_ciphertext_limit")
    compressed = SealedBox(PrivateKey(private_key)).decrypt(raw)
    require(compressed.startswith(COMPRESSION_TAG), "ci_compression_version")
    decoder = lzma.LZMADecompressor(format=lzma.FORMAT_XZ, memlimit=64 * 1024 * 1024)
    plain = decoder.decompress(compressed[len(COMPRESSION_TAG):], max_length=limit + 1)
    require(len(plain) <= limit and decoder.eof and not decoder.unused_data, "ci_decompression_limit_or_trailing_bytes")
    return plain


def chunks(run_id, stage, ciphertext):
    require(type(run_id) is int and run_id > 0 and re.fullmatch(r"[a-z0-9-]{1,80}", stage),
            "ci_route_invalid")
    encoded = base64.b64encode(ciphertext).decode("ascii")
    parts = [encoded[start:start + CHUNK_BYTES] for start in range(0, len(encoded), CHUNK_BYTES)]
    require(0 < len(parts) <= MAX_CHUNKS, "ci_message_chunk_limit")
    return [MARKER + canonical({"contract": CONTRACT, "run_id": run_id, "stage": stage,
            "sha256": digest(ciphertext), "index": index, "count": len(parts), "ciphertext": part}).decode()
            for index, part in enumerate(parts)]


def messages(comments, run_id, stage, *, sender_id=None):
    groups = {}
    for comment in comments:
        author = comment.get("user")
        if sender_id is not None and (not isinstance(author, dict) or author.get("id") != sender_id):
            continue
        body = comment.get("body", "")
        if not isinstance(body, str) or not body.startswith(MARKER) or len(body) > CHUNK_BYTES + 1024:
            continue
        try:
            row = json.loads(body[len(MARKER):])
            if (set(row) != {"contract", "run_id", "stage", "sha256", "index", "count", "ciphertext"}
                    or row["contract"] != CONTRACT or type(row["run_id"]) is not int
                    or row["run_id"] != run_id or row["stage"] != stage
                    or type(row["count"]) is not int or not 0 < row["count"] <= MAX_CHUNKS
                    or type(row["index"]) is not int or not 0 <= row["index"] < row["count"]
                    or not isinstance(row["sha256"], str) or not re.fullmatch(r"[0-9a-f]{64}", row["sha256"])
                    or not isinstance(row["ciphertext"], str) or len(row["ciphertext"]) > CHUNK_BYTES):
                continue
            identity = (row["sha256"], row["count"])
            if identity not in groups and len(groups) >= 256:
                continue
            group = groups.setdefault(identity, {})
            if row["index"] in group and group[row["index"]] != row["ciphertext"]:
                group[row["index"]] = "!"  # Conflicting public chunks cannot win by ordering.
            else:
                group[row["index"]] = row["ciphertext"]
        except (ValueError, TypeError, KeyError):
            continue
    for (sha, count), parts in groups.items():
        if len(parts) != count:
            continue
        try:
            raw = base64.b64decode("".join(parts[index] for index in range(count)), validate=True)
        except (ValueError, binascii.Error):
            continue
        if digest(raw) == sha:
            yield raw


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


class Mailbox:
    def __init__(self, *, run_id, token, since, sender_id=None):
        require(type(run_id) is int and run_id > 0 and isinstance(token, str) and token,
                "ci_mailbox_identity_required")
        self.run_id, self.token, self.since = run_id, token, since
        require(sender_id is None or type(sender_id) is int and sender_id > 0, "ci_mailbox_sender_invalid")
        self.sender_id = sender_id
        self.opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), NoRedirect())

    def _request(self, suffix, value=None):
        url = f"https://api.github.com/repos/{REPOSITORY}/issues/{ISSUE}/comments{suffix}"
        request = urllib.request.Request(url, data=canonical(value) if value is not None else None,
            headers={"Authorization": "Bearer " + self.token, "Accept": "application/vnd.github+json",
                     "Content-Type": "application/json", "X-GitHub-Api-Version": "2022-11-28"})
        with self.opener.open(request, timeout=15) as response:
            raw = response.read(16 * 1024 * 1024 + 1)
        require(len(raw) <= 16 * 1024 * 1024, "ci_mailbox_response_limit")
        return json.loads(raw)

    def publish(self, stage, raw, recipient, *, limit=MAX_PLAIN):
        for body in chunks(self.run_id, stage, seal(raw, recipient, limit=limit)):
            # Ambiguous publication is never retried. The caller stops the run.
            self._request("", {"body": body})

    def comments(self, *, since=None):
        result = []
        for page in range(1, 21):
            rows = self._request(f"?since={since or self.since}&per_page=100&page={page}")
            require(type(rows) is list and all(type(row) is dict for row in rows), "ci_mailbox_schema")
            result.extend(rows)
            if len(rows) < 100:
                return result
        raise ValueError("ci_mailbox_page_limit")

    def receive(self, stage, private_key, validate, deadline, *, limit=MAX_PLAIN):
        seen = set()
        cursor = self.since if stage == "start" else (datetime.now(timezone.utc) - timedelta(seconds=60)).isoformat(timespec="seconds")
        while time.monotonic() < deadline:
            for raw in messages(self.comments(since=cursor), self.run_id, stage, sender_id=self.sender_id):
                sha = digest(raw)
                if sha in seen:
                    continue
                seen.add(sha)
                try:
                    plain = unseal(raw, private_key, limit=limit)
                    value = json.loads(plain)
                    require(canonical(value) == plain, "ci_noncanonical_message")
                    validate(value)
                    return value
                except (ValueError, TypeError, KeyError, CryptoError, lzma.LZMAError):
                    # An unauthenticated public comment cannot change authority.
                    continue
            time.sleep(min(5, max(0, deadline - time.monotonic())))
        raise TimeoutError("ci_signed_review_deadline")


def authenticate(envelope, policy_bytes, evaluator_id, executor_id, contract, expected):
    aliases, keys = validate_policy(policy_bytes)
    evaluator, executor = aliases[evaluator_id], aliases[executor_id]
    require(evaluator["subject_id"] != executor["subject_id"]
            and not set(evaluator["conflict_groups"]) & set(executor["conflict_groups"]), "ci_principal_conflict")
    require(type(envelope) is dict and set(envelope) ==
            {"contract", "algorithm", "key_id", "payload", "signature_hex"}, "ci_signature_schema")
    key = keys.get(envelope["key_id"])
    require(envelope["contract"] == contract and envelope["algorithm"] == "Ed25519" and key is not None
            and not key["revoked"] and key["subject_id"] == evaluator["subject_id"], "ci_unknown_signer")
    payload = envelope["payload"]
    require(type(payload) is dict and all(payload.get(k) == v for k, v in expected.items())
            and payload.get("policy_sha256") == digest(policy_bytes), "ci_stale_review")
    ReviewAuthentication(policy_bytes, b"")._verify_signature(bytes.fromhex(key["public_key_hex"]),
        bytes.fromhex(envelope["signature_hex"]), (contract + "\0").encode() + canonical(payload))
    return payload
