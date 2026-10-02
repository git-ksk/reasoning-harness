# Engine 0.6 evidence relevance holdout v8

Status: runner-freeze preparation only. Successor-v7 semantics are frozen at `engine-0.6-evidence-relevance-successor-v7-semantics-freeze`; no holdout-v8 corpus has been authored or observed.

## Frozen semantic input

- successor semantics tag: `engine-0.6-evidence-relevance-successor-v7-semantics-freeze`
- tag target: `3a508bfba20386218436581dcbb69224b54948b3`
- effective qualification: v9
- materialization: v22
- historical v8/v21 predecessor functions remain frozen and unchanged

## Runner wiring

The dedicated binary is `reason-evidence-relevance-holdout-v8-study` from `crates/reasoning-harness-cli/src/bin/evidence_relevance_holdout_v8.rs`.

The V8 profile is wired before corpus authoring with:

- configuration: `evidence-relevance-live-holdout-v8`
- suite: `evidence-relevance-holdout-v8`
- annotation protocol: `evidence-relevance-effective-qualification-v9`
- fixed core: `evidence-relevance-fixed-core-v8`
- expected relative directory: `fixtures/evidence-relevance-holdout-v8`
- expected case count: 26
- checkpoint profile: holdout / scorable only when complete

The runner completeness test covers every declared profile including V8. No holdout-v8 directory exists at runner-freeze preparation time, so the corpus cannot influence runner wiring.

## Runner freeze

Freeze coordinate: `engine-0.6-evidence-relevance-holdout-v8-runner-freeze`.

Only after this annotated tag is pushed may holdout-v8 corpus authoring begin. The corpus must be authored independently and must have zero overlap in case ID, canonical entity, task, exact signal, and exact 8-token signal n-grams against holdout v1-v7 and all successor-v5/v6/v7 development surfaces. Provider observation remains prohibited until the corpus itself is frozen.
