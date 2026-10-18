from pathlib import Path
import gzip,hashlib,json,subprocess
p=Path('crates/reasoning-harness-core/src/evidence_relevance.rs');s=p.read_text();assert hashlib.sha256(p.read_bytes()).hexdigest()=='2be99a5d6b2acb7312bd904395e82c74116d2bc0f0828103d37a2e37ade192b8'
old='''        "the statement that ",
    ]''';assert s.count(old)==1;s=s.replace(old,'''        "the statement that ",
        "claim that ",
        "assertion that ",
        "statement that ",
    ]''',1)
old='''        || launch_complete_quoted_foreign_event_v17(policy, value)
        || launch_complete_property_proposition_v17(policy, value)''';assert s.count(old)==1;s=s.replace(old,'''        || launch_complete_quoted_foreign_event_v17(policy, value)
        || launch_complete_foreign_role_owner_v17(policy, value).is_some()
        || launch_complete_property_proposition_v17(policy, value)''',1)
old='''    foreign_event: bool,
    declared_value: String,''';assert s.count(old)==1;s=s.replace(old,'''    foreign_event: bool,
    foreign_owner: Option<String>,
    declared_value: String,''',1)
old='''        !matching.is_empty()
            && matching.iter().all(|owner| {
                owner.as_ref().is_some_and(|(name, scope)| {
                    *scope == Scope::Excluded
                        && matching
                            .first()
                            .and_then(|first| first.as_ref())
                            .is_some_and(|first| first.0 == *name)
                })
            })'''
new='''        let (first, _) = matching.first().and_then(|owner| owner.as_ref())?;
        matching.iter().all(|owner| {
            owner.as_ref().is_some_and(|(name, scope)| *scope == Scope::Excluded && name == first)
        }).then(|| first.clone())''';assert s.count(old)==1;s=s.replace(old,new,1)
old='''                let usable = role.is_none() && unique_foreign_owner(None);''';assert s.count(old)==1;s=s.replace(old,'''                let owner = unique_foreign_owner(None);
                let usable = role.is_none() && owner.is_some();''',1)
old='''                    foreign_event: usable,
                    scope:''';assert s.count(old)==1;s=s.replace(old,'''                    foreign_event: usable,
                    foreign_owner: if usable { owner } else { None },
                    scope:''',1)
old='''            let owner_bound = unowned && unique_foreign_owner(Some(label));''';assert s.count(old)==1;s=s.replace(old,'''            let role_owner = unique_foreign_owner(Some(label));
            let owner_bound = unowned && role_owner.is_some();''',1)
old='''                scope,
                foreign_event,
                declared_value: value.to_owned(),''';assert s.count(old)==1;s=s.replace(old,'''                scope,
                foreign_event,
                foreign_owner: if foreign_event {
                    if owner_bound { role_owner } else { launch_complete_foreign_role_owner_v17(policy, value) }
                } else { None },
                declared_value: value.to_owned(),''',1)
# Complete owner modifiers describe the same finite owner field; unknown,
# competing names and a mismatching role still fail the existing resolver.
old='''    let words = words
        .strip_prefix(&["sole"])
        .or_else(|| words.strip_prefix(&["full"]))
        .unwrap_or(words);
    let words = words.strip_prefix(&["product"]).unwrap_or(words);''';assert s.count(old)==1;s=s.replace(old,'''    let modifier_count = words.iter().take_while(|word| {
        ["sole", "full", "complete", "named", "explicitly"].contains(word)
    }).count();
    let words = &words[modifier_count..];
    let words = words.strip_prefix(&["product"]).unwrap_or(words);''',1)
a=s.index('fn launch_typed_owner_v17(');b=s.index('\nfn launch_owner_frame_role_v17(',a);old=s[a:b];new=old.replace('''                    "full",
                    "product",''','''                    "full",
                    "complete",
                    "named",
                    "explicitly",
                    "product",''');assert new!=old;s=s[:a]+new+s[b:]
