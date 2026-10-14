# Independent first-observation protocol

Author: `/root/launch_072_independent`, a separate corpus author/evaluator for Engine #520/#521. The corpus was written without reading implementation bodies, root regression tests, prior reproduction corpora/results, or earlier independent corpus contents. Only public type/function declarations, crate exports/Cargo metadata, workspace instructions, branch metadata, and candidate source hashes were inspected before freezing.

Fixed candidate: `049be6542d95539e275d9a44a5bac89835b43834`.
Fixed source: `crates/reasoning-harness-core/src/evidence_relevance.rs`.
Source SHA-256: `c9d18f43633665191b18af5c5d79cc535b7914be66e91651ea4788bd18a41736`.

The corpus contains 66 new fictional cases: 24 affirmative positive controls and 42 negative controls. All case IDs, inputs and expected v17/v30 outcomes are in `corpus.json`; its SHA-256 and counts are in `manifest.json`. `author_corpus.py` records the exact authorship specification. The corpus, protocol, manifest and API runner must be committed before compilation or first API execution. Record the resulting freeze commit in `evaluation/launch-520-521-independent-v1/freeze.json` after committing, then begin execution only after available disk capacity is confirmed.

The runner calls `derive_effective_evidence_local_qualification_v17` and `materialize_evidence_relevance_v30` independently with each case's frozen `policy`, `candidate`, exact/exact proposal and optimistic exact-target/requested-relation/no-risk raw qualification. It invokes the public APIs directly, with no model/provider.

Positive controls require effective `requested_relation` AND materialized `relevant`. Negative cases require effective relation scope other than `requested_relation` AND materialized disposition other than `relevant`. Every API error fails the case. Source-local qualification is evaluated only as relevance of the source's own asserted claim; a positive is not an assertion of externally verified hard truth.

Safety gate: zero negative RequestedRelation and zero false Relevant, with zero API errors. Positive-recall gate: zero positive misses, with zero API errors. Both gates and unchanged source hash are required for overall PASS. The 24 positive controls cover launch and GA, aliases, dotted/decimal names, neutral prose, and denial/strikethrough/context explicitly scoped to a different target. URL/navigation are neither factual proof nor governing authority. The 42 negative controls cover hypothetical/fictional/false/denied/withdrawn/retracted context across headings, titles, signals, leading/trailing sentences; original negation/strike symbols and multi-sentence strike scope; and wrong-target, conditional, rumor, question, URL/navigation-only cases.

Execution command:

```sh
CARGO_TARGET_DIR=target/engine-072-independent-v1 cargo test -p reasoning-harness-core --test launch_context_markers_independent_v1 -- --nocapture
```

Capture stdout/stderr and exit status in `evaluation/launch-520-521-independent-v1/first-run.log` and `first-run-exit-status.txt`. The runner first verifies the source hash, creates `first-run-start.json` with exclusive creation, then stores each complete input and both unmodified outputs/errors in `first-observations.jsonl` as it evaluates. It writes `first-summary.json` only after all cases. Existing evidence cannot be overwritten by this runner. It emits all cases before asserting gate failures. Verify the corpus and candidate source hash again afterward.

Initial observations are immutable evidence. Do not alter expectations, repair the corpus to convert a failure into a pass, rescore the initial run, rerun into these files, or call a retry a fresh evaluation. Preserve compile/runtime failures and partial observations if they occur. Any later diagnostic work must carry a distinct run identity and must not replace these initial results. Root owns integration and any implementation repairs. No runtime source edits, branch changes, merge, Oracle, or further delegation are permitted in this independent task.

This is an offline synthetic evaluation. It does not establish live-provider, user, device, production, or externally verified factual-truth acceptance.
