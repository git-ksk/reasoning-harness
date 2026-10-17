from pathlib import Path
p=Path('crates/reasoning-harness-core/src/evidence_relevance.rs');s=p.read_text();s=s.replace('        "rim",\n        "rims",\n','',1)
a=s.index('fn launch_bound_property_reference_v17(');b=s.index('\nfn launch_typed_reference_v17(',a);t=s[a:b]
old='''    if !launch_attribute_assertion_object_v17(policy, &topic.join(" ")) {
        return false;
    }
''';assert old in t;t=t.replace(old,'',1)
needle='''    if previous_property && supports(&previous.raw) {
        return true;
    }
'''
assert needle in t;t=t.replace(needle,'''    // A complete, owner-qualified material proposition can bind a component
    // name outside the standalone component lexicon. Event/reference words
    // are never allowed to acquire that property's type.
    let contextual_material = topic.len() == 2 && topic.contains(&"material")
        && topic.iter().all(|word| *word == "material" || !launch_scope_or_reference_word_v17(word))
        && previous_property && supports(&previous.raw);
    if !launch_attribute_assertion_object_v17(policy, &topic.join(" ")) && !contextual_material {
        return false;
    }
'''+needle,1);s=s[:a]+t+s[b:]
a=s.index('fn launch_adverse_segments_v17(');b=s.index('\nfn launch_complete_foreign_denial_report_v17(',a);t=s[a:b]
old='''                                        right
                                            .first()
                                            .is_some_and(|word| launch_retraction_word_v17(word))
'''
if old not in t:
 print(t[t.index('let right = normalized'):t.index('let right = normalized')+750]);raise SystemExit('report predicate formatted anchor not found')
new='''                                        right
                                            .first()
                                            .is_some_and(|word| launch_retraction_word_v17(word)
                                                || ["report", "reports", "record", "records", "state", "states", "explain", "explains"].contains(word))
''';t=t.replace(old,new,1);s=s[:a]+t+s[b:]
p.write_text(s);print('context-bound material references and elided reporting predicates repaired')
