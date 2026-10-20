from pathlib import Path
p=Path('crates/reasoning-harness-core/src/evidence_relevance.rs');s=p.read_text()
a=s.index('fn launch_component_head_v17');b=s.index('\nfn launch_component_qualifier_v17',a);part=s[a:b];needle='''        "label",''';assert part.count(needle)==1;part=part.replace(needle,'''        "lid",
        "lids",
        "label",''');s=s[:a]+part+s[b:]
i=s.index('fn launch_typed_object_scope_v17(')
s=s[:i]+'''fn launch_printed_quoted_property_v17(policy: &EvidenceRelevanceTargetPolicy, value: &str) -> bool {
    let Some(start) = value.find(['"','“','‘']) else { return false; };
    let head = normalized(value[..start].trim());
    if !["the font of the printed words", "the typography of the printed words", "the colour of the printed words", "the color of the printed words"].contains(&head.as_str()) { return false; }
    let quote = value[start..].chars().next().unwrap();
    let closing = match quote { '“'=>'”','‘'=>'’',_=>'"' };
    let quoted = &value[start+quote.len_utf8()..];
    let Some(end) = quoted.find(closing) else { return false; };
    let tail = quoted[end+closing.len_utf8()..].trim();
    target_identity_phrases(policy).iter().any(|target| normalized_phrase_matches(tail,target))
        && launch_attribute_assertion_object_v17(policy,&format!("{head} printed lettering {tail}"))
}

'''+s[i:]
needle='''    if launch_complete_property_proposition_v17(policy, value)
        || launch_compound_attribute_object_v17(policy, value)''';new='''    if launch_complete_property_proposition_v17(policy, value)
        || launch_compound_attribute_object_v17(policy, value)
        || launch_printed_quoted_property_v17(policy, value)''';assert s.count(needle)==1;s=s.replace(needle,new)
# Preserve named owner spelling when checking coordinated objects.
a=s.index('fn launch_coordinated_property_objects_v17');b=s.index('\nfn launch_explicit_property_context_v17',a);part=s[a:b];needle='''    let prefix = words[..=verb].join(" ");
''';new=needle+'''    let clause = launch_other_clause_v17(statement);
    let (raw_left, raw_right) = clause.split_once(" and ").unwrap_or(("", ""));
    let folded_left = raw_left.to_ascii_lowercase();
    let left_object = folded_left.find(&format!(" {} ", words[verb])).map(|index| &raw_left[index+words[verb].len()+2..]);
    let left_statement = format!("{prefix} {}",left_object.unwrap_or("").trim_start_matches("both "));
    let right_statement = format!("{prefix} {raw_right}");
''';assert part.count(needle)==1;part=part.replace(needle,new)
part=part.replace('''launch_retraction_other_object_v17(policy, &format!("{prefix} {}", left.join(" ")))''','''launch_retraction_other_object_v17(policy, &left_statement)''')
part=part.replace('''launch_retraction_other_object_v17(
            policy,
            &format!("{prefix} {}", words[join + 1..].join(" ")),
        )''','''launch_retraction_other_object_v17(policy, &right_statement)''')
s=s[:a]+part+s[b:];p.write_text(s)
