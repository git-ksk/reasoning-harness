"""Independent post-rebase counterexamples for #584/#586; entirely synthetic."""
import copy
import json
import tempfile
import unittest
from pathlib import Path

import collect
from contracts import digest, keyed_digest
from test_measurement import demo_fixture


def replace_capture(plan, folder, filename, value):
    path = folder / filename
    path.write_text(json.dumps(value, ensure_ascii=False, sort_keys=True), encoding="utf-8")
    manifest = json.loads(plan.read_text(encoding="utf-8"))
    manifest["rows"][0]["files"][filename] = digest(path.read_bytes())
    plan.write_text(json.dumps(manifest), encoding="utf-8")


class IndependentFrozenBinding586(unittest.TestCase):
    def test_freeze_must_include_model_identity(self):
        # Retain the original frozen hash, but swap every self-reported model identity.
        for failed in (True, False):
            with self.subTest(failed=failed), tempfile.TemporaryDirectory() as temp:
                plan, lane, _, _, job = demo_fixture(
                    temp, phase="initial_candidate" if failed else None
                )
                manifest = json.loads(plan.read_text())
                original = manifest["rows"][0]["frozen_request_binding_sha256"]
                manifest["model"] = "other-model"
                plan.write_text(json.dumps(manifest))
                trace = json.loads((job / "trace.json").read_text())
                trace["model"] = "other-model"
                for event in trace["events"]:
                    if event["kind"] in ("model_request", "model_response", "model_failure"):
                        event["model"] = "other-model"
                replace_capture(plan, job, "trace.json", trace)
                http = json.loads((job / "http.json").read_text())
                for event in http:
                    event["model"] = "other-model"
                replace_capture(plan, job, "http.json", http)
                if not failed:
                    cli = json.loads((job / "cli.json").read_text())
                    cli["result"]["configuration"]["model"] = "other-model"
                    replace_capture(plan, job, "cli.json", cli)
                self.assertEqual(original, json.loads(plan.read_text())["rows"][0]["frozen_request_binding_sha256"])
                with self.assertRaisesRegex(ValueError, "unbound_model_request_hash"):
                    collect.collect(plan, lane, Path(temp) / "output")

    def test_freeze_must_include_job_identity(self):
        with tempfile.TemporaryDirectory() as temp:
            plan, lane, _, _, job = demo_fixture(temp, phase="initial_candidate")
            renamed = job.with_name("case-01-candidate")
            job.rename(renamed)
            manifest = json.loads(plan.read_text())
            manifest["rows"][0]["job_id"] = renamed.name
            plan.write_text(json.dumps(manifest))
            with self.assertRaisesRegex(ValueError, "unbound_model_request_hash"):
                collect.collect(plan, lane, Path(temp) / "output")

    def test_first_physical_http_request_cannot_be_later_trace_request(self):
        # Trace presents frozen A first; a concurrent request B actually returns
        # and appears in the physical HTTP journal before A.
        with tempfile.TemporaryDirectory() as temp:
            plan, lane, _, _, job = demo_fixture(temp, phase="initial_candidate")
            trace = json.loads((job / "trace.json").read_text())
            old = trace["events"]
            old[2]["request"]["task"] = "Attacker's unrelated first physical request"
            trace["events"] = [old[0], old[2], old[3], old[1]]
            for i, event in enumerate(trace["events"]):
                event["sequence"] = i
            replace_capture(plan, job, "trace.json", trace)
            http = json.loads((job / "http.json").read_text())
            replace_capture(plan, job, "http.json", list(reversed(http)))
            with self.assertRaisesRegex(ValueError, "overlapping_trace_model_requests"):
                collect.collect(plan, lane, Path(temp) / "output")

    def test_duplicate_job_id_rejected_before_output_creation(self):
        with tempfile.TemporaryDirectory() as temp:
            plan, lane, _, _, _ = demo_fixture(temp, phase="initial_candidate")
            manifest = json.loads(plan.read_text())
            second = copy.deepcopy(manifest["rows"][0])
            second["case_id"] = "case-02"
            second["arm"] = "candidate"
            manifest["rows"].append(second)
            plan.write_text(json.dumps(manifest))
            out = Path(temp) / "output"
            with self.assertRaisesRegex(ValueError, "duplicate_job_id"):
                collect.collect(plan, lane, out)
            self.assertFalse(out.exists())

    def test_redacted_request_collision_cannot_hide_actual_secret_change(self):
        from contracts import frozen_request_digest
        with tempfile.TemporaryDirectory() as temp:
            plan, lane, _, _, job = demo_fixture(temp, phase="initial_candidate")
            manifest = json.loads(plan.read_text())
            trace = json.loads((job / "trace.json").read_text())
            redacted = {"task": "[REDACTED]"}
            original_hash = keyed_digest({"task": "Authorization: original"})
            changed_hash = keyed_digest({"task": "Authorization: replaced"})
            trace["events"][0]["request"] = redacted
            trace["events"][0]["request_sha256"] = original_hash
            replace_capture(plan, job, "trace.json", trace)
            manifest = json.loads(plan.read_text())
            row = manifest["rows"][0]
            row["frozen_request_binding_sha256"] = frozen_request_digest(
                job_id=row["job_id"], case_id=row["case_id"],
                arm=row["arm"], question=row["question"], request=redacted,
                request_sha256=original_hash, provider=manifest["provider"],
                model=manifest["model"], receipt_sha256=manifest["receipt_sha256"],
            )
            plan.write_text(json.dumps(manifest))
            good = collect.collect(plan, lane, Path(temp) / "good")
            self.assertEqual(len(good), 1)
            trace["events"][0]["request_sha256"] = changed_hash
            replace_capture(plan, job, "trace.json", trace)
            with self.assertRaisesRegex(ValueError, "unbound_model_request_hash"):
                collect.collect(plan, lane, Path(temp) / "bad")

    def test_missing_original_request_hash_is_rejected(self):
        with tempfile.TemporaryDirectory() as temp:
            plan, lane, _, _, job = demo_fixture(temp, phase="initial_candidate")
            trace = json.loads((job / "trace.json").read_text())
            del trace["events"][0]["request_sha256"]
            replace_capture(plan, job, "trace.json", trace)
            with self.assertRaisesRegex(ValueError, "missing_raw_request_digest"):
                collect.collect(plan, lane, Path(temp) / "output")

    def test_frozen_receipt_identity_cannot_be_substituted(self):
        with tempfile.TemporaryDirectory() as temp:
            plan, lane, _, _, _ = demo_fixture(temp, phase="initial_candidate")
            manifest = json.loads(plan.read_text())
            manifest["receipt_sha256"] = "a" * 64
            plan.write_text(json.dumps(manifest))
            with self.assertRaisesRegex(ValueError, "unbound_model_request_hash"):
                collect.collect(plan, lane, Path(temp) / "output")

    def test_full_request_field_change_even_with_same_question(self):
        with tempfile.TemporaryDirectory() as temp:
            plan, lane, _, _, job = demo_fixture(temp, phase="initial_candidate")
            trace = json.loads((job / "trace.json").read_text())
            trace["events"][0]["request"]["temperature"] = 1
            replace_capture(plan, job, "trace.json", trace)
            with self.assertRaisesRegex(ValueError, "unbound_model_request_hash"):
                collect.collect(plan, lane, Path(temp) / "output")

    def test_nonfinite_request_numbers_are_not_canonical_json(self):
        from contracts import frozen_request_digest
        with self.assertRaises(ValueError):
            frozen_request_digest(
                job_id="case-01-baseline", case_id="case-01", arm="baseline",
                question="What?", request={"task": "What?", "temperature": float("nan")},
                request_sha256="b" * 64, provider="mistral",
                model="fixture-model", receipt_sha256="a" * 64,
            )

    def test_first_request_missing_fails_closed(self):
        with tempfile.TemporaryDirectory() as temp:
            plan, lane, _, _, job = demo_fixture(temp, phase="initial_candidate")
            trace = json.loads((job / "trace.json").read_text())
            trace["events"] = []
            replace_capture(plan, job, "trace.json", trace)
            with self.assertRaisesRegex(ValueError, "invalid_first_model_request"):
                collect.collect(plan, lane, Path(temp) / "output")

    def test_canonical_key_order_and_unicode_are_exact(self):
        a = {"question": "漢字と\u00e9", "request": {"z": ["\u03a9", "\n"], "a": {"b": 1, "a": True}}}
        b = {"request": {"a": {"a": True, "b": 1}, "z": ["\u03a9", "\n"]}, "question": "漢字と\u00e9"}
        self.assertEqual(keyed_digest(a), keyed_digest(b))
        c = copy.deepcopy(a)
        c["question"] = "漢字と" + "e\u0301"
        self.assertNotEqual(keyed_digest(a), keyed_digest(c))
        c = copy.deepcopy(a)
        c["request"]["z"][1] = "\t"
        self.assertNotEqual(keyed_digest(a), keyed_digest(c))


if __name__ == "__main__":
    unittest.main()
