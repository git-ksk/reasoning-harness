//! Independent v8 corpus: normal tests replay without any filesystem writes.
//! Explicit LAUNCH_INDEPENDENT_V8_RECORD_FIRST=1 records immutable first observations.
use reasoning_harness_core::evidence_relevance::{
    EvidenceLocalQualificationV6, EvidenceRelevanceBindingProposal, EvidenceRelevanceCandidate,
    EvidenceRelevanceTargetPolicy, derive_effective_evidence_local_qualification_v17,
    materialize_evidence_relevance_v30,
};
use serde_json::{Value, json};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::process::Command;

fn create_json(path: &Path, value: &Value) {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap_or_else(|err| panic!("refusing to replace evidence {}: {err}", path.display()));
    serde_json::to_writer_pretty(&mut file, value).unwrap();
    file.write_all(b"\n").unwrap();
    file.sync_all().unwrap();
}

fn source_hash(root: &Path) -> String {
    let output = Command::new("shasum")
        .args([
            "-a",
            "256",
            "crates/reasoning-harness-core/src/evidence_relevance.rs",
        ])
        .current_dir(root)
        .output()
        .expect("source hash command");
    assert!(output.status.success());
    String::from_utf8(output.stdout)
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap()
        .to_owned()
}

#[test]
fn independent_launch_context_markers_v8() {
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../fixtures/launch-520-521-independent-v8/corpus.json"
    ))
    .unwrap();
    let manifest: Value = serde_json::from_str(include_str!(
        "../../../fixtures/launch-520-521-independent-v8/manifest.json"
    ))
    .unwrap();
    let first_mode = std::env::var("LAUNCH_INDEPENDENT_V8_RECORD_FIRST").as_deref() == Ok("1");
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let out = root.join("evaluation/launch-520-521-independent-v8");
    let mut initial_hash = None;
    let mut observations: Option<File> = None;
    if first_mode {
        let hash = source_hash(root);
        assert_eq!(
            hash,
            corpus["candidate_source_sha256"].as_str().unwrap(),
            "fixed source changed before first observation"
        );
        let head = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(root)
            .output()
            .unwrap();
        assert!(head.status.success());
        create_json(
            &out.join("first-run-start.json"),
            &json!({
                "run_identity": "independent-v8-first-api-execution",
                "author_identity": corpus["author_identity"],
                "candidate_commit": corpus["candidate_commit"],
                "candidate_source_sha256": hash,
                "corpus_sha256": manifest["corpus_sha256"],
                "checkout_head": String::from_utf8(head.stdout).unwrap().trim(),
                "offline_synthetic": true,
                "external_hard_truth_evaluation": false,
            }),
        );
        observations = Some(
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(out.join("first-observations.jsonl"))
                .expect("first observation file must be new"),
        );
        initial_hash = Some(hash);
    }
    let mut positive_count = 0usize;
    let mut negative_count = 0usize;
    let mut positive_misses = Vec::new();
    let mut negative_requested_relation = Vec::new();
    let mut false_relevant = Vec::new();
    let mut api_errors = Vec::new();
    let mut failures = Vec::new();
    for case in corpus["cases"].as_array().unwrap() {
        let id = case["case_id"].as_str().unwrap();
        let policy: EvidenceRelevanceTargetPolicy =
            serde_json::from_value(case["policy"].clone()).unwrap();
        let candidate: EvidenceRelevanceCandidate =
            serde_json::from_value(case["candidate"].clone()).unwrap();
        let proposal: EvidenceRelevanceBindingProposal =
            serde_json::from_value(case["proposal"].clone()).unwrap();
        let raw: EvidenceLocalQualificationV6 =
            serde_json::from_value(case["raw"].clone()).unwrap();
        let effective_result = derive_effective_evidence_local_qualification_v17(
            &policy,
            &candidate,
            Some(&proposal),
            Some(&raw),
        );
        let assessment_result =
            materialize_evidence_relevance_v30(&policy, &candidate, Some(&proposal), Some(&raw));
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
        let passed = if positive {
            requested && relevant && !error
        } else {
            !requested && !relevant && !error
        };
        if positive {
            positive_count += 1;
            if !passed {
                positive_misses.push(id.to_owned());
            }
        } else {
            negative_count += 1;
            if requested {
                negative_requested_relation.push(id.to_owned());
            }
            if relevant {
                false_relevant.push(id.to_owned());
            }
        }
        if error {
            api_errors.push(id.to_owned());
        }
        if !passed {
            failures.push(id.to_owned());
        }
        if let Some(file) = &mut observations {
            serde_json::to_writer(
                &mut *file,
                &json!({
                    "case_id": id,
                    "input": case,
                    "effective_v17": effective,
                    "assessment_v30": assessment,
                    "effective_error": effective_error,
                    "assessment_error": assessment_error,
                    "requested_relation": requested,
                    "relevant": relevant,
                    "passed": passed,
                }),
            )
            .unwrap();
            file.write_all(b"\n").unwrap();
            file.flush().unwrap();
        }
        println!(
            "CASE {id}: {} relation={} disposition={}",
            if passed { "PASS" } else { "FAIL" },
            effective["relation_scope"],
            assessment["disposition"]
        );
    }
    let after_hash = if first_mode {
        Some(source_hash(root))
    } else {
        None
    };
    let source_unchanged = initial_hash == after_hash;
    let summary = json!({
        "run_identity": if first_mode { "independent-v8-first-api-execution" } else { "side-effect-free-corpus-replay" },
        "candidate_commit": corpus["candidate_commit"],
        "candidate_source_sha256_before": initial_hash,
        "candidate_source_sha256_after": after_hash,
        "source_unchanged": if first_mode { json!(source_unchanged) } else { Value::Null },
        "corpus_sha256": manifest["corpus_sha256"],
        "total_cases": positive_count + negative_count,
        "positive_cases": positive_count,
        "negative_cases": negative_count,
        "passed_cases": positive_count + negative_count - failures.len(),
        "failed_case_ids": failures,
        "positive_miss_ids": positive_misses,
        "negative_requested_relation_ids": negative_requested_relation,
        "false_relevant_ids": false_relevant,
        "api_error_ids": api_errors,
        "safety_gate_passed": negative_requested_relation.is_empty() && false_relevant.is_empty() && api_errors.is_empty(),
        "positive_recall_gate_passed": positive_misses.is_empty() && api_errors.is_empty(),
        "overall_passed": failures.is_empty() && source_unchanged,
        "acceptance_scope": "offline synthetic source-local public-API evaluation; no provider/user/device/production/external-truth acceptance",
    });
    if first_mode {
        observations.as_ref().unwrap().sync_all().unwrap();
        create_json(&out.join("first-summary.json"), &summary);
    }
    println!("{}", serde_json::to_string_pretty(&summary).unwrap());
    assert!(
        source_unchanged,
        "candidate source changed during first observation"
    );
    assert!(
        failures.is_empty(),
        "independent v8 failures: {}",
        failures.join(", ")
    );
}
