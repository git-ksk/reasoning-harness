from pathlib import Path
import hashlib,gzip,subprocess,json
p=Path.cwd()/'crates/reasoning-harness-core/src/evidence_relevance.rs';ident=json.loads(Path('/private/tmp/engine073-launch-trace-current30-identity.json').read_text());assert hashlib.sha256(p.read_bytes()).hexdigest()==ident['instrumented_source_sha256'];s=Path('/private/tmp/engine073-trace-current30-base.rs').read_text();Path('/private/tmp/engine073-source27-stage02-source.rs.gz').write_bytes(gzip.compress(s.encode(),mtime=0))
a='''    let gerund = statement
        .trim()
        .strip_prefix("denying ")
        .map(|body| format!("We deny {body}"));'''
b='''    let gerund = statement
        .trim()
        .strip_prefix("denying ")
        .map(|body| format!("We deny {body}"))
        .or_else(|| ["withdraw ", "retract ", "deny "].iter().find_map(|verb| {
            statement.trim().strip_prefix(verb).map(|body| format!("We {verb}{body}"))
        }));'''
assert s.count(a)==1;s=s.replace(a,b,1)
a='''    let text = normalized(statement);
    let words = text.split_whitespace().collect::<Vec<_>>();
    if !words.iter().enumerate().any(|(i, word)| {'''
b='''    if let Some((head, explanation)) = statement.split_once(':') {
        if let Some(role) = launch_single_retracted_role_v17(head) {
            let explanation = normalized(explanation);
            let words = explanation.split_whitespace().collect::<Vec<_>>();
            let field = match words.as_slice() {
                ["the", "disputed", "assertion" | "claim" | "statement", "concerns", "what", field, "was", "printed", "not", "an", "event"] => Some(*field),
                _ => None,
            };
            let selected = objects.iter().filter(|object| object.label == role).collect::<Vec<_>>();
            if field.is_some_and(|field| ["title", "heading", "label", "sentence"].contains(&field)
                && !selected.is_empty() && selected.iter().all(|object| {
                    object.scope == Scope::Excluded && !object.foreign_event
                        && normalized(&object.declared_value).split_whitespace().any(|word| word == field)
                })) {
                return launch_typed_reference_v17(policy, head, objects, previous, next);
            }
        }
    }
    let text = normalized(statement);
    let words = text.split_whitespace().collect::<Vec<_>>();
    if !words.iter().enumerate().any(|(i, word)| {'''
assert s.count(a)==1;s=s.replace(a,b,1)
a='''    if let Some(value) = apposition {
        let value = value'''
b='''    if let Some(value) = apposition {
        if let [_, _, predicate] = parts.as_slice() {
            let text = normalized(predicate);
            let words = text.split_whitespace().collect::<Vec<_>>();
            let passive = matches!(words.as_slice(), ["is" | "was" | "are" | "were", verb]
                | ["is" | "was" | "are" | "were", verb, "by", "us" | "me" | "author" | "editor" | "publisher"] if launch_retraction_word_v17(verb));
            if !passive && launch_single_retracted_role_v17(predicate).as_deref() != selected_label {
                return Some(false);
            }
        }
        let value = value'''
assert s.count(a)==1;s=s.replace(a,b,1)
# Add new adversarial continuations to the existing new groups.
a='''        for context in [
            "The Object is the printed cover title 'Launch notice on the Silver Lens booklet.'''
b='''        check_current_role_context("The Object is the printed cover title 'Launch notice' on the Silver Lens booklet. We deny this Object: the disputed assertion concerns what title was printed, not an event.", true);
        check_current_role_context("The Object is the printed cover title 'Launch notice' on the Silver Lens booklet. We deny this Object: the disputed assertion concerns what title was printed, not an event, but we deny that Silver Lens was launched.", false);
        for context in [
            "The Object is the printed cover title 'Launch notice on the Silver Lens booklet.'''
assert s.count(a)==1;s=s.replace(a,b,1)
a='''            ("We deny the Subject, the claim that Silver Lens was launched.", false),'''
# Source has been rustfmt formatted; locate by string instead of surrounding tuple formatting.
needle='''        for (context, expected) in [
            (
                "We deny the Subject, the dotted line'''
insert='''        check_current_role_context("We deny this Subject, the dotted line printed around the Silver Lens diagram, but we deny that Silver Lens was launched.", false);
        for (context, expected) in [
            (
                "We deny the Subject, the dotted line'''
assert s.count(needle)==1;s=s.replace(needle,insert,1)
p.write_text(s);subprocess.run(['cargo','fmt','--all'],check=True);print(hashlib.sha256(p.read_bytes()).hexdigest())
