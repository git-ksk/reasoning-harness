//! Fresh independent corpus runner. Expectations and this runner are frozen before first execution.
//! Never overwrite first-run evidence; any subsequent execution must fail before calling APIs.
use reasoning_harness_core::evidence_relevance::{
    EvidenceLocalQualificationV6, EvidenceRelevanceBindingProposal, EvidenceRelevanceCandidate,
    EvidenceRelevanceTargetPolicy, derive_effective_evidence_local_qualification_v17,
    materialize_evidence_relevance_v30,
};
use serde_json::{Value, json};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::process::Command;

fn create_json(path: &Path, value: &Value) {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)
        .unwrap_or_else(|err| panic!("refusing to replace evidence {}: {err}", path.display()));
    serde_json::to_writer_pretty(&mut file, value).unwrap();
    file.write_all(b"\n").unwrap();
    file.sync_all().unwrap();
}

fn source_hash(root: &Path) -> String {
    let output = Command::new("shasum")
        .args(["-a", "256", "crates/reasoning-harness-core/src/evidence_relevance.rs"])
        .current_dir(root).output().expect("source hash command");
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap().split_whitespace().next().unwrap().to_owned()
}

#[test]
#[ignore = "Historical v1 first observation is RED; do not rerun/rescore or overwrite it"]
fn fresh_independent_launch_context_markers_first_observation() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../fixtures/launch-520-521-independent-v1/corpus.json"
    )).unwrap();
    let manifest: Value = serde_json::from_str(include_str!(
        "../../../fixtures/launch-520-521-independent-v1/manifest.json"
    )).unwrap();
    let out = root.join("evaluation/launch-520-521-independent-v1");
    fs::create_dir_all(&out).unwrap();
    let before_hash = source_hash(root);
    assert_eq!(before_hash, corpus["candidate_source_sha256"].as_str().unwrap(), "fixed source changed before test");
    let head = Command::new("git").args(["rev-parse", "HEAD"]).current_dir(root).output().unwrap();
    assert!(head.status.success());
    create_json(&out.join("first-run-start.json"), &json!({
        "run_identity": "first-api-execution-only",
        "author_identity": corpus["author_identity"],
        "candidate_commit": corpus["candidate_commit"],
        "candidate_source_sha256": before_hash,
        "corpus_sha256": manifest["corpus_sha256"],
        "checkout_head": String::from_utf8(head.stdout).unwrap().trim(),
        "offline_synthetic": true,
        "live_provider_acceptance": false,
        "external_hard_truth_evaluation": false,
    }));
    let mut observations = OpenOptions::new().write(true).create_new(true)
        .open(out.join("first-observations.jsonl")).expect("first observations must be new");
    let mut positive_total = 0usize;
    let mut negative_total = 0usize;
    let mut positive_misses = Vec::new();
    let mut negative_requested_relation = Vec::new();
    let mut false_relevant = Vec::new();
    let mut api_errors = Vec::new();
    let mut all_failures = Vec::new();
    for case in corpus["cases"].as_array().unwrap() {
        let id = case["case_id"].as_str().unwrap();
        let policy: EvidenceRelevanceTargetPolicy = serde_json::from_value(case["policy"].clone()).unwrap();
        let candidate: EvidenceRelevanceCandidate = serde_json::from_value(case["candidate"].clone()).unwrap();
        let proposal: EvidenceRelevanceBindingProposal = serde_json::from_value(case["proposal"].clone()).unwrap();
        let raw: EvidenceLocalQualificationV6 = serde_json::from_value(case["raw"].clone()).unwrap();
        let effective_result = derive_effective_evidence_local_qualification_v17(
            &policy, &candidate, Some(&proposal), Some(&raw),
        );
        let assessment_result = materialize_evidence_relevance_v30(
            &policy, &candidate, Some(&proposal), Some(&raw),
        );
        let (effective, effective_error) = match effective_result {
            Ok(value) => (serde_json::to_value(value).unwrap(), Value::Null),
            Err(err) => (Value::Null, json!(format!("{err:?}"))),
        };
        let (assessment, assessment_error) = match assessment_result {
            Ok(value) => (serde_json::to_value(value).unwrap(), Value::Null),
            Err(err) => (Value::Null, json!(format!("{err:?}"))),
        };
        let requested = effective["relation_scope"] == "requested_relation";
        let relevant = assessment["disposition"] == "relevant";
        let positive = case["label"] == "positive";
        let error = !effective_error.is_null() || !assessment_error.is_null();
        let passed = if positive { requested && relevant && !error } else { !requested && !relevant && !error };
        if positive {
            positive_total += 1;
            if !passed { positive_misses.push(id.to_owned()); }
        } else {
            negative_total += 1;
            if requested { negative_requested_relation.push(id.to_owned()); }
            if relevant { false_relevant.push(id.to_owned()); }
        }
        if error { api_errors.push(id.to_owned()); }
        if !passed { all_failures.push(id.to_owned()); }
        let observation = json!({
            "case_id": id,
            "input": case,
            "effective_v17": effective,
            "assessment_v30": assessment,
            "effective_error": effective_error,
            "assessment_error": assessment_error,
            "requested_relation": requested,
            "relevant": relevant,
            "passed": passed,
        });
        serde_json::to_writer(&mut observations, &observation).unwrap();
        observations.write_all(b"\n").unwrap();
        observations.flush().unwrap();
        println!("CASE {id}: {} relation={} disposition={}", if passed { "PASS" } else { "FAIL" },
            effective["relation_scope"], assessment["disposition"]);
    }
    observations.sync_all().unwrap();
    let after_hash = source_hash(root);
    let unchanged = after_hash == before_hash;
    let summary = json!({
        "run_identity": "first-api-execution-only",
        "candidate_commit": corpus["candidate_commit"],
        "candidate_source_sha256_before": before_hash,
        "candidate_source_sha256_after": after_hash,
        "source_unchanged": unchanged,
        "corpus_sha256": manifest["corpus_sha256"],
        "total_cases": positive_total + negative_total,
        "positive_cases": positive_total,
        "negative_cases": negative_total,
        "passed_cases": positive_total + negative_total - all_failures.len(),
        "failed_case_ids": all_failures,
        "positive_miss_ids": positive_misses,
        "negative_requested_relation_ids": negative_requested_relation,
        "false_relevant_ids": false_relevant,
        "api_error_ids": api_errors,
        "safety_gate_passed": false_relevant.is_empty() && negative_requested_relation.is_empty() && api_errors.is_empty(),
        "positive_recall_gate_passed": positive_misses.is_empty() && api_errors.is_empty(),
        "overall_passed": all_failures.is_empty() && unchanged,
        "acceptance_scope": "offline synthetic public-API evaluation; no live-provider, user, device, or external factual-truth acceptance",
    });
    create_json(&out.join("first-summary.json"), &summary);
    println!("{}", serde_json::to_string_pretty(&summary).unwrap());
    assert!(unchanged, "candidate source changed while evaluating");
    assert!(all_failures.is_empty(), "independent first-observation failures: {}", all_failures.join(", "));
}
