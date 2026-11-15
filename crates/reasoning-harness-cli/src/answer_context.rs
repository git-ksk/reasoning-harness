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

fn source_quote(source: &Source, artifact: &ReasoningArtifact) -> Result<Option<String>, String> {
    // A predeclared source is only a candidate until this proposition's actual
    // claim/receipt refers to it. Either kind of reference must still validate;
    // an incomplete or forged reference cannot become a silent omission.
    let referenced = artifact.claims.iter().any(|claim| {
        claim.proposition.as_ref() == Some(&source.proposition)
            && claim.evidence_ids.contains(&source.evidence_id)
    }) || artifact.verification_receipts.iter().any(|receipt| {
        receipt.proposition.as_ref() == Some(&source.proposition)
            && receipt.evidence_ids.contains(&source.evidence_id)
    });
    let matches = artifact
        .evidence
        .iter()
        .filter(|e| e.id == source.evidence_id)
        .collect::<Vec<_>>();
    if matches.is_empty() && !referenced {
        return Ok(None);
    }
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
    // Acquired but unselected candidates remain hash/metadata checked above.
    // Acquisition alone does not attach their quotation to the selected claim.
    if !referenced {
        return Ok(None);
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
    Ok(Some(quote.to_owned()))
}

/// A frozen source record is a task-specific answer candidate, not truth authority.
/// Only promote it to an explicit finalization target when its complete source
/// identity (including version/quote pointers and admitted evidence SHA) and
/// current structured-fact claim/receipt are independently verified.  This
/// keeps a model-generated investigation plan from silently dropping a verified
/// facet when it omits an expected fact key (Issue #613, Redis allocator case).
/// Conflicting or unverified records never supply recovery authority.
pub(super) fn verified_source_targets(
    config: &Config,
    artifact: &ReasoningArtifact,
) -> Vec<Proposition> {
    if config.validate().is_err() {
        return vec![];
    }
    let mut result = Vec::new();
    for source in &config.sources {
        let target = &source.proposition;
        if result.contains(target)
            || config.sources.iter().any(|other| {
                other.proposition.key == target.key && other.proposition.value != target.value
            })
            || !grounded_target_authorized(artifact, target)
        {
            continue;
        }
        if matches!(source_quote(source, artifact), Ok(Some(_))) {
            result.push(target.clone());
        }
    }
    result
}

/// Quarantine only a self-contained, unverified model inference component rooted
/// in one *already refuted* model value. Any independent premise, typed
/// dependent claim, accepted receipt or protected hypothesis vetoes quarantine.
/// The original rejected graph remains in the previously recorded diagnostic
/// phase; this returns a new candidate for independent re-grounding only.
fn quarantine_refuted_component(
    candidate: &mut reasoning_harness_core::ReasoningCandidate,
    artifact: &ReasoningArtifact,
    root_id: &str,
) -> bool {
    let mut tainted = BTreeSet::from([root_id.to_owned()]);
    loop {
        let mut grew = false;
        for inference in &candidate.inferences {
            if inference
                .premise_claim_ids
                .iter()
                .any(|id| tainted.contains(id))
                && tainted.insert(inference.conclusion_claim_id.clone())
            {
                grew = true;
            }
        }
        if !grew {
            break;
        }
    }
    for inference in &candidate.inferences {
        if tainted.contains(&inference.conclusion_claim_id)
            && (inference.premise_claim_ids.is_empty()
                || inference
                    .premise_claim_ids
                    .iter()
                    .any(|p| !tainted.contains(p)))
        {
            return false;
        }
    }
    for id in &tainted {
        let matching = artifact
            .claims
            .iter()
            .filter(|c| &c.id == id)
            .collect::<Vec<_>>();
        let [current] = matching.as_slice() else {
            return false;
        };
        if id != root_id
            && (!matches!(
                current.state,
                EpistemicState::Assumed | EpistemicState::Inferred | EpistemicState::Unknown
            ) || current.proposition.is_some()
                || artifact
                    .verification_receipts
                    .iter()
                    .any(|r| r.claim_id.as_deref() == Some(id))
                || artifact
                    .adversarial_findings
                    .iter()
                    .any(|f| f.claim_id == *id && f.strength == FindingStrength::Hard))
        {
            return false;
        }
        if candidate.claims.iter().filter(|c| &c.id == id).count() != 1 {
            return false;
        }
    }
    for claim in &mut candidate.claims {
        if tainted.contains(&claim.id) {
            claim.proposition = None;
            claim.proposed_state = EpistemicState::Assumed;
            claim.evidence_ids.clear();
        }
    }
    candidate.inferences.retain(|i| {
        !tainted.contains(&i.conclusion_claim_id)
            && i.premise_claim_ids.iter().all(|p| !tainted.contains(p))
    });
    true
}

/// A model-proposed value for an already source-bound fact may be *refuted* by
/// the exact trusted structured value. Preserve its prose as untrusted and
/// untyped rather than letting it create a permanent same-key Reject which
/// blocks the separately verified official answer. This is NOT a synonym rule:
/// it requires an actual refutation receipt for the model claim, a separate
/// current support receipt for the canonical fact, unambiguous pinned source
/// bytes, and unanimity among admitted observations for this fact key.
///
/// No user-requested hypothesis, typed inference dependency, harness-owned
/// fact or competing authoritative evidence is revised. The caller must run
/// the entire grounding pipeline again and preserve the original trace.
pub(super) fn unbind_refuted_model_fact_keys(
    config: &Config,
    artifact: &ReasoningArtifact,
    candidate: &mut reasoning_harness_core::ReasoningCandidate,
) -> Vec<String> {
    if config.validate().is_err() {
        return vec![];
    }
    let verifier = reasoning_harness_core::structured_fact_verifier_for_input(
        &super::input_from_artifact(artifact),
    );
    let mut corrected = Vec::new();
    for source in &config.sources {
        let target = &source.proposition;
        if config.sources.iter().any(|other| {
            other.proposition.key == target.key && other.proposition.value != target.value
        }) || artifact
            .hypotheses
            .iter()
            .any(|p| p.key == target.key && p != target)
            || artifact
                .evidence_requirements
                .iter()
                .any(|r| r.proposition.key == target.key && r.proposition != *target)
            || artifact
                .evidence_qualification_findings
                .iter()
                .any(|f| f.proposition.key == target.key)
            || artifact.evidence.iter().any(|e| {
                e.facts
                    .get(&target.key)
                    .is_some_and(|value| value != &target.value)
            })
        {
            continue;
        }
        let matching = artifact
            .evidence
            .iter()
            .filter(|e| e.id == source.evidence_id)
            .collect::<Vec<_>>();
        let [evidence] = matching.as_slice() else {
            continue;
        };
        if evidence.source != source.source_id
            || evidence_hash(evidence) != source.evidence_sha256
            || evidence.facts.get(&target.key) != Some(&target.value)
            || evidence.metadata.scope.is_none()
            || evidence.metadata.temporal.is_none()
            || evidence
                .metadata
                .provenance_class
                .as_deref()
                .is_none_or(|p| p == "untrusted_context" || p == "untrusted")
        {
            continue;
        }
        let Ok(parsed) = serde_json::from_str::<UniqueJson>(&evidence.observation) else {
            continue;
        };
        let record = &parsed.0;
        if record.pointer(&source.url_pointer).and_then(|v| v.as_str()) != Some(&source.url)
            || record
                .pointer(&source.version_pointer)
                .and_then(|v| v.as_str())
                != Some(&source.source_version)
            || !record
                .pointer(&source.quote_pointer)
                .and_then(|v| v.as_str())
                .is_some_and(|quote| clean(quote) && quote.len() <= 32768)
        {
            continue;
        }
        let supported = artifact.claims.iter().any(|claim| {
            claim.proposition.as_ref() == Some(target)
                && claim.state == EpistemicState::Supported
                && claim.evidence_ids.contains(&source.evidence_id)
                && verifier
                    .verify(claim, &artifact.evidence)
                    .is_some_and(|receipt| {
                        receipt.conclusion == VerificationConclusion::Supported
                            && receipt.evidence_ids.contains(&source.evidence_id)
                    })
                && artifact.verification_receipts.iter().any(|receipt| {
                    matches!(
                        receipt.verifier.as_str(),
                        "structured_fact_equality" | "qualified_structured_fact_equality"
                    ) && receipt.conclusion == VerificationConclusion::Supported
                        && receipt.claim_id.as_deref() == Some(claim.id.as_str())
                        && receipt.proposition.as_ref() == Some(target)
                        && receipt
                            .claim_statement
                            .as_ref()
                            .is_none_or(|s| s == &claim.statement)
                        && receipt.evidence_ids.contains(&source.evidence_id)
                        && receipt
                            .evidence_ids
                            .iter()
                            .all(|id| artifact.evidence.iter().filter(|e| &e.id == id).count() == 1)
                })
        });
        if !supported {
            continue;
        }
        // Snapshot the candidate IDs first. Mutations occur only after the
        // complete dependency graph passes the fail-closed quarantine checks.
        let candidate_ids = candidate
            .claims
            .iter()
            .map(|c| c.id.clone())
            .collect::<Vec<_>>();
        for id in candidate_ids {
            let Some(claim) = candidate.claims.iter().find(|c| c.id == id) else {
                continue;
            };
            if claim.id.starts_with("harness_")
                || !claim
                    .proposition
                    .as_ref()
                    .is_some_and(|p| p.key == target.key && p.value != target.value)
                || !artifact.claims.iter().any(|original| {
                    original.id == claim.id
                        && original.proposition == claim.proposition
                        && original.state == EpistemicState::Contradicted
                })
                || !artifact.verification_receipts.iter().any(|receipt| {
                    receipt.claim_id.as_deref() == Some(claim.id.as_str())
                        && receipt.proposition.as_ref() == claim.proposition.as_ref()
                        && receipt.conclusion == VerificationConclusion::Contradicted
                        && matches!(
                            receipt.verifier.as_str(),
                            "structured_fact_equality" | "qualified_structured_fact_equality"
                        )
                        && receipt.evidence_ids.contains(&source.evidence_id)
                })
            {
                continue;
            }
            if quarantine_refuted_component(candidate, artifact, &id) {
                corrected.push(id);
            }
        }
    }
    corrected.sort();
    corrected.dedup();
    corrected
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
            let Some(quote) = source_quote(source, artifact)? else {
                continue;
            };
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
    use reasoning_harness_core::FinalAnswerCandidate;

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
    fn source_registry_candidates_follow_actual_acquisition_and_receipt_selection() {
        use reasoning_harness_core::{AnswerSafetyProfile, Verdict};
        let (mut config, original, selected, hard) = test_fixture();
        let mut sibling = original.evidence[0].clone();
        sibling.id = "admitted-e2".into();
        sibling.source = "alternate-manual".into();
        let mut binding = config.sources[0].clone();
        binding.evidence_id = sibling.id.clone();
        binding.source_id = sibling.source.clone();
        binding.evidence_sha256 = evidence_hash(&sibling);
        config.sources.push(binding);
        // The registry is fixed before either source is acquired. The same
        // proposition can be established by either exact admitted source.
        for selected_source in 0..2 {
            let mut artifact = original.clone();
            if selected_source == 1 {
                artifact.evidence[0] = sibling.clone();
                artifact.claims[0].evidence_ids = vec![sibling.id.clone()];
                artifact.verification_receipts[0].evidence_ids = vec![sibling.id.clone()];
            }
            let (answer, saved) = compose(&config, &artifact, &selected, hard.clone()).unwrap();
            assert_eq!(saved.citations.len(), 1);
            assert_eq!(saved.citations[0].evidence_id, artifact.evidence[0].id);
            assert_eq!(
                restore(
                    &config,
                    &artifact,
                    Verdict::Accept,
                    AnswerSafetyProfile::VerifiedTargetV1,
                    &saved
                )
                .unwrap()
                .0,
                answer
            );
        }
        let mut artifact = original.clone();
        artifact.evidence.push(sibling.clone());
        // Merely acquiring another agreeing document is not a bound receipt.
        let (_, view) = compose(&config, &artifact, &selected, hard.clone()).unwrap();
        assert_eq!(view.citations.len(), 1);
        for change in 0..4 {
            let mut changed = artifact.clone();
            match change {
                0 => changed.evidence[1].observation.push(' '),
                1 => changed.evidence.push(sibling.clone()),
                2 => changed.claims[0].evidence_ids.push(sibling.id.clone()),
                _ => changed.verification_receipts[0]
                    .evidence_ids
                    .push(sibling.id.clone()),
            }
            assert!(
                compose(&config, &changed, &selected, hard.clone()).is_err(),
                "change={change}"
            );
        }
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

    fn refuted_model_variant_fixture() -> (
        Config,
        ReasoningArtifact,
        reasoning_harness_core::ReasoningCandidate,
    ) {
        use reasoning_harness_core::{CandidateClaim, ReasoningCandidate};
        let (config, original, selected, _) = test_fixture();
        let current = selected[0].proposition.clone();
        let input = super::super::input_from_artifact(&original);
        let variant = Proposition {
            key: current.key.clone(),
            value: "model's unsupported shorter paraphrase".into(),
        };
        let candidate = ReasoningCandidate {
            claims: vec![
                CandidateClaim {
                    id: "model-refuted-variant".into(),
                    statement: "original untrusted model phrasing".into(),
                    proposed_state: EpistemicState::Assumed,
                    proposition: Some(variant),
                    evidence_ids: vec![],
                },
                CandidateClaim {
                    id: "official-source-fact".into(),
                    statement: "canonical source fact".into(),
                    proposed_state: EpistemicState::Assumed,
                    proposition: Some(current),
                    evidence_ids: vec![original.evidence[0].id.clone()],
                },
            ],
            ..Default::default()
        };
        let outcome = super::super::run_standard_grounding(input, candidate.clone()).unwrap();
        assert_eq!(outcome.verdict, reasoning_harness_core::Verdict::Reject);
        (config, outcome.artifact, candidate)
    }

    #[test]
    fn issue613_refuted_model_variant_can_be_unbound_but_verified_fact_is_asserted() {
        use reasoning_harness_core::{
            FinalizationStatus, Verdict, canonical_verified_target_partial_answer,
        };
        let (config, artifact, mut candidate) = refuted_model_variant_fixture();
        let original_id = candidate.claims[0].id.clone();
        assert_eq!(
            unbind_refuted_model_fact_keys(&config, &artifact, &mut candidate),
            vec![original_id]
        );
        assert!(candidate.claims[0].proposition.is_none());
        assert_eq!(candidate.claims[0].proposed_state, EpistemicState::Assumed);
        let replay = super::super::run_standard_grounding(
            super::super::input_from_artifact(&artifact),
            candidate,
        )
        .unwrap();
        assert_eq!(replay.verdict, Verdict::Unknown);
        assert!(
            !replay
                .artifact
                .claims
                .iter()
                .any(|c| c.state == EpistemicState::Contradicted)
        );
        let targets = verified_source_targets(&config, &replay.artifact);
        assert_eq!(targets.len(), 1);
        let (answer, hard) =
            canonical_verified_target_partial_answer(&replay.artifact, replay.verdict, &targets)
                .expect("exact supported source truth is available; model error stays unresolved");
        assert_eq!(hard.status, FinalizationStatus::QualifiedPartialAnswer);
        let (display, view) =
            compose(&config, &replay.artifact, &answer.factual_claims, hard).unwrap();
        assert!(
            display
                .text
                .as_deref()
                .unwrap()
                .contains("\"feature.rule\" = \"limited\"")
        );
        assert!(
            !display
                .text
                .as_deref()
                .unwrap()
                .contains("unsupported shorter paraphrase")
        );
        assert_eq!(view.citations.len(), 1);
    }

    #[test]
    fn issue613_refuted_model_inference_component_quarantines_only_unverified_dependents() {
        use reasoning_harness_core::{
            CandidateClaim, Inference, Verdict, canonical_verified_target_partial_answer,
        };
        let (config, original, mut candidate) = refuted_model_variant_fixture();
        for n in 0..3 {
            candidate.claims.push(CandidateClaim {
                id: format!("model-dependent-{n}"),
                statement: format!("unverified model inference {n}"),
                proposed_state: EpistemicState::Assumed,
                proposition: None,
                evidence_ids: vec![],
            });
            candidate.inferences.push(Inference {
                id: format!("model-inference-{n}"),
                premise_claim_ids: vec!["model-refuted-variant".into()],
                conclusion_claim_id: format!("model-dependent-{n}"),
                method: "unverified_generation".into(),
            });
        }
        let rejected = super::super::run_standard_grounding(
            super::super::input_from_artifact(&original),
            candidate.clone(),
        )
        .unwrap();
        assert_eq!(rejected.verdict, Verdict::Reject);
        let ids = unbind_refuted_model_fact_keys(&config, &rejected.artifact, &mut candidate);
        assert_eq!(ids, vec!["model-refuted-variant"]);
        assert!(candidate.inferences.is_empty());
        assert!(
            candidate
                .claims
                .iter()
                .filter(|c| c.id.starts_with("model-"))
                .all(|c| c.proposition.is_none() && c.proposed_state == EpistemicState::Assumed)
        );
        let revised = super::super::run_standard_grounding(
            super::super::input_from_artifact(&rejected.artifact),
            candidate,
        )
        .unwrap();
        assert_eq!(revised.verdict, Verdict::Unknown);
        let exact = verified_source_targets(&config, &revised.artifact);
        let (answer, hard) =
            canonical_verified_target_partial_answer(&revised.artifact, revised.verdict, &exact)
                .expect(
                    "the actual source-backed fact survives while inferred model prose does not",
                );
        let (display, view) =
            compose(&config, &revised.artifact, &answer.factual_claims, hard).unwrap();
        assert!(
            display
                .text
                .unwrap()
                .contains("\"feature.rule\" = \"limited\"")
        );
        assert_eq!(view.citations.len(), 1);
    }

    #[test]
    fn issue613_refuted_model_variant_fail_closed_without_exact_current_source_authority() {
        let (config, artifact, original) = refuted_model_variant_fixture();
        for change in 0..7 {
            let mut config = config.clone();
            let mut artifact = artifact.clone();
            let mut candidate = original.clone();
            match change {
                0 => config.sources[0].source_version.push_str("-tampered"),
                1 => config.sources[0].evidence_sha256 = "0".repeat(64),
                2 => artifact
                    .verification_receipts
                    .retain(|r| r.conclusion != VerificationConclusion::Supported),
                3 => artifact
                    .verification_receipts
                    .retain(|r| r.conclusion != VerificationConclusion::Contradicted),
                4 => artifact.evidence.push(Evidence {
                    id: "counterexample".into(),
                    source: "different".into(),
                    observation: "other value".into(),
                    facts: BTreeMap::from([("feature.rule".into(), "opposite".into())]),
                    metadata: artifact.evidence[0].metadata.clone(),
                }),
                5 => artifact
                    .hypotheses
                    .push(original.claims[0].proposition.clone().unwrap()),
                _ => candidate
                    .inferences
                    .push(reasoning_harness_core::Inference {
                        id: "link".into(),
                        premise_claim_ids: vec![original.claims[0].id.clone()],
                        conclusion_claim_id: original.claims[1].id.clone(),
                        method: "model".into(),
                    }),
            }
            assert!(
                unbind_refuted_model_fact_keys(&config, &artifact, &mut candidate).is_empty(),
                "unexpected unsupported correction case {change}"
            );
            assert_eq!(candidate.claims[0], original.claims[0]);
        }
    }

    fn two_sourced_facts_with_one_unplanned_target() -> (
        Config,
        ReasoningArtifact,
        crate::acquisition_coverage::Ledger,
        FinalAnswerCandidate,
    ) {
        use reasoning_harness_core::{
            EpistemicState, InvestigationTarget, InvestigationTargetOrigin,
        };
        let (mut config, mut artifact, selected, _) = test_fixture();
        let missing = Proposition {
            key: "allocator.force_libc".into(),
            value: "make MALLOC=libc".into(),
        };
        let mut evidence = artifact.evidence[0].clone();
        evidence.id = "admitted-libc".into();
        evidence.source = "allocator-manual".into();
        evidence.facts = BTreeMap::from([(missing.key.clone(), missing.value.clone())]);
        evidence.observation = serde_json::json!({
            "quote": "Linux defaults to jemalloc; make MALLOC=libc uses libc malloc.",
            "url": "https://example.test/manual", "version": "edition-1"
        })
        .to_string();
        let mut claim = artifact.claims[0].clone();
        claim.id = "verified-libc".into();
        claim.proposition = Some(missing.clone());
        claim.evidence_ids = vec![evidence.id.clone()];
        let mut receipt = artifact.verification_receipts[0].clone();
        receipt.claim_id = Some(claim.id.clone());
        receipt.claim_statement = Some(claim.statement.clone());
        receipt.proposition = Some(missing.clone());
        receipt.evidence_ids = vec![evidence.id.clone()];
        let mut source = config.sources[0].clone();
        source.proposition = missing.clone();
        source.evidence_id = evidence.id.clone();
        source.source_id = evidence.source.clone();
        source.evidence_sha256 = evidence_hash(&evidence);
        config.sources.push(source);
        artifact.evidence.push(evidence);
        artifact.claims.push(claim);
        artifact.verification_receipts.push(receipt);
        let mut unverified = artifact.claims[0].clone();
        unverified.id = "unrelated-model-proposal".into();
        unverified.state = EpistemicState::Assumed;
        unverified.proposition = None;
        unverified.evidence_ids.clear();
        artifact.claims.push(unverified);
        let mut ledger = crate::acquisition_coverage::Ledger::new(vec![
            InvestigationTarget {
                id: "default_allocator".into(),
                question: "What is the default allocator?".into(),
                expected_fact_key: Some(selected[0].proposition.key.clone()),
                origin: InvestigationTargetOrigin::ModelProposedUntrusted,
            },
            InvestigationTarget {
                id: "force_libc".into(),
                question: "How can libc be selected?".into(),
                expected_fact_key: None, // Observed 0.7.3 plan lost this selector.
                origin: InvestigationTargetOrigin::ModelProposedUntrusted,
            },
        ])
        .unwrap();
        ledger.bind_existing(&artifact);
        assert!(ledger.targets(&artifact)[1].verified_proposition.is_none());
        let rendered = FinalAnswerCandidate {
            text: "The default allocator is known".into(),
            factual_claims: selected,
        };
        (config, artifact, ledger, rendered)
    }

    #[test]
    fn issue613_verified_source_recovers_unplanned_answer_fact_not_just_quote() {
        use reasoning_harness_core::{FinalizationStatus, Verdict};
        let (config, artifact, ledger, rendered) = two_sourced_facts_with_one_unplanned_target();
        let targets = verified_source_targets(&config, &artifact);
        assert_eq!(targets.len(), 2);
        let (recovered, hard) = ledger
            .recover_rendered(&artifact, Verdict::Unknown, &rendered, &targets)
            .expect("verified missing facet must be promoted to answer text");
        assert_eq!(hard.status, FinalizationStatus::QualifiedPartialAnswer);
        assert_eq!(hard.covered_claims, 2);
        assert!(recovered.factual_claims.iter().any(|c| {
            c.mode == FinalClaimMode::Grounded
                && c.proposition.key == "allocator.force_libc"
                && c.proposition.value == "make MALLOC=libc"
        }));
        let (display, view) = compose(&config, &artifact, &recovered.factual_claims, hard).unwrap();
        let text = display.text.as_deref().unwrap();
        assert!(text.contains("\"allocator.force_libc\" = \"make MALLOC=libc\""));
        assert_eq!(view.citations.len(), 2);
        // Answer coverage and investigation-plan coverage are independent.
        let (display, coverage) = crate::acquisition_coverage::compose(
            &ledger,
            &artifact,
            &recovered.factual_claims,
            display,
            None,
        )
        .unwrap();
        assert!(!coverage.plan_evidence_complete);
        assert!(!coverage.plan_answer_complete);
        assert!(
            display
                .text
                .as_deref()
                .unwrap()
                .contains("make MALLOC=libc")
        );
        assert!(
            !display
                .text
                .as_deref()
                .unwrap()
                .contains("unrelated-model-proposal")
        );
    }

    #[test]
    fn issue613_tampered_source_and_same_key_conflict_never_gain_answer_authority() {
        use reasoning_harness_core::EpistemicState;
        let (config, artifact, _, _) = two_sourced_facts_with_one_unplanned_target();
        let expected = verified_source_targets(&config, &artifact);
        assert_eq!(expected.len(), 2);
        for change in 0..5 {
            let mut config = config.clone();
            let mut artifact = artifact.clone();
            match change {
                0 => config.sources[1].source_version = "tampered-edition".into(),
                1 => config.sources[1].evidence_sha256 = "0".repeat(64),
                2 => artifact.verification_receipts.retain(|r| {
                    r.proposition
                        .as_ref()
                        .is_none_or(|p| p.key != "allocator.force_libc")
                }),
                3 => {
                    let mut conflicted = artifact.claims[1].clone();
                    conflicted.id = "model-contradiction".into();
                    conflicted.state = EpistemicState::Contradicted;
                    conflicted.proposition.as_mut().unwrap().value =
                        "not-the-supported-value".into();
                    artifact.claims.push(conflicted);
                }
                _ => artifact.evidence[1].observation.push(' '),
            }
            assert!(
                !verified_source_targets(&config, &artifact)
                    .iter()
                    .any(|p| p.key == "allocator.force_libc"),
                "unsafe source must not be promoted: change={change}"
            );
        }
    }

    #[test]
    fn issue613_check_ignore_all_exit_codes_stay_a_single_verified_answer() {
        let (mut config, mut artifact, _, _) = test_fixture();
        let codes = Proposition {
            key: "git.check_ignore.exit_codes".into(),
            value: "0: ignored; 1: not ignored; 128: fatal error".into(),
        };
        artifact.claims[0].proposition = Some(codes.clone());
        artifact.verification_receipts[0].proposition = Some(codes.clone());
        artifact.evidence[0].facts = BTreeMap::from([(codes.key.clone(), codes.value.clone())]);
        artifact.evidence[0].observation = serde_json::json!({
            "quote":"Exit status: 0 ignored, 1 not ignored, 128 fatal error.",
            "url":"https://example.test/manual", "version":"edition-1"
        })
        .to_string();
        config.sources[0].proposition = codes.clone();
        config.sources[0].evidence_sha256 = evidence_hash(&artifact.evidence[0]);
        assert_eq!(
            verified_source_targets(&config, &artifact),
            vec![codes.clone()]
        );
        let answer = reasoning_harness_core::canonical_verified_target_answer(
            &artifact,
            reasoning_harness_core::Verdict::Accept,
            &[codes],
        )
        .unwrap();
        let hard = reasoning_harness_core::finalize_answer(
            &artifact,
            reasoning_harness_core::Verdict::Accept,
            answer.clone(),
            reasoning_harness_core::FinalizationPolicy::default(),
        );
        let (display, view) = compose(&config, &artifact, &answer.factual_claims, hard).unwrap();
        assert!(
            display
                .text
                .unwrap()
                .contains("0: ignored; 1: not ignored; 128: fatal error")
        );
        assert_eq!(view.citations.len(), 1);
    }

    /// The raw original remains private. Supply two *already decrypted*, local-only
    /// paths when deliberately replaying the real issue-613 observation offline.
    #[test]
    #[ignore = "requires RH613_PRIVATE_OBSERVATION and RH613_PUBLIC_CONFIG, never provider calls"]
    fn issue613_real_rw21_offline_artifact_replay() {
        use reasoning_harness_core::Verdict;
        let observation = std::env::var("RH613_PRIVATE_OBSERVATION").unwrap();
        let config_path = std::env::var("RH613_PUBLIC_CONFIG").unwrap();
        let raw: serde_json::Value =
            serde_json::from_slice(&std::fs::read(observation).unwrap()).unwrap();
        let raw_config: serde_json::Value =
            serde_json::from_slice(&std::fs::read(config_path).unwrap()).unwrap();
        let config: Config = serde_json::from_value(raw_config["answer_context"].clone()).unwrap();
        let result = &raw["result"];
        let artifact: ReasoningArtifact =
            serde_json::from_value(result["final_outcome"]["artifact"].clone()).unwrap();
        let ledger: crate::acquisition_coverage::Ledger =
            serde_json::from_value(result["acquisition_coverage"]["ledger"].clone()).unwrap();
        let rendered = FinalAnswerCandidate {
            text: "simulated stochastic renderer omits build setting".into(),
            factual_claims: serde_json::from_value(
                result["acquisition_coverage"]["selected_claims"].clone(),
            )
            .unwrap(),
        };
        assert_eq!(result["final_outcome"]["verdict"], "unknown");
        assert_eq!(ledger.verified_targets(&artifact).len(), 1);
        let targets = verified_source_targets(&config, &artifact);
        assert_eq!(targets.len(), 2);
        let (recovered, hard) = ledger
            .recover_rendered(&artifact, Verdict::Unknown, &rendered, &targets)
            .expect("real rw-21 private artifact must recover both verified facts");
        let (display, view) = compose(&config, &artifact, &recovered.factual_claims, hard).unwrap();
        let text = display.text.unwrap();
        assert!(text.contains("\"redis.allocator.force_libc\" = \"make MALLOC=libc\""));
        assert!(text.contains("\"redis.allocator.linux_default\" = \"jemalloc\""));
        assert_eq!(view.citations.len(), 3); // pinned source has two records for jemalloc.
    }

    /// Replay the three other actual observations without promoting a generated
    /// claim, changing original archives, or making a provider request.
    #[test]
    #[ignore = "requires RH613_PRIVATE_OBSERVATION_DIR and RH613_PUBLIC_CONFIG_DIR"]
    fn issue613_real_conflicts_and_check_ignore_offline_replay() {
        let private =
            std::path::PathBuf::from(std::env::var("RH613_PRIVATE_OBSERVATION_DIR").unwrap());
        let public = std::path::PathBuf::from(std::env::var("RH613_PUBLIC_CONFIG_DIR").unwrap());
        for case in ["rw-02", "rw-19", "rw-08"] {
            let raw: serde_json::Value = serde_json::from_slice(
                &std::fs::read(private.join(format!("{case}-candidate.stdout.json"))).unwrap(),
            )
            .unwrap();
            let cfg: serde_json::Value = serde_json::from_slice(
                &std::fs::read(public.join(format!("{case}.json"))).unwrap(),
            )
            .unwrap();
            let config: Config = serde_json::from_value(cfg["answer_context"].clone()).unwrap();
            let result = &raw["result"];
            let artifact: ReasoningArtifact =
                serde_json::from_value(result["final_outcome"]["artifact"].clone()).unwrap();
            let targets = verified_source_targets(&config, &artifact);
            match case {
                "rw-02" | "rw-19" => {
                    assert_eq!(result["final_outcome"]["verdict"], "reject");
                    assert!(
                        targets.is_empty(),
                        "must preserve conflict withholding: {case}"
                    );
                    assert_eq!(result["finalization"]["status"], "abstain");
                }
                "rw-08" => {
                    assert_eq!(targets.len(), 1);
                    let answer = &result["acquisition_coverage"]["selected_claims"];
                    assert_eq!(answer.as_array().unwrap().len(), 1);
                    let statement = answer[0]["proposition"]["value"].as_str().unwrap();
                    for code in ["0:", "1:", "128:"] {
                        assert!(
                            statement.contains(code),
                            "missing {code} in displayed asserted fact"
                        );
                    }
                    assert_eq!(result["finalization"]["status"], "qualified_partial_answer");
                }
                _ => unreachable!(),
            }
        }
    }

    /// Replay the exact previously captured model claims and source bindings;
    /// never create a fresh provider observation or change the original result.
    #[test]
    #[ignore = "requires RH613_PRIVATE_OBSERVATION_DIR and RH613_PUBLIC_CONFIG_DIR"]
    fn issue613_real_rw02_rw19_offline_corrected_display_replay() {
        use reasoning_harness_core::{
            CandidateClaim, ReasoningCandidate, Verdict, canonical_verified_target_answer,
            canonical_verified_target_partial_answer,
        };
        let private =
            std::path::PathBuf::from(std::env::var("RH613_PRIVATE_OBSERVATION_DIR").unwrap());
        let public = std::path::PathBuf::from(std::env::var("RH613_PUBLIC_CONFIG_DIR").unwrap());
        for case in ["rw-19", "rw-02"] {
            let raw: serde_json::Value = serde_json::from_slice(
                &std::fs::read(private.join(format!("{case}-candidate.stdout.json"))).unwrap(),
            )
            .unwrap();
            let public_cfg: serde_json::Value = serde_json::from_slice(
                &std::fs::read(public.join(format!("{case}.json"))).unwrap(),
            )
            .unwrap();
            let cfg: Config = serde_json::from_value(public_cfg["answer_context"].clone()).unwrap();
            let original: ReasoningArtifact =
                serde_json::from_value(raw["result"]["final_outcome"]["artifact"].clone()).unwrap();
            assert_eq!(raw["result"]["final_outcome"]["verdict"], "reject");
            let mut candidate = ReasoningCandidate {
                claims: original
                    .claims
                    .iter()
                    .map(|c| CandidateClaim {
                        id: c.id.clone(),
                        statement: c.statement.clone(),
                        proposed_state: EpistemicState::Assumed,
                        proposition: c.proposition.clone(),
                        evidence_ids: c.evidence_ids.clone(),
                    })
                    .collect(),
                inferences: original.inferences.clone(),
            };
            let corrected = unbind_refuted_model_fact_keys(&cfg, &original, &mut candidate);
            assert_eq!(
                corrected.len(),
                1,
                "expected one verified model-value correction: {case}"
            );
            let mut input = super::super::input_from_artifact(&original);
            if case == "rw-02" {
                // This source was NOT acquired in the cancelled real attempt:
                // the core Reject short circuit aborted the second resolution.
                // Add the *unchanged* public host-pinned snapshot as a simulated
                // read-only acquisition for a provider-free prospective replay.
                let captured =
                    &public_cfg["resolution"]["investigation"]["capabilities"][0]["evidence"][0];
                let mut acquired = original.evidence[0].clone();
                acquired.id = captured["id"].as_str().unwrap().into();
                acquired.source = captured["source"].as_str().unwrap().into();
                acquired.observation = captured["observation"].as_str().unwrap().into();
                acquired.facts = serde_json::from_value(captured["facts"].clone()).unwrap();
                assert_eq!(acquired.id, cfg.sources[0].evidence_id);
                assert_eq!(evidence_hash(&acquired), cfg.sources[0].evidence_sha256);
                candidate.claims.push(CandidateClaim {
                    id: "harness_local_replay_only_rw02_source0".into(),
                    statement: "typed fact from pinned source-0".into(),
                    proposed_state: EpistemicState::Assumed,
                    proposition: Some(cfg.sources[0].proposition.clone()),
                    evidence_ids: vec![acquired.id.clone()],
                });
                input.evidence.push(acquired);
            }
            let regrounded = super::super::run_standard_grounding(input, candidate).unwrap();
            assert_ne!(
                regrounded.verdict,
                Verdict::Reject,
                "model-origin contradiction persists: {case}"
            );
            let targets = verified_source_targets(&cfg, &regrounded.artifact);
            assert_eq!(targets.len(), if case == "rw-02" { 2 } else { 1 });
            let (answer, hard) = canonical_verified_target_answer(
                &regrounded.artifact,
                regrounded.verdict,
                &targets,
            )
            .map(|a| {
                let result = reasoning_harness_core::finalize_answer(
                    &regrounded.artifact,
                    regrounded.verdict,
                    a.clone(),
                    reasoning_harness_core::FinalizationPolicy::default(),
                );
                (a, result)
            })
            .or_else(|| {
                canonical_verified_target_partial_answer(
                    &regrounded.artifact,
                    regrounded.verdict,
                    &targets,
                )
            })
            .expect("both canonical facts must be re-grounded from the actual artifact");
            let (display, view) =
                compose(&cfg, &regrounded.artifact, &answer.factual_claims, hard).unwrap();
            let text = display.text.as_deref().unwrap();
            for target in targets {
                assert!(
                    text.contains(&format!(
                        "{} = {}",
                        serde_json::to_string(&target.key).unwrap(),
                        serde_json::to_string(&target.value).unwrap(),
                    )),
                    "fact not asserted in final display: {}",
                    target.key
                );
            }
            assert!(view.citations.len() >= if case == "rw-02" { 2 } else { 1 });
            assert!(!text.contains("model's unsupported shorter paraphrase"));
        }
    }

    #[test]
    #[ignore = "requires RH613_PRIVATE_OBSERVATION_DIR and RH613_PUBLIC_CONFIG_DIR"]
    fn issue613_actual_five_negative_controls_never_unbind_or_assert_new_facts() {
        use reasoning_harness_core::{CandidateClaim, ReasoningCandidate};
        let private =
            std::path::PathBuf::from(std::env::var("RH613_PRIVATE_OBSERVATION_DIR").unwrap());
        let public = std::path::PathBuf::from(std::env::var("RH613_PUBLIC_CONFIG_DIR").unwrap());
        for case in ["rw-07", "rw-12", "rw-14", "rw-18", "rw-22"] {
            let raw: serde_json::Value = serde_json::from_slice(
                &std::fs::read(private.join(format!("{case}-candidate.stdout.json"))).unwrap(),
            )
            .unwrap();
            let public_cfg: serde_json::Value = serde_json::from_slice(
                &std::fs::read(public.join(format!("{case}.json"))).unwrap(),
            )
            .unwrap();
            let config: Config =
                serde_json::from_value(public_cfg["answer_context"].clone()).unwrap();
            let artifact: ReasoningArtifact =
                serde_json::from_value(raw["result"]["final_outcome"]["artifact"].clone()).unwrap();
            let mut candidate = ReasoningCandidate {
                claims: artifact
                    .claims
                    .iter()
                    .map(|c| CandidateClaim {
                        id: c.id.clone(),
                        statement: c.statement.clone(),
                        proposed_state: EpistemicState::Assumed,
                        proposition: c.proposition.clone(),
                        evidence_ids: c.evidence_ids.clone(),
                    })
                    .collect(),
                inferences: artifact.inferences.clone(),
            };
            let unchanged = candidate.clone();
            assert!(
                unbind_refuted_model_fact_keys(&config, &artifact, &mut candidate).is_empty(),
                "negative control must not be corrected: {case}"
            );
            assert_eq!(candidate, unchanged);
            assert_eq!(raw["result"]["finalization"]["covered_claims"], 0);
            let status = raw["result"]["finalization"]["status"].as_str().unwrap();
            assert!(
                matches!(status, "abstain" | "requires_verification" | "unresolved"),
                "negative control unexpectedly asserted an answer: {case}"
            );
        }
    }
}
