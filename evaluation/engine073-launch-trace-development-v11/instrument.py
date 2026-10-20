from pathlib import Path
import hashlib,json,difflib,subprocess
r=Path.cwd();p=r/'crates/reasoning-harness-core/src/evidence_relevance.rs';s=p.read_text();expected='4daf64b918532a9004dea9807b8001ec743244b20b5cd1892c7f3a7ce316d1ef';assert hashlib.sha256(p.read_bytes()).hexdigest()==expected
Path('/private/tmp/engine073-trace-source29_v27-base.rs').open('x').write(s)
a=s.index('fn launch_governing_context_blocks_v17(');b=s.index('\nfn target_owned_launch_fact_v17(',a);t=s[a:b]
t=t.replace('for signal in candidate.signals.iter().filter(|signal| {','for (signal_index, signal) in candidate.signals.iter().enumerate().filter(|(_, signal)| {',1)
rules=['collective_event_withdrawal','unknown_owner_note','target_nonexistence','unresolved_non_event_owner','broad_colon_context','typed_reference_not_excluded','target_adverse_clause','denied_heading','unresolved_adverse_scope']
assert t.count('return true;')==len(rules)
for rule in rules:
 i=t.index('return true;')
 # Move forward past the already instrumented return on later iterations.
 if rule!=rules[0]:
  prior=t.index('"'+rules[rules.index(rule)-1]+'"');i=t.index('return true;',t.index('return true;',prior)+len('return true;'))
 insert='#[cfg(test)]\n            tests::record_launch_development_trace_v17(signal_index, index, &signal.text, &segment.raw, "'+rule+'");\n            '
 t=t[:i]+insert+t[i:]
