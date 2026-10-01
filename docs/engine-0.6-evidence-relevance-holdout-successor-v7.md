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
- successor-owned identity authority + scope-risk, materialization, correctness, and utility convergence required on both providers; relation-scope exactness remains diagnostic because v9 must preserve, not retune, the v8 relation axis

The development runner is separate from the frozen v7 holdout runner. Fresh holdout v8 runner/corpus work is not started before semantics freeze.

### Development observation 36889080034

The first reusable run at candidate commit `57b29326f7499f62e24c9d01d16074c00b6d85cf` completed both providers operationally but failed the original full-qualification gate. Mistral was 9/12 and Google 11/12 on full effective-authority qualification; both were 12/12 on materialization, had wrong-target Relevant 0 and utility misses 0, and had identity-scope misses 0 plus scope-risk misses/spurious risks 0. Every miss was relation-scope-only, and the miss sets were disjoint across providers.

That observation does not justify relation-lexicon or provider-specific tuning. Successor v7 changes only identity authority. The generic control therefore requires v9 to preserve the exact relation scope and scope risk produced by v8 for the same proposal/raw inputs while changing only the single-near-sibling identity axis. Live development convergence gates that successor-owned axis plus materialization/correctness/utility and continues to record full relation-scope mismatch metrics diagnostically. Full qualification remains part of the future fresh holdout v8 observation after semantics freeze.

### Development convergence 36890934124

A second reusable run at candidate commit `41d853080a991b3a9f3a976b758d8b338ce2c784` passed the successor-owned development gate. Both providers completed 12/12 with identity-scope misses 0, scope-risk misses/spurious risks 0, materialization 12/12, wrong-target Relevant 0, false relevance rejections 0, relevant-left-Ambiguous 0, and utility misses 0. Mistral retained 3 relation-scope-only diagnostic misses (full qualification 9/12); Google retained 1 (11/12). No production semantic change was made in response to either relation-only pattern.

The successful raw observations and v2 gate summary are captured under `fixtures/evidence-relevance-successor-v7-development/` and replayed offline. The replay verifies that v9 preserves the v8 relation scope/scope risk for the captured provider inputs, that the successor-owned identity axis matches the development labels, and that v22 materializes every case as expected. After this capture the live development workflow is manual-only again.

### Development surface independence audit

The pre-freeze audit found that development-v1 was not actually identity-fresh: `Juniper Vault`, `Ruby Queue`, and `Saffron Bridge` reused canonical entity names from prior holdouts, and the Ruby Queue task text was also reused. This does not contaminate holdout v7 because the development corpus was authored only after the immutable v7 observation and is not holdout acceptance evidence, but it contradicts the documented fresh-identity claim and is therefore not used as the final pre-freeze development surface.

A separate `evidence-relevance-successor-v7-development-v2` surface was authored before any live v2 provider observation. Its 12 cases have zero overlap against holdout v1-v7, successor-v5/v6 development, and development-v1 in case IDs, canonical entities, tasks, exact signals, and exact 8-token signal n-grams. A deterministic CI test enforces those five zero-overlap properties. v2 preserves the same generic semantic families and v9/v22 expected labels; it is a surface/provenance correction, not a production semantic change.

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
