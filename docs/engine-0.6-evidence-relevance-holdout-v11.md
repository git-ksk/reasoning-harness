# Engine 0.6 evidence relevance holdout v11

Status: fresh independent acceptance corpus prepared and still unobserved.

Successor-v10 semantics are frozen at `engine-0.6-evidence-relevance-successor-v10-semantics-freeze` / `2d75c2d8f1710c0553c8b2993cad201566be7d5c`. The dedicated runner is frozen at `engine-0.6-evidence-relevance-holdout-v11-runner-freeze` / `63e3e1bc5a319d2dccbbbaf2fcf643bc3a32dba5`.

No holdout-v11 provider observation has occurred.

## Frozen profile

- binary: `reason-evidence-relevance-holdout-v11-study`
- configuration: `evidence-relevance-live-holdout-v11`
- suite: `evidence-relevance-holdout-v11`
- annotation protocol: `evidence-relevance-effective-qualification-v15-materialization-v28`
- fixed core: `evidence-relevance-fixed-core-v11`
- directory: `fixtures/evidence-relevance-holdout-v11`
- cases: 26
- issue: #468
- effective qualification: v15
- materialization: v28
- seed: 4681110

Historical V1-V10 bindings remain unchanged.

## Fresh corpus

The corpus was authored only after the runner-freeze tag existed.

Distribution:
- Relevant: 8
- Irrelevant: 10
- Ambiguous: 8

Authority modes:
- require_different: 9
- require_requested: 8
- forbid_different: 4
- preserve_risk: 4
- preserve_absence: 1

Freshness is enforced against observed holdout-v1-v10 and successor development surfaces through case ID, canonical entity, task, exact signal, and 8-token signal-window non-reuse.

Offline v15/v28 property tests cover:
- bounded negative relation recovery under conservative model outputs;
- requested-relation recovery;
- instruction/control-schema resistance to adversarial DifferentRelation votes;
- strict requested-relation absence preservation;
- exact expected materialization under the precommitted labels.

## Three-provider acceptance gate

The one-shot canonical acceptance surface restores all three required providers:
- Mistral `ministral-8b-latest`
- Google `gemini-3.5-flash-lite`
- Groq `openai/gpt-oss-120b`

Every required arm must complete all 26 cases with:
- provider/protocol failures 0;
- provider-arm latch absent;
- exact Harness-owned identity/risk constraints;
- require_different -> different_relation;
- require_requested -> requested_relation;
- forbid_different -> not different_relation;
- preserve_absence -> relation_absent;
- preserve_risk -> frozen risk preserved;
- final v28 materialization 26/26 exact;
- wrong-target Relevant 0;
- false relevance rejection 0;
- relevant-left-Ambiguous 0;
- utility miss 0.

A single required-arm miss makes the immutable attempt FAIL. No rerun, rescore, relabel, or tag move is allowed.

## Groq admission

Groq is re-anchored from canonical holdout-v10 run `37085574010`:
- observed Groq tokens: 65,889
- Groq arm completed: `2026-10-03T03:31:56Z`
- prior configured start headroom: 55,000
- prior pacing: 300,000 ms inter-case + 10,000 ms minimum request interval
- v11 required start headroom: 55,000
- v11 self-budget: 70,000
- v11 pacing: 300,000 ms inter-case + 10,000 ms minimum request interval
- conservative not-before floor: `2026-10-03T09:17:01Z`

The workflow fails closed if the modeled floor is not satisfied.

## Freeze ordering

The corpus/workflow/test surface must pass checksum, validate-only, freshness, fmt, Clippy `-D warnings`, v15/v28 replay/regression tests, workflow YAML parse, and exact-head GitHub CI before the annotated tag `engine-0.6-evidence-relevance-holdout-v11-freeze` is pushed.

Tag push triggers the first and only canonical provider observation.
