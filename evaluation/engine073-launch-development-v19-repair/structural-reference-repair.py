from pathlib import Path
import hashlib
p=Path('crates/reasoning-harness-core/src/evidence_relevance.rs');s=p.read_text();assert hashlib.sha256(p.read_bytes()).hexdigest()=='2c2a9cfce355a07dd8387f060e8dac2366d47400124f1425d7f03fa6b5dc4985'
s=s.replace('        "lid",\n        "lids",','        "lid",\n        "lids",\n        "rim",\n        "rims",',1)
s=s.replace('["page", "count" | "number" | "numbering" | "length"]','["page", "count" | "number" | "numbering" | "length" | "width" | "height" | "size" | "thickness"]',1)
s=s.replace('.or_else(|| statement.trim().strip_suffix(" as a separate correction"))','.or_else(|| statement.trim().strip_suffix(" as a separate correction"))\n            .or_else(|| statement.trim().strip_suffix(" as a separate event claim"))',1)
a=s.index('fn launch_printed_quoted_property_v17(');b=s.index('\nfn launch_typed_object_scope_v17(',a);t=s[a:b]
old='''    let head = normalized(value[..start].trim());
    if ![
        "the font of the printed words",
        "the typography of the printed words",
        "the colour of the printed words",
        "the color of the printed words",
    ]
    .contains(&head.as_str())
    {
        return false;
    }
'''
new='''    let head = normalized(value[..start].trim());
    let head_words = head.split_whitespace().collect::<Vec<_>>();
    let field = match head_words.as_slice() {
        ["the", field, "of", "the", "printed", "words" | "name"]
        | ["the", field, "of", "the", "literal", "printed", "words" | "name"]
            if ["font", "typography", "colour", "color"].contains(field) => *field,
        _ => return false,
    };
'''
assert old in t;t=t.replace(old,new,1).replace('&format!("{head} printed lettering {tail}")','&format!("the {field} of the printed words printed lettering {tail}")');s=s[:a]+t+s[b:]
a=s.index('fn launch_property_correction_frame_v17(');b=s.index('\nfn launch_complete_nominal_property_v17(',a);t=s[a:b]
t=t.replace('["we", "i", "hereby", "formally"].contains(word)','["we", "i", "also", "hereby", "formally", "explicitly"].contains(word)',1)
needle='''    if let Some((head, value)) = statement.split_once(", ") {
'''
addition='''    if let Some((head, value)) = statement.split_once(", ") {
        let head = normalized(head);
        if matches!(head.split_whitespace().collect::<Vec<_>>().as_slice(),
            ["the", "claim" | "assertion" | "statement", "we" | "i", verb, "is", "this"]
            if launch_retraction_word_v17(verb)) && property(value) {
            return true;
        }
    }
'''
assert needle in t;t=t.replace(needle,addition+needle,1);s=s[:a]+t+s[b:]
a=s.index('fn launch_typed_reference_v17(');b=s.index('\nfn launch_adverse_segments_v17(',a);t=s[a:b]
needle='''    let selected_label = label.or(previous_role.as_deref());
'''
assert needle in t;t=t.replace(needle,'''    // An explicit foreign event owner is not an unspecified reference to a
    // typed property elsewhere in the source. Named roles still bind first.
    if label.is_none() && previous_role.is_none()
        && launch_retraction_other_object_v17(policy, statement) {
        return None;
    }
'''+needle,1);s=s[:a]+t+s[b:]
helper=r'''
// Resolve a property-specific anaphor only against an adjacent complete
// property proposition or its explicitly named role. No event/global scope is
// inferred from an attribute word, and unrelated roles cannot lend their type.
fn launch_bound_property_reference_v17(
    policy: &EvidenceRelevanceTargetPolicy,
    statement: &str,
    previous: Option<&LaunchFactualSegmentV17>,
    objects: &[LaunchTypedObjectV17],
) -> bool {
    let Some(previous) = previous else { return false; };
    let (head, apposition) = statement.split_once(',').map_or((statement,None), |(head,tail)|(head,Some(tail)));
    if apposition.is_some_and(|tail| {
        let tail=normalized(tail);
        !tail.split_whitespace().all(|word| ["the","a","local","quoted","property","claim","assertion","statement"].contains(&word))
            || !tail.split_whitespace().any(|word| word=="property")
    }) { return false; }
    let text=normalized(head); let words=text.split_whitespace().collect::<Vec<_>>();
    let Some(verb)=words.iter().position(|word|launch_retraction_word_v17(word)) else { return false; };
    let mut topic = if words[..verb].iter().all(|word| ["we","i","also","hereby","formally","explicitly"].contains(word)) {
        &words[verb+1..]
    } else if matches!(words.get(verb.wrapping_sub(1)),Some(&"is" | &"was" | &"are" | &"were"))
        && (verb+1==words.len() || matches!(&words[verb+1..],["by","us" | "me"])) {
        &words[..verb-1]
    } else { return false; };
    if matches!(topic,["the","accuracy","of",..]) {topic=&topic[3..];}
    if !matches!(topic.first(),Some(&"this" | &"that")) {return false;}
    topic=&topic[1..];
    if !matches!(topic.last(),Some(&"claim" | &"assertion" | &"statement")) {return false;}
    topic=&topic[..topic.len()-1];
    let previous_property = launch_complete_property_proposition_v17(policy,&previous.raw)
        || launch_property_correction_frame_v17(policy,&previous.raw)
        || launch_complete_nominal_property_v17(policy,&previous.raw);
    if topic==["complete","document","property"] {
        return previous_property && normalized(&previous.raw).split_whitespace().any(|word| ["page","document","manual","guide","handbook"].contains(&word));
    }
    if !launch_attribute_assertion_object_v17(policy,&topic.join(" ")) {return false;}
    let supports = |value:&str| {
        let text=normalized(value);let value_words=text.split_whitespace().collect::<Vec<_>>();
        topic.iter().all(|word|value_words.contains(word))
    };
    if previous_property && supports(&previous.raw) {return true;}
    let previous_text=normalized(&previous.raw);
    let mut roles=previous_text.split_whitespace().filter(|word|["object","subject","item"].contains(word));
    let Some(role)=roles.next() else {return false;};
    if roles.any(|other|other!=role) {return false;}
    let selected=objects.iter().filter(|object|object.label==role).collect::<Vec<_>>();
    !selected.is_empty() && selected.iter().all(|object|object.scope==LaunchTypedScopeV17::Excluded
        && !object.foreign_event && supports(&object.declared_value))
}

'''
needle='fn launch_typed_reference_v17(';assert s.count(needle)==1;s=s.replace(needle,helper+needle,1)
a=s.index('fn launch_governing_context_blocks_v17(');b=s.index('\nfn target_owned_launch_fact_v17(',a);t=s[a:b]
needle='''            if let Some(excluded) = launch_typed_reference_v17(
'''
addition='''            if launch_bound_property_reference_v17(policy, &segment.raw,
                index.checked_sub(1).and_then(|i| segments.get(i)), &typed_objects) {
                continue;
            }
'''
assert needle in t;t=t.replace(needle,addition+needle,1);s=s[:a]+t+s[b:]
a=s.index('fn launch_adverse_segments_v17(');b=s.index('\nfn launch_complete_foreign_denial_report_v17(',a);t=s[a:b]
old='''                        if let Some(index) = folded.find(marker) {
''';new='''                        for (index, _) in folded.match_indices(marker) {
''';assert old in t;t=t.replace(old,new,1)
old='''                                || (reporting(&raw[..index])
                                    && reporting(&raw[index + marker.len()..]))
'''
new='''                                || (reporting(&raw[..index])
                                    && reporting(&raw[index + marker.len()..])
                                    && {
                                        let right=normalized(&raw[index+marker.len()..]);
                                        let right=right.split_whitespace().collect::<Vec<_>>();
                                        right.first().is_some_and(|word|launch_retraction_word_v17(word))
                                            || right.iter().position(|word|launch_retraction_word_v17(word)).is_some_and(|verb|
                                                right[..verb].iter().any(|word|["is","was","are","were","has","have"].contains(word)))
                                    })
''';assert old in t;t=t.replace(old,new,1)
# Searching later conjunctions must not overwrite a split already selected.
needle='''                        }
                    }
                }
            }
            if let Some((left, right)) = split {
'''
replace='''                        }
                        if split.is_some() { break; }
                    }
                }
            }
            if let Some((left, right)) = split {
''';assert needle in t;t=t.replace(needle,replace,1);s=s[:a]+t+s[b:]
p.write_text(s)
print('structural property/reference and predicate coordination repair applied')
