//! Host-owned, hash-bound source context. This never authorizes a truth claim.
use std::collections::{BTreeMap, BTreeSet};

use reasoning_harness_core::{
    EpistemicState, Evidence, FinalAnswerClaim, FinalClaimMode, FinalizationResult,
    FinalizationStatus, FindingStrength, Proposition, ReasoningArtifact, VerificationConclusion,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize, de};
use sha2::{Digest, Sha256};

pub(super) const CONFIG_ID: &str = "reason-answer-context-config-v1";
pub(super) const POLICY_ID: &str = "harness-bound-source-context-v1";
pub(super) const OUTPUT_ID: &str = "reason-natural-output-v5";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct Config {
    pub schema_version: String,
    #[serde(default)]
    pub sources: Vec<Source>,
    /// Host-supplied input keys, not text supplied by a model or scoring rubric.
    #[serde(default)]
    pub unresolved_inputs: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct Source {
    pub proposition: Proposition,
    pub evidence_id: String,
    pub source_id: String,
    /// SHA256 of compact serde_json::to_vec(Evidence), including admitted metadata.
    pub evidence_sha256: String,
    /// RFC6901 pointers into the unchanged observation; entire strings are exposed.
    pub quote_pointer: String,
    pub url_pointer: String,
    pub version_pointer: String,
    pub url: String,
    pub source_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Citation {
    pub proposition: Proposition,
    pub evidence_id: String,
    pub source_id: String,
    pub evidence_sha256: String,
    pub url: String,
    pub source_version: String,
    pub quote: String,
    pub quote_sha256: String,
    pub quote_display_encoding: String,
    /// UTF8 byte spans in finalization.text, not offsets in a normalized document.
    pub fact_start_byte: usize,
    pub fact_end_byte: usize,
    pub quote_start_byte: usize,
    pub quote_end_byte: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct View {
    pub policy_id: String,
    /// Coverage belongs to these hard claims only, never to the supplementary quotes.
    pub hard_finalization: FinalizationResult,
    pub selected_claims: Vec<FinalAnswerClaim>,
    pub citations: Vec<Citation>,
    pub unresolved_inputs: BTreeSet<String>,
    pub input_reasons: BTreeMap<String, BTreeSet<String>>,
}

fn clean(text: &str) -> bool {
    !text.trim().is_empty()
        && !text.chars().any(|c| {
            (c.is_control() && c != '\n' && c != '\t')
                || matches!(c, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
        })
}

fn pointer_ok(pointer: &str) -> bool {
    if !pointer.starts_with('/') || !clean(pointer) {
        return false;
    }
    let mut chars = pointer.chars();
    while let Some(c) = chars.next() {
        if c == '~' && !matches!(chars.next(), Some('0' | '1')) {
            return false;
        }
    }
    true
}

impl Config {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != CONFIG_ID
            || self.sources.len() > 32
            || self.unresolved_inputs.len() > 32
            || self.sources.is_empty() && self.unresolved_inputs.is_empty()
        {
            return Err("invalid answer_context contract or limits".into());
        }
        let mut seen = BTreeSet::new();
        for source in &self.sources {
            let url =
                url::Url::parse(&source.url).map_err(|_| "invalid answer_context source URL")?;
            if !matches!(url.scheme(), "https" | "http")
                || url.host_str().is_none()
                || !url.username().is_empty()
                || url.password().is_some()
                || !clean(&source.url)
                || source.url.chars().any(char::is_whitespace)
                || !clean(&source.source_version)
                || !clean(&source.source_id)
                || !clean(&source.evidence_id)
                || !clean(&source.proposition.key)
                || !clean(&source.proposition.value)
                || ![
                    &source.quote_pointer,
                    &source.url_pointer,
                    &source.version_pointer,
                ]
                .into_iter()
                .all(|p| pointer_ok(p))
                || source.evidence_sha256.len() != 64
                || !source
                    .evidence_sha256
                    .bytes()
                    .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
                || !seen.insert((
                    source.proposition.key.clone(),
                    source.proposition.value.clone(),
                    source.evidence_id.clone(),
                ))
            {
                return Err("invalid or duplicate answer_context binding".into());
            }
        }
        if self
            .unresolved_inputs
            .iter()
            .any(|key| !clean(key) || key.len() > 256)
        {
            return Err("invalid answer_context unresolved input key".into());
        }
        Ok(())
    }
}

pub(super) fn evidence_hash(evidence: &Evidence) -> String {
    hash(&serde_json::to_vec(evidence).expect("Evidence serializes"))
}

fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn structured_verifier(name: &str) -> bool {
    matches!(
        name,
        "structured_fact_equality" | "qualified_structured_fact_equality"
    )
}

// serde_json::Value normally accepts duplicate object keys. Bind every pointer to
// one unambiguous interpretation instead, recursively (including nested objects).
struct UniqueJson(serde_json::Value);
impl<'de> Deserialize<'de> for UniqueJson {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> de::Visitor<'de> for Visitor {
            type Value = UniqueJson;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("JSON with unique object keys")
            }
            fn visit_bool<E: de::Error>(self, v: bool) -> Result<Self::Value, E> {
                Ok(UniqueJson(v.into()))
            }
            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
                Ok(UniqueJson(v.into()))
            }
            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
                Ok(UniqueJson(v.into()))
            }
            fn visit_f64<E: de::Error>(self, v: f64) -> Result<Self::Value, E> {
                serde_json::Number::from_f64(v)
                    .map(|v| UniqueJson(v.into()))
                    .ok_or_else(|| E::custom("non-finite JSON number"))
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                Ok(UniqueJson(v.into()))
            }
            fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
                Ok(UniqueJson(v.into()))
            }
            fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
                Ok(UniqueJson(serde_json::Value::Null))
            }
            fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut values = Vec::new();
                while let Some(UniqueJson(value)) = seq.next_element()? {
                    values.push(value);
                }
                Ok(UniqueJson(values.into()))
            }
            fn visit_map<A: de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut values = serde_json::Map::new();
                while let Some(key) = map.next_key::<String>()? {
                    if values.contains_key(&key) {
                        return Err(de::Error::custom("duplicate JSON key"));
                    }
                    let UniqueJson(value) = map.next_value()?;
                    values.insert(key, value);
                }
                Ok(UniqueJson(values.into()))
            }
        }
        deserializer.deserialize_any(Visitor)
    }
}

