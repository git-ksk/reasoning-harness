from pathlib import Path
import hashlib
p=Path('crates/reasoning-harness-core/src/evidence_relevance.rs');s=p.read_text();assert hashlib.sha256(p.read_bytes()).hexdigest()=='77727cdc14de86ad4821b1698a43799afb5b42e86d53ef566971623f1d9663f7'
def replace(old,new):
 global s
 assert s.count(old)==1,(s.count(old),old[:100]);s=s.replace(old,new,1)
# A complete direct denial has precedence over attribute words in the target's
# proper name. Only a finite role prefix may be unwrapped; quoted property
# content is not unwrapped.
anchor='fn launch_typed_object_scope_v17(\n'
helper='''fn launch_target_nonexistence_v17(
    policy: &EvidenceRelevanceTargetPolicy,
    statement: &str,
) -> bool {
    let value = statement.split_once(':').map_or(statement, |(label, value)| {
        if ["object", "subject", "item"].contains(&normalized(label).as_str()) {
            value.trim()
        } else {
            statement
        }
    });
    let text = normalized(value);
    target_identity_phrases(policy).iter().any(|target| {
        text.strip_prefix(target).is_some_and(|tail| {
            let tail = tail.trim();
            ["does not exist", "has never existed", "never existed", "is nonexistent", "is not real"]
                .iter().any(|denial| tail == *denial || tail.starts_with(&format!("{denial} ")))
        })
    })
}

'''
replace(anchor,helper+anchor)
replace('''    use LaunchTypedScopeV17 as Scope;
    if launch_complete_property_proposition_v17(policy, value)''','''    use LaunchTypedScopeV17 as Scope;
    if launch_target_nonexistence_v17(policy, value) {
        return Scope::Launch;
    }
    if launch_complete_property_proposition_v17(policy, value)''')
a=s.index('            if targets.iter().any(|target| {',s.index('fn launch_governing_context_blocks_v17('));b=s.index('            if segment.question',a)
assert 'does not exist' in s[a:b];s=s[:a]+'''            if launch_target_nonexistence_v17(policy, &segment.raw) {
                return true;
            }
'''+s[b:]
# Distinct unresolved claims do not inherit the previous foreign event owner.
replace('''                        "additional",
                        "including",
                        "neither",
                        "list",''','''                        "additional",
                        "remaining",
                        "separate",
                        "including",
                        "neither",
                        "list",''')
replace('''    words.contains(&"owner")
        && ((words.contains(&"no")''','''    (words.contains(&"owner") || (words.contains(&"product") && words.contains(&"identity")))
        && ((words.contains(&"no")''')
replace('''                && [
                    index.checked_sub(1).and_then(|i| segments.get(i)),
                    segments.get(index + 1),
                ]
                .into_iter()
                .flatten()
                .any(|neighbor| {''','''                && std::iter::once(segment)
                .chain([
                    index.checked_sub(1).and_then(|i| segments.get(i)),
                    segments.get(index + 1),
                ].into_iter().flatten())
                .any(|neighbor| {''')
# Resolve an explicit role-qualified owner supplement after bounded source-field
# wording, preserving role conflicts/unknown owners and actual event ownership.
replace('''            let text = normalized(value);
            let unowned = [''','''            let text = normalized(value);
            let unowned_text = text.split_once(" whose owner is specified in ")
                .filter(|(_, field)| matches!(field.split_whitespace().collect::<Vec<_>>().as_slice(),
                    ["the" | "this", "source", "title"] | ["the" | "this", "heading"]))
                .map_or(text.as_str(), |(object, _)| object);
            let unowned = [''')
replace('''            .contains(&text.as_str());
            let words = text.split_whitespace().collect::<Vec<_>>();
            let words = words
                .strip_prefix(&["an"])''','''            .contains(&unowned_text);
            let words = unowned_text.split_whitespace().collect::<Vec<_>>();
            let words = words
                .strip_prefix(&["an"])''')
# Split a repeated elided predicate after comma; preserve comma appositions and
# coordinated objects until an actual verb/actor starts the next clause.
replace('''                        "–and ",
                        " and ",
                    ] {''','''                        "–and ",
                        " and ",
                        ", ",
                    ] {''')
