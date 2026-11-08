#!/usr/bin/env python3
"""Offline, metadata-only audit of an interrupted CI v3 native job.

Never retries a provider, authorizes a run, or interprets answer correctness.
Timing projections show risk only; maximum allowed generations are not a
prediction of how many generations an individual task actually requires.
"""
import argparse
import json
import statistics
from pathlib import Path

from contracts import require

CONTRACT = "reason-cli-normal-v3-observed-timing-audit-v1"


def analyze(plan, job_id, execution, journal):
    require(type(plan) is dict and type(plan.get("jobs")) is list, "audit_missing_frozen_jobs")
    matches = [row for row in plan["jobs"] if row.get("id") == job_id]
    require(len(matches) == 1, "audit_ambiguous_job_id")
    job = matches[0]
    require(type(execution) is dict and
            execution.get("operational_failure") == "native_process_timeout" and
            execution.get("streams_complete") is False, "audit_not_timeout_evidence")
    require(type(journal) is list and journal and all(type(row) is dict and
            type(row.get("event")) is dict for row in journal), "audit_invalid_journal")
    events = [row["event"] for row in journal]
    started = [event for event in events if event.get("kind") == "request"]
    require(len(started) >= 2 and len({event.get("request_id") for event in started}) == len(started),
            "audit_insufficient_distinct_requests")
    times = [event.get("start_elapsed_ns") for event in started]
    require(all(type(value) is int and value >= 0 for value in times) and
            times == sorted(times) and len(set(times)) == len(times), "audit_invalid_start_order")
    received = [event for event in events if event.get("kind") == "response" and
                event.get("http_status") == 200]
    completed = [event for event in events if event.get("kind") == "completed"]
    aborted = [event for event in events if event.get("kind") == "aborted"]
    started_ids = {x["request_id"] for x in started}
    for name, rows in (("received", received), ("completed", completed), ("aborted", aborted)):
        ids = [row.get("request_id") for row in rows]
        require(all(ident in started_ids for ident in ids), "audit_"+name+"_without_request")
        require(len(set(ids)) == len(ids), "audit_duplicate_"+name+"_record")
    require(not ({x["request_id"] for x in completed} & {x["request_id"] for x in aborted}),
            "audit_completed_and_aborted_same_request")
    require(type(job.get("timeout_seconds")) in (int, float) and job["timeout_seconds"] > 0,
            "audit_invalid_timeout")
    allowed = job["budget"]["max_generations"]
    require(type(allowed) is int and allowed >= len(started), "audit_invalid_generation_ceiling")
    argv = job.get("argv", [])
    require(type(argv) is list and argv.count("--max-model-calls") == 1,
            "audit_missing_actual_cli_call_ceiling")
    position = argv.index("--max-model-calls")
    require(position + 1 < len(argv) and type(argv[position + 1]) is str
            and argv[position + 1].isdigit(), "audit_invalid_cli_call_ceiling")
    cli_cap = int(argv[position + 1])
    require(0 < cli_cap <= allowed and cli_cap >= len(started),
            "audit_actual_cli_call_ceiling_inconsistent")
    intervals = [(after-before)/1e9 for before, after in zip(times, times[1:])]
    median_gap = statistics.median(intervals)
    # This is a hypothetical 24-request scenario, NOT a forecast of task
    # complexity, provider speed or achieved case coverage.
    hypothetical_final_start = times[0]/1e9 + (cli_cap-1)*median_gap
    reservation_max_start = times[0]/1e9 + (allowed-1)*median_gap
    return {
        "contract": CONTRACT,
        "job_id": job_id,
        "measurement_state": "operational_stop",
        "not_model_quality_failure": True,
        "job_timeout_seconds": job["timeout_seconds"],
        "allowed_generation_ceiling_not_expected_count": allowed,
        "actual_cli_call_ceiling_not_expected_count": cli_cap,
        "recorded_request_starts": len(started),
        "http200_responses": len(received),
        "completed_attempts": len(completed),
        "aborted_attempts": len(aborted),
        "observed_start_seconds": [round(v/1e9, 2) for v in times],
        "observed_median_interstart_seconds": round(median_gap, 2),
        "frozen_min_interstart_seconds": job["http_limits"]["min_generation_start_interval_ms"]/1000,
        "hypothetical_last_start_seconds_if_cli_call_ceiling_reached_at_observed_median":
            round(hypothetical_final_start, 1),
        "hypothetical_last_start_exceeds_job_timeout": hypothetical_final_start >= job["timeout_seconds"],
        "hypothetical_reservation_ceiling_last_start_seconds_not_executable_by_actual_cli":
            round(reservation_max_start, 1),
        "cannot_infer_current_provider_quota_from_http200": True,
        "actual_case_answer_complete": False,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--plan", type=Path, required=True)
    parser.add_argument("--journal", type=Path, required=True)
    parser.add_argument("--execution", type=Path, required=True)
    parser.add_argument("--job-id", required=True)
    args = parser.parse_args()
    plan = json.loads(args.plan.read_bytes())
    execution = json.loads(args.execution.read_bytes())
    journal = [json.loads(row) for row in args.journal.read_text().splitlines() if row.strip()]
    result = analyze(plan, args.job_id, execution, journal)
    print(json.dumps(result, sort_keys=True, ensure_ascii=False))


if __name__ == "__main__":
    main()
