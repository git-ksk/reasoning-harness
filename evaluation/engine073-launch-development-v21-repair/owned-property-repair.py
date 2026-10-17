from pathlib import Path
import hashlib
p=Path('crates/reasoning-harness-core/src/evidence_relevance.rs');s=p.read_text();assert hashlib.sha256(p.read_bytes()).hexdigest()=='1abbec8d424d372a5713d57be7068af2dd231f1415ad6ad232f2b497e16baea6'
def replace(old,new):
 global s
 assert s.count(old)==1,(s.count(old),old[:90]);s=s.replace(old,new,1)
anchor='fn launch_attribute_assertion_object_v17(\n'
helper='''// This recognizes a complete PROPERTY owner, never a launch owner. Require
// a full proper name and a separate finite component/document noun phrase;
// metadata names and quoted event names cannot provide this declaration.
fn launch_named_property_owner_tail_v17(value: &str) -> bool {
    let words = value.split_whitespace().collect::<Vec<_>>();
    let end = words.iter().take_while(|word| {
        word.chars().next().is_some_and(char::is_uppercase)
    }).count();
    end > 0 && end < words.len()
        && launch_named_subject_v17(&words[..end].join(" "))
        && words[end..].iter().all(|word| {
            let text = normalized(word);
            !text.is_empty() && text.split_whitespace().all(|word| {
                (!launch_scope_or_reference_word_v17(word) || ["document", "page", "book"].contains(&word))
                    && !["and", "or", "but", "not", "never", "no", "does", "do", "exist", "exists", "existed", "is", "was", "are", "were", "has", "have", "uses", "use", "because", "although", "while", "when", "if", "since", "as"].contains(&word)
            })
        })
}

fn launch_owned_material_object_v17(value: &str) -> bool {
    let value = value.trim();
    let folded = value.to_ascii_lowercase();
    let mut start = 0;
    for prefix in ["the ", "a ", "an "] {
        if folded.starts_with(prefix) { start = prefix.len(); break; }
    }
    if folded[start..].starts_with("paper ") { start += 6; }
    if !folded[start..].starts_with("material ") { return false; }
    start += 9;
    for noun in ["claim ", "assertion ", "statement "] {
        if folded[start..].starts_with(noun) { start += noun.len(); break; }
    }
    let Some(link) = ["of ", "about ", "concerning "].iter().find(|link| folded[start..].starts_with(**link)) else { return false; };
    launch_named_property_owner_tail_v17(value[start + link.len()..].trim())
}

fn launch_complete_printed_property_correction_v17(
    policy: &EvidenceRelevanceTargetPolicy,
    statement: &str,
) -> bool {
    let Some(start) = statement.find(['"', '“', '‘']) else { return false; };
    let head = normalized(&statement[..start]);
    let words = head.split_whitespace().collect::<Vec<_>>();
    let Some(verb) = words.iter().position(|word| launch_retraction_word_v17(word)) else { return false; };
    if words[..verb].is_empty() || !words[..verb].iter().all(|word| ["we", "i", "also", "hereby", "explicitly", "formally"].contains(word)) { return false; }
    let ["the", field, "claim" | "assertion" | "statement", "about", "the", "printed", "sentence" | "words"] = &words[verb + 1..] else { return false; };
    if !["font", "typography", "colour", "color"].contains(field) { return false; }
    launch_printed_quoted_property_v17(policy, &format!("the {field} of the literal printed words {}", &statement[start..]))
}

'''
replace(anchor,helper+anchor)
replace('''    if launch_complete_property_proposition_v17(policy, value) {
        return true;
    }
    let text = normalized(value).replace''','''    if launch_owned_material_object_v17(value) || launch_complete_property_proposition_v17(policy, value) {
        return true;
    }
    let text = normalized(value).replace''')
a=s.index('fn launch_compound_attribute_object_v17(');b=s.index('// A finite local Object',a);t=s[a:b].replace('''    let text = normalized(value);
    let Some((left, right)) = text.split_once(" and ")''','''    let Some((left, right)) = value.split_once(" and ")''');assert t!=s[a:b];s=s[:a]+t+s[b:]
replace('''    !values.is_empty()
        && launch_specific_property_object_v17(policy, &words[..predicate])''','''    let owned_material = [" is ", " was ", " are ", " were ", " uses ", " use "]
        .iter().filter_map(|copula| value.to_ascii_lowercase().find(copula))
        .min().is_some_and(|index| launch_owned_material_object_v17(value[..index].trim()));
    !values.is_empty()
        && (owned_material || launch_specific_property_object_v17(policy, &words[..predicate]))''')
