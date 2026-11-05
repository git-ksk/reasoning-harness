"""Public-mailbox adversaries and zero-provider checks for same-job CI."""
import base64
import copy
import json
import lzma
import os
from pathlib import Path
import tempfile
import time
import random
import unittest
import urllib.parse
from unittest.mock import patch

from nacl.public import PrivateKey, SealedBox
from nacl.signing import SigningKey

import ci_exchange
from ci_exchange import APPROVAL, Mailbox, authenticate, chunks, messages, seal, unseal
from ci_operator import decode_packet, unpack
from ci_prepare import context, gold_cases, pack_files
from ci_run import drive
from contracts import canonical, digest
from native_lane import private_bytes


class CiphertextTests(unittest.TestCase):
    def test_roundtrip_tamper_wrong_key_and_inflation(self):
        key = PrivateKey.generate()
        source = canonical({"private_source": "full confidential bytes"})
        ciphertext = seal(source, bytes(key.public_key))
        self.assertNotIn(b"confidential", ciphertext)
        self.assertEqual(unseal(ciphertext, bytes(key)), source)
        for candidate, secret in ((ciphertext[:-1] + bytes([ciphertext[-1] ^ 1]), bytes(key)),
                                  (ciphertext, bytes(PrivateKey.generate()))):
            with self.assertRaises(Exception): unseal(candidate, secret)
        oversized = SealedBox(key.public_key).encrypt(ci_exchange.COMPRESSION_TAG + lzma.compress(b"x" * 1025))
        trailing = SealedBox(key.public_key).encrypt(ci_exchange.COMPRESSION_TAG + lzma.compress(b"data") + b"trailing")
        for candidate in (oversized, trailing):
            with self.assertRaisesRegex(ValueError, "decompression"):
                unseal(candidate, bytes(key), limit=1024)

    def test_chunk_order_duplicate_conflict_foreign_route_and_missing_chunk(self):
        ciphertext = os.urandom(90000)
        comments = [{"body": body} for body in chunks(42, "job-00", ciphertext)]
        self.assertEqual(list(messages(list(reversed(comments)) + comments[:1], 42, "job-00")), [ciphertext])
        self.assertEqual(list(messages(comments, 43, "job-00")), [])
        self.assertEqual(list(messages(comments, 42, "job-01")), [])
        self.assertEqual(list(messages(comments[:-1], 42, "job-00")), [])
        conflict = json.loads(comments[0]["body"][len(ci_exchange.MARKER):])
        conflict["ciphertext"] = "A" + conflict["ciphertext"][1:]
        poisoned = comments + [{"body": ci_exchange.MARKER + canonical(conflict).decode()}]
        self.assertEqual(list(messages(poisoned, 42, "job-00")), [])
        authorized = [{**comment, "user": {"id": 123}} for comment in comments]
        outsider = {"body": ci_exchange.MARKER + canonical(conflict).decode(), "user": {"id": 456}}
        self.assertEqual(list(messages(authorized + [outsider], 42, "job-00", sender_id=123)), [ciphertext])
        self.assertTrue(all(len(row["body"]) < 65536 for row in comments))

    def test_archive_large_binary_mode_and_path_escape_are_checked_before_writes(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder).resolve()
            path = root / "binary"
            private_bytes(path, b"binary"); path.chmod(0o700)
            raw = pack_files({"binary/baseline": path})
            unpack(raw, root / "decoded")
            self.assertEqual((root / "decoded/binary/baseline").read_bytes(), b"binary")
            malicious = canonical({"contract": "reason-cli-normal-v3-ci-private-files-v1",
                                   "files": {"../escape": base64.b64encode(b"bad").decode()}})
            with self.assertRaisesRegex(ValueError, "escape"):
                unpack(malicious, root / "rejected")
            self.assertFalse((root / "rejected").exists())
            path.chmod(0o666)
            with self.assertRaisesRegex(ValueError, "unsafe"): pack_files({"binary": path})

    def test_full_48_source_complete_observation_and_packet_archive_roundtrips(self):
        key = PrivateKey.generate()
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder).resolve()
            # An incompressible full document is larger than zlib's window.
            # Repetition across 96 source-complete files must still deduplicate.
            source = base64.b64encode(random.Random(42).randbytes(265000)).decode()
            row = canonical({"sources": {"full": {"text": source}}})
            files = {}
            for index in range(48):
                for group in ("observations", "packets"):
                    path = root / (group + "-" + str(index) + ".json")
                    private_bytes(path, row)
                    files[group + "/" + str(index) + ".json"] = path
            raw = pack_files(files)
            self.assertGreater(len(raw), ci_exchange.MAX_PLAIN)
            published = []
            box = Mailbox(run_id=42, token="fixture-only", since="2026-11-05T00:00:00Z")
            with patch.object(box, "_request", side_effect=lambda suffix, value: published.append(value)):
                box.publish("final", raw, bytes(key.public_key), limit=ci_exchange.MAX_ARCHIVE)
            cipher = list(messages(published, 42, "final"))
            self.assertEqual(len(cipher), 1)
            decoded = unseal(cipher[0], bytes(key), limit=ci_exchange.MAX_ARCHIVE)
            self.assertEqual(decoded, raw)
            restored = root / "restored"
            unpack(decoded, restored)
            self.assertEqual((restored / "observations/47.json").read_bytes(), row)
            self.assertEqual((restored / "packets/47.json").read_bytes(), row)


