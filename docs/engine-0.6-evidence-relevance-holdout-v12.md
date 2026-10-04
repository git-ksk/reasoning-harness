# Engine 0.6 evidence relevance holdout v12

Status: runner-only preparation. Successor-v11 semantics are frozen at `engine-0.6-evidence-relevance-successor-v11-semantics-freeze` / `d8d9459d9bd225a16442e0ce32de55aa7563a08f`. No holdout-v12 corpus exists yet and no provider observation is allowed.

## Runner binding

Dedicated binary: `reason-evidence-relevance-holdout-v12-study`.

V12 profile:
- configuration: `evidence-relevance-live-holdout-v12`
- suite: `evidence-relevance-holdout-v12`
- annotation protocol: `evidence-relevance-effective-qualification-v17-materialization-v30`
- fixed core: `evidence-relevance-fixed-core-v12`
- expected directory: `fixtures/evidence-relevance-holdout-v12`
- expected cases: 26
- issue: #468
- effective qualification: v17
- materialization: v30

Historical v1-v11 and reusable development profiles retain their original semantic bindings. A runner regression explicitly verifies that V11 preserves the old ExactTarget state for the immutable development-v1 omitted-ownership case while V12 applies the frozen v17 Unresolved identity floor and v30 remains fail-closed Ambiguous.

## Ordering constraint

Runner freeze coordinate: `engine-0.6-evidence-relevance-holdout-v12-runner-freeze`.

Only after exact-head CI passes and that annotated tag is pushed may `fixtures/evidence-relevance-holdout-v12` be created. The fresh corpus must be independently authored after runner freeze, with no reuse of observed holdout-v1-v11 or successor-development case IDs, canonical entities, tasks, exact signals, or 8-token windows.

The later one-shot acceptance surface must require Mistral, Google, and Groq. Groq is not to be invoked during runner preparation or corpus authoring.