fn source_quote(source: &Source, artifact: &ReasoningArtifact) -> Result<String, String> {
    let matches = artifact
        .evidence
        .iter()
        .filter(|e| e.id == source.evidence_id)
        .collect::<Vec<_>>();
    let [evidence] = matches.as_slice() else {
        return Err("answer_context evidence missing or ambiguous".into());
    };
    if evidence.source != source.source_id
        || evidence_hash(evidence) != source.evidence_sha256
        || evidence.facts.get(&source.proposition.key) != Some(&source.proposition.value)
        || evidence
            .metadata
            .provenance_class
            .as_deref()
            .is_none_or(|p| p == "untrusted_context" || p == "untrusted")
        || evidence.metadata.scope.is_none()
        || evidence.metadata.temporal.is_none()
    {
        return Err("answer_context evidence identity, fact or admitted metadata mismatch".into());
    }
    let data: UniqueJson = serde_json::from_str(&evidence.observation)
        .map_err(|_| "answer_context observation must be unambiguous JSON")?;
    if data.0.pointer(&source.url_pointer).and_then(|v| v.as_str()) != Some(&source.url)
        || data
            .0
            .pointer(&source.version_pointer)
            .and_then(|v| v.as_str())
            != Some(&source.source_version)
    {
        return Err("answer_context source URL/version mismatch".into());
    }
    let quote = data
        .0
        .pointer(&source.quote_pointer)
        .and_then(|v| v.as_str())
        .ok_or("answer_context quote pointer must select a complete string")?;
    if !clean(quote) || quote.len() > 32768 {
        return Err("answer_context quote is unsafe or exceeds its bound; no truncation".into());
    }
    let matching_claims = artifact
        .claims
        .iter()
        .filter(|c| c.proposition.as_ref() == Some(&source.proposition))
        .collect::<Vec<_>>();
    let safe = !matching_claims.is_empty()
        && matching_claims
            .iter()
            .all(|c| matches!(c.state, EpistemicState::Known | EpistemicState::Supported));
    let verifier = reasoning_harness_core::structured_fact_verifier_for_input(
        &super::input_from_artifact(artifact),
    );
    let receipt_bound = matching_claims.iter().any(|claim| {
        let current = verifier.verify(claim, &artifact.evidence);
        if !current.as_ref().is_some_and(|receipt| {
            structured_verifier(&receipt.verifier)
                && receipt.conclusion == VerificationConclusion::Supported
                && receipt.evidence_ids.contains(&source.evidence_id)
        }) {
            return false;
        }
        artifact.verification_receipts.iter().any(|r| {
            r.conclusion == VerificationConclusion::Supported
                && structured_verifier(&r.verifier)
                && r.proposition.as_ref() == Some(&source.proposition)
                && r.claim_id.as_deref() == Some(&claim.id)
                && r.claim_statement
                    .as_ref()
                    .is_none_or(|s| s == &claim.statement)
                && r.evidence_ids.contains(&source.evidence_id)
                && claim.evidence_ids.contains(&source.evidence_id)
                && !r.evidence_ids.is_empty()
                && r.evidence_ids
                    .iter()
                    .all(|id| artifact.evidence.iter().filter(|e| &e.id == id).count() == 1)
        })
    });
    let unsafe_key = artifact.claims.iter().any(|c| {
        c.proposition.as_ref().is_some_and(|p| {
            p.key == source.proposition.key
                && (p.value != source.proposition.value || c.state == EpistemicState::Contradicted)
        })
    }) || artifact.verification_receipts.iter().any(|r| {
        r.proposition
            .as_ref()
            .is_some_and(|p| p.key == source.proposition.key)
            && r.conclusion == VerificationConclusion::Contradicted
    }) || artifact
        .evidence_qualification_findings
        .iter()
        .any(|f| f.proposition.key == source.proposition.key)
        || artifact.adversarial_findings.iter().any(|f| {
            f.proposition.key == source.proposition.key && f.strength == FindingStrength::Hard
        });
    if !safe || !receipt_bound || unsafe_key {
        return Err("answer_context source has no exact current authorized claim/receipt".into());
    }
    Ok(quote.to_owned())
}