replace('''                            if ![" and ", "—and ", "–and "].contains(&marker)''','''                            if ![" and ", "—and ", "–and ", ", "].contains(&marker)''')
# Existing complete correction frames: accept a generic singular claim
# apposition only when its whole value is independently a physical property.
replace('''            if matches!(
                normalized(head).as_str(),
                "the other claim"''','''            let head_text = normalized(head);
            let predicate_text = normalized(predicate);
            let plain_claim = matches!(head_text.split_whitespace().collect::<Vec<_>>().as_slice(),
                ["the", "claim" | "statement" | "assertion"]);
            let passive = matches!(predicate_text.split_whitespace().collect::<Vec<_>>().as_slice(),
                ["is" | "was", verb] | ["is" | "was", verb, "by", "us" | "me"] if launch_retraction_word_v17(verb));
            if plain_claim && passive && property(value) { return true; }
            if matches!(
                normalized(head).as_str(),
                "the other claim"''')
old='''        if matches!(
            head.as_str(),
            "we retract the claim"
                | "i retract the claim"
                | "we withdraw the claim"
                | "i withdraw the claim"
                | "we deny the claim"
                | "i deny the claim"
        ) && property(value)'''
new='''        let words = head.split_whitespace().collect::<Vec<_>>();
        let complete_reporting_head = words.iter().position(|word| launch_retraction_word_v17(word))
            .is_some_and(|verb| !words[..verb].is_empty()
                && words[..verb].iter().all(|word| ["we", "i", "also", "hereby", "formally", "explicitly"].contains(word))
                && matches!(&words[verb + 1..], ["the", "claim" | "statement" | "assertion"]
                    | ["the", "remaining" | "other", "claim" | "statement" | "assertion"]));
        if complete_reporting_head && property(value)'''
replace(old,new)
# A quoted string carried by a complete property object is data. Require the
# owner and property outside the quote, balanced quotes and no appended clause.
a=s.index('    let field = match head_words.as_slice() {',s.index('fn launch_printed_quoted_property_v17('));b=s.index('    let quote = value[start..]',a);field=s[a:b];s=s[:a]+s[b:]
needle='''    let tail = quoted[end + closing.len_utf8()..].trim();
    target_identity_phrases(policy)'''
new='''    let tail = quoted[end + closing.len_utf8()..].trim();
    if let Some(property) = head.strip_suffix(" carrying the literal text") {
        return tail.is_empty()
            && target_identity_phrases(policy).iter().any(|target| normalized_phrase_matches(property, target))
            && launch_attribute_assertion_object_v17(policy, property);
    }
'''+field+'''    target_identity_phrases(policy)'''
replace(needle,new)
# Accuracy wrapper binds only the explicit role; does not add those tokens to
# arbitrary reference tails.
replace('''    let tail = &words[verb + 1..];
    let plain_tail = tail.iter().all(|word| {''','''    let mut tail = &words[verb + 1..];
    if label.is_some() && tail.starts_with(&["the", "accuracy", "of"]) {
        tail = &tail[3..];
    }
    let plain_tail = tail.iter().all(|word| {''')
# A short material apposition needs a complete excluded property in every
# selected explicit role. No standalone component lexicon or event inference.
replace('''        return Some(
            launch_typed_object_scope_v17(policy, value) == Scope::Excluded
                && !selected.iter().any(|object| object.scope == Scope::Launch),
        );''','''        let mut description = value_words.split_whitespace().collect::<Vec<_>>();
        while matches!(description.first(), Some(&"only" | &"the")) { description.remove(0); }
        if matches!(description.last(), Some(&"claim" | &"assertion" | &"statement")) { description.pop(); }
        if label.is_some() && description.len() == 2 && description.contains(&"material")
            && description.iter().all(|word| *word == "material" || !launch_scope_or_reference_word_v17(word))
            && selected.iter().all(|object| object.scope == Scope::Excluded && !object.foreign_event
                && description.iter().all(|word| normalized(&object.declared_value).split_whitespace().any(|declared| declared == *word)))
        { return Some(true); }
        return Some(
            launch_typed_object_scope_v17(policy, value) == Scope::Excluded
                && !selected.iter().any(|object| object.scope == Scope::Launch),
        );''')
p.write_text(s)
print('bounded source20 repair written')
