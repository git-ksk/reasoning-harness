from pathlib import Path
p=Path('crates/reasoning-harness-core/src/evidence_relevance.rs');s=p.read_text()
s=s.replace('''    let printed_label_date = object.contains(&"printed")
        && object.contains(&"date")
        && object.iter().any(|word| ["label", "labels"].contains(word));''','''    let printed_label_date = object.contains(&"printed")
        && object.contains(&"date")
        && object.iter().any(|word| ["label", "labels", "card", "carton", "property"].contains(word));''',1)
a=s.index('fn launch_property_correction_frame_v17(');b=s.index('    if let Some((head, value)) = statement.split_once',a);part=s[a:b];c=part.index('    let property = |value: &str| {');part=part[:c]+'''    let property = |value: &str| launch_attribute_assertion_object_v17(policy, value);
''';s=s[:a]+part+s[b:]
i=s.index('// A finite local Object/Subject/Item frame')
s=s[:i]+'''fn launch_attribute_assertion_object_v17(policy: &EvidenceRelevanceTargetPolicy, value: &str) -> bool {
    if launch_complete_property_proposition_v17(policy, value) { return true; }
    let text = normalized(value).replace("those literal words", "printed lettering");
    let mut words = text.split_whitespace().collect::<Vec<_>>();
    while matches!(words.first(), Some(&"the" | &"a" | &"an" | &"this" | &"that" | &"those" | &"these" | &"only" | &"fully" | &"identified")) { words.remove(0); }
    if matches!(words.first(), Some(&"claim" | &"assertion" | &"statement")) && matches!(words.get(1),Some(&"about" | &"concerning")) { words.drain(..2); }
    if let Some(head) = words.iter().position(|word| ["claim", "assertion", "statement"].contains(word)) {
        if head == 0 { return false; }
        words.remove(head);
        if matches!(words.get(head), Some(&"about" | &"on" | &"concerning")) { words[head] = "of"; }
    }
    if words.iter().any(|word| ["is", "was", "are", "were", "does", "do", "did", "not", "never", "exist", "exists", "existed", "and", "or", "but", "that", "which", "who", "while", "when", "before", "after", "because"].contains(word)) { return false; }
    if words.windows(2).any(|pair| matches!(pair, ["carton" | "label" | "card", "text"])) {
        for word in &mut words { if *word == "text" { *word = "lettering"; } }
    }
    words.iter().any(|word| ["colour", "color", "font", "material", "width", "height", "thickness", "title", "typography", "length", "size", "weight", "printed", "lettering", "page", "count"].contains(word))
        && launch_specific_property_object_v17(policy, &words)
}

fn launch_compound_attribute_object_v17(policy: &EvidenceRelevanceTargetPolicy, value: &str) -> bool {
    let text = normalized(value);
    let Some((left,right)) = text.split_once(" and ") else { return false; };
    !right.contains(" and ") && launch_attribute_assertion_object_v17(policy,left) && launch_attribute_assertion_object_v17(policy,right)
}

''' +s[i:]
a=s.index('fn launch_typed_object_scope_v17(');b=s.index('    if let Some((head, explanation))',a);part=s[a:b];part=part.replace('''if launch_complete_property_proposition_v17(policy, value)''','''if launch_complete_property_proposition_v17(policy, value) || launch_compound_attribute_object_v17(policy,value)''');part+='''    let raw = value.trim();
    if let Some(quote) = raw.chars().next().filter(|quote| ['"','“','‘'].contains(quote)) {
        let closing = match quote { '“'=>'”', '‘'=>'’', _=>'"' };
        let quoted = &raw[quote.len_utf8()..];
        if let Some(end) = quoted.find(closing) {
            let tail = quoted[end+closing.len_utf8()..].trim();
            if launch_complete_property_proposition_v17(policy,&quoted[..end])
                && tail.strip_prefix(',').is_some_and(|tail| launch_attribute_assertion_object_v17(policy,tail.trim())) { return Scope::Excluded; }
        }
    }
''';s=s[:a]+part+s[b:]
a=s.index('fn launch_complete_nominal_property_v17(');b=s.index('\nfn launch_property_apposition_v17',a);part=s[a:b]
part=part.replace('''        "this denial is limited to ",''','''        "this denial is limited to ",
        "a local explanation identifies the denied claim as ",
        "the corrected object is a fully identified ",
        "the denial concerns only ",
        "the statement i deny is ",
        "the statement we deny is ",
        "the claim i withdraw is ",
        "this notice gives no date value but the correction object is ",''')
part=part.replace('''launch_specific_property_object_v17(policy, &topic.split_whitespace().collect::<Vec<_>>())''','''launch_specific_property_object_v17(policy, &topic.split_whitespace().collect::<Vec<_>>()) || launch_attribute_assertion_object_v17(policy,topic)''')
part=part.replace('''    let property_words = launch_property_words_v17(policy, statement);''','''    let original = normalized(statement);
    if let Some(subject) = original.strip_suffix(" is the entire local correction") {
        if launch_attribute_assertion_object_v17(policy,subject) { return true; }
    }
    let property_words = launch_property_words_v17(policy, statement);''',1)
