use serde::{Deserialize, Serialize};

use crate::evidence_relevance::{
    EVIDENCE_RELEVANCE_BINDING_PROPOSAL_V5_CONTRACT_ID, EvidenceLocalBlockingReason,
    EvidenceLocalIdentityScope, EvidenceLocalQualificationV6, EvidenceLocalRelationScope,
    EvidenceRelevanceAssessment, EvidenceRelevanceAssessmentPath, EvidenceRelevanceBinding,
    EvidenceRelevanceBindingProposal, EvidenceRelevanceCandidate, EvidenceRelevanceDisposition,
    EvidenceRelevanceError, EvidenceRelevanceIdentityRequirement, EvidenceRelevanceReason,
    EvidenceRelevanceRelationKind, EvidenceRelevanceSignalKind, EvidenceRelevanceTargetPolicy,
    classify_deterministic_local_scope_risk,
};

pub const EVIDENCE_RELEVANCE_LOCAL_AUTHORITY_V1_CONTRACT_ID: &str =
    "reason-evidence-relevance-local-authority-v1";
pub const EVIDENCE_RELEVANCE_BINDING_MATERIALIZATION_POLICY_V20_ID: &str =
    "target-evidence-relevance-binding-materialization-v20";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceAuthoritySource {
    HarnessDeterministic,
    HarnessCorroborated,
    ModelConsensus,
    Unresolved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceLocalAuthorityV1 {
    pub identity_scope: EvidenceLocalIdentityScope,
    pub identity_source: EvidenceAuthoritySource,
    pub relation_scope: EvidenceLocalRelationScope,
    pub relation_source: EvidenceAuthoritySource,
    pub scope_risk: EvidenceLocalBlockingReason,
}

fn normalized(value: &str) -> String {
    let folded = value
        .chars()
        .flat_map(char::to_lowercase)
        .map(|character| {
            if character.is_alphanumeric() {
                character
            } else {
                ' '
            }
        })
        .collect::<String>();
    folded.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn normalized_phrase_matches(text: &str, phrase: &str) -> bool {
    let text = normalized(text);
    let phrase = normalized(phrase);
    if phrase.is_empty() {
        return false;
    }
    if phrase.is_ascii() {
        let text_tokens = text.split_whitespace().collect::<Vec<_>>();
        let phrase_tokens = phrase.split_whitespace().collect::<Vec<_>>();
        return !phrase_tokens.is_empty()
            && text_tokens
                .windows(phrase_tokens.len())
                .any(|window| window == phrase_tokens.as_slice());
    }
    text.contains(&phrase)
}

fn target_phrases(policy: &EvidenceRelevanceTargetPolicy) -> Vec<String> {
    let Some(entity) = &policy.entity else {
        return Vec::new();
    };
    let mut phrases = vec![normalized(&entity.canonical_name)];
    phrases.extend(entity.aliases.iter().map(|alias| normalized(alias)));
    phrases.retain(|phrase| !phrase.is_empty());
    phrases.sort();
    phrases.dedup();
    phrases
}

fn signal_can_own_identity(kind: EvidenceRelevanceSignalKind) -> bool {
    !matches!(
        kind,
        EvidenceRelevanceSignalKind::CanonicalUrl | EvidenceRelevanceSignalKind::NavigationOrFooter
    )
}

fn candidate_local_text(candidate: &EvidenceRelevanceCandidate) -> String {
    candidate
        .signals
        .iter()
        .filter(|signal| signal.kind != EvidenceRelevanceSignalKind::CanonicalUrl)
        .map(|signal| normalized(&signal.text))
        .collect::<Vec<_>>()
        .join(" ")
}

fn target_in_signal(policy: &EvidenceRelevanceTargetPolicy, text: &str) -> bool {
    target_phrases(policy)
        .iter()
        .any(|phrase| normalized_phrase_matches(text, phrase))
}

fn target_in_navigation(
    policy: &EvidenceRelevanceTargetPolicy,
    candidate: &EvidenceRelevanceCandidate,
) -> bool {
    candidate.signals.iter().any(|signal| {
        signal.kind == EvidenceRelevanceSignalKind::NavigationOrFooter
            && target_in_signal(policy, &signal.text)
    })
}

fn target_in_url(
    policy: &EvidenceRelevanceTargetPolicy,
    candidate: &EvidenceRelevanceCandidate,
) -> bool {
    candidate.signals.iter().any(|signal| {
        signal.kind == EvidenceRelevanceSignalKind::CanonicalUrl
            && target_in_signal(policy, &signal.text)
    })
}

fn target_occurrence_is_context_only(text: &str, phrase: &str) -> Option<bool> {
    let text_tokens = normalized(text)
        .split_whitespace()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let phrase_tokens = normalized(phrase)
        .split_whitespace()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if text_tokens.is_empty() || phrase_tokens.is_empty() || phrase_tokens.len() > text_tokens.len()
    {
        return None;
    }

    let mut saw = false;
    for index in 0..=text_tokens.len() - phrase_tokens.len() {
        if text_tokens[index..index + phrase_tokens.len()] != phrase_tokens {
            continue;
        }
        saw = true;
        let before = &text_tokens[..index];
        let ends_with = |suffix: &[&str]| {
            before.len() >= suffix.len()
                && before[before.len() - suffix.len()..]
                    .iter()
                    .map(String::as_str)
                    .eq(suffix.iter().copied())
        };
        let contextual = ends_with(&["unlike"])
            || ends_with(&["versus"])
            || ends_with(&["vs"])
            || ends_with(&["compared", "to"])
            || ends_with(&["compared", "with"])
            || ends_with(&["rather", "than"])
            || ends_with(&["in", "contrast", "to"])
            || ends_with(&["as", "opposed", "to"]);
        if !contextual {
            return Some(false);
        }
    }
    saw.then_some(true)
}

fn target_mentions_are_context_only(
    policy: &EvidenceRelevanceTargetPolicy,
    candidate: &EvidenceRelevanceCandidate,
) -> bool {
    let phrases = target_phrases(policy);
    let mut saw = false;
    for signal in candidate
        .signals
        .iter()
        .filter(|signal| signal_can_own_identity(signal.kind))
    {
        for phrase in &phrases {
            let Some(context_only) = target_occurrence_is_context_only(&signal.text, phrase) else {
                continue;
            };
            saw = true;
            if !context_only {
                return false;
            }
        }
    }
    saw
}

fn target_has_direct_local_mention(
    policy: &EvidenceRelevanceTargetPolicy,
    candidate: &EvidenceRelevanceCandidate,
) -> bool {
    let phrases = target_phrases(policy);
    candidate
        .signals
        .iter()
        .filter(|signal| signal_can_own_identity(signal.kind))
        .any(|signal| {
            phrases.iter().any(|phrase| {
                target_occurrence_is_context_only(&signal.text, phrase) == Some(false)
            })
        })
}

fn repeated_authorized_target(
    policy: &EvidenceRelevanceTargetPolicy,
    candidate: &EvidenceRelevanceCandidate,
) -> bool {
    let phrases = target_phrases(policy);
    let subject_kind = |kind| {
        matches!(
            kind,
            EvidenceRelevanceSignalKind::SourceTitle
                | EvidenceRelevanceSignalKind::Heading
                | EvidenceRelevanceSignalKind::StructuredMetadata
        )
    };
    let body_kind = |kind| {
        matches!(
            kind,
            EvidenceRelevanceSignalKind::Excerpt
                | EvidenceRelevanceSignalKind::Fact
                | EvidenceRelevanceSignalKind::StructuredMetadata
        )
    };

    phrases.iter().any(|phrase| {
        candidate
            .signals
            .iter()
            .enumerate()
            .any(|(left_index, left)| {
                subject_kind(left.kind)
                    && target_occurrence_is_context_only(&left.text, phrase) == Some(false)
                    && candidate
                        .signals
                        .iter()
                        .enumerate()
                        .any(|(right_index, right)| {
                            right_index != left_index
                                && body_kind(right.kind)
                                && target_occurrence_is_context_only(&right.text, phrase)
                                    == Some(false)
                        })
            })
    })
}

fn noise_token(token: &str) -> bool {
    matches!(
        token,
        "pricing"
            | "price"
            | "cost"
            | "billing"
            | "billed"
            | "availability"
            | "available"
            | "regional"
            | "region"
            | "regions"
            | "coverage"
            | "release"
            | "released"
            | "notes"
            | "launch"
            | "launches"
            | "launched"
            | "new"
            | "update"
            | "updated"
            | "change"
            | "changes"
            | "limit"
            | "limits"
            | "quota"
            | "overview"
            | "guide"
            | "deployment"
            | "service"
            | "services"
            | "product"
            | "products"
            | "documentation"
            | "docs"
            | "supported"
            | "support"
            | "rate"
            | "rates"
            | "monthly"
            | "month"
            | "retention"
            | "policy"
            | "definition"
            | "benefit"
            | "benefits"
    )
}

fn repeated_non_target_subject(
    policy: &EvidenceRelevanceTargetPolicy,
    candidate: &EvidenceRelevanceCandidate,
) -> bool {
    let target_tokens = target_phrases(policy)
        .iter()
        .flat_map(|phrase| phrase.split_whitespace())
        .map(str::to_owned)
        .collect::<Vec<_>>();

    let header_kind = |kind| {
        matches!(
            kind,
            EvidenceRelevanceSignalKind::SourceTitle
                | EvidenceRelevanceSignalKind::Heading
                | EvidenceRelevanceSignalKind::StructuredMetadata
        )
    };
    let body_kind = |kind| {
        matches!(
            kind,
            EvidenceRelevanceSignalKind::Excerpt
                | EvidenceRelevanceSignalKind::Fact
                | EvidenceRelevanceSignalKind::StructuredMetadata
        )
    };

    for (header_index, header) in candidate.signals.iter().enumerate() {
        if !header_kind(header.kind) {
            continue;
        }
        let header_tokens = normalized(&header.text)
            .split_whitespace()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        let max_width = usize::min(4, header_tokens.len());
        for width in 2..=max_width {
            for phrase in header_tokens.windows(width) {
                if phrase.iter().all(|token| noise_token(token)) {
                    continue;
                }
                let phrase_text = phrase.join(" ");
                if target_phrases(policy)
                    .iter()
                    .any(|target| normalized_phrase_matches(&phrase_text, target))
                {
                    continue;
                }
                let has_distinguishing_token = phrase.iter().any(|token| {
                    token.len() >= 4
                        && !noise_token(token)
                        && !target_tokens.iter().any(|target| target == token)
                });
                if !has_distinguishing_token {
                    continue;
                }
                for (body_index, body) in candidate.signals.iter().enumerate() {
                    if body_index == header_index || !body_kind(body.kind) {
                        continue;
                    }
                    let body_tokens = normalized(&body.text)
                        .split_whitespace()
                        .map(str::to_owned)
                        .collect::<Vec<_>>();
                    if body_tokens
                        .windows(width)
                        .any(|candidate_phrase| candidate_phrase == phrase)
                    {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn explicit_separation_present(candidate: &EvidenceRelevanceCandidate) -> bool {
    let text = candidate_local_text(candidate);
    let separation = text.contains("separate") || text.contains("distinct");
    let object = text.contains("service") || text.contains("product");
    separation && object
}

fn explicit_target_absence_present(
    policy: &EvidenceRelevanceTargetPolicy,
    candidate: &EvidenceRelevanceCandidate,
) -> bool {
    let text = candidate_local_text(candidate);
    let phrases = target_phrases(policy);
    let named_absence = phrases.iter().any(|phrase| {
        [
            format!("no {phrase} entry"),
            format!("no {phrase} product"),
            format!("no {phrase} service"),
            format!("no {phrase} information"),
            format!("no information about {phrase}"),
            format!("{phrase} is absent"),
            format!("{phrase} is not listed"),
            format!("{phrase} not listed"),
            format!("without {phrase}"),
        ]
        .iter()
        .any(|pattern| text.contains(pattern))
    });
    let generic_product_absence = text.contains("generic")
        && (text.contains("no product specific")
            || text.contains("no product-specific")
            || text.contains("no specific product"));
    let generic_scope_absence = text.contains("generic")
        && (text.contains("catalog")
            || text.contains("directory")
            || text.contains("site")
            || text.contains("index"));
    let generic_browse_scope = (text.contains("browse") || text.contains("explore"))
        && (text.contains("service") || text.contains("services"))
        && !target_has_direct_local_mention(policy, candidate);
    named_absence || generic_product_absence || generic_scope_absence || generic_browse_scope
}

fn single_signal_near_sibling(
    policy: &EvidenceRelevanceTargetPolicy,
    candidate: &EvidenceRelevanceCandidate,
) -> bool {
    if target_has_direct_local_mention(policy, candidate)
        || target_in_url(policy, candidate)
        || explicit_separation_present(candidate)
    {
        return false;
    }
    let identity_signals = candidate
        .signals
        .iter()
        .filter(|signal| signal_can_own_identity(signal.kind))
        .collect::<Vec<_>>();
    if identity_signals.len() != 1 {
        return false;
    }

    let target_tokens = target_phrases(policy)
        .iter()
        .flat_map(|phrase| phrase.split_whitespace())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let signal_tokens = normalized(&identity_signals[0].text)
        .split_whitespace()
        .map(str::to_owned)
        .collect::<Vec<_>>();

    signal_tokens
        .iter()
        .any(|token| token.len() >= 4 && target_tokens.iter().any(|target| target == token))
        && signal_tokens.iter().any(|token| {
            token.len() >= 4
                && !target_tokens.iter().any(|target| target == token)
                && !noise_token(token)
        })
}

fn relation_kind_locally_present(
    relation: EvidenceRelevanceRelationKind,
    candidate: &EvidenceRelevanceCandidate,
) -> bool {
    let text = candidate_local_text(candidate);
    let contains_any = |needles: &[&str]| needles.iter().any(|needle| text.contains(needle));
    match relation {
        EvidenceRelevanceRelationKind::Availability => {
            contains_any(&["available", "availability", "offered", "regional", "region"])
        }
        EvidenceRelevanceRelationKind::Pricing => contains_any(&[
            "pricing",
            "price",
            "cost",
            "billed",
            "billing",
            "charge",
            "credit",
            "allowance",
        ]),
        EvidenceRelevanceRelationKind::Limit => contains_any(&[
            "limit",
            "quota",
            "maximum",
            "allows",
            "allowed",
            "per minute",
            "per second",
            "up to",
        ]),
        EvidenceRelevanceRelationKind::ChangeOrLaunch => contains_any(&[
            "new",
            "add",
            "launch",
            "release",
            "update",
            "change",
            "introduc",
            "improvement",
        ]),
        EvidenceRelevanceRelationKind::Definition => {
            contains_any(&["definition", "concept", "means", "defined", "category"])
        }
        EvidenceRelevanceRelationKind::BenefitOrUseCase => {
            contains_any(&["use case", "benefit", "reduce", "help", "combine", "useful"])
        }
        EvidenceRelevanceRelationKind::General => true,
    }
}

fn requested_relation_locally_present(
    policy: &EvidenceRelevanceTargetPolicy,
    candidate: &EvidenceRelevanceCandidate,
) -> bool {
    if policy.relation != EvidenceRelevanceRelationKind::General {
        return relation_kind_locally_present(policy.relation, candidate);
    }

    let text = candidate_local_text(candidate);
    let entity_tokens = target_phrases(policy)
        .iter()
        .flat_map(|phrase| phrase.split_whitespace())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let stop = [
        "what", "which", "where", "when", "does", "how", "work", "works", "about", "the", "this",
        "that", "with", "from", "into", "for", "support", "supports",
    ];

    normalized(&policy.target_question)
        .split_whitespace()
        .filter(|token| token.len() >= 4)
        .filter(|token| !stop.contains(token))
        .filter(|token| !entity_tokens.iter().any(|entity| entity == *token))
        .any(|token| text.contains(token))
}

fn different_relation_locally_present(
    policy: &EvidenceRelevanceTargetPolicy,
    candidate: &EvidenceRelevanceCandidate,
) -> bool {
    let all = [
        EvidenceRelevanceRelationKind::Availability,
        EvidenceRelevanceRelationKind::Pricing,
        EvidenceRelevanceRelationKind::Limit,
        EvidenceRelevanceRelationKind::ChangeOrLaunch,
        EvidenceRelevanceRelationKind::Definition,
        EvidenceRelevanceRelationKind::BenefitOrUseCase,
    ];
    !relation_kind_locally_present(policy.relation, candidate)
        && all
            .into_iter()
            .filter(|relation| *relation != policy.relation)
            .any(|relation| relation_kind_locally_present(relation, candidate))
}

fn target_specific_requested_relation_absence(
    policy: &EvidenceRelevanceTargetPolicy,
    candidate: &EvidenceRelevanceCandidate,
) -> bool {
    let text = candidate_local_text(candidate);
    let terms: &[&str] = match policy.relation {
        EvidenceRelevanceRelationKind::Availability => &["availability", "available"],
        EvidenceRelevanceRelationKind::Pricing => &["pricing", "price", "cost", "billing"],
        EvidenceRelevanceRelationKind::Limit => &["limit", "quota"],
        EvidenceRelevanceRelationKind::ChangeOrLaunch => &["change", "launch", "release", "update"],
        EvidenceRelevanceRelationKind::Definition => &["definition"],
        EvidenceRelevanceRelationKind::BenefitOrUseCase => &["benefit", "use case"],
        EvidenceRelevanceRelationKind::General => return false,
    };

    target_phrases(policy).iter().any(|target| {
        terms.iter().any(|term| {
            [
                format!("no {target} {term}"),
                format!("{target} has no {term}"),
                format!("{target} does not list {term}"),
                format!("{target} does not provide {term}"),
                format!("{target} {term} is not listed"),
                format!("{target} {term} not listed"),
            ]
            .iter()
            .any(|pattern| text.contains(pattern))
        })
    })
}

fn requested_relation_explicitly_absent(
    policy: &EvidenceRelevanceTargetPolicy,
    candidate: &EvidenceRelevanceCandidate,
) -> bool {
    let text = candidate_local_text(candidate);
    let terms: &[&str] = match policy.relation {
        EvidenceRelevanceRelationKind::Availability => &["availability", "available"],
        EvidenceRelevanceRelationKind::Pricing => &["pricing", "price", "cost", "billing"],
        EvidenceRelevanceRelationKind::Limit => &["limit", "quota"],
        EvidenceRelevanceRelationKind::ChangeOrLaunch => &["change", "launch", "release", "update"],
        EvidenceRelevanceRelationKind::Definition => &["definition"],
        EvidenceRelevanceRelationKind::BenefitOrUseCase => &["benefit", "use case"],
        EvidenceRelevanceRelationKind::General => return false,
    };

    let target_patterns = target_phrases(policy);
    terms.iter().any(|term| {
        let direct_patterns = [
            format!("no {term}"),
            format!("{term} is not listed"),
            format!("{term} not listed"),
            format!("does not list {term}"),
            format!("does not provide {term}"),
            format!("contains no {term}"),
            format!("no product specific {term}"),
            format!("no product-specific {term}"),
        ];
        direct_patterns.iter().any(|pattern| text.contains(pattern))
            || target_patterns.iter().any(|target| {
                [
                    format!("no {target} {term}"),
                    format!("{target} has no {term}"),
                    format!("{target} does not list {term}"),
                    format!("{target} does not provide {term}"),
                ]
                .iter()
                .any(|pattern| text.contains(pattern))
            })
    })
}

fn requested_relation_explicitly_excluded(
    policy: &EvidenceRelevanceTargetPolicy,
    candidate: &EvidenceRelevanceCandidate,
) -> bool {
    let text = candidate_local_text(candidate);
    let terms: &[&str] = match policy.relation {
        EvidenceRelevanceRelationKind::Availability => &["availability", "available"],
        EvidenceRelevanceRelationKind::Pricing => &["pricing", "price", "cost", "billing"],
        EvidenceRelevanceRelationKind::Limit => &["limit", "quota"],
        EvidenceRelevanceRelationKind::ChangeOrLaunch => &["change", "launch", "release", "update"],
        EvidenceRelevanceRelationKind::Definition => &["definition", "defined"],
        EvidenceRelevanceRelationKind::BenefitOrUseCase => &["benefit", "use case"],
        EvidenceRelevanceRelationKind::General => return false,
    };

    terms.iter().any(|term| {
        [
            format!("rather than {term}"),
            format!("instead of {term}"),
            format!("not about {term}"),
            format!("does not document {term}"),
            format!("does not describe {term}"),
            format!("no {term}"),
        ]
        .iter()
        .any(|pattern| text.contains(pattern))
    })
}

fn deterministic_risk(candidate: &EvidenceRelevanceCandidate) -> EvidenceLocalBlockingReason {
    classify_deterministic_local_scope_risk(candidate)
}

pub fn resolve_evidence_local_authority_v1(
    policy: &EvidenceRelevanceTargetPolicy,
    candidate: &EvidenceRelevanceCandidate,
    proposal: Option<&EvidenceRelevanceBindingProposal>,
    raw: Option<&EvidenceLocalQualificationV6>,
) -> EvidenceLocalAuthorityV1 {
    use EvidenceAuthoritySource as Source;
    use EvidenceLocalBlockingReason as Risk;
    use EvidenceLocalIdentityScope as Identity;
    use EvidenceLocalRelationScope as Relation;
    use EvidenceRelevanceBinding as Binding;

    let risk = deterministic_risk(candidate);
    if risk != Risk::None {
        return EvidenceLocalAuthorityV1 {
            identity_scope: Identity::Unresolved,
            identity_source: Source::Unresolved,
            relation_scope: Relation::Unresolved,
            relation_source: Source::Unresolved,
            scope_risk: risk,
        };
    }

    let direct_target = target_has_direct_local_mention(policy, candidate);
    let repeated_target = repeated_authorized_target(policy, candidate);
    let context_only_target = target_mentions_are_context_only(policy, candidate);
    let navigation_target = target_in_navigation(policy, candidate);
    let url_target = target_in_url(policy, candidate);
    let explicit_absence = explicit_target_absence_present(policy, candidate);
    let explicit_separation = explicit_separation_present(candidate);
    let target_relation_absence = target_specific_requested_relation_absence(policy, candidate);
    let repeated_other = repeated_non_target_subject(policy, candidate);
    let weak_single_sibling = single_signal_near_sibling(policy, candidate);
    let strict_identity =
        policy.identity_requirement == EvidenceRelevanceIdentityRequirement::RequireHarnessAnchor;

    let raw_identity = raw.map(|value| value.identity_scope);
    let proposal_identity = proposal.map(|value| value.target_binding);

    let generic_or_explicit_absence = explicit_absence && !direct_target;
    let absent_proof = explicit_absence
        && (raw_identity == Some(Identity::TargetAbsent)
            || generic_or_explicit_absence
            || proposal_identity != Some(Binding::Exact));
    let decisive_structural_distinct = explicit_separation
        || (context_only_target && repeated_other)
        || (strict_identity && navigation_target && repeated_other)
        || (strict_identity && !direct_target && !url_target && repeated_other);
    let corroborated_structural_distinct = repeated_other
        && (raw_identity == Some(Identity::DistinctTarget)
            || proposal_identity == Some(Binding::Different))
        && (strict_identity || context_only_target || explicit_separation);
    let navigation_consensus_distinct = navigation_target
        && !direct_target
        && raw_identity == Some(Identity::DistinctTarget)
        && proposal_identity == Some(Binding::Different);
    let non_near_single_consensus_distinct = strict_identity
        && !direct_target
        && !url_target
        && !weak_single_sibling
        && raw_identity == Some(Identity::DistinctTarget)
        && proposal_identity == Some(Binding::Different);
    let distinct_proof = !weak_single_sibling
        && (decisive_structural_distinct
            || corroborated_structural_distinct
            || navigation_consensus_distinct
            || non_near_single_consensus_distinct);
    let exact_proof = !distinct_proof
        && (target_relation_absence
            || repeated_target
            || (direct_target
                && !context_only_target
                && !absent_proof
                && raw_identity != Some(Identity::TargetAbsent)
                && raw_identity != Some(Identity::DistinctTarget)));

    let (identity_scope, identity_source) = if absent_proof {
        (Identity::TargetAbsent, Source::HarnessCorroborated)
    } else if exact_proof && distinct_proof {
        (Identity::Unresolved, Source::Unresolved)
    } else if (target_relation_absence || repeated_target) && !distinct_proof {
        (Identity::ExactTarget, Source::HarnessDeterministic)
    } else if exact_proof {
        (Identity::ExactTarget, Source::HarnessCorroborated)
    } else if distinct_proof {
        (Identity::DistinctTarget, Source::HarnessCorroborated)
    } else if policy.identity_requirement
        == EvidenceRelevanceIdentityRequirement::AllowSemanticEquivalent
        && raw_identity == Some(Identity::ExactTarget)
        && proposal_identity != Some(Binding::Different)
    {
        (Identity::ExactTarget, Source::ModelConsensus)
    } else {
        (Identity::Unresolved, Source::Unresolved)
    };

    let raw_relation = raw.map(|value| value.relation_scope);
    let proposal_relation = proposal.map(|value| value.relation_binding);
    let requested_present = requested_relation_locally_present(policy, candidate);
    let requested_absent = requested_relation_explicitly_absent(policy, candidate);
    let requested_excluded = requested_relation_explicitly_excluded(policy, candidate);
    let other_relation_present = different_relation_locally_present(policy, candidate);

    let (relation_scope, relation_source) =
        if requested_absent && raw_relation == Some(Relation::RelationAbsent) {
            (Relation::RelationAbsent, Source::HarnessCorroborated)
        } else if ((requested_excluded || other_relation_present)
            && (raw_relation == Some(Relation::DifferentRelation)
                || proposal_relation == Some(Binding::Different)))
            || (!requested_present
                && raw_relation == Some(Relation::DifferentRelation)
                && proposal_relation != Some(Binding::Exact))
        {
            (Relation::DifferentRelation, Source::HarnessCorroborated)
        } else if requested_present
            && (raw_relation == Some(Relation::RequestedRelation)
                || proposal_relation == Some(Binding::Exact))
        {
            (Relation::RequestedRelation, Source::HarnessCorroborated)
        } else if raw_relation == Some(Relation::DifferentRelation)
            && proposal_relation == Some(Binding::Different)
        {
            (Relation::DifferentRelation, Source::ModelConsensus)
        } else if raw_relation == Some(Relation::RelationAbsent)
            && proposal_relation != Some(Binding::Exact)
            && !requested_present
        {
            (Relation::RelationAbsent, Source::HarnessCorroborated)
        } else if raw_relation == Some(Relation::RequestedRelation)
            && proposal_relation == Some(Binding::Exact)
        {
            (Relation::RequestedRelation, Source::ModelConsensus)
        } else {
            (Relation::Unresolved, Source::Unresolved)
        };

    EvidenceLocalAuthorityV1 {
        identity_scope,
        identity_source,
        relation_scope,
        relation_source,
        scope_risk: risk,
    }
}

pub fn materialize_evidence_relevance_v20(
    policy: &EvidenceRelevanceTargetPolicy,
    candidate: &EvidenceRelevanceCandidate,
    proposal: Option<&EvidenceRelevanceBindingProposal>,
    raw: Option<&EvidenceLocalQualificationV6>,
) -> Result<EvidenceRelevanceAssessment, EvidenceRelevanceError> {
    use EvidenceLocalBlockingReason as Risk;
    use EvidenceLocalIdentityScope as Identity;
    use EvidenceLocalRelationScope as Relation;

    let authority = resolve_evidence_local_authority_v1(policy, candidate, proposal, raw);

    let (disposition, path, reasons) = if authority.scope_risk != Risk::None {
        (
            EvidenceRelevanceDisposition::Ambiguous,
            EvidenceRelevanceAssessmentPath::ConservativeFallback,
            vec![
                EvidenceRelevanceReason::LocalQualificationBlockingCuePresent,
                EvidenceRelevanceReason::ModelAmbiguous,
            ],
        )
    } else {
        match (authority.identity_scope, authority.relation_scope) {
            (Identity::ExactTarget, Relation::RequestedRelation) => (
                EvidenceRelevanceDisposition::Relevant,
                EvidenceRelevanceAssessmentPath::ModelAssisted,
                vec![
                    EvidenceRelevanceReason::PositiveTargetLocalBindingConfirmed,
                    EvidenceRelevanceReason::PositiveCandidateSafeToAccept,
                ],
            ),
            (Identity::ExactTarget, Relation::DifferentRelation | Relation::RelationAbsent) => (
                EvidenceRelevanceDisposition::Irrelevant,
                EvidenceRelevanceAssessmentPath::ModelAssisted,
                vec![
                    EvidenceRelevanceReason::LocalQualificationRejectsRelation,
                    EvidenceRelevanceReason::NegativeCandidateSafeToReject,
                ],
            ),
            (Identity::DistinctTarget, _) => (
                EvidenceRelevanceDisposition::Irrelevant,
                EvidenceRelevanceAssessmentPath::ModelAssisted,
                vec![
                    EvidenceRelevanceReason::NegativeTargetDistinctEntityConfirmed,
                    EvidenceRelevanceReason::NegativeCandidateSafeToReject,
                ],
            ),
            (Identity::TargetAbsent, _) => (
                EvidenceRelevanceDisposition::Irrelevant,
                EvidenceRelevanceAssessmentPath::ModelAssisted,
                vec![
                    EvidenceRelevanceReason::NegativeLocalTargetAbsenceConfirmed,
                    EvidenceRelevanceReason::NegativeCandidateSafeToReject,
                ],
            ),
            _ => (
                EvidenceRelevanceDisposition::Ambiguous,
                EvidenceRelevanceAssessmentPath::ConservativeFallback,
                vec![
                    EvidenceRelevanceReason::LocalQualificationDisagreement,
                    EvidenceRelevanceReason::ModelAmbiguous,
                ],
            ),
        }
    };

    Ok(EvidenceRelevanceAssessment {
        contract_id: EVIDENCE_RELEVANCE_BINDING_PROPOSAL_V5_CONTRACT_ID.into(),
        materialization_policy_id: EVIDENCE_RELEVANCE_BINDING_MATERIALIZATION_POLICY_V20_ID.into(),
        policy_id: policy.policy_id.clone(),
        target_id: policy.target_id.clone(),
        evidence_id: candidate.evidence_id.clone(),
        source_id: candidate.source_id.clone(),
        disposition,
        path,
        reasons,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence_relevance::{
        EvidenceRelevanceAssessmentBudget, EvidenceRelevanceSignal, EvidenceTargetEntityIdentity,
    };

    fn policy() -> EvidenceRelevanceTargetPolicy {
        EvidenceRelevanceTargetPolicy {
            policy_id: "authority-policy".into(),
            target_id: "target".into(),
            target_question: "Where is Atlas Queue available?".into(),
            entity: Some(EvidenceTargetEntityIdentity {
                canonical_id: "atlas.queue".into(),
                canonical_name: "Atlas Queue".into(),
                aliases: vec!["AQX".into()],
            }),
            relation: EvidenceRelevanceRelationKind::Availability,
            identity_requirement: EvidenceRelevanceIdentityRequirement::RequireHarnessAnchor,
            assessment_budget: EvidenceRelevanceAssessmentBudget::default(),
        }
    }

    fn candidate(signals: Vec<(EvidenceRelevanceSignalKind, &str)>) -> EvidenceRelevanceCandidate {
        EvidenceRelevanceCandidate {
            evidence_id: "evidence".into(),
            source_id: "source".into(),
            signals: signals
                .into_iter()
                .map(|(kind, text)| EvidenceRelevanceSignal {
                    kind,
                    text: text.into(),
                })
                .collect(),
        }
    }

    fn proposal(
        target: EvidenceRelevanceBinding,
        relation: EvidenceRelevanceBinding,
    ) -> EvidenceRelevanceBindingProposal {
        EvidenceRelevanceBindingProposal {
            target_binding: target,
            relation_binding: relation,
        }
    }

    fn raw(
        identity: EvidenceLocalIdentityScope,
        relation: EvidenceLocalRelationScope,
        risk: EvidenceLocalBlockingReason,
    ) -> EvidenceLocalQualificationV6 {
        EvidenceLocalQualificationV6 {
            identity_scope: identity,
            relation_scope: relation,
            scope_risk: risk,
        }
    }

    #[test]
    fn repeated_authorized_subject_owns_positive_even_when_identity_votes_disagree() {
        let local = candidate(vec![
            (
                EvidenceRelevanceSignalKind::SourceTitle,
                "AQX regional coverage",
            ),
            (
                EvidenceRelevanceSignalKind::Excerpt,
                "AQX is available in West and East.",
            ),
        ]);
        let authority = resolve_evidence_local_authority_v1(
            &policy(),
            &local,
            Some(&proposal(
                EvidenceRelevanceBinding::Different,
                EvidenceRelevanceBinding::Exact,
            )),
            Some(&raw(
                EvidenceLocalIdentityScope::DistinctTarget,
                EvidenceLocalRelationScope::RequestedRelation,
                EvidenceLocalBlockingReason::None,
            )),
        );
        assert_eq!(
            authority.identity_scope,
            EvidenceLocalIdentityScope::ExactTarget
        );
        assert_eq!(
            materialize_evidence_relevance_v20(
                &policy(),
                &local,
                Some(&proposal(
                    EvidenceRelevanceBinding::Different,
                    EvidenceRelevanceBinding::Exact
                )),
                Some(&raw(
                    EvidenceLocalIdentityScope::DistinctTarget,
                    EvidenceLocalRelationScope::RequestedRelation,
                    EvidenceLocalBlockingReason::None,
                )),
            )
            .unwrap()
            .disposition,
            EvidenceRelevanceDisposition::Relevant
        );
    }

    #[test]
    fn navigation_target_does_not_own_other_repeated_subject() {
        let local = candidate(vec![
            (
                EvidenceRelevanceSignalKind::NavigationOrFooter,
                "Atlas Queue",
            ),
            (EvidenceRelevanceSignalKind::Heading, "Boreal Queue regions"),
            (
                EvidenceRelevanceSignalKind::Excerpt,
                "Boreal Queue is available in North.",
            ),
        ]);
        let authority = resolve_evidence_local_authority_v1(
            &policy(),
            &local,
            Some(&proposal(
                EvidenceRelevanceBinding::Exact,
                EvidenceRelevanceBinding::Exact,
            )),
            Some(&raw(
                EvidenceLocalIdentityScope::ExactTarget,
                EvidenceLocalRelationScope::DifferentRelation,
                EvidenceLocalBlockingReason::None,
            )),
        );
        assert_eq!(
            authority.identity_scope,
            EvidenceLocalIdentityScope::DistinctTarget
        );
        assert_eq!(
            materialize_evidence_relevance_v20(
                &policy(),
                &local,
                Some(&proposal(
                    EvidenceRelevanceBinding::Exact,
                    EvidenceRelevanceBinding::Exact
                )),
                Some(&raw(
                    EvidenceLocalIdentityScope::ExactTarget,
                    EvidenceLocalRelationScope::DifferentRelation,
                    EvidenceLocalBlockingReason::None,
                )),
            )
            .unwrap()
            .disposition,
            EvidenceRelevanceDisposition::Irrelevant
        );
    }

    #[test]
    fn explicit_target_absence_beats_name_occurrence_inside_absence_statement() {
        let local = candidate(vec![
            (
                EvidenceRelevanceSignalKind::SourceTitle,
                "Service directory",
            ),
            (
                EvidenceRelevanceSignalKind::Excerpt,
                "This generic directory has no Atlas Queue entry and no product-specific regional data.",
            ),
        ]);
        let authority = resolve_evidence_local_authority_v1(
            &policy(),
            &local,
            Some(&proposal(
                EvidenceRelevanceBinding::Exact,
                EvidenceRelevanceBinding::Exact,
            )),
            Some(&raw(
                EvidenceLocalIdentityScope::TargetAbsent,
                EvidenceLocalRelationScope::RelationAbsent,
                EvidenceLocalBlockingReason::None,
            )),
        );
        assert_eq!(
            authority.identity_scope,
            EvidenceLocalIdentityScope::TargetAbsent
        );
        assert_eq!(
            materialize_evidence_relevance_v20(
                &policy(),
                &local,
                Some(&proposal(
                    EvidenceRelevanceBinding::Exact,
                    EvidenceRelevanceBinding::Exact
                )),
                Some(&raw(
                    EvidenceLocalIdentityScope::TargetAbsent,
                    EvidenceLocalRelationScope::RelationAbsent,
                    EvidenceLocalBlockingReason::None,
                )),
            )
            .unwrap()
            .disposition,
            EvidenceRelevanceDisposition::Irrelevant
        );
    }

    #[test]
    fn explicit_separate_service_creates_negative_ownership_without_model_consensus() {
        let local = candidate(vec![
            (EvidenceRelevanceSignalKind::Heading, "Boreal Queue regions"),
            (
                EvidenceRelevanceSignalKind::Excerpt,
                "Boreal Queue is a separate service from Atlas Queue and is available in North.",
            ),
        ]);
        let authority = resolve_evidence_local_authority_v1(
            &policy(),
            &local,
            Some(&proposal(
                EvidenceRelevanceBinding::Exact,
                EvidenceRelevanceBinding::Exact,
            )),
            Some(&raw(
                EvidenceLocalIdentityScope::DistinctTarget,
                EvidenceLocalRelationScope::RelationAbsent,
                EvidenceLocalBlockingReason::None,
            )),
        );
        assert_eq!(
            authority.identity_scope,
            EvidenceLocalIdentityScope::DistinctTarget
        );
        assert_eq!(
            materialize_evidence_relevance_v20(
                &policy(),
                &local,
                Some(&proposal(
                    EvidenceRelevanceBinding::Exact,
                    EvidenceRelevanceBinding::Exact
                )),
                Some(&raw(
                    EvidenceLocalIdentityScope::DistinctTarget,
                    EvidenceLocalRelationScope::RelationAbsent,
                    EvidenceLocalBlockingReason::None,
                )),
            )
            .unwrap()
            .disposition,
            EvidenceRelevanceDisposition::Irrelevant
        );
    }

    #[test]
    fn single_signal_near_sibling_remains_unresolved() {
        let local = candidate(vec![(
            EvidenceRelevanceSignalKind::Excerpt,
            "Atlas View is available in North.",
        )]);
        let authority = resolve_evidence_local_authority_v1(
            &policy(),
            &local,
            Some(&proposal(
                EvidenceRelevanceBinding::Different,
                EvidenceRelevanceBinding::Exact,
            )),
            Some(&raw(
                EvidenceLocalIdentityScope::DistinctTarget,
                EvidenceLocalRelationScope::RequestedRelation,
                EvidenceLocalBlockingReason::None,
            )),
        );
        assert_eq!(
            authority.identity_scope,
            EvidenceLocalIdentityScope::Unresolved
        );
        assert_eq!(
            materialize_evidence_relevance_v20(
                &policy(),
                &local,
                Some(&proposal(
                    EvidenceRelevanceBinding::Different,
                    EvidenceRelevanceBinding::Exact
                )),
                Some(&raw(
                    EvidenceLocalIdentityScope::DistinctTarget,
                    EvidenceLocalRelationScope::RequestedRelation,
                    EvidenceLocalBlockingReason::None,
                )),
            )
            .unwrap()
            .disposition,
            EvidenceRelevanceDisposition::Ambiguous
        );
    }

    #[test]
    fn blocking_risk_has_absolute_precedence() {
        let local = candidate(vec![
            (EvidenceRelevanceSignalKind::Heading, "Boreal Queue regions"),
            (
                EvidenceRelevanceSignalKind::Excerpt,
                "Boreal Queue is available in North. The document does not establish whether Boreal Queue is an alias for Atlas Queue.",
            ),
        ]);
        let authority = resolve_evidence_local_authority_v1(
            &policy(),
            &local,
            Some(&proposal(
                EvidenceRelevanceBinding::Different,
                EvidenceRelevanceBinding::Exact,
            )),
            Some(&raw(
                EvidenceLocalIdentityScope::DistinctTarget,
                EvidenceLocalRelationScope::RequestedRelation,
                EvidenceLocalBlockingReason::IdentityMapping,
            )),
        );
        assert_eq!(
            authority.identity_scope,
            EvidenceLocalIdentityScope::Unresolved
        );
        assert_eq!(
            authority.scope_risk,
            EvidenceLocalBlockingReason::IdentityMapping
        );
    }
}