replace('''            && target_identity_phrases(policy)
                .iter()
                .any(|target| normalized_phrase_matches(property, target))
            && launch_attribute_assertion_object_v17(policy, property);''','''            && (target_identity_phrases(policy).iter().any(|target| normalized_phrase_matches(property, target))
                || launch_owned_material_object_v17(value[..start].trim().strip_suffix(" carrying the literal text").unwrap_or("")))
            && launch_attribute_assertion_object_v17(policy, value[..start].trim().strip_suffix(" carrying the literal text").unwrap_or(property));''')
replace('''    target_identity_phrases(policy)
        .iter()
        .any(|target| normalized_phrase_matches(tail, target))
        && launch_attribute_assertion_object_v17(''','''    (target_identity_phrases(policy).iter().any(|target| normalized_phrase_matches(tail, target))
        || tail.strip_prefix("on ").is_some_and(launch_named_property_owner_tail_v17))
        && launch_attribute_assertion_object_v17(''')
replace('''    if launch_complete_property_proposition_v17(policy, value)
        || launch_compound_attribute_object_v17(policy, value)''','''    if launch_owned_material_object_v17(value)
        || launch_complete_property_proposition_v17(policy, value)
        || launch_compound_attribute_object_v17(policy, value)''')
a=s.index('fn launch_property_correction_frame_v17(');b=s.index('    let original = normalized(statement);',a);s=s[:b]+'''    if launch_complete_printed_property_correction_v17(policy, statement) { return true; }
'''+s[b:]
replace('''                "is a physical property claim"
                    | "is a physical-property claim"''','''                "is a physical property claim"
                    | "is a physical property"
                    | "is a physical-property claim"''')
# The exact bounded reference suffix is removed before the older shorter
# suffixes. Ownership still comes from the selected role declarations.
replace('''    let statement = statement
        .strip_suffix(" described in the heading")''','''    let statement = [" whose owner is in the heading", " whose owner is specified in the source title", " whose owner is specified in the heading", " whose owner is in the source title"]
        .iter().find_map(|suffix| statement.strip_suffix(suffix)).unwrap_or(statement);
    let statement = statement
        .strip_suffix(" described in the heading")''')
# Complete-property anaphors preserve their field and immediate antecedent.
a=s.index('fn launch_bound_property_reference_v17(');b=s.index('    let (head, apposition) = statement',a)
s=s[:b]+'''    let note = normalized(statement);
    if let ["only", "the", field, "of", "that", "sentence", "is", "the", verb, "property"] = note.split_whitespace().collect::<Vec<_>>().as_slice() {
        if launch_retraction_word_v17(verb)
            && launch_complete_printed_property_correction_v17(policy, &previous.raw)
            && normalized(&previous.raw).split_whitespace().any(|word| word == *field)
        { return true; }
    }
'''+s[b:]
needle='''    let supports = |value: &str| {
        let text = normalized(value);'''
replace(needle,'''    if topic.first() == Some(&"complete") { topic = &topic[1..]; }
    let supports = |value: &str| {
        let text = normalized(value);''')
# A finite foreign adjunct cannot change the explicitly named event owner.
a=s.index('fn launch_retraction_other_object_v17(');b=s.index('    let folded = statement.to_ascii_lowercase();',a)
s=s[:b]+'''    let statement = statement.rsplit_once(" as ").filter(|(_, adjunct)| matches!(
        normalized(adjunct).split_whitespace().collect::<Vec<_>>().as_slice(),
        ["a", "separate" | "further" | "additional", "event", "claim" | "correction"]
    )).map_or(statement, |(event, _)| event);
'''+s[b:]
# The same complete literal proof is used for source governance and successor
# lexical-risk projection. All original outside text and raw risks remain.
replace('''            if segment.raw.split_once(':').is_some_and(|(label, value)| {
                ["object", "subject", "item"].contains(&normalized(label).as_str())
                    && launch_printed_quoted_property_v17(policy, value.trim())
            }) {''','''            if segment.raw.split_once(':').is_some_and(|(label, value)| {
                ["object", "subject", "item"].contains(&normalized(label).as_str())
                    && launch_printed_quoted_property_v17(policy, value.trim())
            }) || launch_complete_printed_property_correction_v17(policy, &segment.raw) {''')
replace('''                if !segment.question && !segment.denied && printed_property {''','''                if !segment.question && !segment.denied && (printed_property || launch_complete_printed_property_correction_v17(policy, &segment.raw)) {''')
p.write_text(s)
print('bounded property-owner/source-reference repair written')
