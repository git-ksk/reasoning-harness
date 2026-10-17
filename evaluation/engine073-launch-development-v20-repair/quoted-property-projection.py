from pathlib import Path
p=Path('crates/reasoning-harness-core/src/evidence_relevance.rs');s=p.read_text()
# Full, balanced typed printed property declarations are checked before any
# event/unknown-owner words inside their literal value.
a=s.index('fn launch_governing_context_blocks_v17(');b=s.index('            if launch_unavailable_owner_note_v17',a)
s=s[:b]+'''            if segment.raw.split_once(':').is_some_and(|(label, value)| {
                ["object", "subject", "item"].contains(&normalized(label).as_str())
                    && launch_printed_quoted_property_v17(policy, value.trim())
            }) {
                continue;
            }
'''+s[b:]
# Extend the existing successor-only view, keeping its exact raw bindings,
# target-owned launch and governing-source checks. Replace only the balanced
# quote body of a proven printed property; all outside source bytes remain.
a=s.index('fn launch_scope_annotation_view_v17(');b=s.index('                let text = normalized(&segment.raw);',a)
s=s[:b]+'''                let printed_property = segment.raw.split_once(':').is_some_and(|(label, value)| {
                    ["object", "subject", "item"].contains(&normalized(label).as_str())
                        && launch_printed_quoted_property_v17(policy, value.trim())
                });
                if !segment.question && !segment.denied && printed_property {
                    let start = segment.raw.find(['"', '“', '‘']).unwrap();
                    let quote = segment.raw[start..].chars().next().unwrap();
                    let closing = match quote { '“' => '”', '‘' => '’', _ => '"' };
                    let body = start + quote.len_utf8();
                    let end = body + segment.raw[body..].find(closing).unwrap();
                    signal_changed = true;
                    return format!("{}printed lettering{}", &segment.raw[..body], &segment.raw[end..]);
                }
'''+s[b:]
p.write_text(s)
p=Path('crates/reasoning-harness-core/tests/launch_context_markers_520_521.rs');s=p.read_text();old='''    for quote in [
        "Silver Lens does not exist",
        "we withdraw the entire source",
        "Silver Lens was never launched",
    ] {''';new='''    for quote in [
        "Silver Lens does not exist",
        "we withdraw the entire source",
        "Silver Lens was never launched",
        "we withdraw a launch claim with unknown owner",
        "required source context is missing",
    ] {''';assert s.count(old)==1;s=s.replace(old,new)
s+='''

#[test]
fn literal_property_projection_keeps_raw_risks_and_actual_omissions() {
    let declaration = "Subject: the material of the Silver Lens label plate carrying the literal text \\\"we withdraw a launch claim with unknown owner\\\". We withdraw this Subject, only the plate-material assertion.";
    for tail in ["", " The product column is missing.", " The required source context is omitted."] {
        let case = json!({"signals":[{"kind":"excerpt","text":format!("Silver Lens was launched. {declaration}{tail}")} ]});
        let (policy,candidate,proposal,mut raw)=inputs(&case);
        for risk in ["none","ownership_scope","context_gap","identity_mapping","multiple"] {
            raw.scope_risk=serde_json::from_value(json!(risk)).unwrap();
            let q=derive_effective_evidence_local_qualification_v17(&policy,&candidate,Some(&proposal),Some(&raw)).unwrap();
            let a=materialize_evidence_relevance_v30(&policy,&candidate,Some(&proposal),Some(&raw)).unwrap();
            if tail.is_empty() && risk=="none" { assert_eq!(a.disposition,Disposition::Relevant,"{q:?}"); }
            else { assert_ne!(a.disposition,Disposition::Relevant,"{tail}:{risk}:{q:?}"); assert_ne!(q.scope_risk,reasoning_harness_core::EvidenceLocalBlockingReason::None); }
        }
    }
}
'''
p.write_text(s)
print('successor-only bounded quote projection written; raw risks/outside bytes preserved')
