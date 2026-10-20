from pathlib import Path
import gzip,hashlib,json,subprocess
p=Path('crates/reasoning-harness-core/src/evidence_relevance.rs');s=p.read_text();assert s==gzip.decompress(Path('/private/tmp/engine073-source28-red/source.rs.gz').read_bytes()).decode()
a=s.index('fn launch_owned_material_object_v17(');b=s.index('\nfn launch_complete_printed_property_correction_v17(',a)
old=s[a:b]
new=old.replace('''    if folded[start..].starts_with("paper ") {
        start += 6;
    }
    if !folded[start..].starts_with("material ") {
        return false;
    }
    start += 9;''','''    let material = if folded[start..].starts_with("material ") {
        start
    } else if let Some(index) = folded[start..].find(" material ") {
        start + index + 1
    } else {
        return false;
    };
    let modifier = normalized(&value[start..material]);
    let modifiers = modifier.split_whitespace().collect::<Vec<_>>();
    if modifiers.len() > 3 || modifiers.iter().any(|word| {
        launch_scope_or_reference_word_v17(word)
            || launch_adverse_context_word_v17(word)
            || launch_retraction_word_v17(word)
            || ["and", "or", "but", "not", "never", "is", "was", "has", "have"].contains(word)
    }) {
        return false;
    }
    start = material + 9;''')
assert new!=old
new=new.replace('''    launch_named_property_owner_tail_v17(value[start + link.len()..].trim())''','''    let owner = value[start + link.len()..].trim();
    let owner = ["the ", "a ", "an "].iter()
        .find_map(|article| owner.strip_prefix(article)).unwrap_or(owner);
    launch_named_property_owner_tail_v17(owner)''');s=s[:a]+new+s[b:]
helper='''// A non-adverse local correction label may introduce one complete copular
// role declaration. The role/value still supplies its own type and owner;
// target/global denials and unknown labels never acquire that property's scope.
fn launch_local_role_declaration_body_v17(statement: &str) -> &str {
    let Some((label, value)) = statement.split_once(':') else { return statement; };
    if launch_owner_reply_label_v17(label) { return value.trim(); }
    let text = normalized(label);
    let words = text.split_whitespace().collect::<Vec<_>>();
    let Some(context) = words.strip_suffix(&["correction"]) else { return statement; };
    let value = value.trim();
    let copular_role = value.to_ascii_lowercase().find(" is ").is_some_and(|index| {
        let head = normalized(&value[..index]);
        ["object", "subject", "item"].contains(&head.strip_prefix("the ").unwrap_or(&head))
    });
    if copular_role && (1..=3).contains(&context.len()) && context.iter().all(|word| {
        !launch_scope_or_reference_word_v17(word)
            && !launch_adverse_context_word_v17(word)
            && !launch_retraction_word_v17(word)
            && !["all", "entire", "whole", "both", "not", "no", "never"].contains(word)
    }) { value } else { statement }
}

'''
needle='fn launch_printed_declaration_v17(policy: &EvidenceRelevanceTargetPolicy, statement: &str) -> bool {';assert s.count(needle)==1;s=s.replace(needle,helper+needle,1)
old='''    let statement = statement
        .split_once(':')
        .map_or(statement, |(label, value)| {
            if launch_owner_reply_label_v17(label) {
                value.trim()
            } else {
                statement
            }
        });''';assert s.count(old)==1;s=s.replace(old,'''    let statement = launch_local_role_declaration_body_v17(statement);''',1)
old='''            let raw = segment
                .raw
                .split_once(':')
                .map_or(segment.raw.as_str(), |(label, value)| {
                    if launch_owner_reply_label_v17(label) {
                        value.trim()
                    } else {
                        segment.raw.as_str()
                    }
                });''';assert s.count(old)==1;s=s.replace(old,'''            let raw = launch_local_role_declaration_body_v17(&segment.raw);''',1)
needle='''        let head = normalized(head);
        if [
            "the printed cover title",''';assert s.count(needle)==1
replacement='''        let head = normalized(head);
        let words = head.split_whitespace().collect::<Vec<_>>();
        let spacing = matches!(words.as_slice(),
            ["the", "physical", "letter" | "word" | "line", "spacing", "in", "the", "printed", "sentence"]);
        let position = matches!(words.as_slice(),
            ["the", "position", "of", "the", "printed", "heading" | "title" | "label"]);
        if spacing || position {
            return tail.trim().strip_prefix("on ").or_else(|| tail.trim().strip_prefix("in "))
                .is_some_and(|owner| launch_owned_appearance_subject_v17(policy, owner)
                    || (spacing && matches!(normalized(owner).split_whitespace().collect::<Vec<_>>().as_slice(),
                        ["a" | "the", "poster" | "label" | "sheet"])));
        }
        if [
            "the printed cover title",'''
s=s.replace(needle,replacement,1);p.write_text(s);subprocess.run(['cargo','fmt','--all'],check=True)
d=Path('/private/tmp/engine073-source28-stage01');d.mkdir();(d/'source.rs.gz').write_bytes(gzip.compress(p.read_bytes(),mtime=0));(d/'identity.json').write_text(json.dumps({'source_sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'scope':'private development; complete local copular declarations, qualified material, printed layout proofs'},indent=2)+'\n');print('stage01 shared property declaration proofs written')
