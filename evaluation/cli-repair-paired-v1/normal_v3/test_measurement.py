"""Entirely offline prospective measurement controls; no real provider/corpus."""
import copy
import json
import shutil
import tempfile
import unittest
from pathlib import Path

import annotate
import collect
import contracts
import score


def demo_fixture(root, *, phase=None, label="positive", text="", status="requires_verification"):
    """Standalone synthetic data; does not claim source/runtime admission."""
    root = Path(root)
    lane = root / "lane"
    job = lane / "case-01-baseline"
    job.mkdir(parents=True)
    receipt = root / "receipt.json"
    receipt.write_text('{"purpose":"synthetic-v3-HTTP-zero"}\n')
    receipt_sha = contracts.digest(receipt.read_bytes())
    diagnostic = None
    if phase:
        diagnostic = {"origin": "local_model_output_decode", "phase": phase}
    failure = {"failure_class": "protocol", "message": "fixture decode"}
    if diagnostic:
        failure["model_output_diagnostic"] = diagnostic
    attempt_count = 2 if phase else 1
    usage = {
        "tracked_from_session_start": True, "model_calls": 1,
        "provider_attempts": attempt_count,
        "input_tokens": attempt_count * 2,
        "output_tokens": attempt_count * 3,
        "total_tokens": attempt_count * 5,
    }
    is_initial = phase == "initial_candidate"
    result = (
        {
            "status": "failed", "failure": failure,
            "partial_diagnostics": {"usage": {"run": usage}, "complete": False},
        }
        if is_initial
        else {
            "output_contract": "reason-natural-output-v6",
            "configuration": {"provider": "mistral", "model": "fixture-model"},
            "usage": {"run": usage},
            "finalization": {"text": text, "status": status, "uncovered_propositions": []},
        }
    )
    if not is_initial:
        result["task"] = "What is the answer?"
    if phase and not is_initial:
        if phase == "final_render":
            result["rendering_failure"] = failure
        else:
            result["investigation"] = {"generation_failure": failure}
    cli = {"schema_version": "reason-cli-output-v1", "command": "ask", "result": result}
    http = [
        {"request_id": "call-" + str(i), "provider": "mistral", "model": "fixture-model",
         "http_status": 200, "input_tokens": 2, "output_tokens": 3, "total_tokens": 5}
        for i in range(attempt_count)
    ]
    events = []
    for i in range(attempt_count):
        identity = {"correlation_id": "c1", "attempt": i + 1,
                    "provider": "mistral", "model": "fixture-model"}
        events.append({"sequence": len(events), "kind": "model_request", **identity,
                       "request": {"task": "What is the answer? with structured JSON"}})
        events.append({
            "sequence": len(events), "kind": "model_response", **identity,
            "response": {"provider_attempts": 1,
                         "usage": {"input_tokens": 2, "output_tokens": 3, "total_tokens": 5}},
        })
    if not is_initial:
        events.append({
            "sequence": len(events), "kind": "finalization_state",
            "phase": "finalization", "round": 1, "stage": "answer_safety",
            "result": {"text": text, "status": status, "uncovered_propositions": []},
        })
    trace = {
        "contract": "reason-natural-diagnostic-trace-v1", "provider": "mistral",
        "model": "fixture-model", "completion_status": "partial" if is_initial else "completed",
        "events": events,
    }
    citations = (
        [{"id": "cit-1", "source_id": "s1", "source_version": "synthetic", "start_byte": 0, "end_byte": 6}]
        if text and status == "grounded_answer" else []
    )
    capture = {
        "display_artifacts": [{"id": "answer", "kind": "final_answer", "text": text}],
        "display_citations": citations,
        "admitted_source_ids": ["s1"] if citations else [],
    }
    sources = {"s1": {"text": "Answer is four", "source_version": "synthetic"}}
    files = {"cli.json": cli, "trace.json": trace, "http.json": http,
             "capture.json": capture, "sources.json": sources}
    hashes = {}
    for name, value in files.items():
        path = job / name
        path.write_text(json.dumps(value, sort_keys=True), encoding="utf-8")
        hashes[name] = contracts.digest(path.read_bytes())
    manifest = {
        "contract": contracts.PLAN_ID, "provider": "mistral",
        "model": "fixture-model", "executor_id": "fixture-executor",
        "receipt_sha256": receipt_sha,
        "rows": [{
            "job_id": "case-01-baseline", "case_id": "case-01",
            "arm": "baseline", "question": "What is the answer?", "files": hashes,
        }],
    }
    plan = root / "capture-manifest.json"
    plan.write_text(json.dumps(manifest), encoding="utf-8")
    gold = {
        "contract": "reason-cli-normal-v3-gold-v1", "receipt_sha256": receipt_sha,
        "cases": [{"case_id": "case-01", "question": "What is the answer?", "label": label,
                   "required_groups": ["g1"] if label == "positive" else []}],
    }
    gold_file = root / "gold.json"
    gold_file.write_text(json.dumps(gold), encoding="utf-8")
    return plan, lane, gold_file, receipt, job


