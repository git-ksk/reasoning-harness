// The quoted body is inert lettering only when its complete outside grammar
// proves a physical/printed attribute and an independently owned component.
fn launch_complete_printed_attribute_v17(
    policy: &EvidenceRelevanceTargetPolicy,
    value: &str,
) -> bool {
    let value = launch_complete_assertion_body_v17(value);
    let Some(start) = value.find(['"', '“', '‘']) else { return false; };
    let quote = value[start..].chars().next().unwrap();
    let closing = match quote { '“'=>'”', '‘'=>'’', _=>'"' };
    let quoted = &value[start + quote.len_utf8()..];
    let Some(end) = quoted.find(closing) else { return false; };
    let head = normalized(&value[..start]);
    let tail = quoted[end + closing.len_utf8()..].trim();
    if tail.contains(['"','“','”','‘','’','?',',']) { return false; }
    let owned = |owner: &str| launch_owned_appearance_subject_v17(policy, owner.trim());
    if head == "the sentence" {
        return tail.strip_prefix("is printed in ").and_then(|tail| tail.split_once(" ink on "))
            .is_some_and(|(colour, owner)| {
                let colour = normalized(colour);
                let words = colour.split_whitespace().collect::<Vec<_>>();
                !words.is_empty() && words.len() <= 2
                    && words.iter().all(|word| !launch_scope_or_reference_word_v17(word)
                        && !["and","or","not","never","is","was","has"].contains(word))
                    && owned(owner)
            });
    }
    if ["the heading", "the section heading"].contains(&head.as_str()) {
        if let Some((owner, style)) = tail.strip_prefix("in ").and_then(|tail| tail.split_once(" uses ")) {
            return owned(owner) && ["small capitals","uppercase letters","lowercase letters","bold lettering"].contains(&normalized(style).as_str());
        }
        if let Some((count, owner)) = tail.strip_prefix("occupies ").and_then(|tail| tail.split_once(" on ")) {
            let count=normalized(count);let words=count.split_whitespace().collect::<Vec<_>>();
            return matches!(words.as_slice(), [number,"line"|"lines"]
                if ["one","two","three","four","five","six","seven","eight","nine","ten"].contains(number)
                    || (!number.is_empty() && number.chars().all(|c|c.is_ascii_digit()))) && owned(owner);
        }
    }
    if head == "the engraving" {
        return tail.strip_prefix("on ").and_then(|tail|tail.split_once(" is "))
            .is_some_and(|(owner,depth)| owned(owner) && ["shallow","deep"].contains(&normalized(depth).as_str()));
    }
    if ["the footer","the literal footer"].contains(&head.as_str()) {
        return tail.strip_prefix("in ").and_then(|tail|tail.split_once(" is "))
            .is_some_and(|(owner,style)|owned(owner) && ["underlined","italicized","bold"].contains(&normalized(style).as_str()));
    }
    false
}