helper='''// Resolve an event anaphor only from an immediately adjacent complete
// full-name event/claim. A following explicit product confirmation can narrow
// that existing owner, never invent or replace it.
fn launch_adjacent_foreign_event_reference_v17(
    policy: &EvidenceRelevanceTargetPolicy,
    statement: &str,
    previous: Option<&LaunchFactualSegmentV17>,
    next: Option<&LaunchFactualSegmentV17>,
) -> bool {
    let Some(previous) = previous else { return false; };
    let text = normalized(statement);
    let words = text.split_whitespace().collect::<Vec<_>>();
    let Some(verb) = words.iter().position(|word| launch_retraction_word_v17(word)) else { return false; };
    if words[..verb].is_empty() || !words[..verb].iter().all(|word| {
        ["we", "i", "also", "hereby", "formally", "explicitly"].contains(word)
    }) { return false; }
    let noun = match &words[verb+1..] {
        ["this" | "that", noun @ ("claim" | "assertion" | "statement")] => *noun,
        ["this" | "that", "product", "s", "launch" | "release", noun @ ("claim" | "assertion" | "statement")] => *noun,
        _ => return false,
    };
    let raw = previous.raw.trim();
    let folded = raw.to_ascii_lowercase();
    let body = ["the correction concerns ", "the denial concerns ", "the withdrawal concerns ", "the retraction concerns "]
        .iter().find(|prefix| folded.starts_with(**prefix))
        .map_or(raw, |prefix| raw[prefix.len()..].trim());
    let non_event_owner = ["no launch took place for ", "no release took place for "]
        .iter().find(|prefix| folded.starts_with(**prefix))
        .map(|prefix| raw[prefix.len()..].trim())
        .filter(|owner| launch_named_subject_v17(owner) && !target_identity_phrases(policy).iter()
            .any(|target| normalized_phrase_matches(owner,target)))
        .map(normalized);
    let Some(owner) = non_event_owner.or_else(|| launch_complete_foreign_role_owner_v17(policy,body)) else { return false; };
    if let Some(next) = next {
        if let Some(index) = next.raw.to_ascii_lowercase().find(" is ") {
            let label = normalized(&next.raw[..index]);
            let label_words = label.split_whitespace().collect::<Vec<_>>();
            let confirmation = matches!(label_words.as_slice(),
                ["the", role, "s", "explicitly", "named", "product"]
                    | ["the", role, "s", "named", "product"] if *role == noun);
            if confirmation {
                let stated = next.raw[index+4..].trim();
                return launch_named_subject_v17(stated) && normalized(stated)==owner;
            }
        }
    }
    true
}

'''
needle='fn launch_typed_reference_v17(';assert s.count(needle)==1;s=s.replace(needle,helper+needle,1)
a=s.index('fn launch_typed_reference_v17(');b=s.index('\nfn launch_adverse_segments_v17(',a);old=s[a:b];new=old.replace('''    if let Some((head, restatement)) = statement.split_once(" and only this fully named ") {''','''    if launch_adjacent_foreign_event_reference_v17(policy, statement, previous, next) {
        return Some(true);
    }
    if let Some((head, restatement)) = statement.split_once(" and only this fully named ") {''',1)
needle='''    let event_reference = selected.iter().all(|object| object.foreign_event);
''';assert new.count(needle)==1;new=new.replace(needle,needle+'''    if let Some((head, owner)) = statement.split_once(" owned by ") {
        if label.is_some() && launch_single_retracted_role_v17(head).as_deref() == label {
            let owner = owner.trim();
            let name = normalized(owner);
            return Some(launch_named_subject_v17(owner) && event_reference
                && selected.iter().all(|object| object.scope == Scope::Excluded
                    && object.foreign_owner.as_deref() == Some(name.as_str())));
        }
    }
''',1)
needle='''            &["the", "statement", "represented", "by"][..],''';assert new.count(needle)==1;new=new.replace(needle,needle+'''\n            &["the", "assertion", "represented", "by"][..],''',1);assert new!=old;s=s[:a]+new+s[b:]
p.write_text(s);subprocess.run(['cargo','fmt','--all'],check=True);d=Path('/private/tmp/engine073-source28-stage02');d.mkdir();(d/'source.rs.gz').write_bytes(gzip.compress(p.read_bytes(),mtime=0));(d/'identity.json').write_text(json.dumps({'source_sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'scope':'private development; preserve exact foreign owner in typed roles and adjacent finite event references'},indent=2)+'\n');print('stage02 complete foreign owner/reference proofs written')
