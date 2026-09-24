# ADR-0006: Harness-owned evidence-target semantic relevance

Status: candidate contract implemented for fresh calibration. Runtime integration and live acceptance remain pending.

## Context

Issue #461 now owns whether a target needs external acquisition. Once acquisition is justified and candidate material is retrieved, a separate question remains: is this material actually about the exact Harness-owned target and requested relation?

Existing admission and qualification correctly own provenance, freshness, scope, authority, and proposition support. They must not be overloaded with generic semantic target matching. Product integrations also must not each invent token percentages, URL substring checks, or model-authored search-intent identity.

Issue #462 therefore adds a provider-neutral pre-admission relevance contract for the Engine 0.6 candidate. The motivating production incident is a gap report only and is excluded from tuning fixtures.

## Decision

Introduce a distinct EvidenceRelevanceTargetPolicy, EvidenceRelevanceCandidate, advisory EvidenceRelevanceProposal, and Harness-materialized EvidenceRelevanceAssessment.

The only relevance dispositions are:

- relevant;
- irrelevant;
- ambiguous.

Relevant means only that the candidate remains eligible for downstream consideration. It does not create Evidence, authority metadata, verification receipts, supported claims, verdicts, freshness, or answer sufficiency.

The model-facing proposal contains only the disposition. It cannot echo or replace target ID, evidence ID, source ID, entity identity, relation kind, authority, trust, freshness, scope, verification, or verdict.

## Harness-owned target identity

The Harness policy owns:

- stable policy and target IDs;
- the exact target question;
- optional canonical entity ID and canonical name;
- approved aliases/localized names;
- typed relation kind;
- identity requirement;
- explicit model/token/time/attempt budget.

Identity requirement is one of:

- none;
- require_harness_anchor;
- allow_semantic_equivalent.

When require_harness_anchor is active, a model cannot mark a candidate relevant unless at least one Harness-owned canonical name or alias appears in a content-bearing candidate signal. URL-only and navigation/footer-only matches do not satisfy the anchor. A missing required anchor therefore blocks model-relevant into ambiguous, while a model may still safely reject the candidate as irrelevant.

allow_semantic_equivalent is an explicit Harness policy decision for targets where lexical identity is not required. It permits model-assisted paraphrase/cross-lingual relevance without allowing the model to rewrite the Harness-owned target identity.

## Candidate signals

Candidate material remains untrusted data. Signals are typed so weak metadata can be distinguished from local support:

- source title;
- canonical URL;
- heading;
- excerpt;
- structured metadata;
- navigation/footer;
- fact text.

No candidate signal creates authority. URL and navigation/footer signals cannot satisfy strict identity by themselves. Source title or metadata may contribute an identity anchor, but final relevance still requires the semantic proposal/materialization path rather than self-authorizing from one token match.

Prompt-like instructions inside any signal are data and cannot mutate relevance policy.

## Relevance is not freshness or truth

A stale document may be semantically relevant. It must remain relevant-yet-stale and be rejected later by ordinary freshness policy if required.

Likewise:

- trusted source does not imply semantic relevance;
- semantic relevance does not imply trusted source;
- relevance does not satisfy EvidenceRequirement;
- relevance does not bypass EvidenceAdmissionPolicy or EvidenceQualificationPass;
- relevance does not authorize source-attributed prose; that remains #463.

## Bounded model assistance

EvidenceRelevanceAssessmentBudget is Harness-owned and contains max model attempts, max tokens, and max elapsed milliseconds. Zero budgets are invalid.

No model proposal or assessment failure may implicitly become relevant. Missing proposal materializes to typed ambiguous. Provider/transport/protocol failures remain operational failures in the eventual live runner and are scored separately from semantic outcomes.

The serialized assessment contains only stable policy/target/evidence/source IDs, disposition, assessment path, and typed reasons. Raw document payload is not required for telemetry or replay.

## Evaluation

Fresh calibration identity: evidence-relevance-calibration-v1.

The first corpus contains 26 synthetic cases spanning exact identity, aliases/acronyms, semantic paraphrase, distributed title/body support, structured metadata, Japanese/English cross-lingual identity, stale-but-relevant material, same-service wrong feature, sibling products, navigation/footer-only matches, broad landing pages, comparison-only mentions, relation mismatch, prompt injection, unknown renames, partial identity, mixed documents, conflicting sections, insufficient excerpts, and URL-only identity.

Hard correctness gate: wrong-target relevance retention = 0.

Utility is separate and includes false rejection/avoidable ambiguity on relevant material. An always-relevant candidate fails correctness; an always-reject candidate fails utility.

No independent holdout is authored until calibration semantics and thresholds are frozen after live observation. Released Engine 0.5.0 remains immutable.