def review_of(obs, gold, *, decision="invalid", groups=None, violations=None):
    return {
        "contract": contracts.REVIEW_ID,
        "job_id": obs["job_id"],
        "observation_sha256": obs["_sha"],
        "gold_sha256": contracts.keyed_digest(gold),
        "reviewer": {
            "role": "agent", "id": "independent-synthetic-reviewer",
            "independent_of_execution": True, "full_source_context_reviewed": True,
        },
        "decision": decision, "correct_groups": groups or [], "incorrect_assertions": 0,
        "hard_violations": violations or [],
        "citation_checks": [
            {"id": c["id"], "supported": True, "literal_exact": True}
            for c in obs["display_citations"]
        ],
        "readable_and_actionable": decision == "answer",
        "segments": (
            [{"kind": "fact", "start_byte": 0,
              "end_byte": len(obs["display_artifacts"][0]["text"].encode("utf-8")),
              "citation_ids": [c["id"] for c in obs["display_citations"]]}]
            if decision == "answer" else
            [{"kind": "nonfact", "start_byte": 0,
              "end_byte": len(obs["display_artifacts"][0]["text"].encode("utf-8")),
              "citation_ids": []}]
            if obs["display_artifacts"][0]["text"] else []
        ),
        "safety_audit_complete": True,
        "notes": "Independent synthetic source-level and display review, not model inference.",
    }


