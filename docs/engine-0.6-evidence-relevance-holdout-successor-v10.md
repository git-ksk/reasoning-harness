# Engine 0.6 evidence relevance holdout successor v10

Status: pre-freeze successor after immutable holdout-v10 FAIL and successor-v10 development v1/v2/v3 FAIL followed by development v4 PASS. Candidate semantics are effective qualification v15 plus materialization v28. Fresh acceptance-holdout authoring is prohibited until the successor-v10 semantics-freeze tag exists.

## Historical boundary

Canonical holdout-v10 run 37085574010 remains immutable FAIL under frozen v11/v23. Successor-v10 development v1 run 37097092197, v2 run 37100490212, and v3 run 37105677785 remain immutable FAIL development evidence. None is rerun, rescored, relabelled, or used as acceptance evidence.

v4 repairs only the pre-adjudicated successor gaps while preserving historical boundaries:
- canonical Groq v10h18: Ambiguous -> Irrelevant;
- v2 Mistral sv10v2_18: Irrelevant -> Relevant;
- v3 Mistral sv10v3_08: Ambiguous -> Irrelevant;
- v3 Mistral sv10v3_20: Irrelevant -> Ambiguous;
- the available v3 Google partial direct-definition miss is corrected without unrelated terminal changes.

Frozen v11, v14, v23, and v27 remain unchanged.

## Effective qualification v15

v15 derives from v14 and makes two bounded successor-only changes.

First, model-only DifferentRelation is demoted to Unresolved for exact-target, risk-none candidates containing untrusted instruction/control text when there is no independent clean Harness-owned negative-relation cue, no Harness-owned requested-relation authority, and no strict absence rule. Control-schema instructions that target advisory labels are included in this one-sided safety floor. This path can remove model-only negative authority but cannot create RequestedRelation or DifferentRelation authority.

Second, v15 recognizes direct target-owned Definition wording such as "is defined as" / "defined as" as bounded other-relation evidence when the requested relation is not Definition and the inherited exact-target, no-risk, local-authority restrictions hold. Comparison-only, other-entity, context-gap, strict-absence, and instruction-shaped content remain fail-closed.

## Materialization v28

v28 derives v15 and synchronizes both advisory proposal bindings and local qualification to the Harness-owned effective state before delegating through frozen v23 behavior. It prevents stale raw proposal fields from recreating relation or identity authority already corrected by v15.

v28 does not independently invent requested-relation or different-relation authority.

## Fresh development v4 evidence

The 24-case development-v4 surface was authored before v4 provider observation and has zero case ID, entity, task, exact-signal, or 8-token-window reuse against the 80 previously observed holdout-v10 / development-v1 / development-v2 / development-v3 cases.

One-shot run 37111376390 from annotated tag engine-0.6-evidence-relevance-successor-v10-development-v4-freeze at d9a5bd49d3c6f3ee5e79f9ba6704f0dc502a00c0 is immutable PASS development evidence.

- Mistral ministral-8b-latest: 24/24 complete, authority / identity-risk / materialization failures 0, effective v15 exact 24/24, final v28 exact 24/24.
- Google gemini-3.5-flash-lite: 24/24 complete, authority / identity-risk / materialization failures 0, effective v15 exact 23/24, final v28 exact 24/24.
- Groq was not observed and remains reserved for the fresh independent acceptance holdout.
- holdout_acceptance_evidence=false.

The run and summary are captured under fixtures/evidence-relevance-successor-v10-development-v4-result/.

## Pre-freeze audit

At development-evidence commit a661ba648aba3573bbcf01d2efb05ff4675d0179:
- exact-head GitHub CI is 8/8 PASS;
- the working tree was clean and branch HEAD matched origin;
- semantic candidate files are byte-for-byte unchanged from the observed v4 freeze commit d9a5bd49d3c6f3ee5e79f9ba6704f0dc502a00c0;
- v4 validate-only reports 24 planned / 0 completed and is non-scorable;
- checksum, fmt, core/CLI Clippy -D warnings, core library tests, v4/v28 tests, historical replay, successor-v9, holdout-v10, and predecessor regression suites are green;
- production semantic source/export scan contains no v4 case IDs or synthetic entity literals.

The semantics-freeze commit may add only freeze evidence/documentation. No behavior code or observed development surface may change.

Intended freeze coordinate: engine-0.6-evidence-relevance-successor-v10-semantics-freeze.

Only after that annotated tag is pushed may a fresh independent acceptance holdout runner/corpus be authored. That holdout must not reuse observed development surfaces and must restore Groq alongside Mistral and Google.