fn grounded_target_authorized(artifact: &ReasoningArtifact, target: &Proposition) -> bool {
    let matching = artifact
        .claims
        .iter()
        .filter(|c| c.proposition.as_ref() == Some(target))
        .collect::<Vec<_>>();
    !matching.is_empty()
        && matching
            .iter()
            .all(|c| matches!(c.state, EpistemicState::Known | EpistemicState::Supported))
        && !artifact.claims.iter().any(|c| {
            c.proposition.as_ref().is_some_and(|p| {
                p.key == target.key
                    && (p.value != target.value || c.state == EpistemicState::Contradicted)
            })
        })
        && !artifact.verification_receipts.iter().any(|r| {
            r.proposition.as_ref().is_some_and(|p| p.key == target.key)
                && r.conclusion == VerificationConclusion::Contradicted
        })
        && !artifact
            .evidence_qualification_findings
            .iter()
            .any(|f| f.proposition.key == target.key)
        && !artifact
            .adversarial_findings
            .iter()
            .any(|f| f.proposition.key == target.key && f.strength == FindingStrength::Hard)
}

/// A saved View is display data, not authority to release an answer. Reconstruct
/// its hard answer using the checkpoint's real verdict, then recheck the pure
/// portion of the current safety gate. Model-dependent safety needs live
/// reevaluation; a serialized claim that it passed is insufficient.
pub(super) fn validate_restored_hard(
    artifact: &ReasoningArtifact,
    verdict: reasoning_harness_core::Verdict,
    profile: reasoning_harness_core::AnswerSafetyProfile,
    selected: &[FinalAnswerClaim],
    hard: &FinalizationResult,
) -> Result<(), String> {
    use reasoning_harness_core::*;
    if matches!(
        hard.status,
        FinalizationStatus::GroundedAnswer | FinalizationStatus::QualifiedPartialAnswer
    ) {
        let candidate = FinalAnswerCandidate {
            text: String::new(),
            factual_claims: selected.to_vec(),
        };
        let baseline = finalize_answer(artifact, verdict, candidate, FinalizationPolicy::default());
        let targets = selected
            .iter()
            .map(|c| c.proposition.clone())
            .collect::<Vec<_>>();
        let partial = canonical_verified_target_partial_answer(artifact, verdict, &targets)
            .or_else(|| {
                canonical_verified_target_reject_partial_answer(artifact, verdict, &targets)
            });
        let matches_recovery = partial.as_ref().is_some_and(|(candidate, result)| {
            candidate.factual_claims.as_slice() == selected && result == hard
        });
        if hard != &baseline && !matches_recovery {
            return Err("saved source context is not authorized by the checkpoint verdict".into());
        }
        for claim in selected
            .iter()
            .filter(|c| c.mode == FinalClaimMode::Grounded)
        {
            if profile == AnswerSafetyProfile::Baseline {
                continue;
            }
            let target = &claim.proposition;
            let request = SoftJudgeRequest {
                id: format!("answer-d3:{}={}", target.key, target.value),
                task: artifact.task.clone(),
                kind: SemanticDiagnosticKind::UnsupportedPremise,
                target: SemanticDiagnosticTarget::Proposition {
                    proposition: target.clone(),
                },
                context: vec![],
            };
            let decidability =
                assess_semantic_decidability(&request, artifact).map_err(|e| e.to_string())?;
            if decidability.disposition == SemanticDecidabilityDisposition::ForceAbstain {
                return Err("saved source context is blocked by current semantic safety".into());
            }
            // This is deliberately narrower than live execution: only the
            // VerifiedTargetV1 trusted short circuit can be replayed offline.
            let matching = artifact
                .claims
                .iter()
                .filter(|c| c.proposition.as_ref() == Some(target))
                .collect::<Vec<_>>();
            if profile != AnswerSafetyProfile::VerifiedTargetV1
                || !grounded_target_authorized(artifact, target)
                || matching.iter().any(|c| {
                    c.state != EpistemicState::Supported
                        || !artifact.verification_receipts.iter().any(|r| {
                            r.claim_id.as_deref() == Some(c.id.as_str())
                                && r.proposition.as_ref() == Some(target)
                                && r.conclusion == VerificationConclusion::Supported
                        })
                })
            {
                return Err("source context safety requires live reevaluation; cannot restore a model-dependent approval".into());
            }
        }
    } else if hard.text.is_some() {
        return Err("blocked saved source context contains exposed text".into());
    }
    Ok(())
}

