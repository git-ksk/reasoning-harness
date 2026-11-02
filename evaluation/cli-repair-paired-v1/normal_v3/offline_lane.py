#!/usr/bin/env python3
"""HTTP-zero synthetic tests of the ACTUAL durable subprocess scheduler.

Not admitted for model use. No Mistral credentials or real model requests.
The trustworthy provider journal, source and image proof remain future gates.
"""
import argparse
import importlib.util
import json
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
LEGACY = ROOT / "evaluation/engine07-realworld-runner-v1/runner.py"
sys.path.insert(0, str(LEGACY.parent))
spec = importlib.util.spec_from_file_location("v3_preserved_scheduler", LEGACY)
scheduler = importlib.util.module_from_spec(spec)
spec.loader.exec_module(scheduler)
from guard import Outcome, FailureClass
# Versioned copy of the current ordinary-CLI operational bridge; the
# preserved legacy runner and historical v1/v2 bridge remain unchanged.
import current_bridge
scheduler.outcome_from_normal = current_bridge.outcome_from_normal
from collect import collect_row
from contracts import FILES, digest, require, safe_id

INERT_PURPOSE = "SYNTHETIC_V3_HTTP_ZERO_CONTROL_NOT_MODEL_EXECUTION"
# Independently inspected Python fixture: reads five local JSON files, prints
# one local CLI envelope. No socket, HTTP library, subprocess or provider.
# The caller cannot substitute arbitrary fixture code via a new plan digest.
PINNED_INERT_DRIVER_SHA256 = (
    "4a10d86620503cd4e7998078a2906dc00b534c6568853990e13d8a3c5ff17902"
)


def verify_inert_plan(plan, receipt, fixture_executable):
    require(type(plan) is dict and plan.get("purpose") == INERT_PURPOSE
            and plan.get("provider") == "mistral"
            and plan.get("model") == "fixture-model"
            and plan.get("schema") == "engine07-operational-plan-v1",
            "inert_only_no_live_provider")
    require(type(receipt) is dict and receipt.get("purpose") == INERT_PURPOSE
            and receipt.get("model_calls_allowed") is False, "inert_receipt_required")
    require(plan.get("receipt_sha256") == digest(
        json.dumps(receipt, sort_keys=True).encode()), "inert_receipt_binding")
    require(type(fixture_executable) is str
            and fixture_executable.startswith("/")
            and fixture_executable in plan["artifacts"], "unfrozen_fixture_driver")
    require(digest(Path(fixture_executable).read_bytes())
            == PINNED_INERT_DRIVER_SHA256, "unknown_inert_driver_bytes")
    ids = plan.get("case_ids")
    require(type(ids) is list and len(ids) == 24 and len(set(ids)) == 24,
            "inert_fixed_24_case_inventory")
    jobs = plan.get("jobs")
    require(type(jobs) is list and len(jobs) == 48, "inert_exact_48_jobs")
    seen_fixture = set()
    for i, case in enumerate(ids):
        safe_id(case, "unsafe_case_id")
        arms = ("baseline", "candidate") if i % 2 == 0 else ("candidate", "baseline")
        for j, arm in enumerate(arms):
            job = jobs[2 * i + j]
            require(job.get("id") == case + "-" + arm
                    and job.get("contract") == "normal"
                    and job.get("arm") == arm, "inert_pair_order_mismatch")
            argv = job.get("argv")
            require(type(argv) is list and len(argv) == 11
                    and argv[:9] == [sys.executable, "-I", "-S", fixture_executable,
                                     "--provider", "mistral", "--model",
                                     "fixture-model", "--fixture"]
                    and argv[9].startswith("/")
                    and argv[10] == "--inert-no-network"
                    and argv[9] in plan["artifacts"], "inert_fixture_command_mismatch")
            seen_fixture.add(argv[9])
    require(len(seen_fixture) == 48, "inert_duplicate_fixture")
    require(set(plan["artifacts"]) ==
            {sys.executable, fixture_executable} | seen_fixture,
            "inert_extra_or_missing_artifacts")
    require(type(plan.get("questions")) is dict
            and set(plan["questions"]) == set(ids)
            and all(type(q) is str and q for q in plan["questions"].values()),
            "inert_question_inventory")


