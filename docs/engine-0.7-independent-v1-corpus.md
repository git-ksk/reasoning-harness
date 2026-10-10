# Engine 0.7.0 — Independent v1 frozen corpus

**Runner frozen before corpus creation:** `engine-0.7-independent-v1-runner-freeze` at `1124e5a7c34e42c088d81908c3cdc75aae2db309`.

This freshly authored, independent evaluation corpus was created **after** the runner, scorecard, threshold protocol, and CI were merged and tagged. It has twelve brand-new cases covering fifteen target relations. Frozen novelty verification rejected reuse of case, target, or source IDs, or eight-consecutive-word quote windows from existing fixtures and runner development tests.

- Six explicitly compatible target pairs
- Eight opposed, unknown, scope/version-mismatched targets
- One identical-quotation control that needs no advisory call
- Numeric, polarity, qualifier, time, context and injection negatives, plus a mixed target case

The three provider arms are validated without model calls. Each must gain at least one **source-local, attributed** compatible target against unchanged Engine 0.6.1 while preserving every legacy citation and conflict and triggering no hard violations or provider failures. All sources refer to fictitious entities: these results validate controlled semantic behavior, **not verified external factual truth** or measured real-user task value. An advisory model's label is not itself authorization.

The canonical GitHub tag `engine-0.7-independent-v1-corpus-freeze` invokes the first real provider observations. The corpus and the first FAIL or PASS are immutable, and any later candidate must use a distinct successor identity.

[Acceptance issue #492](https://github.com/git-ksk/reasoning-harness/issues/492)
