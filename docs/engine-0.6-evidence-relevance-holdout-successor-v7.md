# Engine 0.6 evidence relevance holdout successor v7

Status: pre-freeze successor candidate after immutable holdout v7 FAIL. Effective qualification v9 + materialization v22 are under generic-control, immutable-replay, and two-provider development validation. Fresh holdout v8 is prohibited until the successor-v7 semantics freeze is committed and tagged.

## Observed boundary

Canonical holdout v7 run `36800959088` attempt 1 completed all 26 cases on Mistral, Google, and Groq with provider failures 0 and wrong-target Relevant 0. The required correctness gate passed. All three providers were 25/26 on effective authority qualification. Mistral and Google materialized 26/26; Groq materialized 25/26 and had one utility miss.

All three authority misses were the same fresh family: URL-only Harness target identity plus one substantive near-sibling signal. The candidate relation was semantically availability but expressed without the small deterministic availability lexicon. Mistral and Google remained Ambiguous through other conservative paths; Groq had `different + distinct_target + no risk` and fell through to historical negative-target materialization.

The v7 run, tag, labels, and artifacts are immutable. They are replay evidence only.

## Generic root cause

The existing single-near-sibling identity floor was gated by `requested_relation_locally_present()`. That predicate is a bounded lexical helper for the relation axis. Consequently, an identity-ownership rule could disappear when the same requested relation was paraphrased outside that lexical set.

This is cross-axis coupling:

- whether one near-sibling signal is sufficient identity authority is an identity question;
- whether the substantive proposition expresses the requested relation is a separate relation question;
- model/provider disagreement can expose the coupling but is not the source of authority.

The repair must not add holdout wording, provider names, fixture IDs, entities, or case families to production logic.

## Versioning decision

- effective qualification: introduce v9
- materialization: introduce v22
- v8/v21 implementations remain unchanged
- local qualification model contract remains v8; no additional model stage is introduced
- historical v1-v7 observations and result artifacts remain unchanged

## Successor-v7 identity floor

For strict `require_harness_anchor` policy, a single substantive near-sibling identity signal is insufficient to create distinct-target authority when no content-bearing Harness target anchor exists.

The floor is relation-orthogonal. It applies whether the requested relation is lexically detected, model-classified as requested, classified as different, or remains unresolved. v9 changes only the identity axis to `unresolved`; it preserves the relation scope produced by the prior semantics.

The floor does not apply when stronger Harness-owned distinct-identity evidence exists, including explicit separate/distinct/replacement/successor evidence recognized by deterministic controls.

URL-only and navigation/footer-only target occurrences remain non-owning context.

## Materialization boundary

v22 derives v9 first. If the relation-orthogonal single-near-sibling identity floor is active, v22 materializes Ambiguous before delegating to v21. This prevents historical negative paths from converting advisory `different/distinct_target` agreement into Irrelevant when Harness-owned identity authority remains unresolved.

Otherwise v22 delegates to v21 unchanged.

## Generic controls

The pre-freeze controls require:

- URL-only target + one near sibling + non-lexical relation paraphrase => identity Unresolved, final Ambiguous;
- navigation-only target + one near sibling + non-lexical relation paraphrase => Unresolved / Ambiguous;
- no target anchor + one near sibling + non-lexical relation paraphrase => Unresolved / Ambiguous;
- relation scope is preserved independently; the identity floor must not manufacture `requested_relation`;
- explicit distinct identity evidence remains DistinctTarget;
- a content-bearing exact target anchor is not downgraded;
- prior positive, repeated-sibling negative, mapping/ownership risk, absence, injection, and contradiction behavior remains unchanged through immutable replay.

No production branch may key on provider, fixture ID, synthetic entity, family, or exact holdout text.

## Immutable replay

Successor-v7 replay currently covers:

- v23 fixed regression surface;
- holdout v1;
- holdout v2;
- holdout v3a;
- holdout v4;
- holdout v5;
- immutable holdout v7 canonical observations;
- captured successor-v6 Mistral + Google development observations.

The historical Groq v7h25 disposition remains Irrelevant in the captured v7 replay fixture. Under v9/v22 only, the same immutable proposal/raw observation derives the frozen expected authority and Ambiguous materialization. Wrong-target Relevant remains 0.

## Development policy

Before freeze, a fresh reusable development profile is used only for candidate shaping:

- Mistral `ministral-8b-latest`
- Google `gemini-3.5-flash-lite`
- Groq excluded from iterative development
- 12 synthetic non-holdout cases with fresh identities, including relation-paraphrase single-sibling boundaries and orthogonal relation controls
- exact effective-authority, materialization, correctness, and utility convergence required on both providers

The development runner is separate from the frozen v7 holdout runner. Fresh holdout v8 runner/corpus work is not started before semantics freeze.

## Freeze blockers

Do not freeze successor-v7 semantics until all are green:

- successor-v7 generic controls;
- development-label deterministic self-consistency;
- immutable replay through holdout v7;
- wrong-target Relevant 0 throughout replay;
- Mistral + Google development convergence;
- provider/fixture/entity/text special-case scan;
- relation/identity orthogonality audit;
- repeated-sibling and explicit-distinct regression audit;
- mapping, ownership, truncation, and context-gap audit;
- prompt-injection and contradictory-local-evidence audit;
- core and affected CLI tests;
- all-target Clippy with `-D warnings`;
- rustfmt;
- workflow YAML validation;
- semantics checksum.

Only after the semantics freeze tag is pushed may a fresh independent holdout v8 be authored.
