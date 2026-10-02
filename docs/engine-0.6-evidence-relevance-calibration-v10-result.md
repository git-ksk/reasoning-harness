# Engine 0.6 evidence-target relevance calibration v10 result

Status: **immutable FAIL**. Canonical run `36135286772`, attempt 1, frozen tag `engine-0.6-evidence-relevance-calibration-v10-freeze`, freeze commit `2acaf526823add813c07d96d0c258a39d864c20c`. Do not rerun, rescore, or overwrite the tag.

## Operational result

v10 fixed the v9 transport/provider failures. All three provider arms completed 56/56 with zero provider failures:

- Mistral `ministral-8b-latest`: 56/56, provider failures 0
- Groq `openai/gpt-oss-120b`: 56/56, provider failures 0
- Google `gemini-3.5-flash-lite` replication: 56/56, provider failures 0

The structured JsonSchema/JsonObject safety transport completed without the v9 raw-Text protocol abort, and Google seed normalization eliminated the v9 signed-32-bit HTTP 400 failure.

## Semantic result

Mistral improved materialized exactness from v9's 38/47 to 49/56, but still failed the required gate: unsafe negative rejection 1, unsafe positive acceptance 1, false relevance rejection 1, utility misses 7, materialized exact 49/56. Its primary binding remained weak (22/56 exact), and several failures came from a wrong primary route that never invoked the expected safety stage.

Groq was operationally complete but semantically unsafe: unsafe negative rejections 14, unsafe positive acceptance 1, wrong-target relevance retention 1, relevant left ambiguous 1, utility misses 17, materialized exact 38/56. The action wording `safe_to_reject` was too easy for this provider to select on open-world identity/ownership ambiguity.

Google replication showed the same structural problem rather than a provider-specific one: unsafe negative rejections 9, unsafe positive acceptances 2, wrong-target relevance retention 1, utility misses 9, materialized exact 46/56.

## Root cause

The v10 transport correction was successful, but the semantic architecture still allowed one model classification to control a hard final disposition too directly.

Two failure modes remained:

1. **Primary-route bypass**: an incorrect primary `target/relation` binding could bypass the safety stage entirely. The clearest example is case `55_fresh_positive_injection_ignored`, where Mistral classified an otherwise clear positive relation as `different`, and v6 deterministically materialized `irrelevant` without positive review.
2. **Action-label overreach**: a secondary model output of `safe_to_reject` or `safe_to_accept` directly enabled a hard disposition. Groq repeatedly selected `safe_to_reject` on rename/alias/successor/shared-ownership/partial-context cases that should remain ambiguous. Groq and Google also positively accepted some shared/uncertain ownership cases.

This is not solved by relaxing safety gates or by another transport tweak. The successor must make hard disposition require agreement between independent semantic signals, and the secondary stage must report evidence qualifications/risks rather than a final action.

## Successor implication

v11 will use a two-key materialization boundary:

- primary target/relation binding remains advisory and cannot hard-reject an exact-target case by itself;
- an independent structured local-qualification guard reports target-local support and explicit ambiguity risks, not `accept/reject` actions;
- `Relevant` and `Irrelevant` require agreement between the primary proposal, Harness-owned anchors, and the independent guard;
- disagreement or any identity/ownership/context risk becomes `Ambiguous`;
- exact-target `relation=different` is no longer an unconditional deterministic rejection path;
- hard safety gates remain zero-tolerance.

No independent holdout is authored from v10. Holdout authoring remains blocked until a fresh canonical successor passes.