s=s[:a]+t+s[b:]
anchor='mod tests {\n    use super::*;';assert s.count(anchor)==1
module=r'''

    std::thread_local! {
        static LAUNCH_DEVELOPMENT_TRACE: std::cell::RefCell<Vec<serde_json::Value>> = const { std::cell::RefCell::new(Vec::new()) };
    }

    pub(super) fn record_launch_development_trace_v17(signal_index: usize, segment_index: usize, original: &str, raw: &str, rule: &str) {
        let spans=original.match_indices(raw).map(|(start, matched)| (start,start+matched.len())).collect::<Vec<_>>();
        let exact_span=if spans.len()==1 { Some(spans[0]) } else { None };
        LAUNCH_DEVELOPMENT_TRACE.with_borrow_mut(|trace|trace.push(serde_json::json!({"signal_index":signal_index,"segment_index":segment_index,"rule":rule,"segment_raw":raw,"exact_original_utf8_byte_span":exact_span,"span_limit":"none means reconstructed or repeated segment; no guessed source offset"})));
    }

    #[test]
    fn development_consumed_source29_v27_rule_trace() {
        let corpus: serde_json::Value=serde_json::from_str(include_str!("../../../evaluation/engine073-launch-independent-v27/cases.json")).unwrap();
        let failed=["li27-005", "li27-025", "li27-029", "li27-045", "li27-049", "li27-053"];
        let mut count=0;
        for case in corpus["cases"].as_array().unwrap() {
            let id=case["id"].as_str().unwrap();if !failed.contains(&id){continue;}
            let input=&case["input"];
            let policy: EvidenceRelevanceTargetPolicy=serde_json::from_value(input["policy"].clone()).unwrap();
            let candidate: EvidenceRelevanceCandidate=serde_json::from_value(input["candidate"].clone()).unwrap();
            let proposal: EvidenceRelevanceBindingProposal=serde_json::from_value(input["proposal"].clone()).unwrap();
            let raw: EvidenceLocalQualificationV6=serde_json::from_value(input["raw"].clone()).unwrap();
            LAUNCH_DEVELOPMENT_TRACE.with_borrow_mut(Vec::clear);
            let effective=derive_effective_evidence_local_qualification_v17(&policy,&candidate,Some(&proposal),Some(&raw)).unwrap();
            let derive_rule_trace=LAUNCH_DEVELOPMENT_TRACE.with_borrow(|trace|trace.clone());
            let governing_blocks=launch_governing_context_blocks_v17(&policy,&candidate);
            let target_fact=target_owned_launch_fact_v17(&policy,&candidate);
            let rule_trace=LAUNCH_DEVELOPMENT_TRACE.with_borrow(|trace|trace.clone());
            let objects=launch_typed_objects_v17(&policy,&candidate);
            let typed=objects.iter().map(|object|serde_json::json!({"label":object.label,"scope":match object.scope {LaunchTypedScopeV17::Excluded=>"excluded",LaunchTypedScopeV17::Launch=>"launch",LaunchTypedScopeV17::Unresolved=>"unresolved"},"foreign_event":object.foreign_event,"foreign_owner":object.foreign_owner,"declared_value":object.declared_value})).collect::<Vec<_>>();
            let mut segments=Vec::new();
            for (signal_index,signal) in candidate.signals.iter().enumerate(){
                let parts=launch_adverse_segments_v17(&policy,&signal.text);
                for (index,segment) in parts.iter().enumerate(){
                    segments.push(serde_json::json!({"signal_index":signal_index,"segment_index":index,"raw":segment.raw,"denied":segment.denied,"foreign_object":launch_retraction_other_object_v17(&policy,&segment.raw),"property_context":launch_explicit_property_context_v17(&policy,&segment.raw),"nominal_property":launch_complete_nominal_property_v17(&policy,&segment.raw),"bound_property":launch_bound_property_reference_v17(&policy,&segment.raw,index.checked_sub(1).and_then(|i|parts.get(i)),&objects),"declared_foreign":launch_declared_foreign_object_v17(&policy,&segment.raw),"resolved_foreign":launch_resolved_foreign_reference_v17(&policy,&segment.raw,index.checked_sub(1).and_then(|i|parts.get(i)),parts.get(index+1)),"complete_foreign_report":launch_complete_foreign_denial_report_v17(&policy,&segment.raw),"nonlaunch":launch_explicit_nonlaunch_statement_v17(&policy,&segment.raw),"correction_frame":launch_property_correction_frame_v17(&policy,&segment.raw),"apposition":launch_property_apposition_v17(&policy,&segment.raw),"unknown_owner":launch_unavailable_owner_note_v17(&segment.raw),"typed_reference":launch_typed_reference_v17(&policy,&segment.raw,&objects,index.checked_sub(1).and_then(|i|parts.get(i)),parts.get(index+1))}));
                }
            }
            println!("DEVELOPMENT_TRACE {}",serde_json::json!({"id":id,"effective":effective,"governing_blocks":governing_blocks,"target_fact":target_fact,"rule_trace":rule_trace,"derive_rule_trace":derive_rule_trace,"typed_objects":typed,"segments":segments,"scope":"consumed diagnostic, not fresh score"}));
            count+=1;
        }
        assert_eq!(count,6);
    }
'''
s=s.replace(anchor,anchor+module,1);p.write_text(s)
subprocess.run(['cargo','fmt','--all'],check=True)
new=p.read_text();patch=''.join(difflib.unified_diff(Path('/private/tmp/engine073-trace-source29_v27-base.rs').read_text().splitlines(keepends=True),new.splitlines(keepends=True),fromfile='a/crates/reasoning-harness-core/src/evidence_relevance.rs',tofile='b/crates/reasoning-harness-core/src/evidence_relevance.rs'))
Path('/private/tmp/engine073-launch-trace-source29_v27-instrumentation.patch').open('x').write(patch)
Path('/private/tmp/engine073-launch-trace-source29_v27-identity.json').open('x').write(json.dumps({'base_source_sha256':expected,'instrumented_source_sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'changes':'cfg(test) records before existing true returns; no return predicates changed','temporary_diagnostic_not_fresh_acceptance':True},indent=2)+'\n')
print('temporary rule trace inserted; source patch preserved')
