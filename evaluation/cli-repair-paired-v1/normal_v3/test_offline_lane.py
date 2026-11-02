"""Actual durable scheduler/subprocess integration with synthetic model data."""
import json
import os
from concurrent.futures import ThreadPoolExecutor
import subprocess
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

import offline_lane
from contracts import digest
from test_measurement import demo_fixture


SCRIPT = r'''
import argparse
import json
from pathlib import Path
import shutil
import sys
p=argparse.ArgumentParser()
p.add_argument("--provider",required=True)
p.add_argument("--model",required=True)
p.add_argument("--fixture",required=True)
p.add_argument("--inert-no-network",action="store_true",required=True)
a=p.parse_args()
assert a.provider=="mistral" and a.model=="fixture-model" and a.inert_no_network
for source in (Path(a.fixture).parent/"snapshot").glob("*.json"):
    shutil.copyfile(source,Path.cwd()/source.name)
value=json.loads(Path("cli.json").read_bytes())
print(json.dumps(value,sort_keys=True),flush=True)
sys.exit(1 if value["result"].get("status")=="failed" else 0)
'''


def make_plan(folder, scenario=None):
    """One 48-job inert plan; no credentials, sockets, or actual provider calls."""
    root = Path(folder).resolve()
    ledger = root / "ledger"
    ledger.mkdir(mode=0o700)
    receipt = {
        "purpose": offline_lane.INERT_PURPOSE,
        "model_calls_allowed": False,
        "ledger_dir": str(ledger),
    }
    receipt_file = root / "receipt.json"
    receipt_file.write_text(json.dumps(receipt, sort_keys=True))
    script = root / "fixture.py"
    script.write_text(SCRIPT)
    fixtures = root / "fixtures"
    fixtures.mkdir()
    artifacts = {
        str(script): digest(script.read_bytes()),
        sys.executable: digest(Path(sys.executable).read_bytes()),
    }
    ids = [f"synthetic-{i:02d}" for i in range(24)]
    jobs = []
    for i, cid in enumerate(ids):
        arms = ("baseline", "candidate") if i % 2 == 0 else ("candidate", "baseline")
        for arm in arms:
            jid = cid + "-" + arm
            typ = scenario.get(jid) if scenario else None
            folder = fixtures / jid
            folder.mkdir()
            plan, lane, gold, rec, source = demo_fixture(
                folder,
                phase=("initial_candidate" if typ in ("invalid", "inner_retry", "later_429")
                       else "final_render" if typ == "rendered_decode"
                       else None),
                text=("Unsafe unknown assertion" if typ in ("visible", "rendered_decode") else ""),
                status="grounded_answer" if typ in ("visible", "rendered_decode") else "unresolved",
            )
            fixture = folder / "snapshot"
            fixture.mkdir()
            for item in source.glob("*.json"):
                shutil.copyfile(item, fixture / item.name)
            if typ == "missing_usage":
                http = json.loads((fixture / "http.json").read_bytes())
                http[0]["output_tokens"] = None
                (fixture / "http.json").write_text(json.dumps(http))
            if typ == "inner_retry":
                cli = json.loads((fixture / "cli.json").read_bytes())
                cli["result"]["partial_diagnostics"]["usage"]["run"]["model_calls"] = 1
                (fixture / "cli.json").write_text(json.dumps(cli))
                http = json.loads((fixture / "http.json").read_bytes())
                http[1] = {**http[1], "logical_attempt": 1}
                (fixture / "http.json").write_text(json.dumps(http))
                trace = json.loads((fixture / "trace.json").read_bytes())
                trace["events"] = trace["events"][:2]
                trace["events"][1]["response"]["provider_attempts"] = 2
                trace["events"][1]["response"]["usage"] = {
                    "input_tokens": 4, "output_tokens": 6, "total_tokens": 10
                }
                (fixture / "trace.json").write_text(json.dumps(trace))
            if typ == "later_429":
                http = json.loads((fixture / "http.json").read_bytes())
                http[1]["http_status"] = 429
                (fixture / "http.json").write_text(json.dumps(http))
                trace = json.loads((fixture / "trace.json").read_bytes())
                trace["events"][3].pop("response")
                trace["events"][3]["kind"] = "model_failure"
                trace["events"][3]["failure_class"] = "rate_limit"
                (fixture / "trace.json").write_text(json.dumps(trace))
            if typ == "quota":
                cli = json.loads((fixture / "cli.json").read_bytes())
                cli["result"]["status"] = "failed"
                cli["result"].pop("output_contract", None)
                cli["result"].pop("configuration", None)
                cli["result"].pop("finalization", None)
                cli["result"].pop("usage", None)
                cli["result"]["failure"] = {"failure_class": "quota"}
                cli["result"]["partial_diagnostics"] = {
                    "usage": {"run": {"tracked_from_session_start": True,
                                     "model_calls": 1,
                                     "provider_attempts": 1,
                                     "input_tokens": 2, "output_tokens": 3,
                                     "total_tokens": 5}}}
                (fixture / "cli.json").write_text(json.dumps(cli))
                # Trace stopped on operational failure, not a clean completed response.
                trace = json.loads((fixture / "trace.json").read_bytes())
                trace["completion_status"] = "partial"
                trace["events"] = trace["events"][:2]
                (fixture / "trace.json").write_text(json.dumps(trace))
            # Each fixture artifact is frozen and hashed before the process.
            artifacts[str(fixture)] = digest_fixture(fixture)
            jobs.append({
                "id": jid, "arm": arm, "contract": "normal",
                "timeout_seconds": 10, "cancel_grace_seconds": 0.3,
                "budget": {
                    "max_generations": 2,
                    "max_input_tokens_per_generation": 10,
                    "max_output_tokens_per_generation": 10,
                    "max_attempts_per_generation": 1,
                },
                "argv": [sys.executable, "-I", "-S", str(script),
                         "--provider", "mistral", "--model", "fixture-model",
                         "--fixture", str(fixture), "--inert-no-network"],
            })
    # Hash directories cannot be used by legacy plan validator which expects
    # ordinary files: use a sealed manifest file per fixture instead.
    for job in jobs:
        path = Path(job["argv"][9])
        manifest = path.parent / "fixture-hashes.json"
        manifest.write_text(json.dumps({
            name: digest((path / name).read_bytes())
            for name in ("cli.json", "trace.json", "http.json", "capture.json", "sources.json")
        }, sort_keys=True))
        artifacts.pop(str(path))
        artifacts[str(manifest)] = digest(manifest.read_bytes())
        job["argv"][9] = str(manifest)
    plan = {
        "schema": "engine07-operational-plan-v1",
        "purpose": offline_lane.INERT_PURPOSE,
        "provider": "mistral", "model": "fixture-model",
        "receipt_sha256": digest(receipt_file.read_bytes()),
        "case_ids": ids,
        "questions": {cid: "What is the answer?" for cid in ids},
        "token_budget": 48 * 40,
        "artifacts": artifacts,
        "jobs": jobs,
    }
    return plan, receipt_file, script, ledger


