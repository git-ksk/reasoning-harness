from pathlib import Path
p=Path('crates/reasoning-harness-core/tests/launch_context_markers_520_521.rs');s=p.read_text();a=s.index('fn foreign_printed_properties_keep_outside_ownership_and_raw_risks');b=s.index('\n#[test]',a);t=s[a:b];start=t.index('    let case = json!({"signals":[{"kind":"excerpt","text":"Silver Lens was launched. Object: the font');t=t[:start]+'''    // Historical qualification re-evaluates local scope risks; unsupported
    // advisory risk alone is not authority. New views require raw None, so a
    // non-None advisory risk must retain the inherited qualification risk.
    for tail in ["", " The product column is missing."] {
        let case=json!({"signals":[{"kind":"excerpt","text":format!("Silver Lens was launched. Object: the font of the literal printed words \\\"no launch owner supplied\\\" on Maple Court label. We deny the accuracy of this Object, only its font assertion.{tail}")} ]});
        let (policy,candidate,proposal,mut raw)=inputs(&case);
        for risk in ["ownership_scope","context_gap","identity_mapping","multiple"] {
            raw.scope_risk=serde_json::from_value(json!(risk)).unwrap();
            let inherited=reasoning_harness_core::derive_effective_evidence_local_qualification_v16(&policy,&candidate,Some(&proposal),Some(&raw)).unwrap();
            let effective=derive_effective_evidence_local_qualification_v17(&policy,&candidate,Some(&proposal),Some(&raw)).unwrap();
            assert_eq!(effective.scope_risk,inherited.scope_risk,"{tail}:{risk}");
            assert_eq!(serde_json::to_value(&raw).unwrap()["scope_risk"],risk);
            if !tail.is_empty() {
                assert_ne!(effective.scope_risk,reasoning_harness_core::EvidenceLocalBlockingReason::None);
                assert_ne!(materialize_evidence_relevance_v30(&policy,&candidate,Some(&proposal),Some(&raw)).unwrap().disposition,Disposition::Relevant);
            }
        }
    }
}
''';p.write_text(s[:a]+t+s[b:]);print('root expectation corrected to frozen v16 advisory/local-risk contract; actual gap controls retained')
