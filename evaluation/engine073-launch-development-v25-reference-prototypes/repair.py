from pathlib import Path
import gzip,hashlib,subprocess
p=Path.cwd()/'crates/reasoning-harness-core/src/evidence_relevance.rs';s=p.read_text();Path('/private/tmp/engine073-source27-red-source.rs.gz').write_bytes(gzip.compress(p.read_bytes(),mtime=0))
def rep(a,b):
 global s
 assert s.count(a)==1,(a[:80],s.count(a));s=s.replace(a,b,1)
anchor='fn launch_factual_segments_v17('
helper=r'''// Split one balanced literal while leaving possessive apostrophes alone.
// Outside grammar, rather than words inside the literal, establishes its scope.
fn launch_literal_parts_v17(value: &str) -> Option<(&str, &str, &str)> {
    let (start, quote) = value.char_indices().find(|(index, c)| {
        ['"', '“', '‘', '\''].contains(c)
            && (*c != '\'' || !value[..*index].chars().next_back().is_some_and(char::is_alphanumeric))
    })?;
    let close = match quote { '“' => '”', '‘' => '’', c => c };
    let rest = &value[start + quote.len_utf8()..];
    let end = rest.find(close)?;
    if rest[..end].trim().is_empty() { return None; }
    Some((&value[..start], &rest[..end], &rest[end + close.len_utf8()..]))
}

'''+anchor
rep(anchor,helper)
rep('''    let mut prepared = String::with_capacity(value.len());''','''    let mut literals = Vec::new();
    let mut cursor = 0;
    while let Some((head, _, tail)) = launch_literal_parts_v17(&value[cursor..]) {
        let end = value.len() - tail.len();
        literals.push(cursor + head.len()..end);
        cursor = end;
    }
    let mut prepared = String::with_capacity(value.len());''')
rep("        if !keep_period && ['.', ';', '\\n', '\\r', '!', '?', '。', '！', '？'].contains(&character)","        let in_literal = literals.iter().any(|span| span.start < index && index + character.len_utf8() < span.end);\n        if !keep_period && !in_literal && ['.', ';', '\\n', '\\r', '!', '?', '。', '！', '？'].contains(&character)")
# A carrier's name is complete; the artifact itself is not an event.
rep('''            "glossary",
        ]
        .contains(&word)
    };
    let components''','''            "glossary",
            "backplate",
        ]
        .contains(&word)
    };
    let components''')
anchor='fn launch_printed_quoted_property_v17(policy: &EvidenceRelevanceTargetPolicy, value: &str) -> bool {'
helpers=r'''fn launch_owned_printed_nominal_v17(policy: &EvidenceRelevanceTargetPolicy, value: &str) -> bool {
    [" printed on ", " printed around ", " printed in ", " on "].iter().any(|link| {
        value.split_once(link).is_some_and(|(head, owner)| {
            let text = normalized(head);
            let words = text.split_whitespace().collect::<Vec<_>>();
            let artifact = words.last().is_some_and(|w| ["illustration", "line", "lettering", "drawing"].contains(w));
            artifact && (*link != " on " || words.last() == Some(&"lettering"))
                && words.iter().all(|w| {
                    (!launch_scope_or_reference_word_v17(w) || *w == "opening")
                        && !["not", "never", "unknown", "missing", "and", "or", "is", "was", "are", "were", "has", "have"].contains(w)
                        && !launch_retraction_word_v17(w)
                })
                && launch_owned_appearance_subject_v17(policy, owner.trim())
        })
    })
}

fn launch_unowned_quoted_event_v17(value: &str) -> bool {
    let Some((head, body, tail)) = launch_literal_parts_v17(value) else { return false; };
    let head = normalized(head);
    let body = normalized(body);
    matches!(head.as_str(), "the quoted launch claim" | "the quoted release claim" | "a quoted launch claim" | "a quoted release claim")
        && tail.trim().is_empty()
        && matches!(body.as_str(), "the product was launched" | "the product was released" | "the product was not launched" | "the product was not released")
}

// A finite single-role retraction binds only its own apposition or immediately
// adjacent explicit target field. An existing unresolved role is never erased.
fn launch_single_retracted_role_v17(statement: &str) -> Option<String> {
    let text = normalized(statement);
    let words = text.split_whitespace().collect::<Vec<_>>();
    let verb = words.iter().position(|w| launch_retraction_word_v17(w))?;
    if words[..verb].is_empty() || !words[..verb].iter().all(|w| ["we", "i", "also", "hereby", "formally", "explicitly"].contains(w)) { return None; }
    match &words[verb+1..] {
        ["the" | "this" | "that", role @ ("object" | "subject" | "item")] => Some((*role).to_owned()),
        _ => None,
    }
}

'''+anchor
rep(anchor,helpers)
rep(anchor+'\n',anchor+r'''
    if let Some((head, _, tail)) = launch_literal_parts_v17(value) {
        let head = normalized(head);
        if ["the printed cover title", "the printed title", "the printed heading"].contains(&head.as_str()) {
            return tail.trim().strip_prefix("on ").or_else(|| tail.trim().strip_prefix("in "))
                .is_some_and(|owner| launch_owned_appearance_subject_v17(policy, owner));
        }
    }
''')
rep('''    if launch_owned_material_object_v17(value)
        || launch_complete_quoted_foreign_event_v17(policy, value)''','''    if launch_owned_material_object_v17(value)
        || launch_owned_printed_nominal_v17(policy, value)
        || launch_complete_quoted_foreign_event_v17(policy, value)''')