s=s[:a]+part+s[b:]
# Complete appositions may declare the correction before or after their property.
a=s.index('fn launch_property_correction_frame_v17(');b=s.index('\nfn launch_complete_nominal_property_v17',a);part=s[a:b]
part=part.replace('''"the other claim" | "the other statement" | "the other assertion"''','''"the other claim" | "the other statement" | "the other assertion" | "the claim we withdraw" | "the claim i withdraw"''')
part=part.replace('''"is withdrawn"
                    | "is also withdrawn"''','''"is a physical property claim" | "is a physical-property claim" | "is withdrawn"
                    | "is also withdrawn"''')
needle='''    if let Some(verb) = words''';i=part.index(needle);part=part[:i]+'''    if let Some((head,value)) = statement.split_once(", ") {
        let head = normalized(head);
        if matches!(head.as_str(), "we retract the claim" | "i retract the claim" | "we withdraw the claim" | "i withdraw the claim" | "we deny the claim" | "i deny the claim") && property(value) { return true; }
    }
'''+part[i:];s=s[:a]+part+s[b:]
# Coordinate complete property/foreign objects independently under one verb.
a=s.index('fn launch_coordinated_property_objects_v17(');b=s.index('\nfn launch_explicit_property_context_v17(',a);part=s[a:b]
part=part.replace('''&& launch_explicit_property_context_v17(policy, &format!("{prefix} {}", left.join(" ")))''','''&& (launch_explicit_property_context_v17(policy, &format!("{prefix} {}", left.join(" "))) || launch_retraction_other_object_v17(policy, &format!("{prefix} {}", left.join(" "))))''')
part=part.replace('''        && launch_explicit_property_context_v17(
            policy,
            &format!("{prefix} {}", words[join + 1..].join(" ")),
        )''','''        && (launch_explicit_property_context_v17(policy, &format!("{prefix} {}", words[join + 1..].join(" "))) || launch_retraction_other_object_v17(policy,&format!("{prefix} {}", words[join+1..].join(" "))))''')
s=s[:a]+part+s[b:]
# Resolve every explicit typed role; tolerate an elided repeated actor.
a=s.index('    if let ["we" | "i", verb, "the", left, "and", "the", right]',s.index('fn launch_typed_reference_v17('));b=s.index('    let pointer = words',a)
s=s[:a]+'''    if let Some(verb) = words.iter().position(|word| launch_retraction_word_v17(word)) {
        let tail = &words[verb+1..];
        let roles = tail.iter().copied().filter(|word| ["object","subject","item"].contains(word)).collect::<Vec<_>>();
        if roles.len() >= 2 && tail.iter().all(|word| ["the","and","object","subject","item"].contains(word))
            && tail.iter().filter(|word| **word == "and").count()+1 == roles.len()
            && words[..verb].iter().all(|word| ["we","i","also"].contains(word)) {
            return Some(roles.iter().all(|role| {
                let selected = objects.iter().filter(|object| object.label == *role).collect::<Vec<_>>();
                !selected.is_empty() && selected.iter().all(|object| object.scope == Scope::Excluded)
            }));
        }
    }
'''+s[b:]
s=s.replace('''        .or_else(|| statement.strip_suffix(" in the source title"))''','''        .or_else(|| statement.strip_suffix(" in the source title"))
        .or_else(|| statement.strip_suffix(" whose owner is specified in this heading"))
        .or_else(|| statement.strip_suffix(" whose owner is specified in this source title"))''',1)
a=s.index('    if let Some(value) = apposition {',s.index('fn launch_typed_reference_v17('));b=s.index('        if let Some(owner)',a);part=s[a:b];part+='''        let value_words = normalized(value);
        if matches!(value_words.split_whitespace().collect::<Vec<_>>().as_slice(), ["these","two", .., "assertions" | "claims"])
            && selected.iter().all(|object| object.scope == Scope::Excluded && launch_compound_attribute_object_v17(policy,&object.declared_value)) {
            let topic = value_words.strip_prefix("these two ").unwrap().rsplit_once(' ').unwrap().0;
            if launch_attribute_assertion_object_v17(policy,topic) { return Some(true); }
        }
''';s=s[:a]+part+s[b:]
# Nominal reports retain the same positive reporting modifier semantics.
a=s.index('fn launch_complete_foreign_denial_report_v17');b=s.index('\nfn launch_governing_context_blocks_v17',a);part=s[a:b];part=part.replace('''                    "reports",
                    "records",''','''                    "reports",
                    "also",
                    "explicitly",
                    "formally",
                    "clearly",
                    "records",''');s=s[:a]+part+s[b:]
# Parentheses and explicit active clauses are independent scopes.
a=s.index('fn launch_adverse_segments_v17');b=s.index('\nfn launch_unavailable_owner_note_v17',a);part=s[a:b];needle='''            let mut split = None;
            if reporting(&raw) {''';new='''            let mut split = None;
            if let Some((head,body)) = raw.split_once(" (") {
                if let Some(body) = body.strip_suffix(')') {
                    if !head.contains(['"','“','‘']) && !body.contains(['(',')']) && reporting(body) { split = Some((head.to_owned(),body.to_owned())); }
                }
            }
            if split.is_none() && reporting(&raw) {''';assert part.count(needle)==1;part=part.replace(needle,new)
needle='''                                    && reporting(&raw[index + marker.len()..]))''';new='''                                    && reporting(&raw[index + marker.len()..]))
                                || (reporting(&raw[index+marker.len()..]) && ["i ","we ","the editor ","the author ","the publisher "].iter().any(|actor| folded[index+marker.len()..].trim_start().starts_with(actor)))''';assert part.count(needle)==1;part=part.replace(needle,new);s=s[:a]+part+s[b:]
p.write_text(s)
