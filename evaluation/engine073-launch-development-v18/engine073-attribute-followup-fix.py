from pathlib import Path
import json
p=Path('crates/reasoning-harness-core/src/evidence_relevance.rs');s=p.read_text()
s=s.replace('''    let printed_label_date = object.contains(&"printed")
        && object.contains(&"date")
        && object
            .iter()
            .any(|word| ["label", "labels", "card", "carton", "property"].contains(word));''','''    let printed_label_date = object.contains(&"printed") && object.contains(&"date");''',1)
a=s.index('fn launch_retraction_other_object_v17(');b=s.index('    let folded = statement.to_ascii_lowercase();',a);s=s[:b]+'''    let statement = if target_identity_phrases(policy).iter().any(|target| normalized_phrase_matches(statement,target)) {
        statement
    } else {
        statement.trim().strip_suffix(" separately")
            .or_else(|| statement.trim().strip_suffix(" as a separate correction"))
            .unwrap_or(statement)
    };
    let folded = statement.to_ascii_lowercase();
    for prefix in ["No actual launch of ", "No launch of ", "No actual release of "] {
        if let Some(owner) = statement.strip_prefix(prefix).and_then(|body| body.strip_suffix(" occurred")) {
            if launch_named_subject_v17(owner) && !target_identity_phrases(policy).iter().any(|target| normalized_phrase_matches(owner,target)) { return true; }
        }
    }
''' +s[b+len('    let folded = statement.to_ascii_lowercase();\n'):]
needle='''                embedded
                    || (!target_identity_phrases(policy)''';new='''                let owned_foreign = ["the launch claim owned by ", "the release claim owned by "].iter().any(|prefix| {
                    object.strip_prefix(prefix).is_some_and(|owner| launch_named_subject_v17(owner) && !target_identity_phrases(policy).iter().any(|target| normalized_phrase_matches(owner,target)))
                });
                embedded || owned_foreign
                    || (!target_identity_phrases(policy)''';assert s.count(needle)==1;s=s.replace(needle,new)
needle='''split = Some((head.to_owned(), body.to_owned()));''';assert s.count(needle)==1;s=s.replace(needle,'''split = Some((head.to_owned(), body.strip_prefix("and ").unwrap_or(body).to_owned()));''',1)
cases=[]
for v in [12,18]:
 for c in json.loads(Path(f'evaluation/engine073-launch-independent-v{v}/cases.json').read_text())['cases']:
  if c['id'] in ['li12-011','li18-001','li18-015','li18-017','li18-031','li18-053','li18-057','li18-059']:cases.append(c['input'])
Path('/private/tmp/engine073-development-helper-diagnostics18.json').write_text(json.dumps(cases,indent=2)+'\n')
s+='''
#[cfg(test)]
mod development_scope_diagnostics18 {
    use super::*;
    #[test]
    fn saved_consumed_clause_diagnostics() {
        let cases: serde_json::Value=serde_json::from_str(include_str!("/private/tmp/engine073-development-helper-diagnostics18.json")).unwrap();
        for input in cases.as_array().unwrap() {
            let policy: EvidenceRelevanceTargetPolicy=serde_json::from_value(input["policy"].clone()).unwrap();
            let candidate: EvidenceRelevanceCandidate=serde_json::from_value(input["candidate"].clone()).unwrap();
            let objects=launch_typed_objects_v17(&policy,&candidate);
            eprintln!("CASE {} governing={}",input["id"],launch_governing_context_blocks_v17(&policy,&candidate));
            for signal in &candidate.signals {
                let segments=launch_adverse_segments_v17(&policy,&signal.text);
                for (i,segment) in segments.iter().enumerate() {
                    eprintln!("SEG {:?} property={} frame={} nominal={} foreign={} typed={:?}",segment.raw,launch_explicit_property_context_v17(&policy,&segment.raw),launch_property_correction_frame_v17(&policy,&segment.raw),launch_complete_nominal_property_v17(&policy,&segment.raw),launch_retraction_other_object_v17(&policy,&segment.raw),launch_typed_reference_v17(&policy,&segment.raw,&objects,i.checked_sub(1).and_then(|x|segments.get(x)),segments.get(i+1)));
                }
            }
        }
    }
}
''';p.write_text(s)