# Add roles from syntactically explicit local appositions / target fields.
rep('''        for (index, segment) in segments.iter().enumerate() {
            if launch_quoted_claim_property_v17''','''        for (index, segment) in segments.iter().enumerate() {
            if let Some((head, value)) = segment.raw.split_once(',') {
                if let Some(role) = launch_single_retracted_role_v17(head) {
                    declarations.push((role, value.split_once(',').map_or(value, |(first, _)| first).trim().to_owned()));
                }
            }
            if let Some((label, value)) = segment.raw.split_once(':') {
                if normalized(label) == "denial target" {
                    if let Some(role) = index.checked_sub(1).and_then(|i| launch_single_retracted_role_v17(&segments[i].raw)) {
                        declarations.push((role, value.trim().to_owned()));
                    }
                }
            }
            if launch_quoted_claim_property_v17''')
rep('''            let unowned = unowned
                || matches!(''','''            let unowned = unowned || launch_unowned_quoted_event_v17(value)
                || matches!(''')
rep('''["denial object", "denial explanation"].contains(&normalized(label).as_str())''','''["denial object", "denial explanation", "denial target"].contains(&normalized(label).as_str())''')
# Possessive properties must be present in every selected complete property value.
rep('''    if !(plain_tail || property_tail) {
''','''    let possessive_property = label.is_some() && !event_reference && {
        let mut topic = tail;
        for prefix in [&["the", "claim", "about"][..], &["the", "statement", "about"][..], &["the", "assertion", "about"][..]] {
            if let Some(rest) = topic.strip_prefix(prefix) { topic = rest; break; }
        }
        if matches!(topic.first(), Some(&"this" | &"that" | &"the")) { topic = &topic[1..]; }
        matches!(topic, [role, "s", property @ ..] if Some(*role) == label
            && !property.is_empty()
            && launch_specific_property_object_v17(policy, property)
            && selected.iter().all(|object| {
                let value=normalized(&object.declared_value);
                let words=value.split_whitespace().collect::<Vec<_>>();
                object.scope == Scope::Excluded && !object.foreign_event
                    && words.windows(property.len()).any(|window| window == property)
            }))
    };
    if !(plain_tail || property_tail || possessive_property) {
''')
# Printed projection uses the same balanced literal interpretation.
a='''                    let start = segment.raw.find(['"', '“', '‘']).unwrap();
                    let quote = segment.raw[start..].chars().next().unwrap();
                    let closing = match quote {
                        '“' => '”',
                        '‘' => '’',
                        _ => '"',
                    };
                    let body = start + quote.len_utf8();
                    let end = body + segment.raw[body..].find(closing).unwrap();
                    signal_changed = true;
                    return format!(
                        "{}printed lettering{}",
                        &segment.raw[..body],
                        &segment.raw[end..]
                    );'''
b='''                    if let Some((head, _, tail)) = launch_literal_parts_v17(&segment.raw) {
                        signal_changed = true;
                        return format!("{head}printed lettering {tail}");
                    }'''
rep(a,b)
# Only this private successor view renames the already proven target for the
# historical risk computation. Public policy/IDs/raw risk are unchanged.
a='''fn launch_opaque_name_context_view_v17(''';start=s.index(a);end=s.index('\nfn launch_foreign_context_view_v17(',start);f=s[start:end]
f=f.replace('Result<Option<EvidenceRelevanceCandidate>, EvidenceRelevanceError>', 'Result<Option<(EvidenceRelevanceTargetPolicy, EvidenceRelevanceCandidate)>, EvidenceRelevanceError>',1)
f=f.replace('''    let mut view = candidate.clone();''','''    let mut view_policy = policy.clone();
    let Some(entity) = view_policy.entity.as_mut() else { return Ok(None); };
    entity.canonical_name = "anchored target".to_owned();
    entity.aliases.clear();
    let mut view = candidate.clone();''',1)
f=f.replace('''            anchor = true;
            continue;''','''            anchor = true;''',1)
f=f.replace('derive_effective_evidence_local_qualification_v16(policy, &view, proposal, raw)?','derive_effective_evidence_local_qualification_v16(&view_policy, &view, proposal, raw)?',1)
f=f.replace('.then_some(view)', '.then_some((view_policy, view))',1)
s=s[:start]+f+s[end:]
rep('''    if let Some(view) = launch_opaque_name_context_view_v17(policy, candidate, proposal, raw)? {
        let scoped =
            derive_effective_evidence_local_qualification_v16(policy, &view, proposal, raw)?;''','''    if let Some((view_policy, view)) = launch_opaque_name_context_view_v17(policy, candidate, proposal, raw)? {
        let scoped =
            derive_effective_evidence_local_qualification_v16(&view_policy, &view, proposal, raw)?;''')
rep('''        match launch_scope_annotation_view_v17(policy, candidate, proposal, raw_qualification)?.or(''','''        match launch_scope_annotation_view_v17(policy, candidate, proposal, raw_qualification)?.map(|view| (policy.clone(), view)).or(''')
rep('''                launch_foreign_context_view_v17(policy, candidate, proposal, raw_qualification)?
''','''                launch_foreign_context_view_v17(policy, candidate, proposal, raw_qualification)?.map(|view| (policy.clone(), view))
''')
rep('''        policy,
        scoped_candidate.as_ref().unwrap_or(candidate),''','''        scoped_candidate.as_ref().map_or(policy, |(view_policy, _)| view_policy),
        scoped_candidate.as_ref().map_or(candidate, |(_, view)| view),''')
p.write_text(s);subprocess.run(['cargo','fmt','--all'],check=True);print(hashlib.sha256(p.read_bytes()).hexdigest())
