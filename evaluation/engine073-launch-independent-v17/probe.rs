use reasoning_harness_core::{
    EvidenceLocalQualificationV6, EvidenceRelevanceBindingProposal, EvidenceRelevanceCandidate,
    EvidenceRelevanceTargetPolicy, derive_effective_evidence_local_qualification_v17,
    materialize_evidence_relevance_v29, materialize_evidence_relevance_v30,
};
use serde_json::{Value, json};
fn main() {
    let path = std::env::args().nth(1).expect("prebound input path");
    let cases: Vec<Value> = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    for case in cases {
        // Expected labels stay outside this engine process.
        let policy: EvidenceRelevanceTargetPolicy =
            serde_json::from_value(case["policy"].clone()).unwrap();
        let candidate: EvidenceRelevanceCandidate =
            serde_json::from_value(case["candidate"].clone()).unwrap();
        let proposal: EvidenceRelevanceBindingProposal =
            serde_json::from_value(case["proposal"].clone()).unwrap();
        let raw: EvidenceLocalQualificationV6 =
            serde_json::from_value(case["raw"].clone()).unwrap();
        let effective = derive_effective_evidence_local_qualification_v17(
            &policy,
            &candidate,
            Some(&proposal),
            Some(&raw),
        )
        .unwrap();
        let v29 =
            materialize_evidence_relevance_v29(&policy, &candidate, Some(&proposal), Some(&raw))
                .unwrap();
        let v30 =
            materialize_evidence_relevance_v30(&policy, &candidate, Some(&proposal), Some(&raw))
                .unwrap();
        println!(
            "{}",
            json!({"id": case["id"], "engine_version": reasoning_harness_core::ENGINE_VERSION,
            "effective_v17":effective,"v29":v29,"v30":v30})
        );
    }
}
