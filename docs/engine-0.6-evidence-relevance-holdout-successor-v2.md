# Engine 0.6 evidence relevance holdout successor v2 design

Status: successor semantics implemented and replay-validated; fresh independent holdout not yet frozen or observed.

This successor exists because immutable independent holdout v1 failed the zero-wrong-target hard gate. Holdout v1 remains immutable FAIL and is not a tuning surface.

## Versioned successor

- effective qualification: **v4**
- materialization: **v17**
- predecessor calibration semantics: effective qualification v3 + materialization v16
- issue: #462

No provider name, fixture ID, synthetic entity name, or holdout-specific case branch is present in the production rule.

## Authority precedence

The v1 failure exposed an identity-authority precedence defect.

A Harness-owned canonical/alias occurrence proves that the name occurs in candidate material. It does not prove proposition ownership.

v4 therefore applies these rules:

1. Deterministic scope-risk blockers remain fail-closed and unchanged.
2. If advisory target binding is Exact but the independent raw verifier says `distinct_target` with no scope risk, a lexical Harness anchor cannot upgrade the conflict to `exact_target`.
3. If Harness-owned target-local syntax shows that every substantive target-name occurrence is context-only/comparative, the conflict is corroborated as `distinct_target`; otherwise identity becomes `unresolved`.
4. Even when both model outputs say exact, a target name that appears only in a bounded context-only frame cannot create exact-target authority; identity becomes `unresolved`.
5. Relation remains orthogonal. Presence of pricing/availability/limit/etc. language cannot establish which entity owns that relation.

Current deterministic context-only frames are narrowly target-local: `unlike TARGET`, `versus TARGET`, `vs TARGET`, `not TARGET`, `compared to/with TARGET`, `rather than TARGET`, `in contrast to TARGET`, and `as opposed to TARGET`. Canonical names and aliases are normalized as identity phrases, longest spans are evaluated first, and fully overlapped shorter aliases are deduplicated; a later non-overlapped substantive alias occurrence still prevents context-only classification.

This cue is negative/abstention authority only. It never creates positive relevance.

## Materialization v17

v17 intercepts only positive identity-authority conflicts where the advisory proposal claims exact target identity.

- raw `distinct_target` + deterministic target-local context-only corroboration -> Irrelevant
- exact/raw-distinct conflict without deterministic corroboration -> Ambiguous
- both model outputs exact but target occurrence is context-only -> Ambiguous
- existing proposal=Different negative paths remain delegated to v16 and are not weakened
- ordinary exact-target positive material remains delegated to v16

The hard gates are unchanged:

- wrong-target relevance retention = 0
- false relevance rejection
- Relevant utility
- materialization exactness
- authority qualification
- operational failures separated from semantic failures

## Regression evidence

Canonical-artifact replay fixtures are immutable-observation derivatives, not new observations and not rescoring of the frozen runs.

- v23 canonical run `36400085595`: 144/144 provider observations preserve expected authority/materialization under v4/v17
- holdout v1 run `36495389012`: Groq remains 26/26; Mistral remains 24/26 with zero wrong-target Relevant; Google becomes 25/26 with the former h14 wrong-target Relevant safely rejected
- focused generic unit tests cover corroborated comparison-only conflict, uncorroborated exact/raw-distinct conflict, two-exact context-only conflict, ordinary positive preservation, and unrelated comparison-marker non-interference

## Fresh successor holdout rule

A new independent holdout must be authored only after these successor semantics are fixed.

It must not copy the v1 failed case, entity names, candidate text, or a superficial h14 paraphrase merely to test the repair. It must use a new suite/surface identity and independently cover positive ownership, context-only mentions, sibling/comparison structures, relation ownership, ambiguous identity, aliases, distributed evidence, and non-comparison controls.

Holdout v1 observations may be used only as postmortem replay evidence. The fresh successor holdout must independently satisfy the original zero-wrong-target hard gate before #462 can close.

## Groq operational admission

Before any successor live freeze:

- recompute modeled TPD headroom from the actual 65,244-token Groq holdout-v1 consumption and the observation timestamps;
- treat tiny readiness as transport/credential/TPM/RPD evidence only, never TPD-headroom proof;
- precommit a fail-closed TPD admission floor, pacing, observed-token cap, and reserve;
- invalidate/re-anchor the model for known or suspected material organization-level Groq use;
- keep operational failure separate from semantic failure.

No successor live freeze may occur until this admission calculation, local validation, exact surface checksum, public-safety scan, and PR CI are green.
