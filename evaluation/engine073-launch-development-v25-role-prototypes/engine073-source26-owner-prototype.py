from pathlib import Path
p=Path('crates/reasoning-harness-core/src/evidence_relevance.rs');s=p.read_text()
needle='''    let statement = if target_identity_phrases(policy)''';a=s.index('fn launch_retraction_other_object_v17(');i=s.index(needle,a)
s=s[:i]+'''    let gerund = statement.trim().strip_prefix("denying ")
        .map(|body| format!("We deny {body}"));
    let statement = gerund.as_deref().unwrap_or(statement);
'''+s[i:]
needle='''                        "the claim ",
                        "our statement ",''';assert s.count(needle)==1;s=s.replace(needle,'''                        "the claim ",
                        "the separate claim ",
                        "a separate claim ",
                        "our statement ",''')
a=s.index('fn launch_owner_frame_role_v17(');i=s.index('    if let [\n        "owner",\n        "statement",',a)
s=s[:i]+'''    let short = statement_words.strip_prefix(&["the"]).unwrap_or(statement_words);
    if let [role @ ("object" | "subject" | "item"), "owner"] = short {
        return Some(Some(role));
    }
'''+s[i:]
a=s.index('fn launch_owner_reply_label_v17(');b=s.index('\nfn launch_typed_objects_v17(',a);old=s[a:b]
old=old.replace('    words == ["reply"]','''    matches!(words.as_slice(), [actor,"statement"] if ["author","editor","publisher"].contains(actor))
        || matches!(words.as_slice(), [kind,actor,"reply"|"statement"] if ["immediate","first","second","third","additional","another","alternative","competing","subsequent"].contains(kind) && ["author","editor","publisher"].contains(actor))
        || words == ["reply"]''');s=s[:a]+old+s[b:]
a=s.index('fn launch_typed_objects_v17(');i=s.index('            if let Some((label, value)) = raw.split_once',a)
s=s[:i]+'''            let copular_role = raw.to_ascii_lowercase().find(" is ").and_then(|index| {
                let label = normalized(&raw[..index]);
                let role = label.strip_prefix("the ").unwrap_or(&label);
                ["object", "subject", "item"].contains(&role)
                    .then(|| (role.to_owned(), raw[index + 4..].trim().to_owned()))
            });
            if let Some(declaration) = copular_role {
                declarations.push(declaration);
            } else '''+s[i:].lstrip() # keep the original colon/owner fallbacks
# Existing copular-role branch is now handled once above, including values containing a colon.
old='''                let role = label.strip_prefix("the ").unwrap_or(&label);
                if ["object", "subject", "item"].contains(&role) {
                    // Copular and colon declarations carry the same local role.
                    // The value still needs its own complete scope/owner proof.
                    declarations.push((role.to_owned(), raw[index + 4..].trim().to_owned()));
                } else if launch_owner_frame_role_v17(&label).is_some() {'''
assert s.count(old)==1;s=s.replace(old,'''                if launch_owner_frame_role_v17(&label).is_some() {''')
# A role's explicit event and all owner statements must agree. This only adds a veto.
a=s.index('fn launch_typed_objects_v17(');i=s.index('launch_complete_quoted_foreign_event_owner_v17(policy, value)',a);s=s[:i]+s[i:].replace('launch_complete_quoted_foreign_event_owner_v17(policy, value)','launch_complete_foreign_role_owner_v17(policy, value)',1)
i=s.index('fn launch_owner_reply_label_v17(')
s=s[:i]+'''fn launch_complete_foreign_role_owner_v17(
    policy: &EvidenceRelevanceTargetPolicy,
    value: &str,
) -> Option<String> {
    if let Some(owner) = launch_complete_quoted_foreign_event_owner_v17(policy, value) {
        return Some(owner);
    }
    let body = launch_complete_assertion_body_v17(value);
    let owner = [" was launched", " was released", " was not launched", " was not released"]
        .iter().find_map(|suffix| body.strip_suffix(suffix))
        .or_else(|| ["the launch of ", "the release of ", "launch of ", "release of "]
            .iter().find_map(|prefix| body.strip_prefix(prefix)))?;
    (launch_named_subject_v17(owner) && !target_identity_phrases(policy).iter()
        .any(|target| normalized_phrase_matches(owner, target))).then(|| normalized(owner))
}

'''+s[i:]
p.write_text(s)
