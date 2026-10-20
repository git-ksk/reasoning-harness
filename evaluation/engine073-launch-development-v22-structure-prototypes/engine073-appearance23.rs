fn launch_complete_assertion_body_v17(value: &str) -> &str {
    let value = value.trim();
    let folded = value.to_ascii_lowercase();
    ["the claim that ", "the assertion that ", "the statement that "]
        .iter()
        .find(|prefix| folded.starts_with(**prefix))
        .map_or(value, |prefix| value[prefix.len()..].trim())
}

// These subjects identify one local physical/document component and its own
// complete owner. They cannot identify an event merely by naming its publisher.
fn launch_owned_appearance_subject_v17(
    policy: &EvidenceRelevanceTargetPolicy,
    value: &str,
) -> bool {
    let component = |word: &str| {
        ["certificate", "card", "manual", "case", "folder", "notice", "sleeve",
         "bulletin", "booklet", "document", "title", "cover", "label", "table",
         "caption", "clasp", "sheet", "insert", "badge", "sticker", "catalog",
         "page", "spine", "binding", "panel", "seal", "hinge", "edge"]
            .contains(&word)
    };
    let components = |text: &str| {
        let text = normalized(text);
        let words = text.split_whitespace().collect::<Vec<_>>();
        !words.is_empty() && words.len() <= 5
            && words.last().is_some_and(|word| component(word))
            && words.iter().all(|word| {
                (!launch_scope_or_reference_word_v17(word)
                    || ["document", "notice", "bulletin", "page"].contains(word))
                    && !["and", "or", "not", "never", "has", "have", "is", "was",
                         "are", "were", "uses", "prints", "because", "although"].contains(word)
            })
    };
    let value = value.trim();
    let value = value.strip_prefix("the ").or_else(|| value.strip_prefix("a ")).unwrap_or(value);
    let owner_and_component = |value: &str| {
        let words = value.split_whitespace().collect::<Vec<_>>();
        let end = words.iter().take_while(|word| word.chars().next().is_some_and(char::is_uppercase)).count();
        end > 0 && end < words.len()
            && launch_named_subject_v17(&words[..end].join(" "))
            && components(&words[end..].join(" "))
    };
    if owner_and_component(value) {
        return true;
    }
    let folded = normalized(value);
    if target_identity_phrases(policy).iter().any(|target| {
        folded.strip_prefix(target).is_some_and(|tail| {
            tail.starts_with(' ') && components(tail.trim_start().strip_prefix("s ").unwrap_or(tail.trim_start()))
        })
    }) {
        return true;
    }
    for link in [" supplied with ", " in ", " on ", " for "] {
        if let Some((head, owner)) = value.split_once(link) {
            let owner = owner.strip_prefix("the ").unwrap_or(owner).trim();
            if components(head) && (launch_named_subject_v17(owner) || owner_and_component(owner)) {
                return true;
            }
        }
    }
    false
}

fn launch_complete_appearance_proposition_v17(
    policy: &EvidenceRelevanceTargetPolicy,
    value: &str,
) -> bool {
    let value = launch_complete_assertion_body_v17(value);
    if value.contains(['"', '“', '‘', '?']) {
        return false;
    }
    if let Some((head, appearance)) = value.split_once(" has ") {
        let text = normalized(appearance);
        let words = text.split_whitespace().collect::<Vec<_>>();
        let field = words.last().is_some_and(|word| {
            ["watermark", "spine", "edge", "corner", "corners", "tie", "border",
             "flap", "page", "hinge", "period", "seal", "clasp", "coating",
             "handle", "label", "binding", "panel", "aperture", "groove"]
                .contains(word)
        });
        return launch_owned_appearance_subject_v17(policy, head) && field
            && words.len() <= 5
            && words.iter().all(|word| {
                (!launch_scope_or_reference_word_v17(word) || *word == "page")
                    && !["and", "or", "not", "never", "is", "was", "has", "have",
                         "because", "although", "while", "when", "if", "since"].contains(word)
            });
    }
    if let Some((head, printed)) = value.split_once(" prints ") {
        if !launch_owned_appearance_subject_v17(policy, head) {
            return false;
        }
        let text = normalized(printed);
        let tail = ["its units in ", "its address in ", "its revision mark inside a "]
            .iter().find_map(|prefix| text.strip_prefix(prefix));
        return tail.is_some_and(|tail| {
            let words = tail.split_whitespace().collect::<Vec<_>>();
            !words.is_empty() && words.len() <= 3
                && words.iter().all(|word| {
                    !launch_scope_or_reference_word_v17(word)
                        && !["and", "or", "not", "never", "is", "was", "has", "have",
                             "because", "although", "while", "when", "if", "since"].contains(word)
                })
        });
    }
    false
}
