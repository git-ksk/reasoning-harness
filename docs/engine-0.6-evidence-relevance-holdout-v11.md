# Engine 0.6 evidence relevance holdout v11

Status: runner-only preparation. Successor-v10 semantics are frozen at engine-0.6-evidence-relevance-successor-v10-semantics-freeze / 2d75c2d8f1710c0553c8b2993cad201566be7d5c. No holdout-v11 corpus exists yet and no provider observation is allowed.

## Runner binding

Dedicated binary: reason-evidence-relevance-holdout-v11-study.

V11 profile:
- configuration: evidence-relevance-live-holdout-v11
- suite: evidence-relevance-holdout-v11
- annotation protocol: evidence-relevance-effective-qualification-v15-materialization-v28
- fixed core: evidence-relevance-fixed-core-v11
- expected directory: fixtures/evidence-relevance-holdout-v11
- expected cases: 26
- issue: #468
- effective qualification: v15
- materialization: v28

Historical v1-v10 and reusable development profiles remain available for checkpoint/replay compatibility with their original semantic bindings unchanged.

## Ordering constraint

Runner freeze coordinate: engine-0.6-evidence-relevance-holdout-v11-runner-freeze.

Only after that annotated tag is pushed may fixtures/evidence-relevance-holdout-v11 be created. The fresh corpus must be independently authored after runner freeze, with no reuse of observed holdout-v1-v10 or successor-development case IDs, entities, tasks, exact signals, or 8-token windows. Development-v4 is development evidence only, not acceptance evidence.

The later one-shot acceptance surface must restore Mistral, Google, and Groq.