class SignedInboxTests(unittest.TestCase):
    def setUp(self):
        self.signing = SigningKey.generate()
        self.policy = canonical({"contract": "reason-cli-normal-v3-review-policy-v1", "policy_id": "ci-test",
            "principals": [{"subject_id": "executor", "aliases": ["exec"], "role": "agent", "conflict_groups": ["exec"]},
                           {"subject_id": "reviewer", "aliases": ["review"], "role": "agent", "conflict_groups": ["review"]}],
            "keys": [{"key_id": "active", "subject_id": "reviewer", "public_key_hex": bytes(self.signing.verify_key).hex(), "revoked": False}]})
        self.expected = {"contract": APPROVAL, "run_id": 42, "stage": "start", "policy_sha256": digest(self.policy),
                         "session_sha256": "a" * 64}
        payload = {**self.expected, "decision": "approve"}
        self.signed = {"contract": APPROVAL, "algorithm": "Ed25519", "key_id": "active", "payload": payload,
                       "signature_hex": self.signing.sign((APPROVAL + "\0").encode() + canonical(payload)).signature.hex()}

    def verify(self, value):
        return authenticate(value, self.policy, "review", "exec", APPROVAL, self.expected)

    def test_foreign_session_mutation_revocation_and_executor_alias_are_rejected(self):
        self.assertEqual(self.verify(self.signed)["decision"], "approve")
        for mutate in (lambda x: x["payload"].update(run_id=43), lambda x: x["payload"].update(decision="stop"),
                       lambda x: x.update(key_id="unknown")):
            bad = copy.deepcopy(self.signed); mutate(bad)
            with self.assertRaises(ValueError): self.verify(bad)
        with self.assertRaises(ValueError):
            authenticate(self.signed, self.policy, "exec", "exec", APPROVAL, self.expected)
        revoked = json.loads(self.policy); revoked["keys"][0]["revoked"] = True
        with self.assertRaises(ValueError):
            authenticate(self.signed, canonical(revoked), "review", "exec", APPROVAL, self.expected)

    def test_public_garbage_does_not_consume_review_or_skip_signature(self):
        key = PrivateKey.generate()
        wrong = copy.deepcopy(self.signed); wrong["payload"]["run_id"] = 43
        malformed = SealedBox(key.public_key).encrypt(ci_exchange.COMPRESSION_TAG + b"not-an-xz-stream")
        comments = [{"body": item} for item in chunks(42, "start", malformed)]
        comments += [{"body": item} for value in (wrong, self.signed) for item in
                    chunks(42, "start", seal(canonical(value), bytes(key.public_key)))]
        box = Mailbox(run_id=42, token="fixture-only", since="2026-11-05T00:00:00Z")
        with patch.object(box, "comments", return_value=comments):
            self.assertEqual(box.receive("start", bytes(key), self.verify, time.monotonic() + 1), self.signed)
        with patch.object(box, "comments") as read:
            with self.assertRaises(TimeoutError): box.receive("start", bytes(key), self.verify, time.monotonic() - 1)
            read.assert_not_called()

    def test_poll_cursor_keeps_utc_offset_in_the_actual_github_query(self):
        box = Mailbox(run_id=42, token="fixture-only", since="2026-11-05T00:00:00Z")
        cursor = "2026-11-05T01:02:03+00:00"
        with patch.object(box, "_request", return_value=[]) as get:
            self.assertEqual(box.comments(since=cursor), [])
        query = urllib.parse.parse_qs(get.call_args.args[0][1:])
        self.assertEqual(query["since"], [cursor])
        self.assertEqual(query["per_page"], ["100"])

    def test_fake_packet_cannot_prevent_reading_the_current_bot_packet(self):
        key = PrivateKey.generate()
        session = {"run_id": 42, "github_bot_id": 123}
        session_raw = canonical(session)
        packet = {"ci_session_sha256": digest(session_raw), "expected": {"evaluation_id": "ci-v3-42", "job_id": "case-baseline"},
                  "packet": {"job_id": "case-baseline"}}
        comments = []
        for raw, author in ((os.urandom(100), 123), (seal(canonical(packet), bytes(key.public_key)), 456),
                            (seal(canonical(packet), bytes(key.public_key)), 123)):
            comments += [{"body": text, "user": {"id": author}} for text in chunks(42, "job-00", raw)]
        self.assertEqual(decode_packet(comments, 42, "job-00", bytes(key), session, session_raw), canonical(packet))
        with self.assertRaises(ValueError):
            decode_packet(comments[:-1], 42, "job-00", bytes(key), session, session_raw)


