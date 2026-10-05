# Engine 0.6 evidence relevance successor v11 development

Status: pre-observation fresh development candidate after immutable holdout-v11 FAIL.

Historical boundary:
- holdout-v11 freeze: engine-0.6-evidence-relevance-holdout-v11-freeze / 4343e7939e964e91d466a0788358733d629e92f2
- canonical run: 37182677114, immutable FAIL
- recorded result: 08885938bd989bd26ca0b3a3f193b372b149bbba
- Mistral and Google passed; Groq alone missed v11h21_ambiguous_generic_no_relation
- no rerun, rescore, relabel, or freeze-tag movement

## Candidate semantics

Effective qualification v16 derives from frozen v15.

The core rule is symmetric negative-authority hardening: advisory model output alone cannot manufacture DifferentRelation for an exact target when the Harness cannot identify an affirmative target-owned proposition about another coarse relation.

When a model-only DifferentRelation survives v15 but no Harness-owned alternate-relation cue exists, v16 demotes relation scope to Unresolved. This removes unsupported terminal negative authority without creating RequestedRelation, identity, or scope-risk authority.

Successor-only alternate-relation cue v4 retains all v3 cues and adds bounded affirmative wording needed by historical contracts without changing frozen global semantic frames:
- target-owned limited to plus a numeric value is Limit evidence when Limit is not requested;
- target-owned direct launched / launches / released / rollout wording is ChangeOrLaunch evidence when that relation is not requested.

v16 may therefore restore DifferentRelation only from Harness-owned target-local affirmative alternate-relation structure.

Materialization v29 preserves v28 full composition but feeds v16 effective identity/relation state into frozen v23. Stale advisory proposal bindings cannot recreate negative authority removed by v16.

## Historical replay

Offline replay covers:
- successor-v9 development observations;
- holdout-v10 Mistral / Google / Groq;
- successor-v10 development v2 / v3 / v4 observations;
- holdout-v11 Mistral / Google / Groq.

Observed holdout-v11 terminal behavior changes only one case:
- Groq v11h21_ambiguous_generic_no_relation: Irrelevant -> Ambiguous.

No new terminal miss appears across the replay set.

## Fresh development surface

Suite: evidence-relevance-successor-v11-development
Cases: 22
Status: fresh_independent_development
Protocol: effective qualification v16 + materialization v29
Fixed core: evidence-relevance-fixed-core-successor-v11-development-v1

Distribution:
- require_different: 8
- forbid_different: 6
- require_requested: 4
- preserve_risk: 2
- preserve_absence: 2
- final Irrelevant: 10
- final Ambiguous: 8
- final Relevant: 4

Freshness is checked against observed holdout-v1-v11 and successor-v5-v10 development surfaces by case ID, canonical entity, task, exact signal, and 8-token signal window.

Property controls require:
- all true alternate-relation cues recover DifferentRelation under conservative model output;
- all six generic/no-relation controls remain non-Different and Ambiguous even under adversarial model DifferentRelation votes;
- strict target-specific relation absence remains RelationAbsent / Irrelevant;
- requested relation positives and scope-risk controls remain unchanged.

## Provider policy

Development observation is Mistral + Google only.

Groq is excluded from candidate shaping. It may re-enter only after successor-v11 semantics are separately frozen and a new fresh independent holdout is authored.

Intended development freeze coordinate:
engine-0.6-evidence-relevance-successor-v11-development-v1-freeze
