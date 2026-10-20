from pathlib import Path
p=Path('crates/reasoning-harness-core/src/evidence_relevance.rs');s=p.read_text();old='''                    "a",
                    "separate" | "further" | "additional",
                    "event",''';new='''                    "a" | "an",
                    "separate" | "further" | "additional",
                    "event",''';assert s.count(old)==1;p.write_text(s.replace(old,new,1))
p=Path('crates/reasoning-harness-core/tests/launch_context_markers_520_521.rs');s=p.read_text();old='''            "We withdraw the entire source.",
            "The product column is missing.",
        ] {
            cases.push(json!({"id":format!("outside:{quoted}:{denial}")''';new='''            "We withdraw the entire source.",
        ] {
            cases.push(json!({"id":format!("outside:{quoted}:{denial}")''';assert s.count(old)==1;s=s.replace(old,new,1)
needle='''fn foreign_printed_properties_keep_outside_ownership_and_raw_risks() {
    let mut cases = Vec::new();''';addition='''fn foreign_printed_properties_keep_outside_ownership_and_raw_risks() {
    // Local relation and blocking risk are orthogonal: a visible launch can
    // remain RequestedRelation while a missing owner column blocks acceptance.
    for tail in ["The product column is missing.", "Required source context is omitted."] {
        let case=json!({"signals":[{"kind":"excerpt","text":format!("Silver Lens was launched. We deny the font assertion about the printed sentence \\\"required source context is missing\\\" on Maple Court guide. {tail}")} ]});
        let (policy,candidate,proposal,raw)=inputs(&case);
        let q=derive_effective_evidence_local_qualification_v17(&policy,&candidate,Some(&proposal),Some(&raw)).unwrap();
        assert_ne!(q.scope_risk,reasoning_harness_core::EvidenceLocalBlockingReason::None);
        assert_ne!(materialize_evidence_relevance_v30(&policy,&candidate,Some(&proposal),Some(&raw)).unwrap().disposition,Disposition::Relevant);
    }
    let mut cases = Vec::new();''';assert s.count(needle)==1;s=s.replace(needle,addition,1);p.write_text(s)
print('a/an finite adjunct fixed; root omission check uses actual orthogonal risk/disposition contract')