class SameJobTests(unittest.TestCase):
    def test_no_next_spawn_before_review_and_failure_stops_the_loop(self):
        class Lane:
            next = 0
            pending = False
            starts = 0
            def run_next(self):
                self.starts += 1
                if self.pending: raise AssertionError("spawn before review")
                self.pending = True
                return {"index": self.next}, {"index": self.next}
            def accept_review(self, raw):
                self.pending = False
                self.next += 1
                return json.loads(raw)
        lane = Lane()
        def receive(stage, expected, deadline):
            self.assertTrue(lane.pending)
            self.assertEqual(lane.starts, lane.next + 1)
            if lane.next == 2: raise TimeoutError("review deadline")
            return expected
        with self.assertRaises(TimeoutError):
            drive(lane, lambda stage, raw: None, receive, time.monotonic() + 1000)
        self.assertEqual((lane.starts, lane.next), (3, 2))
        lane = Lane()
        rows = drive(lane, lambda stage, raw: None, lambda stage, expected, deadline: expected, time.monotonic() + 1000)
        self.assertEqual((lane.starts, lane.next, len(rows)), (48, 48, 48))
        with self.assertRaises(ValueError):
            drive(Lane(), lambda stage, raw: None, receive, time.monotonic() - 1)

    def test_attempt_two_wrong_tag_and_expired_job_refuse_before_credentials(self):
        environment = {"GITHUB_REPOSITORY": "git-ksk/reasoning-harness", "GITHUB_RUN_ID": "42",
            "GITHUB_RUN_ATTEMPT": "1", "GITHUB_SHA": "a" * 40, "GITHUB_EVENT_NAME": "push",
            "GITHUB_REF": "refs/tags/cli-normal-real24-development-v3", "V3_JOB_STARTED_UNIX": str(time.time())}
        self.assertEqual(context(environment, "live")["run_id"], 42)
        for changes in ({"GITHUB_RUN_ATTEMPT": "2"}, {"GITHUB_REF": "refs/heads/main"},
                        {"V3_JOB_STARTED_UNIX": str(time.time() - 331 * 60)}):
            with self.assertRaises(ValueError): context({**environment, **changes}, "live")
        self.assertEqual(context({**environment, "GITHUB_EVENT_NAME": "workflow_dispatch"}, "prepare")["mode"], "prepare")

    def test_gold_uses_only_existing_normal_groups_and_question_bytes(self):
        corpus = {"cases": [{"id": str(i), "question": "question " + str(i)} for i in range(24)]}
        records = [{"track": "e2e_normal", "case_id": str(i), "label": "positive" if i < 17 else "negative",
                    "groups": [{"id": "group-" + str(i)}] if i < 17 else []} for i in range(24)]
        gold = gold_cases(corpus, {"records": records + [{"track": "helper_opt_in"}]})
        self.assertEqual(sum(row["label"] == "positive" for row in gold), 17)
        self.assertEqual(gold[-1]["question"], "question 23")
        with self.assertRaises(ValueError): gold_cases(corpus, {"records": records[:-1]})


if __name__ == "__main__":
    unittest.main()