def digest_fixture(path):
    return digest(b"".join(
        name.encode() + b"\0" + (Path(path) / name).read_bytes()
        for name in ("cli.json", "trace.json", "http.json", "capture.json", "sources.json")
    ))


class OfflineV3LaneTests(unittest.TestCase):
    def test_inherited_python_usercustomize_cannot_execute_in_pinned_child(self):
        # Startup hooks are not part of the SHA-pinned fixture code. An
        # inherited PYTHONUSERBASE must not activate arbitrary usercustomize.
        with tempfile.TemporaryDirectory() as t:
            root = Path(t)
            env = {**os.environ, "PYTHONUSERBASE": str(root / "user-base")}
            site = subprocess.check_output(
                [sys.executable, "-c", "import site;print(site.getusersitepackages())"],
                env=env, text=True,
            ).strip()
            Path(site).mkdir(parents=True)
            marker = root / "unexpected-child-startup"
            (Path(site) / "usercustomize.py").write_text(
                "from pathlib import Path\n"
                + f"Path({str(marker)!r}).write_text('executed')\n"
            )
            plan, receipt, script, ledger = make_plan(root)
            plan["token_budget"] = 40
            previous = os.environ.get("PYTHONUSERBASE")
            try:
                os.environ["PYTHONUSERBASE"] = str(root / "user-base")
                offline_lane.OfflineV3Runner(
                    plan, root / "lane", receipt, str(script)
                ).run()
            finally:
                if previous is None:
                    os.environ.pop("PYTHONUSERBASE", None)
                else:
                    os.environ["PYTHONUSERBASE"] = previous
            self.assertFalse(marker.exists(), "unfrozen Python startup code executed")

    def test_local_decode_then_later_429_stops_with_known_usage(self):
        # First 200 model content is unparseable, but subsequent 429
        # dominates; no later subprocess may be admitted.
        with tempfile.TemporaryDirectory() as t:
            plan, receipt, script, ledger = make_plan(
                t, {"synthetic-00-baseline": "later_429"}
            )
            root = Path(t) / "lane"
            report = offline_lane.OfflineV3Runner(
                plan, root, receipt, str(script)
            ).run()
            self.assertEqual(len(list(root.glob("*/spawn.json"))), 1)
            self.assertEqual(report["known_provider_attempts"], 2)
            self.assertEqual(report["unknown_usage_rows"], 0)
            self.assertNotEqual(report["rows"][0].get("v3_state"), "quality_invalid")
            self.assertIsNotNone(report["latch"])
            self.assertTrue(all(not x["admission"]["allowed"] for x in report["rows"][1:]))
            self.assertTrue((root / "state.json").is_file())
            self.assertTrue((root / "summary.json").is_file())

    def test_same_receipt_concurrent_processes_admit_one_child(self):
        with tempfile.TemporaryDirectory() as t:
            root = Path(t)
            plan, receipt, script, ledger = make_plan(t)
            plan["token_budget"] = 40
            plan_file = root / "run-plan.json"
            plan_file.write_text(json.dumps(plan))
            lane_script = str(Path(offline_lane.__file__).resolve())

            def attempt(i):
                return subprocess.run(
                    [sys.executable, lane_script,
                     "--plan", str(plan_file), "--receipt", str(receipt),
                     "--fixture-executable", str(script),
                     "--output", str(root / f"concurrent-{i}")],
                    capture_output=True, text=True,
                    env={"PATH": "/usr/bin:/bin"}, timeout=30,
                )

            with ThreadPoolExecutor(max_workers=2) as pool:
                results = list(pool.map(attempt, range(2)))
            # Both processes exit nonzero: one was rejected by O_EXCL, the
            # other persisted a deterministic budget suppression after row 1.
            self.assertTrue(all(r.returncode != 0 for r in results))
            spawned = list(root.glob("concurrent-*/*/spawn.json"))
            self.assertEqual(len(spawned), 1)
            self.assertEqual(len(list(ledger.glob("*.claim.json"))), 1)
            states = list(root.glob("concurrent-*/state.json"))
            self.assertEqual(len(states), 1)
            self.assertEqual(json.loads(states[0].read_text())["known_provider_attempts"], 1)

    def test_two_generate_calls_exceed_generation_cap_and_suppress_children(self):
        # One adapter correlation c1 can perform two distinct generate()
        # calls (attempts 1 and 2). max_generations=1 must stop even
        # though provider_attempts and tokens were completely accounted.
        with tempfile.TemporaryDirectory() as t:
            plan, receipt, script, ledger = make_plan(
                t, {"synthetic-00-baseline": "invalid"}
            )
            plan["jobs"][0]["budget"]["max_generations"] = 1
            report = offline_lane.OfflineV3Runner(
                plan, Path(t) / "lane", receipt, str(script)).run()
            self.assertIsNotNone(report["latch"])
            self.assertEqual(len(list((Path(t) / "lane").glob("*/spawn.json"))), 1)
            self.assertTrue(all(not r["admission"]["allowed"] for r in report["rows"][1:]))
            self.assertEqual(report["known_provider_attempts"], 2)

    def test_full48_inert_all_measured_including_invalid_and_new_run_forbidden(self):
        with tempfile.TemporaryDirectory() as t:
            # Third job initial-candidate decode, with 2 HTTP200 attempts and
            # complete usage: must be charged and followed by job four.
            chosen = {"synthetic-01-candidate": "invalid"}
            plan, receipt, script, ledger = make_plan(t, chosen)
            output = Path(t) / "lane"
            report = offline_lane.OfflineV3Runner(plan, output, receipt, str(script)).run()
            self.assertEqual(len(report["rows"]), 48)
            self.assertIsNone(report["latch"])
            self.assertTrue(report["complete"])
            invalid_rows = [r for r in report["rows"] if r.get("v3_state") == "quality_invalid"]
            self.assertEqual(len(invalid_rows), 1)
            self.assertTrue(invalid_rows[0]["complete"])
            self.assertEqual(invalid_rows[0]["outcome"]["provider_attempts"], 2)
            self.assertGreater(report["charged_reservations"], report["known_total_tokens"])
            self.assertEqual(report["unknown_usage_rows"], 0)
            self.assertEqual((output / "event-000001.json").exists(), True)
            with self.assertRaises(FileExistsError):
                offline_lane.OfflineV3Runner(
                    plan, Path(t) / "alternative-lane", receipt, str(script))
            self.assertFalse((Path(t) / "alternative-lane").exists())

    def test_unsafe_display_latches_and_suppresses_all_following(self):
        with tempfile.TemporaryDirectory() as t:
            plan, receipt, script, ledger = make_plan(
                t, {"synthetic-00-baseline": "visible"}
            )
            report = offline_lane.OfflineV3Runner(
                plan, Path(t) / "lane", receipt, str(script)).run()
            self.assertEqual(report["rows"][0]["v3_state"], "safety_hold")
            self.assertFalse(report["rows"][0]["complete"])
            self.assertEqual(len(list((Path(t) / "lane").glob("*/spawn.json"))), 1)
            self.assertTrue(all(r["admission"]["allowed"] is False for r in report["rows"][1:]))
            self.assertEqual(len(report["rows"]), 48)
            self.assertEqual(report["known_provider_attempts"], 1)

    def test_quota_and_unknown_origin_stop_before_second_child(self):
        for scenario in ("quota", "rendered_decode"):
            with self.subTest(scenario=scenario), tempfile.TemporaryDirectory() as t:
                plan, receipt, script, ledger = make_plan(
                    t, {"synthetic-00-baseline": scenario}
                )
                report = offline_lane.OfflineV3Runner(
                    plan, Path(t) / "lane", receipt, str(script)).run()
                self.assertIsNotNone(report["latch"])
                self.assertEqual(len(list((Path(t) / "lane").glob("*/spawn.json"))), 1)
                self.assertTrue(all(not r["admission"]["allowed"] for r in report["rows"][1:]))

    def test_missing_usage_and_unapproved_internal_retry_stop(self):
        for scenario in ("missing_usage", "inner_retry"):
            with self.subTest(scenario=scenario), tempfile.TemporaryDirectory() as t:
                plan, receipt, script, ledger = make_plan(
                    t, {"synthetic-00-baseline": scenario}
                )
                report = offline_lane.OfflineV3Runner(
                    plan, Path(t) / "lane", receipt, str(script)).run()
                self.assertEqual(len(list((Path(t) / "lane").glob("*/spawn.json"))), 1)
                self.assertTrue(all(not r["admission"]["allowed"] for r in report["rows"][1:]))
                self.assertNotEqual(report["latch"], None)
                self.assertGreater(report["charged_reservations"], 0)

    def test_full_cohort_token_reserve_stops_before_next_spawn(self):
        with tempfile.TemporaryDirectory() as t:
            plan, receipt, script, ledger = make_plan(t)
            plan["token_budget"] = 40  # Only one 80-token reserve fits.
            report = offline_lane.OfflineV3Runner(
                plan, Path(t) / "lane", receipt, str(script)).run()
            self.assertEqual(len(list((Path(t) / "lane").glob("*/spawn.json"))), 1)
            self.assertEqual(report["charged_reservations"], 40)
            self.assertEqual(report["latch"], "arm_reserved_budget_exhausted")
            self.assertTrue(all(not r["admission"]["allowed"] for r in report["rows"][1:]))

    def test_corrupt_fixture_and_missing_child_output_fail_closed(self):
        for sabotage in ("changed_capture", "missing_child_cli"):
            with self.subTest(sabotage=sabotage), tempfile.TemporaryDirectory() as t:
                plan, receipt, script, ledger = make_plan(t)
                first_fixture_manifest = Path(plan["jobs"][0]["argv"][9])
                folder = first_fixture_manifest.parent / "snapshot"
                if sabotage == "changed_capture":
                    (folder / "capture.json").write_text('{"malicious":"changed"}')
                else:
                    (folder / "cli.json").unlink()
                report = offline_lane.OfflineV3Runner(
                    plan, Path(t) / "lane", receipt, str(script)).run()
                self.assertEqual(len(list((Path(t) / "lane").glob("*/spawn.json"))), 1)
                self.assertFalse(report["rows"][0]["complete"])
                self.assertTrue(all(not row["admission"]["allowed"]
                                    for row in report["rows"][1:]))
                self.assertEqual(report["latch"], "interrupted_accounting")
                self.assertGreater(report["charged_reservations"], 0)
                with self.assertRaises(FileExistsError):
                    offline_lane.OfflineV3Runner(
                        plan, Path(t) / "second-lane", receipt, str(script))

    def test_inert_ledger_and_provider_reject_live_swap(self):
        with tempfile.TemporaryDirectory() as t:
            plan, receipt, script, ledger = make_plan(t)
            malicious = json.loads(json.dumps(plan))
            malicious["model"] = "ministral-8b-2512"
            with self.assertRaisesRegex(ValueError, "inert_only_no_live_provider"):
                offline_lane.OfflineV3Runner(
                    malicious, Path(t) / "lane", receipt, str(script))
            self.assertEqual(list(ledger.glob("*")), [])
            malicious = json.loads(json.dumps(plan))
            malicious["jobs"][0]["argv"][7] = "paid-model"
            with self.assertRaisesRegex(ValueError, "inert_fixture_command_mismatch"):
                offline_lane.OfflineV3Runner(
                    malicious, Path(t) / "lane", receipt, str(script))
            self.assertEqual(list(ledger.glob("*")), [])


if __name__ == "__main__":
    unittest.main()