class V3Measurement(unittest.TestCase):
    def test_six_phases_actual_json_paths(self):
        for phase in sorted(contracts.PHASES):
            with self.subTest(phase=phase), tempfile.TemporaryDirectory() as d:
                plan, lane, gold, receipt, job = demo_fixture(d, phase=phase)
                observations = Path(d) / "observations"
                collected = collect.collect(plan, lane, observations)
                self.assertEqual(len(collected), 1)
                self.assertEqual(collected[0]["measurement_state"], "local_decode_observed")
                self.assertEqual(collected[0]["diagnostics"][0]["phase"], phase)
                self.assertEqual(collected[0]["accounted_usage"]["total_tokens"], 10)
                self.assertFalse(collect.read_json(observations / "index.json")["continuation_authorized"])
                packets = annotate.make_packets(observations, gold, Path(d) / "packets")
                self.assertEqual(packets[0]["review_state"], "PENDING_INDEPENDENT_SEMANTIC_ADJUDICATION")
                self.assertFalse(packets[0]["auto_graded"])

    def test_provider_protocol_not_local(self):
        with tempfile.TemporaryDirectory() as d:
            plan, lane, gold, receipt, job = demo_fixture(d, phase="initial_candidate")
            data = collect.read_json(job / "cli.json")
            data["result"]["failure"].pop("model_output_diagnostic")
            self.rewrite(plan, job, "cli.json", data)
            row = collect.collect(plan, lane, Path(d) / "obs")[0]
            self.assertEqual(row["measurement_state"], "operational_stop")
            self.assertFalse(row["diagnostics"][0]["local_decode"])

    def test_forged_diagnostic_is_not_permitted(self):
        for origin in ("provider_protocol", "local_model_output_decode"):
            with tempfile.TemporaryDirectory() as d:
                plan, lane, gold, receipt, job = demo_fixture(d, phase="initial_candidate")
                payload = collect.read_json(job / "cli.json")
                payload["result"]["failure"]["model_output_diagnostic"]["origin"] = origin
                if origin == "local_model_output_decode":
                    payload["result"]["failure"]["failure_class"] = "rate_limit"
                self.rewrite(plan, job, "cli.json", payload)
                with self.assertRaisesRegex(ValueError, "invalid_typed_model_output_diagnostic"):
                    collect.collect(plan, lane, Path(d) / "obs")

    def test_recovered_fallback_can_be_semantically_correct(self):
        with tempfile.TemporaryDirectory() as d:
            plan, lane, gold_file, receipt, job = demo_fixture(
                d, phase="final_render", text="Answer is 4", status="grounded_answer"
            )
            obs_dir = Path(d) / "obs"
            row = collect.collect(plan, lane, obs_dir)[0]
            self.assertEqual(row["measurement_state"], "local_decode_observed")
            self.assertEqual(row["terminal_status"], "grounded_answer")
            obs = {**row, "_sha": contracts.digest((obs_dir / "case-01-baseline.json").read_bytes())}
            review = review_of(obs, annotate.read_json(gold_file)["cases"][0],
                               decision="answer", groups=["g1"])
            annotate.validate_review(review, row, obs["_sha"], annotate.read_json(gold_file)["cases"][0], "fixture-executor")
            scored = score.score_case(row, review, annotate.read_json(gold_file)["cases"][0])
            self.assertEqual(scored["tp"], 1)
            self.assertFalse(scored["quality_invalid"])
        self.assertTrue(scored["recovered_local_decode"])

    def test_missing_or_changed_http_trace_display_fails_closed(self):
        cases = [
            ("http.json", lambda a: a[0].update(total_tokens=99), "inconsistent_http_usage"),
            ("http.json", lambda a: a[0].update(http_status=429), None),
            ("http.json", lambda a: a[0].pop("input_tokens"), "http_attempt_missing_keys"),
            ("trace.json", lambda a: (a["events"].pop(-2), a["events"][-1].update(sequence=len(a["events"])-1)), "pending_model_call"),
            ("trace.json", lambda a: a["events"][1]["response"]["usage"].update(output_tokens=7), "trace_http_token_mismatch"),
            ("trace.json", lambda a: a.update(model="wrong-model"), "trace_provider_or_contract_mismatch"),
            ("capture.json", lambda a: a["display_artifacts"][0].update(text="forged"), "display_or_terminal_mismatch"),
        ]
        for filename, change, expected in cases:
            with self.subTest(filename=filename, expected=expected), tempfile.TemporaryDirectory() as d:
                plan, lane, gold, receipt, job = demo_fixture(d, text="Answer is 4", status="grounded_answer")
                data = collect.read_json(job / filename)
                change(data)
                self.rewrite(plan, job, filename, data)
                if expected is None:
                    row = collect.collect(plan, lane, Path(d) / "obs")[0]
                    self.assertEqual(row["measurement_state"], "operational_stop")
                else:
                    with self.assertRaises(ValueError) as err:
                        collect.collect(plan, lane, Path(d) / "obs")
                    self.assertIn(expected, str(err.exception))

    def test_unreported_usage_and_modified_raw_are_rejected(self):
        with tempfile.TemporaryDirectory() as d:
            plan, lane, gold, receipt, job = demo_fixture(d, phase="candidate_regeneration")
            data = collect.read_json(job / "http.json")
            data[0]["output_tokens"] = None
            self.rewrite(plan, job, "http.json", data)
            with self.assertRaisesRegex(ValueError, "unknown_http_usage"):
                collect.collect(plan, lane, Path(d) / "obs")
        with tempfile.TemporaryDirectory() as d:
            plan, lane, gold, receipt, job = demo_fixture(d)
            (job / "trace.json").write_text("{}")
            with self.assertRaisesRegex(ValueError, "changed_capture_file"):
                collect.collect(plan, lane, Path(d) / "obs")

    def test_unsafe_negative_overrides_invalid_and_positive_invalid_is_fn(self):
        positive = {"case_id": "positive", "question": "q+", "label": "positive", "required_groups": ["g1", "g2"]}
        negative = {"case_id": "negative", "question": "q-", "label": "negative", "required_groups": []}
        obs = {"job_id": "a", "case_id": "positive", "arm": "baseline",
               "measurement_state": "local_decode_observed",
               "diagnostics": [{"failure_class": "protocol", "phase": "investigation_plan"}]}
        review = {"decision": "invalid", "correct_groups": [], "incorrect_assertions": 0,
                  "hard_violations": [], "citation_checks": [], "segments": []}
        item = score.score_case(obs, review, positive)
        self.assertEqual((item["fn"], item["claim_fn"], item["tp"]), (1, 2, 0))
        obs["case_id"] = "negative"
        item = score.score_case(obs, review, negative)
        self.assertEqual((item["tp"], item["fp"], item["fn"], item["tn"]), (0, 0, 0, 0))
        self.assertTrue(item["quality_invalid"])
        review["hard_violations"] = ["unsafe displayed assertion"]
        item = score.score_case(obs, review, negative)
        self.assertEqual((item["fp"], item["tn"], item["hard_violations"]), (1, 0, 1))
        self.assertFalse(item["quality_invalid"])

    def test_strict_semantic_review_cannot_promote_failed_or_blank_to_withhold(self):
        with tempfile.TemporaryDirectory() as d:
            plan, lane, gold_file, receipt, job = demo_fixture(d, phase="initial_candidate", label="negative")
            obs_dir = Path(d) / "obs"
            row = collect.collect(plan, lane, obs_dir)[0]
            gold = annotate.read_json(gold_file)["cases"][0]
            obs_hash = contracts.digest((obs_dir / "case-01-baseline.json").read_bytes())
            review = review_of({**row, "_sha": obs_hash}, gold, decision="withhold")
            with self.assertRaisesRegex(ValueError, "empty_or_failed_output_cannot_be_withhold"):
                annotate.validate_review(review, row, obs_hash, gold, "fixture-executor")
            review["reviewer"]["id"] = "fixture-executor"
            review["decision"] = "invalid"
            with self.assertRaisesRegex(ValueError, "nonindependent_or_unreviewed_sources"):
                annotate.validate_review(review, row, obs_hash, gold, "fixture-executor")

    def test_partial_score_is_null_and_exclusive(self):
        with tempfile.TemporaryDirectory() as d:
            plan, lane, gold_file, receipt, job = demo_fixture(d, phase="initial_candidate")
            obs_dir = Path(d) / "obs"
            packet_dir = Path(d) / "packets"
            collect.collect(plan, lane, obs_dir)
            annotate.make_packets(obs_dir, gold_file, packet_dir)
            row = annotate.read_observations(obs_dir)[1]["case-01-baseline"]
            gold = annotate.read_json(gold_file)["cases"][0]
            review = review_of({**row[0], "_sha": row[1]}, gold)
            reviews_dir = Path(d) / "reviews"
            reviews_dir.mkdir()
            review_path = reviews_dir / "case-01-baseline.review.json"
            review_path.write_text(json.dumps(review))
            freeze = {
                "contract": "reason-cli-normal-v3-review-index-v1",
                "receipt_sha256": contracts.digest(receipt.read_bytes()),
                "observation_index_sha256": contracts.digest((obs_dir / "index.json").read_bytes()),
                "packet_index_sha256": contracts.digest((packet_dir / "packet-index.json").read_bytes()),
                "executor_id": "fixture-executor",
                "reviews": {"case-01-baseline": contracts.digest(review_path.read_bytes())},
            }
            (reviews_dir / "review-index.json").write_text(json.dumps(freeze))
            result = score.score(obs_dir, packet_dir, gold_file, reviews_dir, receipt, Path(d) / "score")
            self.assertFalse(result["cohort_measured"])
            self.assertIsNone(result["arms"]["baseline"]["case_metrics"])
            self.assertFalse(result["quality_go"])
            with self.assertRaises(FileExistsError):
                score.score(obs_dir, packet_dir, gold_file, reviews_dir, receipt, Path(d) / "score")
            review_path.write_text(json.dumps({**review, "decision": "withhold"}))
            with self.assertRaisesRegex(ValueError, "modified_review_file"):
                score.score(obs_dir, packet_dir, gold_file, reviews_dir, receipt, Path(d) / "score-tampered")

    def test_full24_metrics_never_promote_negative_invalid(self):
        gold = {}
        rows = []
        for n in range(24):
            cid = "case-" + str(n)
            gold[cid] = {"case_id": cid, "question": "q" + str(n), "label": "positive" if n < 12 else "negative",
                         "required_groups": ["g1"] if n < 12 else []}
            for arm in ("baseline", "candidate"):
                invalid = n == 13 and arm == "candidate"
                observation = {
                    "job_id": arm + "-" + cid, "case_id": cid, "arm": arm,
                    "measurement_state": "local_decode_observed" if invalid else "normal_observed",
                    "diagnostics": [
                        {"failure_class": "protocol", "phase": "candidate_regeneration"}
                    ] if invalid else [],
                }
                decision = "invalid" if invalid else ("answer" if n < 12 else "withhold")
                review = {
                    "decision": decision, "correct_groups": ["g1"] if n < 12 else [],
                    "incorrect_assertions": 0, "hard_violations": [],
                    "citation_checks": [{"id": "citation", "supported": True, "literal_exact": True}]
                    if n < 12 else [],
                    "segments": [
                        {"kind": "fact", "start_byte": 0, "end_byte": 1,
                         "citation_ids": ["citation"]}
                    ] if n < 12 else [],
                }
                rows.append(score.score_case(observation, review, gold[cid]))
        result = score.report(rows, gold)
        self.assertTrue(result["cohort_measured"])
        candidate = result["arms"]["candidate"]
        self.assertEqual(candidate["quality_invalid"], 1)
        self.assertEqual(candidate["case_confusion"]["TN"], 11)
        self.assertEqual(candidate["case_confusion"]["FP"], 0)
        self.assertEqual(candidate["invalid_rate"], 1/24)
        self.assertEqual(candidate["valid_answer_coverage"], 12/24)
        self.assertEqual(candidate["valid_finalization_coverage"], 23/24)
        self.assertEqual(candidate["case_metrics"]["f1"], 1.0)
        self.assertFalse(candidate["semantic_thresholds_met"])
        self.assertFalse(result["all_semantic_thresholds_met"])

    def test_cross_case_binding_rejects_wrong_question_and_gold(self):
        with tempfile.TemporaryDirectory() as d:
            plan, lane, gold_file, receipt, job = demo_fixture(d, text="Answer is 4")
            payload = collect.read_json(job / "cli.json")
            payload["result"]["task"] = "Some other case"
            self.rewrite(plan, job, "cli.json", payload)
            with self.assertRaisesRegex(ValueError, "cli_case_question_mismatch"):
                collect.collect(plan, lane, Path(d) / "obs")
        with tempfile.TemporaryDirectory() as d:
            plan, lane, gold_file, receipt, job = demo_fixture(d, phase="initial_candidate")
            trace = collect.read_json(job / "trace.json")
            trace["events"][0]["request"]["task"] = "unrelated prompt"
            self.rewrite(plan, job, "trace.json", trace)
            with self.assertRaisesRegex(ValueError, "unbound_failed_job_question"):
                collect.collect(plan, lane, Path(d) / "obs")
        with tempfile.TemporaryDirectory() as d:
            plan, lane, gold_file, receipt, job = demo_fixture(d, text="Answer is 4")
            observed = Path(d) / "obs"
            collect.collect(plan, lane, observed)
            gold = annotate.read_json(gold_file)
            gold["cases"][0]["question"] = "wrong frozen gold question"
            gold_file.write_text(json.dumps(gold))
            with self.assertRaisesRegex(ValueError, "gold_case_question_mismatch"):
                annotate.make_packets(observed, gold_file, Path(d) / "packets")

    def test_success_trace_request_must_match_case_question(self):
        with tempfile.TemporaryDirectory() as d:
            plan, lane, gold_file, receipt, job = demo_fixture(
                d, text="Answer is 4", status="grounded_answer"
            )
            trace = collect.read_json(job / "trace.json")
            trace["events"][0]["request"]["task"] = "Another unrelated question"
            self.rewrite(plan, job, "trace.json", trace)
            with self.assertRaisesRegex(ValueError, "unbound_job_question"):
                collect.collect(plan, lane, Path(d) / "obs")

    def test_cross_source_citation_version_and_range_mismatch_rejected(self):
        for change, expected in [
            (lambda x: x["display_citations"][0].update(source_version="forged"),
             "citation_identity_or_version_mismatch"),
            (lambda x: x["display_citations"][0].update(end_byte=999),
             "citation_outside_actual_display"),
            (lambda x: x["admitted_source_ids"].clear(),
             "citation_outside_admitted_sources"),
        ]:
            with tempfile.TemporaryDirectory() as d:
                plan, lane, gold_file, receipt, job = demo_fixture(d, text="Answer is 4", status="grounded_answer")
                capture = collect.read_json(job / "capture.json")
                change(capture)
                self.rewrite(plan, job, "capture.json", capture)
                with self.assertRaisesRegex(ValueError, expected):
                    collect.collect(plan, lane, Path(d) / "obs")

    def test_stale_receipt_and_rebuilt_review_packet_block_scoring(self):
        with tempfile.TemporaryDirectory() as d:
            plan, lane, gold_file, receipt, job = demo_fixture(d, text="Answer is 4", status="grounded_answer")
            obs_dir = Path(d) / "obs"
            packets = Path(d) / "packets"
            collect.collect(plan, lane, obs_dir)
            annotate.make_packets(obs_dir, gold_file, packets)
            row, obs_sha = annotate.read_observations(obs_dir)[1]["case-01-baseline"]
            gold = annotate.read_json(gold_file)["cases"][0]
            review = review_of({**row, "_sha": obs_sha}, gold, decision="answer", groups=["g1"])
            reviews = Path(d) / "reviews"
            reviews.mkdir()
            review_path = reviews / "case-01-baseline.review.json"
            review_path.write_text(json.dumps(review))
            frozen = {
                "contract": "reason-cli-normal-v3-review-index-v1",
                "receipt_sha256": contracts.digest(receipt.read_bytes()),
                "observation_index_sha256": contracts.digest((obs_dir / "index.json").read_bytes()),
                "packet_index_sha256": contracts.digest((packets / "packet-index.json").read_bytes()),
                "executor_id": "fixture-executor",
                "reviews": {"case-01-baseline": contracts.digest(review_path.read_bytes())},
            }
            (reviews / "review-index.json").write_text(json.dumps(frozen))
            receipt.write_text('{"purpose":"tampered"}')
            with self.assertRaisesRegex(ValueError, "receipt_byte_mismatch"):
                score.score(obs_dir, packets, gold_file, reviews, receipt, Path(d) / "no-score")
            receipt.write_text('{"purpose":"synthetic-v3-HTTP-zero"}\n')
            p = packets / "case-01-baseline.review-packet.json"
            packet = annotate.read_json(p)
            packet["full_sources"] = {}
            p.write_text(json.dumps(packet))
            index_path = packets / "packet-index.json"
            index = annotate.read_json(index_path)
            index["rows"][0]["sha256"] = contracts.digest(p.read_bytes())
            index_path.write_text(json.dumps(index))
            frozen["packet_index_sha256"] = contracts.digest(index_path.read_bytes())
            (reviews / "review-index.json").write_text(json.dumps(frozen))
            with self.assertRaisesRegex(ValueError, "packet_observation_or_gold_mismatch"):
                score.score(obs_dir, packets, gold_file, reviews, receipt, Path(d) / "no-score-packet")

    def test_operational_error_never_creates_full_cohort_f1(self):
        gold = {f"c{i}": {"case_id": f"c{i}", "question": "question",
                          "label": "positive" if i < 12 else "negative",
                          "required_groups": ["g"] if i < 12 else []}
                for i in range(24)}
        rows = []
        for arm in ("baseline", "candidate"):
            for i in range(24):
                row = {"job_id": f"{arm}-{i}", "case_id": f"c{i}", "arm": arm,
                       "measurement_state": "operational_stop" if i == 10 and arm == "candidate" else "normal_observed",
                       "diagnostics": []}
                reviewer = {"decision": "invalid" if i == 10 and arm == "candidate" else
                            "answer" if i < 12 else "withhold",
                            "correct_groups": ["g"] if i < 12 and not (i == 10 and arm == "candidate") else [],
                            "incorrect_assertions": 0, "hard_violations": [],
                            "citation_checks": [{"id": "c", "supported": True, "literal_exact": True}]
                            if i < 12 else [],
                            "segments": [{"kind": "fact", "start_byte": 0, "end_byte": 1, "citation_ids": ["c"]}]
                            if i < 12 and not (i == 10 and arm == "candidate") else []}
                rows.append(score.score_case(row, reviewer, gold[f"c{i}"]))
        result = score.report(rows, gold)
        self.assertTrue(result["arms"]["baseline"]["observation_complete"])
        self.assertFalse(result["arms"]["candidate"]["observation_complete"])
        self.assertIsNone(result["arms"]["candidate"]["case_metrics"])
        self.assertFalse(result["all_semantic_thresholds_met"])

    def test_full48_synthetic_capture_review_score(self):
        """Real three-stage integration, but entirely local/synthetic and no runtime admission."""
        with tempfile.TemporaryDirectory() as work:
            root = Path(work)
            common_lane = root / "full-lane"
            common_lane.mkdir()
            receipt = root / "full-receipt.json"
            receipt.write_text('{"purpose":"synthetic-v3-HTTP-zero"}\n')
            gold_cases = []
            capture_rows = []
            for n in range(24):
                cid = f"task-{n:02d}"
                question = f"What is answer to synthetic task {n}?"
                positive = n < 12
                gold_cases.append({
                    "case_id": cid, "question": question,
                    "label": "positive" if positive else "negative",
                    "required_groups": ["g1"] if positive else [],
                })
                for arm in ("baseline", "candidate"):
                    invalid = (n == 13 and arm == "candidate")
                    sub = root / ("fixture-" + cid + "-" + arm)
                    sub.mkdir()
                    manifest_path, _, _, _, old_job = demo_fixture(
                        sub,
                        phase="final_render" if invalid else None,
                        text="" if invalid else ("Answer is 4" if positive else "Withhold: unresolved"),
                        status="grounded_answer" if positive else "requires_verification",
                        label="positive" if positive else "negative",
                    )
                    cli = collect.read_json(old_job / "cli.json")
                    cli["result"]["task"] = question
                    trace = collect.read_json(old_job / "trace.json")
                    trace["events"][0]["request"]["task"] = question + " structured JSON"
                    (old_job / "cli.json").write_text(json.dumps(cli))
                    (old_job / "trace.json").write_text(json.dumps(trace))
                    job_id = cid + "-" + arm
                    new_job = common_lane / job_id
                    shutil.copytree(old_job, new_job)
                    hashes = {
                        name: contracts.digest((new_job / name).read_bytes())
                        for name in contracts.FILES
                    }
                    capture_rows.append({
                        "job_id": job_id, "case_id": cid, "arm": arm,
                        "question": question, "files": hashes,
                    })
            receipt_sha = contracts.digest(receipt.read_bytes())
            manifest = {
                "contract": contracts.PLAN_ID, "provider": "mistral",
                "model": "fixture-model", "executor_id": "fixture-executor",
                "receipt_sha256": receipt_sha, "rows": capture_rows,
            }
            capture_manifest = root / "full-manifest.json"
            capture_manifest.write_text(json.dumps(manifest))
            gold_file = root / "full-gold.json"
            gold_file.write_text(json.dumps({
                "contract": "reason-cli-normal-v3-gold-v1",
                "receipt_sha256": receipt_sha, "cases": gold_cases,
            }))
            observations = root / "full-observations"
            packets = root / "full-packets"
            reviews = root / "full-reviews"
            out = root / "full-first-score"
            collected = collect.collect(capture_manifest, common_lane, observations)
            self.assertEqual(len(collected), 48)
            self.assertEqual(len(annotate.make_packets(observations, gold_file, packets)), 48)
            reviews.mkdir()
            obs_index, obs_rows = annotate.read_observations(observations)
            gold_by_id = annotate.validate_gold(annotate.read_json(gold_file))
            review_hashes = {}
            for job_id, (observation, obs_hash) in obs_rows.items():
                cid = observation["case_id"]
                positive = gold_by_id[cid]["label"] == "positive"
                invalid = cid == "task-13" and observation["arm"] == "candidate"
                decision = "invalid" if invalid else ("answer" if positive else "withhold")
                review = review_of(
                    {**observation, "_sha": obs_hash},
                    gold_by_id[cid], decision=decision,
                    groups=["g1"] if positive else [],
                )
                review_file = reviews / (job_id + ".review.json")
                review_file.write_text(json.dumps(review))
                review_hashes[job_id] = contracts.digest(review_file.read_bytes())
            (reviews / "review-index.json").write_text(json.dumps({
                "contract": "reason-cli-normal-v3-review-index-v1",
                "receipt_sha256": receipt_sha,
                "observation_index_sha256": contracts.digest((observations / "index.json").read_bytes()),
                "packet_index_sha256": contracts.digest((packets / "packet-index.json").read_bytes()),
                "executor_id": "fixture-executor", "reviews": review_hashes,
            }))
            result = score.score(observations, packets, gold_file, reviews, receipt, out)
            self.assertTrue(result["cohort_measured"])
            self.assertTrue(result["arms"]["baseline"]["observation_complete"])
            self.assertTrue(result["arms"]["candidate"]["observation_complete"])
            self.assertEqual(result["arms"]["candidate"]["quality_invalid"], 1)
            self.assertEqual(result["arms"]["candidate"]["case_confusion"]["TN"], 11)
            self.assertEqual(result["arms"]["candidate"]["case_metrics"]["f1"], 1.0)
            self.assertEqual(result["arms"]["candidate"]["invalid_rate"], 1/24)
            self.assertEqual(result["arms"]["candidate"]["valid_answer_coverage"], 12/24)
            self.assertEqual(result["arms"]["candidate"]["valid_finalization_coverage"], 23/24)
            self.assertEqual(result["arms"]["candidate"]["recovered_local_decode_rows"], 0)
            self.assertFalse(result["arms"]["candidate"]["semantic_thresholds_met"])
            self.assertFalse(result["quality_go"])
            self.assertEqual(annotate.read_json(out / "report.json")["contract"], contracts.SCORE_ID)

    def test_forged_final_display_against_actual_trace_is_rejected(self):
        with tempfile.TemporaryDirectory() as d:
            plan, lane, gold, receipt, job = demo_fixture(d, text="Answer is 4", status="grounded_answer")
            trace = collect.read_json(job / "trace.json")
            trace["events"][-1]["result"]["text"] = "A different actual trace answer"
            self.rewrite(plan, job, "trace.json", trace)
            with self.assertRaisesRegex(ValueError, "trace_finalization_mismatch"):
                collect.collect(plan, lane, Path(d) / "obs")

    def test_partial_trace_is_not_a_completed_final_answer(self):
        with tempfile.TemporaryDirectory() as d:
            plan, lane, gold, receipt, job = demo_fixture(d, text="Answer is 4", status="grounded_answer")
            trace = collect.read_json(job / "trace.json")
            trace["completion_status"] = "partial"
            self.rewrite(plan, job, "trace.json", trace)
            with self.assertRaisesRegex(ValueError, "successful_cli_trace_not_completed"):
                collect.collect(plan, lane, Path(d) / "obs")

    def test_independent_factual_span_coverage_cannot_be_inflated(self):
        gold = {"case_id": "p", "question": "q", "label": "positive", "required_groups": ["g1", "g2"]}
        observation = {
            "job_id": "p-baseline", "case_id": "p", "arm": "baseline",
            "measurement_state": "normal_observed", "diagnostics": [],
        }
        review = {
            "decision": "answer", "correct_groups": ["g1", "g2"],
            "incorrect_assertions": 0, "hard_violations": [],
            "citation_checks": [{"id": "c", "supported": True, "literal_exact": True}],
            "segments": [
                {"kind": "fact", "start_byte": 0, "end_byte": 6, "citation_ids": ["c"]},
                {"kind": "fact", "start_byte": 6, "end_byte": 12, "citation_ids": []},
            ],
        }
        scored = score.score_case(observation, review, gold)
        self.assertEqual(scored["factual_claims"], 2)
        self.assertEqual(scored["supported_cited_claims"], 1)
        rows = []
        corpus = {}
        for n in range(24):
            cid = "case-" + str(n)
            corpus[cid] = {"case_id": cid, "question": "q", "label": "positive" if n < 12 else "negative",
                           "required_groups": ["g1"] if n < 12 else []}
            for arm in ("baseline", "candidate"):
                row = dict(scored)
                row.update(job_id=arm+"-"+cid, case_id=cid, arm=arm)
                if n >= 12:
                    row.update(tp=0, fn=0, tn=1, claim_tp=0, claim_fn=0,
                               claim_denominator=0, factual_claims=0, supported_cited_claims=0,
                               citation_count=0, citation_passed_count=0, citation_evaluable=False)
                rows.append(row)
        result = score.report(rows, corpus)
        self.assertEqual(result["arms"]["candidate"]["citation_coverage"], 0.5)
        self.assertFalse(result["arms"]["candidate"]["semantic_thresholds_met"])

    def test_review_must_cover_every_display_byte_and_citation(self):
        with tempfile.TemporaryDirectory() as d:
            plan, lane, gold_file, receipt, job = demo_fixture(
                d, text="Answer is 4", status="grounded_answer"
            )
            observations = Path(d) / "obs"
            row = collect.collect(plan, lane, observations)[0]
            gold = annotate.read_json(gold_file)["cases"][0]
            obs_hash = contracts.digest((observations / "case-01-baseline.json").read_bytes())
            review = review_of({**row, "_sha": obs_hash}, gold, decision="answer", groups=["g1"])
            review["segments"] = [
                {"kind": "fact", "start_byte": 0, "end_byte": 6, "citation_ids": ["cit-1"]}
            ]
            with self.assertRaisesRegex(ValueError, "display_segments_incomplete"):
                annotate.validate_review(review, row, obs_hash, gold, "fixture-executor")
            review["segments"][0]["end_byte"] = len("Answer is 4".encode())
            annotate.validate_review(review, row, obs_hash, gold, "fixture-executor")
            review["segments"][0]["citation_ids"] = []
            with self.assertRaisesRegex(ValueError, "orphan_display_citation"):
                annotate.validate_review(review, row, obs_hash, gold, "fixture-executor")

    @staticmethod
    def rewrite(plan, job, filename, value):
        path = job / filename
        path.write_text(json.dumps(value, sort_keys=True), encoding="utf-8")
        manifest = collect.read_json(plan)
        manifest["rows"][0]["files"][filename] = contracts.digest(path.read_bytes())
        plan.write_text(json.dumps(manifest), encoding="utf-8")


if __name__ == "__main__":
    unittest.main()