pub(super) fn restore(
    config: &Config,
    artifact: &ReasoningArtifact,
    verdict: reasoning_harness_core::Verdict,
    profile: reasoning_harness_core::AnswerSafetyProfile,
    saved: &View,
) -> Result<(FinalizationResult, View), String> {
    validate_restored_hard(
        artifact,
        verdict,
        profile,
        &saved.selected_claims,
        &saved.hard_finalization,
    )?;
    compose(
        config,
        artifact,
        &saved.selected_claims,
        saved.hard_finalization.clone(),
    )
}

pub(super) fn compose(
    config: &Config,
    artifact: &ReasoningArtifact,
    selected: &[FinalAnswerClaim],
    hard: FinalizationResult,
) -> Result<(FinalizationResult, View), String> {
    config.validate()?;
    let mut view = View {
        policy_id: POLICY_ID.into(),
        hard_finalization: hard.clone(),
        selected_claims: selected.to_vec(),
        citations: vec![],
        unresolved_inputs: BTreeSet::new(),
        input_reasons: BTreeMap::new(),
    };
    if !matches!(
        hard.status,
        FinalizationStatus::GroundedAnswer | FinalizationStatus::QualifiedPartialAnswer
    ) || hard.text.is_none()
    {
        return Ok((hard, view));
    }
    let mut text = String::new();
    let mut total_quote_bytes = 0;
    if hard.factual_claims != selected.len()
        || selected.iter().any(|selected| {
            if selected.mode == FinalClaimMode::Grounded
                && !grounded_target_authorized(artifact, &selected.proposition)
            {
                return true;
            }
            !artifact.claims.iter().any(|c| {
                c.proposition.as_ref() == Some(&selected.proposition)
                    && match selected.mode {
                        FinalClaimMode::Grounded => {
                            matches!(c.state, EpistemicState::Known | EpistemicState::Supported)
                        }
                        FinalClaimMode::Uncertain => c.state != EpistemicState::Contradicted,
                    }
            })
        })
    {
        return Err("answer_context selected claims do not match the hard finalization".into());
    }
    for claim in selected {
        if !clean(&claim.proposition.key) || !clean(&claim.proposition.value) {
            return Err("unsafe answer_context proposition".into());
        }
        if !text.is_empty() {
            text.push('\n');
        }
        let fact_start = text.len();
        if claim.mode == FinalClaimMode::Uncertain {
            text.push_str("uncertain(");
        }
        text.push_str(&format!(
            "{} = {}",
            serde_json::to_string(&claim.proposition.key).unwrap(),
            serde_json::to_string(&claim.proposition.value).unwrap()
        ));
        if claim.mode == FinalClaimMode::Uncertain {
            text.push(')');
        }
        let fact_end = text.len();
        for source in config.sources.iter().filter(|s| {
            s.proposition == claim.proposition && claim.mode == FinalClaimMode::Grounded
        }) {
            let quote = source_quote(source, artifact)?;
            total_quote_bytes += quote.len();
            if total_quote_bytes > 65536 {
                return Err("answer_context total quote limit exceeded; no truncation".into());
            }
            text.push_str(&format!("\nSource {} (version {}): {}\nSource quotation (including its conditions; source-local; JSON string):\n", serde_json::to_string(&source.source_id).unwrap(), serde_json::to_string(&source.source_version).unwrap(), serde_json::to_string(&source.url).unwrap()));
            let quote_start = text.len();
            text.push_str(&serde_json::to_string(&quote).unwrap());
            let quote_end = text.len();
            view.citations.push(Citation {
                proposition: source.proposition.clone(),
                evidence_id: source.evidence_id.clone(),
                source_id: source.source_id.clone(),
                evidence_sha256: source.evidence_sha256.clone(),
                url: source.url.clone(),
                source_version: source.source_version.clone(),
                quote_sha256: hash(quote.as_bytes()),
                quote_display_encoding: "json_string".into(),
                quote,
                fact_start_byte: fact_start,
                fact_end_byte: fact_end,
                quote_start_byte: quote_start,
                quote_end_byte: quote_end,
            });
        }
    }
    for key in &config.unresolved_inputs {
        // Unknown input is a limitation of this run, never an assertion of falsity.
        if artifact.claims.iter().any(|c| {
            c.proposition.as_ref().is_some_and(|p| &p.key == key)
                && matches!(c.state, EpistemicState::Known | EpistemicState::Supported)
        }) {
            return Err("answer_context unresolved input is already established".into());
        }
        view.unresolved_inputs.insert(key.clone());
        let mut reasons = BTreeSet::new();
        for finding in artifact
            .evidence_qualification_findings
            .iter()
            .filter(|f| &f.proposition.key == key)
        {
            reasons.insert(format!(
                "evidence_qualification:{}",
                serde_json::to_string(&finding.reason).unwrap()
            ));
        }
        for claim in artifact
            .claims
            .iter()
            .filter(|c| c.proposition.as_ref().is_some_and(|p| &p.key == key))
        {
            reasons.insert(format!(
                "unestablished_claim:{}",
                serde_json::to_string(&claim.state).unwrap()
            ));
        }
        if artifact.evidence.iter().any(|e| e.facts.contains_key(key)) {
            reasons.insert("provided_evidence_is_not_verified_for_this_input".into());
        }
        if reasons.is_empty() {
            reasons.insert("not_established_from_this_run".into());
        }
        view.input_reasons.insert(key.clone(), reasons);
    }
    if !view.unresolved_inputs.is_empty() {
        text.push_str("\nInputs not established in this run (no claim about their actual value):");
        for key in &view.unresolved_inputs {
            text.push_str(&format!(
                "\n- {} ({})",
                serde_json::to_string(key).unwrap(),
                view.input_reasons[key]
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }
    if view.citations.is_empty() && view.unresolved_inputs.is_empty() {
        return Ok((hard, view));
    }
    if selected.is_empty() {
        return Err("answer_context cannot replace an empty hard answer".into());
    }
    if hard.status == FinalizationStatus::QualifiedPartialAnswer {
        text.push_str("\nThis is a partial answer; other unresolved claims are not established.");
    }
    let mut result = hard;
    result.text = Some(text);
    result.status = FinalizationStatus::QualifiedPartialAnswer;
    Ok((result, view))
}

#[cfg(test)]
pub(super) fn test_fixture() -> (
    Config,
    ReasoningArtifact,
    Vec<FinalAnswerClaim>,
    FinalizationResult,
) {
    use reasoning_harness_core::*;
    let proposition = Proposition {
        key: "feature.rule".into(),
        value: "limited".into(),
    };
    let scope = BTreeMap::from([(
        "platform".into(),
        ScopeCoverage::Values {
            values: BTreeSet::from(["demo".into()]),
        },
    )]);
    let evidence = Evidence {
        id: "admitted-e1".into(), source: "manual".into(),
        observation: serde_json::json!({"quote":"条件Xの場合だけ利用可能です。\nOther conditions do not apply.", "url":"https://example.test/manual", "version":"edition-1"}).to_string(),
        facts: BTreeMap::from([(proposition.key.clone(), proposition.value.clone())]),
        metadata: EvidenceMetadata { temporal: Some(TemporalValidity { effective_from_unix_seconds: Some(100), effective_until_unix_seconds: Some(300) }), scope: Some(scope.clone()), provenance_class: Some("primary".into()) },
    };
    let input = HarnessInput {
        task: "Explain the scoped rule".into(),
        evidence: vec![evidence.clone()],
        hypotheses: vec![proposition.clone()],
        evidence_requirements: vec![EvidenceRequirement {
            proposition: proposition.clone(),
            as_of_unix_seconds: Some(200),
            scope: Some(scope),
            minimum_authority_class: Some("primary".into()),
        }],
        authority_policy: EvidenceAuthorityPolicy {
            ranks: BTreeMap::from([("primary".into(), 20)]),
        },
        ..Default::default()
    };
    let candidate = ReasoningCandidate {
        claims: vec![CandidateClaim {
            id: "claim-1".into(),
            statement: "UNTRUSTED MODEL PROSE".into(),
            proposed_state: EpistemicState::Known,
            proposition: Some(proposition.clone()),
            evidence_ids: vec![evidence.id.clone()],
        }],
        ..Default::default()
    };
    let outcome = super::run_standard_grounding(input, candidate).unwrap();
    assert_eq!(outcome.verdict, Verdict::Accept);
    let selected = vec![FinalAnswerClaim {
        proposition: proposition.clone(),
        mode: FinalClaimMode::Grounded,
    }];
    let hard = finalize_answer(
        &outcome.artifact,
        outcome.verdict,
        FinalAnswerCandidate {
            text: "MODEL CLAIMS ALL CONDITIONS ARE SATISFIED".into(),
            factual_claims: selected.clone(),
        },
        FinalizationPolicy::default(),
    );
    let config = Config {
        schema_version: CONFIG_ID.into(),
        sources: vec![Source {
            proposition,
            evidence_id: evidence.id.clone(),
            source_id: evidence.source.clone(),
            evidence_sha256: evidence_hash(&evidence),
            quote_pointer: "/quote".into(),
            url_pointer: "/url".into(),
            version_pointer: "/version".into(),
            url: "https://example.test/manual".into(),
            source_version: "edition-1".into(),
        }],
        unresolved_inputs: BTreeSet::from(["project.setting".into()]),
    };
    (config, outcome.artifact, selected, hard)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saved_self_consistent_accept_cannot_override_actual_reject_or_unknown() {
        use reasoning_harness_core::{AnswerSafetyProfile, Verdict};
        let (config, artifact, selected, hard) = test_fixture();
        let (_, forged) = compose(&config, &artifact, &selected, hard).unwrap();
        assert!(
            restore(
                &config,
                &artifact,
                Verdict::Accept,
                AnswerSafetyProfile::VerifiedTargetV1,
                &forged
            )
            .is_ok()
        );
        for actual in [Verdict::Reject, Verdict::Unknown] {
            let error = restore(
                &config,
                &artifact,
                actual,
                AnswerSafetyProfile::VerifiedTargetV1,
                &forged,
            )
            .unwrap_err();
            assert!(error.contains("checkpoint verdict"));
        }
    }

    #[test]
    fn saved_supported_receipt_cannot_override_current_semantic_safety() {
        use reasoning_harness_core::{AnswerSafetyProfile, Verdict};
        let (config, mut artifact, selected, hard) = test_fixture();
        let (_, saved) = compose(&config, &artifact, &selected, hard).unwrap();
        artifact.evidence_requirements[0].as_of_unix_seconds = Some(400);
        let error = restore(
            &config,
            &artifact,
            Verdict::Accept,
            AnswerSafetyProfile::VerifiedTargetV1,
            &saved,
        )
        .unwrap_err();
        assert!(error.contains("current semantic safety"));
    }

    #[test]
    fn saved_model_dependent_approval_cannot_authorize_offline_restore() {
        use reasoning_harness_core::{AnswerSafetyProfile, Verdict};
        let (config, mut artifact, selected, hard) = test_fixture();
        let (_, saved) = compose(&config, &artifact, &selected, hard).unwrap();
        artifact.claims[0].state = EpistemicState::Known;
        let error = restore(
            &config,
            &artifact,
            Verdict::Accept,
            AnswerSafetyProfile::VerifiedTargetV1,
            &saved,
        )
        .unwrap_err();
        assert!(error.contains("requires live reevaluation"));
        let error = restore(
            &config,
            &artifact,
            Verdict::Accept,
            AnswerSafetyProfile::D3SufficiencyV2,
            &saved,
        )
        .unwrap_err();
        assert!(error.contains("requires live reevaluation"));
    }

    #[test]
    fn complete_conditions_citations_and_unknown_input_survive_without_model_prose() {
        let (config, artifact, selected, hard) = test_fixture();
        assert!(!hard.text.as_ref().unwrap().contains("条件"));
        let original = artifact.clone();
        let (answer, view) = compose(&config, &artifact, &selected, hard.clone()).unwrap();
        assert_eq!(artifact, original);
        assert_eq!(view.hard_finalization, hard);
        assert_eq!(answer.status, FinalizationStatus::QualifiedPartialAnswer);
        assert_eq!(answer.factual_claims, 1); // never counts the entire quote as verified truth
        let text = answer.text.unwrap();
        assert!(text.contains("条件Xの場合だけ"));
        assert!(text.contains("project.setting"));
        assert!(!text.contains("UNTRUSTED MODEL") && !text.contains("ALL CONDITIONS"));
        let citation = &view.citations[0];
        assert_eq!(
            serde_json::from_str::<String>(
                &text[citation.quote_start_byte..citation.quote_end_byte]
            )
            .unwrap(),
            citation.quote
        );
        assert_eq!(
            &text[citation.fact_start_byte..citation.fact_end_byte],
            "\"feature.rule\" = \"limited\""
        );
        assert_eq!(
            serde_json::from_str::<View>(&serde_json::to_string(&view).unwrap()).unwrap(),
            view
        );
    }

    #[test]
    fn evidence_changes_including_scope_time_and_version_fail_closed() {
        let (config, original, selected, hard) = test_fixture();
        for change in 0..6 {
            let mut artifact = original.clone();
            match change {
                0 => artifact.evidence[0].source = "other".into(),
                1 => artifact.evidence[0].metadata.scope = Some(BTreeMap::new()),
                2 => {
                    artifact.evidence[0]
                        .metadata
                        .temporal
                        .as_mut()
                        .unwrap()
                        .effective_until_unix_seconds = Some(150)
                }
                3 => artifact.evidence[0]
                    .facts
                    .insert("feature.rule".into(), "different".into())
                    .map(|_| ())
                    .unwrap(),
                4 => artifact.evidence[0].observation.push(' '),
                _ => artifact.evidence.clear(),
            }
            assert!(
                compose(&config, &artifact, &selected, hard.clone()).is_err(),
                "change={change}"
            );
        }
    }

    #[test]
    fn exact_receipt_and_claim_authority_are_required() {
        let (config, original, selected, hard) = test_fixture();
        for change in 0..6 {
            let mut artifact = original.clone();
            match change {
                0 => artifact.verification_receipts.clear(),
                1 => artifact.verification_receipts[0].claim_id = Some("another-claim".into()),
                2 => {
                    artifact.verification_receipts[0].conclusion =
                        VerificationConclusion::Contradicted
                }
                3 => artifact.claims[0].state = EpistemicState::Assumed,
                4 => artifact.claims[0].evidence_ids.clear(),
                _ => artifact.verification_receipts[0]
                    .evidence_ids
                    .push("missing".into()),
            }
            assert!(
                compose(&config, &artifact, &selected, hard.clone()).is_err(),
                "change={change}"
            );
        }
    }

    #[test]
    fn changed_current_qualification_cannot_reuse_old_supported_receipt() {
        let (config, mut artifact, selected, hard) = test_fixture();
        artifact.evidence_requirements[0].as_of_unix_seconds = Some(400);
        assert!(compose(&config, &artifact, &selected, hard).is_err());
    }

    #[test]
    fn acquisition_admission_and_structured_receipt_are_supported_without_qualification_pass() {
        let (config, mut artifact, selected, hard) = test_fixture();
        // The ordinary investigation lane admits scope/time before creating the
        // structured receipt, so it need not repeat requirements in the artifact.
        artifact.evidence_requirements.clear();
        artifact.authority_policy.ranks.clear();
        artifact.verification_receipts[0].verifier = "structured_fact_equality".into();
        assert!(compose(&config, &artifact, &selected, hard).is_ok());
    }

    #[test]
    fn blocked_finalization_cannot_expose_source_context_or_change_status() {
        let (config, artifact, selected, mut hard) = test_fixture();
        for status in [
            FinalizationStatus::Abstain,
            FinalizationStatus::RequiresVerification,
            FinalizationStatus::Unresolved,
        ] {
            hard.status = status;
            hard.text = None;
            let (answer, view) = compose(&config, &artifact, &selected, hard.clone()).unwrap();
            assert_eq!(answer, hard);
            assert!(view.citations.is_empty() && view.unresolved_inputs.is_empty());
        }
    }

    #[test]
    fn ambiguous_json_invalid_pointer_and_bad_url_are_refused() {
        let (config, mut artifact, selected, hard) = test_fixture();
        let mut ambiguous = config.clone();
        artifact.evidence[0].observation = r#"{"quote":"a","quote":"b","url":"https://example.test/manual","version":"edition-1"}"#.into();
        ambiguous.sources[0].evidence_sha256 = evidence_hash(&artifact.evidence[0]);
        assert!(compose(&ambiguous, &artifact, &selected, hard.clone()).is_err());
        let mut wrong = config.clone();
        wrong.sources[0].quote_pointer = "/~2quote".into();
        assert!(wrong.validate().is_err());
        wrong = config.clone();
        wrong.sources[0].url = "javascript:alert(1)".into();
        assert!(wrong.validate().is_err());
        wrong = config.clone();
        wrong.sources[0].source_version = "edition-2".into();
        let (_, artifact, _, _) = test_fixture();
        assert!(compose(&wrong, &artifact, &selected, hard).is_err());
    }

    #[test]
    fn no_silent_truncation_or_terminal_control_in_quotes() {
        let (mut config, mut artifact, selected, hard) = test_fixture();
        for quote in [
            "x".repeat(32769),
            "escaped \u{1b}[2J output".into(),
            "spoof \u{202e} version".into(),
        ] {
            let mut json: serde_json::Value =
                serde_json::from_str(&artifact.evidence[0].observation).unwrap();
            json["quote"] = quote.into();
            artifact.evidence[0].observation = json.to_string();
            config.sources[0].evidence_sha256 = evidence_hash(&artifact.evidence[0]);
            assert!(compose(&config, &artifact, &selected, hard.clone()).is_err());
        }
    }

    #[test]
    fn unselected_context_is_not_exposed_and_partial_remains_partial() {
        let (mut config, artifact, selected, mut hard) = test_fixture();
        config.sources[0].proposition.key = "unselected".into();
        config.unresolved_inputs.clear();
        let (answer, view) = compose(&config, &artifact, &selected, hard.clone()).unwrap();
        assert_eq!(answer, hard);
        assert!(view.citations.is_empty());
        config.sources[0].proposition = selected[0].proposition.clone();
        hard.status = FinalizationStatus::QualifiedPartialAnswer;
        let (answer, _) = compose(&config, &artifact, &selected, hard).unwrap();
        assert_eq!(answer.status, FinalizationStatus::QualifiedPartialAnswer);
        assert!(
            answer
                .text
                .unwrap()
                .contains("other unresolved claims are not established")
        );
    }
}
