from pathlib import Path
import hashlib,subprocess
p=Path('crates/reasoning-harness-core/src/evidence_relevance.rs');s=p.read_text()
assert 'fn v30_parenthetical_correction_coverage_requires_same_property_role()' in s
assert 'fn launch_bound_property_coverage_v17(' not in s
def edit(a,b):
 global s
 if s.count(a)==1:s=s.replace(a,b,1)
 else:
  start=s.index('fn launch_complete_foreign_role_owner_v17(');end=s.index('\nfn launch_owner_reply_label_v17(',start);part=s[start:end]
  assert part.count(a)==1,(a[:80],s.count(a),part.count(a));s=s[:start]+part.replace(a,b,1)+s[end:]
edit('''    [" printed on ", " printed around ", " printed in ", " on "]''','''    [" printed on ", " printed around ", " printed along ", " printed in ", " on "]''')
edit('''["illustration", "line", "lettering", "drawing"].contains(w)''','''["illustration", "line", "lettering", "drawing", "watermark"].contains(w)''')
edit('''            "backplate",
        ]''','''            "backplate",
            "tag",
        ]''')
edit('''            | "a quoted release claim"
    )''','''            | "a quoted release claim"
            | "the quoted claim"
            | "the quoted assertion"
            | "the quoted statement"
    )''')
edit('''["author","editor","publisher","first","second","third","fourth","fifth","additional","another","alternative","competing","further","subsequent"].contains(kind)''','''["author","editor","publisher","immediate","first","second","third","fourth","fifth","additional","another","alternative","competing","further","subsequent"].contains(kind)''')
edit('''        if [
            "the printed cover title",
            "the printed title",
            "the printed heading",
        ]
        .contains(&head.as_str())''','''        // The outside printed type is unchanged by an optional article.
        if ["printed cover title", "printed title", "printed heading"]
            .contains(&head.strip_prefix("the ").unwrap_or(&head))''')
edit('''        if spacing || position {''','''        let lettering = matches!(words.as_slice(),
            ["the", "uppercase" | "lowercase" | "bold" | "italic", "lettering", "of", "the", "word"]);
        if lettering {
            return tail.trim().strip_prefix("printed on ")
                .is_some_and(|owner| launch_owned_appearance_subject_v17(policy, owner));
        }
        if spacing || position {''')
edit('''    let owner = [
        " was launched",
        " was released",
        " was not launched",
        " was not released",
    ]''','''    let possessive_owner = ["the claim about ", "the statement about ", "the assertion about "]
        .iter().find_map(|prefix| body.strip_prefix(prefix))
        .and_then(|value| ["'s launch", "’s launch", "'s release", "’s release"]
            .iter().find_map(|suffix| value.strip_suffix(suffix)));
    let owner = [
        " was launched",
        " was released",
        " was not launched",
        " was not released",
    ]''')
edit('''        .find_map(|prefix| body.strip_prefix(prefix))
    })?;''','''        .find_map(|prefix| body.strip_prefix(prefix))
    }).or(possessive_owner)?;''')
edit('''        let roles = tail
            .iter()
            .copied()
            .filter(|word| ["object", "subject", "item"].contains(word))
            .collect::<Vec<_>>();
        if roles.len() >= 2
            && tail
                .iter()
                .all(|word| ["the", "and", "object", "subject", "item"].contains(word))
            && tail.iter().filter(|word| **word == "and").count() + 1 == roles.len()''','''        // A plural claim wrapper coordinates finite role pointers. Each role
        // still needs its own complete excluded type; no unbounded list words.
        let tail = match tail {
            ["the", "assertions" | "claims" | "statements", "about", rest @ ..] => rest,
            _ => tail,
        };
        let groups = tail.split(|word| *word == "and").collect::<Vec<_>>();
        let roles = groups.iter().filter_map(|group| match *group {
            ["the" | "this" | "that", role @ ("object" | "subject" | "item")]
            | [role @ ("object" | "subject" | "item")] => Some(*role),
            _ => None,
        }).collect::<Vec<_>>();
        if roles.len() >= 2 && groups.len() == roles.len()''')
edit('''        return Some(
            launch_typed_object_scope_v17(policy, value) == Scope::Excluded
                && !selected.iter().any(|object| object.scope == Scope::Launch),
        );''','''        if let Some(owner) = launch_complete_foreign_role_owner_v17(policy, value) {
            return Some(selected.iter().all(|object| object.scope == Scope::Excluded
                && object.foreign_event && object.foreign_owner.as_deref() == Some(owner.as_str())));
        }
        return Some(
            launch_typed_object_scope_v17(policy, value) == Scope::Excluded
                && !selected.iter().any(|object| object.scope == Scope::Launch),
        );''')
helper=r'''
// A bounded correction-coverage note can restate only properties already
// present in the immediately preceding resolved same-role retraction.
fn launch_bound_property_coverage_v17(
    policy: &EvidenceRelevanceTargetPolicy,
    statement: &str,
    previous: &LaunchFactualSegmentV17,
    objects: &[LaunchTypedObjectV17],
) -> bool {
    let text=normalized(statement);
    let words=text.split_whitespace().collect::<Vec<_>>();
    let ["the", "withdrawal" | "retraction" | "denial", "covers", body @ ..] = words.as_slice() else {return false;};
    let Some(property)=body.strip_suffix(&["description", "only"]) else {return false;};
    if !(2..=4).contains(&property.len()) || !property.iter().any(|word|
        ["finish", "colour", "color", "font", "material", "width", "height", "weight", "lettering"].contains(word))
        || property.iter().any(|word| launch_scope_or_reference_word_v17(word)
            || launch_retraction_word_v17(word)
            || ["and", "or", "not", "never", "is", "was", "has", "have"].contains(word)) {
        return false;
    }
    let prior=normalized(&previous.raw);
    let roles=prior.split_whitespace().filter(|word|["object","subject","item"].contains(word)).collect::<Vec<_>>();
    let [role]=roles.as_slice() else {return false;};
    if launch_typed_reference_v17(policy,&previous.raw,objects,None,None)!=Some(true) {return false;}
    let selected=objects.iter().filter(|object|object.label==*role).collect::<Vec<_>>();
    !selected.is_empty() && selected.iter().all(|object| {
        let declared=normalized(&object.declared_value);
        object.scope==LaunchTypedScopeV17::Excluded && !object.foreign_event
            && property.iter().all(|word|declared.split_whitespace().any(|part|part==*word))
    })
}

'''
anchor='fn launch_bound_property_reference_v17(';assert s.count(anchor)==1;s=s.replace(anchor,helper+anchor,1)
edit('''    let note = normalized(statement);
    if let Some(property) = note''','''    if launch_bound_property_coverage_v17(policy, statement, previous, objects) {
        return true;
    }
    let note = normalized(statement);
    if let Some(property) = note''')
# A different complete foreign apposition cannot replace the frozen role owner.
needle='''    fn v30_generic_quoted_claim_requires_unique_same_role_owner() {'''
assert s.count(needle)==1
idx=s.index(needle)+len(needle)
s=s[:idx]+'''        check_current_role_context("The Item is the quoted claim: 'The product was launched.' Immediate reply: Item owner is Violet Bell. We withdraw this Item, the claim about Cedar Basket's launch.",false);\n'''+s[idx:]
p.write_text(s);subprocess.run(['cargo','fmt','--all'],check=True)
print('shared outside-type, role coordination and bounded coverage repair applied in private prototype')
