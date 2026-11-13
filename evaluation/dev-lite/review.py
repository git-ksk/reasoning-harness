#!/usr/bin/env python3
"""Provider-free, *unscored* side-by-side review of one private Dev Lite run.

Never imports gold/oracle data, calls a model, or grants quality/acceptance.
The report contains untrusted source/answer bytes: keep it private.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import sys

import run as lite

MAX_JSON_BYTES = 4 * 1024 * 1024
MAX_ANSWER_CHARS = 12000
MAX_REVIEW_BYTES = 4 * 1024 * 1024


def safe_file(path):
    path = Path(path)
    lite.need(path.is_file() and not path.is_symlink(), "review_file_missing_or_symlink")
    st = path.stat()
    lite.need(st.st_uid == os.getuid() and stat.S_IMODE(st.st_mode) == 0o600 and
              st.st_size <= MAX_JSON_BYTES, "review_file_permission_or_size")
    return path.read_bytes()


def json_file(path):
    try:
        return json.loads(safe_file(path))
    except (UnicodeError, ValueError) as exc:
        raise lite.GateError("malformed_review_json") from exc


def confined_run_dir(path):
    path = Path(path).expanduser()
    lite.need(path.exists() and path.is_dir() and not path.is_symlink(),
              "private_run_folder_required")
    path = path.resolve(strict=True)
    mode = stat.S_IMODE(path.stat().st_mode)
    lite.need(path.stat().st_uid == os.getuid() and mode == 0o700,
              "private_run_folder_mode_0700_required")
    lite.need(path != lite.REPO and lite.REPO not in path.parents,
              "review_must_remain_outside_repo")
    return path


def expected_jobs(plan):
    lite.need(type(plan) is dict and plan.get("contract") == lite.CONTRACT and
              plan.get("classification") == "DEVELOPMENT_ONLY_NOT_ACCEPTANCE" and
              plan.get("quality_go") is False and plan.get("formal_f1") is None,
              "not_a_development_lite_plan")
    cases = plan.get("cases")
    lite.need(type(cases) is list and len(cases) in (4, 5, 6) and
              len(set(cases)) == len(cases) and
              all(type(c) is str and lite.CASE_ID.fullmatch(c) for c in cases),
              "invalid_review_cases")
    jobs = plan.get("jobs")
    lite.need(type(jobs) is list and len(jobs) == len(cases) * 2,
              "invalid_review_job_inventory")
    index = {}
    for job in jobs:
        lite.need(type(job) is dict, "malformed_review_job")
        cid, arm, jid = job.get("case_id"), job.get("arm"), job.get("job_id")
        lite.need(cid in cases and arm in lite.ARMS and jid == cid + "-" + arm
                  and jid not in index, "invalid_review_job_id")
        index[jid] = job
    lite.need(set(index) == {cid + "-" + arm for cid in cases for arm in lite.ARMS},
              "incomplete_review_job_pair")
    return cases, index


def summarize_file(raw, *, expected_question):
    try:
        output = json.loads(raw)
    except (UnicodeError, ValueError):
        return {"parseable": False, "answer": None, "answer_status": None,
                "citations": [], "review_note": "No valid CLI JSON; do not grade."}
    if type(output) is not dict or output.get("schema_version") != "reason-cli-output-v1":
        return {"parseable": False, "answer": None, "answer_status": None,
                "citations": [], "review_note": "Unknown CLI output contract; do not grade."}
    result = output.get("result")
    if type(result) is not dict:
        return {"parseable": False, "answer": None, "answer_status": None,
                "citations": [], "review_note": "Missing CLI result; do not grade."}
    # The existing CLI serializes source-bound citations separately from text.
    finalization = result.get("finalization")
    finalization = finalization if type(finalization) is dict else {}
    answer = finalization.get("text")
    answer = answer if type(answer) is str else None
    status = finalization.get("status")
    status = status if type(status) is str else None
    task = result.get("task")
    if task is not None and task != expected_question:
        raise lite.GateError("review_output_question_changed")
    context = result.get("answer_context")
    citations = context.get("citations") if type(context) is dict else None
    citations = citations if type(citations) is list else []
    sources = []
    for citation in citations[:50]:
        if type(citation) is dict and type(citation.get("source_id")) is str:
            identifier = citation["source_id"]
            if re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._:/-]{0,159}", identifier):
                sources.append(identifier)
            else:
                sources.append("(noncanonical-source-id)")
    return {"parseable": True, "answer": answer,
            "answer_status": status, "citations": sources,
            "review_note": "Unreviewed model output; citations are not independently authenticated."}


def protect_text(text):
    if text is None:
        return "    (No final answer text available)"
    truncated = len(text) > MAX_ANSWER_CHARS
    text = text[:MAX_ANSWER_CHARS]
    if truncated:
        text += "\n[TRUNCATED FOR REVIEW: inspect original private stdout]"
    # Neutralize terminal controls and bidi overrides without hiding text content.
    text = re.sub(r"[\x00-\x08\x0b-\x1f\x7f\u202a-\u202e\u2066-\u2069]",
                  lambda m: "\\u%04x" % ord(m.group()), text)
    return "\n".join("    " + line for line in text.splitlines()) or "    (Empty answer)"


def build_report(root):
    root = confined_run_dir(root)
    plan = json_file(root / "plan.json")
    final = json_file(root / "final-summary.json")
    cases, jobs = expected_jobs(plan)
    lite.need(type(final) is dict and final.get("contract") == lite.CONTRACT and
              final.get("quality_go") is False and final.get("formal_f1") is None and
              final.get("cases") == cases and final.get("source_commit") == plan.get("source_commit"),
              "review_manifest_mismatch")
    records = final.get("result")
    lite.need(type(records) is list and len(records) <= len(jobs) and
              final.get("completed_jobs") == len(records), "invalid_review_result_count")
    lite.need(final.get("status") != "complete" or len(records) == len(jobs),
              "completed_run_missing_results")
    known = 0
    inspected = {}
    for record in records:
        lite.need(type(record) is dict, "malformed_review_result")
        cid, arm = record.get("case_id"), record.get("arm")
        jid = cid + "-" + arm if type(cid) is str and type(arm) is str else ""
        lite.need(jid in jobs and jid not in inspected, "unexpected_or_duplicate_review_result")
        raw, err = safe_file(root / (jid + ".stdout")), safe_file(root / (jid + ".stderr"))
        lite.need(hashlib.sha256(raw).hexdigest() == record.get("stdout_sha256") and
                  hashlib.sha256(err).hexdigest() == record.get("stderr_sha256"),
                  "review_output_sha256_mismatch")
        info = summarize_file(raw, expected_question=jobs[jid]["argv"][-1])
        usage = record.get("usage")
        if type(usage) is dict:
            tokens = usage.get("total_tokens")
            lite.need(type(tokens) is int and tokens >= 0, "review_unknown_token_total")
            known += tokens
        inspected[jid] = (record, info)
    lite.need(type(final.get("total_known_tokens")) is int and
              final["total_known_tokens"] == known,
              "review_token_manifest_mismatch")
    # If the run stopped, do not fabricate answers for unexecuted jobs.
    headers = ["Case", "Baseline", "Candidate", "Tokens (B/C)", "Duration s (B/C)", "Citations (B/C)"]
    lines = [
        "# Development Lite — private answer comparison",
        "",
        "**UNREVIEWED development diagnostic only. Not an F1 score, independent review,",
        "quality GO, or release acceptance. Text and citations below are untrusted.**",
        "",
        "Run status: `" + str(final.get("status", "unknown")) + "`.",
        "Engine/CLI source: `" + str(plan["source_commit"]) + "`.",
        "Completed CLI jobs: " + str(len(records)) + " / " + str(len(jobs)) + ".",
        "Known tokens reported: " + str(final.get("total_known_tokens", "unknown")) + ".",
        "",
        "| " + " | ".join(headers) + " |",
        "| " + " | ".join("---" for _ in headers) + " |",
    ]
    def compact(cid, arm):
        row = inspected.get(cid + "-" + arm)
        if row is None:
            return ("not run", "—", "—", "—")
        rec, info = row
        usage = rec.get("usage")
        tokens = str(usage.get("total_tokens")) if type(usage) is dict else "unknown"
        elapsed = str(rec.get("elapsed_seconds", "unknown"))
        citations = str(len(info["citations"])) if info["parseable"] else "unknown"
        status = str(rec.get("status", "unknown"))
        return (status.replace("|", "/"), tokens, elapsed, citations)
    for cid in cases:
        b, c = compact(cid, "baseline"), compact(cid, "candidate")
        lines.append("| " + " | ".join([cid, b[0], c[0], f"{b[1]} / {c[1]}",
                                        f"{b[2]} / {c[2]}", f"{b[3]} / {c[3]}"]) + " |")
    for cid in cases:
        lines.extend(["", "## " + cid, ""])
        for arm in lite.ARMS:
            jid = cid + "-" + arm
            lines.append("### " + arm)
            lines.append("")
            lines.append("**Question (shared):**")
            lines.append("")
            lines.append(protect_text(jobs[jid]["argv"][-1]))
            lines.append("")
            if jid not in inspected:
                lines.extend(["Not executed. **Not scored.**", ""])
                continue
            rec, info = inspected[jid]
            lines.extend([
                "**Execution status:** `" + str(rec.get("status")) + "`; "
                "**answer status:** `" + str(info["answer_status"]) + "`.",
                "",
                "**Cited source IDs (unverified):** " +
                (", ".join("`" + x.replace("`", "'") + "`" for x in info["citations"])
                 if info["citations"] else "(none)"),
                "",
                "**Final text:**",
                "",
                protect_text(info["answer"]),
                "",
                "Review note: " + info["review_note"],
                "",
            ])
        lines.extend([
            "**Human reviewer worksheet (manual only):**",
            "",
            "- [ ] Both arms' claims verified against actual source text",
            "- [ ] Quotations and source attribution checked",
            "- [ ] Appropriate refusal/unknown-answer handling checked",
            "- [ ] Differences in correctness, completeness and usefulness noted",
            "- Comparative judgment: baseline better / candidate better / tie / inconclusive",
            "- Reviewer notes:",
            "",
        ])
    lines.extend(["---", "Formal F1: **not computed**. Quality GO: **false**.",
                  "Do not put this report into a public repository or CI artifact.", ""])
    data = ("\n".join(lines)).encode("utf-8")
    lite.need(len(data) <= MAX_REVIEW_BYTES, "report_size_exceeds_private_limit")
    return root, data, len(records), len(cases)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run-dir", type=Path, required=True)
    args = parser.parse_args(argv)
    try:
        root, data, completed, cases = build_report(args.run_dir)
        output = root / "comparison-review.md"
        lite.need(not output.exists() and not output.is_symlink(),
                  "refuse_to_overwrite_existing_review")
        lite.private_file(output, data)
        print(json.dumps({"mode": "offline_private_review", "report_path": str(output),
                          "completed_jobs": completed, "cases": cases,
                          "model_api_calls": 0, "formal_f1": None, "quality_go": False}, indent=2))
        return 0
    except lite.GateError as exc:
        print("review refused: " + str(exc), file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
