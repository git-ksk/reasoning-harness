# Harness Engine 0.7.0 — Evidence Reconciliation and Answer Sufficiency

[日本語](engine-0.7.0-roadmap.ja.md) | English

**Status: planned, not implemented, not released.** [Milestone #10](https://github.com/git-ksk/reasoning-harness/milestone/10) · [Parent Issue #486](https://github.com/git-ksk/reasoning-harness/issues/486)

This is an **independent Harness Engine source** track. It does not advance the published Reason CLI 0.5.4 / Engine 0.6.1 pair, imply a future CLI version, or change any existing release tag.

## Product hypothesis

Engine 0.6.1 is already good at deciding whether individual evidence is target-relevant, temporally/scope/authority qualified, verified, or safely source-attributed. The next suspected product gap concerns **reconciliation across multiple pieces of evidence for one target**: when evidence repeats a single origin, revisions change over time, statements partly disagree, or available evidence covers only some of the user's required information.

The goal is to **recover more useful source-grounded answers without increasing unsupported conclusions**, not merely to increase the amount of text output.

**Hypothesis, not a proven production defect:** no proposed semantic mechanism ships until the frozen 0.6.1 baseline shows a real correctness/utility residual and fresh acceptance demonstrates a benefit.

## Reuse rather than rebuild

| Already present in Engine 0.6.1 | Candidate 0.7.0 boundary |
| --- | --- |
| `EvidenceQualificationInspector`: temporal validity, scope and provenance class; structured-fact conflicts | Explicit source-version relationships and as-of reconciliation; do not replace existing qualification |
| `QualifiedStructuredFactVerifier`: withholds hard receipts on conflicting qualified facts | Preserve unresolved conflicts; no source-majority or latest-document truth |
| `EvidenceNeedMaterialization`: target-local need floors, stale/mismatched reuse rejection | Decide if remaining required information warrants bounded acquisition, not a bypass of these floors |
| `SourceAttributionState`: exact source spans, per-binding attribution/citations, conflict state and replay | Trusted origin lineage, genuine independent corroboration and narrowly allowed multi-source synthesis |
| `EvidenceSufficiency`: advisory sufficient/insufficient/mixed research | Harness-owned target-local answerability decision, without model-created authority |

Source-attributed claim conflict markers presently distinguish trimmed statement strings for one target. A different string is **not automatically a logical contradiction**, and identical strings do not prove common origin or independent support. Source URLs, IDs, publication/retrieval times and version strings are useful bindings but do not by themselves certify provenance, factual recency, independence, or truth.

## Sequenced work and dependencies

### Phase 0 — P0: Freeze 0.6.1 residual baseline ([#487](https://github.com/git-ksk/reasoning-harness/issues/487))

Audit the existing runtime and deterministic tests. Precommit fixtures and a scoring contract for mirrored-origin articles, asserted-independent sources, temporal revisions, opposite vs paraphrased claims, partial required-information, wrong targets, source injection, mixed tasks and session replay. Record useful responses, avoidable abstention, violations, budget and operational failures **separately**. Choose numeric utility/efficiency thresholds from the measured development baseline **before** writing the fresh independent holdout.

Deliverable: reproducible gap matrix marking each proposed feature *demonstrated*, *already handled*, *unproven*, or *out of scope*. Stop or narrow 0.7.0 if no meaningful gap is observed.

### Phase 1 — P1: Provenance lineage / corroboration ([#488](https://github.com/git-ksk/reasoning-harness/issues/488))

Conditional on #487. Distinguish evidence and citations from **independent originating sources**. Proven common-origin items may collapse into one corroboration origin; unknown lineage must never receive independent-support credit. Provenance relationships require Harness-owned validated inputs, not a model's guess, a URL count, a copied article or domain heuristics. Retain every original citation and do not promote corroboration to trusted verification.

### Phase 2 — P1: Version / time consistency ([#489](https://github.com/git-ksk/reasoning-harness/issues/489))

Conditional on #487, and #488 when origin lineage is required. Keep `effective_from/until`, query `as_of`, observed `retrieved_at`, publication/source version and **explicitly established supersession** distinct. A more recently retrieved record is not necessarily a more truthful record. Missing metadata, time/scope overlap or unproven supersession yields qualified disagreement or withheld judgment rather than an invented current fact.

### Phase 3 — P1: Conflict-preserving synthesis ([#490](https://github.com/git-ksk/reasoning-harness/issues/490))

Conditional on #487 and any adopted lineage/temporal controls. Combine only statements whose target/scope/time and per-binding atomic source support are demonstrably compatible; distinguish paraphrase from opposition while retaining conflicting source statements and citations separately. Advisory semantic comparison cannot erase a conflict, create provenance, strengthen claim modality, or yield `Known`/`Supported` without the existing trusted verification boundary.

### Phase 4 — P1: Answerability / bounded reacquisition ([#491](https://github.com/git-ksk/reasoning-harness/issues/491))

Conditional on #487 and any adopted source modules. An exact target's **required-information coverage** and the existing acquisition/verifier policy determine whether to (a) provide an attributable/qualified answer, (b) acquire additional read-only evidence within explicit budgets, or (c) preserve a partial/mixed/unknown result. An advisory model sufficiency label cannot relax hard verification, scope, freshness, conflict or source-identity floors. Replaying a stored answer must not redo side effects.

### Phase 5 — P0: Independent holdout and Engine source release ([#492](https://github.com/git-ksk/reasoning-harness/issues/492))

Freeze candidate/runtime identity, scoring, evaluator and thresholds, then author a *separately fresh* holdout with no reused development cases/targets/passages or exact quote windows. Run deterministic and live **Mistral, Google and Groq** acceptance independently; separate provider/protocol/quota failure from semantic scoring. Preserve any first-run FAIL immutably and use a fresh successor identity if remediation is needed.

Only after all hard gates, precommitted utility criteria and exact-head CI pass may a reviewed release PR advance `reasoning-harness-core` to **0.7.0** and publish immutable `engine-v0.7.0`. Reason CLI adoption is a separate release with independent installer/update/rollback/provenance checks.

## Release-gate scorecard

| Dimension | Acceptance requirement |
| --- | --- |
| Unsupported factual promotion | **Zero** `Known`/`Supported` without trusted existing verification |
| Provenance / corroboration | **Zero** fabricated origins, source bindings, independent-source credits or citations |
| Temporal / scope / target safety | **Zero** stale-policy bypass, unproven newest-wins, wrong-target or cross-scope conclusion |
| Conflict preservation | **Zero** material disagreements silently erased, including during paraphrase/synthesis |
| Replay and bounded resources | **Zero** replayed external side effects, unbounded acquisition or silent provider switch |
| Product usefulness | Fresh baseline-relative answerability, avoidable abstention, reacquisition and citation coverage measured separately; numeric minimums precommitted after #487 |
| Operations | Provider calls, tokens, latency, retry/quota/transport errors reported separately, never relabeled as `unknown` |
| Compatibility | Existing machine/session contract IDs unchanged or explicitly versioned, with read/rollback/replay regression |

If a proposed phase cannot pass these gates, **defer or omit it**; its existence is not a prerequisite to shipping unrelated safe improvements.

## Exclusions / coordination

Not an autonomous coding agent, browser/RAG crawler, source-credibility or model-confidence ranking, majority-vote verifier, domain-specific trusted-source list, or a silent model/provider fallback system. Historical Engine freeze tags, existing releases, and the legacy frozen MCP v1 path remain untouched. The separate [#465 CI fixture flake](https://github.com/git-ksk/reasoning-harness/issues/465) and [#375 WinGet external moderation](https://github.com/git-ksk/reasoning-harness/issues/375) are not Engine 0.7.0 blockers unless fresh evidence proves a direct release impact.

See [Product roadmap](product-roadmap.md), [Evidence qualification](evidence-qualification.md), [ADR-0005](adr/0005-target-local-evidence-need-routing.md), [ADR-0006](adr/0006-evidence-target-semantic-relevance.md), and [ADR-0007](adr/0007-source-attributed-qualified-prose.md).