class OfflineV3Runner(scheduler.OperationalRunner):
    """Real scheduler persistence/guards; all provider attempts are simulated."""
    def __init__(self, plan, output, receipt_file, fixture_executable):
        receipt_file = Path(receipt_file).resolve()
        receipt = json.loads(receipt_file.read_bytes())
        verify_inert_plan(plan, receipt, fixture_executable)
        ledger = Path(receipt["ledger_dir"])
        require(ledger.is_absolute() and ledger.is_dir()
                and ledger.resolve() == ledger, "inert_ledger_identity")
        self.receipt_sha256 = plan["receipt_sha256"]
        claim = ledger / (self.receipt_sha256 + ".claim.json")
        # This is an O_EXCL durable claim before any credential retrieval,
        # even when an alternate output directory is requested.
        scheduler.write_new(claim, {
            "receipt_sha256": self.receipt_sha256,
            "run_dir": str(Path(output).resolve()),
            "purpose": INERT_PURPOSE, "replay_allowed": False,
        })
        super().__init__(plan, output, lambda _provider: {})
        scheduler.write_new(self.output / "v3-inert-attestation.json", {
            "scope": INERT_PURPOSE, "live_run_authorized": False,
            "real_provider_journal": "NOT_VERIFIED",
        })

    def process(self, job, env, directory):
        # Only synthetic CLI input is needed. Do not propagate ANY host
        # environment variable, including unrecognized key names, proxy
        # settings, Python user-site paths or dynamic-loader injection.
        # The pinned Python command also requires -I -S: isolated, no site.
        env = {}
        outcome, diagnostics = super().process(job, env, directory)
        if diagnostics["termination"] is not None:
            self.guard.latch = self.guard.latch or "inert_process_terminated"
            return outcome, diagnostics
        received = json.loads((directory / "stdout.json").read_bytes())
        require(received == json.loads((directory / "cli.json").read_bytes()),
                "fixture_stdout_cli_mismatch")
        fixture_path = Path(job["argv"][9])
        require(digest(fixture_path.read_bytes())
                == self.plan["artifacts"][str(fixture_path)],
                "fixture_changed_during_run")
        frozen_hashes = json.loads(fixture_path.read_bytes())
        require(type(frozen_hashes) is dict and set(frozen_hashes) == set(FILES),
                "inert_fixture_file_inventory")
        files = {name: digest((directory / name).read_bytes()) for name in FILES}
        require(files == frozen_hashes, "inert_fixture_bytes_mismatch")
        case_id = job["id"].rsplit("-", 1)[0]
        observation = collect_row(
            {"job_id": job["id"], "case_id": case_id,
             "arm": job["arm"], "question": self.plan["questions"][case_id],
             "files": files},
            self.output, self.plan["provider"], self.plan["model"],
            self.receipt_sha256, "v3-offline-fixture",
        )
        measured = observation["accounted_usage"]
        if outcome.failure_class is None:
            require(
                outcome.tokens == measured["total_tokens"]
                and outcome.provider_attempts == measured["provider_attempts"],
                "bridge_vs_collector_usage_mismatch",
            )
        state = observation["measurement_state"]
        if outcome.failure_class not in (None, FailureClass.PROTOCOL) or (
            outcome.failure_class is FailureClass.PROTOCOL
            and state != "local_decode_observed"
        ):
            self.guard.latch = self.guard.latch or "v3_unexpected_bridge_failure"
            diagnostics["v3_state"] = "operational_stop"
            return Outcome(FailureClass.UNKNOWN, measured["total_tokens"],
                           measured["provider_attempts"]), diagnostics
        http_attempts = json.loads((directory / "http.json").read_bytes())
        # Every trace model_request is one CLI generation. A correlation
        # spans repeated generate() calls on the same adapter; its attempt
        # field distinguishes generations. Provider-internal retries are
        # counted as physical HTTP requests within that (correlation, attempt).
        per_generation = {}
        for item in http_attempts:
            key = (item["correlation_id"], item["logical_attempt"])
            per_generation[key] = per_generation.get(key, 0) + 1
        if (
            len(per_generation) > job["budget"]["max_generations"]
            or any(n > job["budget"]["max_attempts_per_generation"]
                   for n in per_generation.values())
        ):
            self.guard.latch = self.guard.latch or "v3_generation_attempt_cap_exceeded"
            diagnostics["v3_state"] = "operational_stop"
            return Outcome(FailureClass.UNKNOWN, measured["total_tokens"],
                           measured["provider_attempts"]), diagnostics
        no_assertions = (
            observation["display_artifacts"][0]["text"] == ""
            and not observation["display_citations"]
            and not observation["admitted_source_ids"]
        )
        if state == "operational_stop" or not no_assertions:
            self.guard.latch = self.guard.latch or (
                "v3_operational_stop" if state == "operational_stop"
                else "v3_unreviewed_display_hold"
            )
            diagnostics["v3_state"] = (
                "operational_stop" if state == "operational_stop"
                else "safety_hold"
            )
        elif (state == "local_decode_observed"
              and observation["terminal_status"] in (
                  "failed", "unresolved", "requires_verification", "rejected"
              )):
            diagnostics["v3_state"] = "quality_invalid"
        else:
            diagnostics["v3_state"] = "observed_no_display"
        diagnostics["v3_usage_attested_by"] = "SYNTHETIC_ONLY"
        diagnostics["v3_source_bytes_hash"] = digest(
            json.dumps(observation, sort_keys=True).encode()
        )
        # The inherited scheduler records row + observed checkpoint durably
        # BEFORE considering the next job. A measured invalid has complete=True
        # for observation only; it is never declared a quality success.
        return Outcome(
            FailureClass.UNKNOWN if diagnostics["v3_state"] in (
                "operational_stop", "safety_hold"
            ) else None,
            measured["total_tokens"], measured["provider_attempts"],
        ), diagnostics


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--plan", type=Path, required=True)
    parser.add_argument("--receipt", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--fixture-executable", required=True)
    a = parser.parse_args()
    report = OfflineV3Runner(
        json.loads(a.plan.read_bytes()), a.output, a.receipt, a.fixture_executable,
    ).run()
    print(json.dumps({
        "observed_rows": sum(row["complete"] for row in report["rows"]),
        "quality_invalid": sum(
            row.get("v3_state") == "quality_invalid" for row in report["rows"]
        ),
        "latch": report["latch"], "live_run_authorized": False,
    }))
    return 0 if report["complete"] and report["latch"] is None else 1


if __name__ == "__main__":
    raise SystemExit(main())
