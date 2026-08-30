use std::{collections::BTreeMap, fs, path::PathBuf};

use reasoning_harness_core::{
    CorpusCaseStatus, CorpusManifest, CorpusSuite, validate_corpus_manifest,
};

#[test]
fn corpus_v1_covers_every_primary_fixture_with_stable_identity() {
    let fixture_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures");
    let manifest_path = fixture_root.join("corpus/v1.json");
    let manifest: CorpusManifest =
        serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
    validate_corpus_manifest(&manifest).unwrap();

    assert_eq!(manifest.corpus_version, "1.0.0");
    assert_eq!(manifest.score_compatibility_id, "corpus-v1");
    assert_eq!(manifest.cases.len(), 41);
    assert!(
        manifest
            .cases
            .iter()
            .all(|case| case.status == CorpusCaseStatus::Active)
    );

    let mut suite_counts = BTreeMap::new();
    for case in &manifest.cases {
        *suite_counts.entry(case.suite).or_insert(0usize) += 1;
        let path = fixture_root.join(&case.fixture_path);
        assert!(
            path.is_file(),
            "manifest path is missing: {}",
            path.display()
        );
        let value: serde_json::Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
        assert_eq!(value["id"].as_str(), Some(case.fixture_id.as_str()));
        let suite_prefix = match case.suite {
            CorpusSuite::Claim => "claim",
            CorpusSuite::Causal => "causal",
            CorpusSuite::Assumption => "assumption",
            CorpusSuite::EvidenceQualification => "evidence_qualification",
        };
        assert_eq!(case.case_id, format!("{suite_prefix}:{}", case.fixture_id));
    }

    assert_eq!(suite_counts.get(&CorpusSuite::Claim), Some(&20));
    assert_eq!(suite_counts.get(&CorpusSuite::Causal), Some(&8));
    assert_eq!(suite_counts.get(&CorpusSuite::Assumption), Some(&5));
    assert_eq!(
        suite_counts.get(&CorpusSuite::EvidenceQualification),
        Some(&8)
    );
}

#[test]
fn public_corpus_manifest_is_provider_neutral_and_secret_free() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/corpus/v1.json");
    let text = fs::read_to_string(path).unwrap().to_lowercase();
    for forbidden in [
        "api_key",
        "authorization",
        "bearer ",
        "mistral",
        "gemini",
        "nvidia",
        "openai",
    ] {
        assert!(
            !text.contains(forbidden),
            "manifest contains forbidden term: {forbidden}"
        );
    }
}
