# Engine 0.6 evidence relevance holdout v9

Status: runner-freeze preparation only. Successor-v9 semantics are frozen at `engine-0.6-evidence-relevance-successor-v9-semantics-freeze`; no holdout-v9 corpus has been authored or observed.

## Frozen semantic input

- successor semantics tag: `engine-0.6-evidence-relevance-successor-v9-semantics-freeze`
- tag target: `3e49a9fd2f7827b7c707516d2150e2f0f16e9e17`
- effective qualification: v11
- materialization: v23
- historical v9/v22 predecessor functions remain frozen and unchanged

## Runner wiring

The dedicated binary is `reason-evidence-relevance-holdout-v9-study` from `crates/reasoning-harness-cli/src/bin/evidence_relevance_holdout_v9.rs`.

The V9 profile is wired before corpus authoring with:

- configuration: `evidence-relevance-live-holdout-v9`
- suite: `evidence-relevance-holdout-v9`
- annotation protocol: `evidence-relevance-effective-qualification-v11`
- fixed core: `evidence-relevance-fixed-core-v9`
- expected relative directory: `fixtures/evidence-relevance-holdout-v9`
- expected case count: 26
- checkpoint profile: holdout / scorable only when complete
- effective qualification: v11
- materialization: v23

The runner completeness test covers every declared profile including V9. No `fixtures/evidence-relevance-holdout-v9` directory exists at runner-freeze preparation time, so the fresh corpus cannot influence runner wiring.

## Runner freeze

Freeze coordinate: `engine-0.6-evidence-relevance-holdout-v9-runner-freeze`.

Only after this annotated tag is pushed may holdout-v9 corpus authoring begin. The corpus must be independently authored with zero overlap in case ID, canonical entity, task, exact signal, and exact 8-token candidate-signal n-grams against holdout v1-v8 and all successor-v5/v6/v7/v8/v9 development surfaces. Provider observation remains prohibited until the corpus itself is frozen.
