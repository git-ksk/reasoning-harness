# Preparation contract

`protocol.json` describes the intended consumed-set paired development measurement. Its current state is `PREPARATION_NO_LIVE_ALLOWED`. There is no new F1 result, M0 receipt, release acceptance, provider-readiness approval or fresh independent corpus in this checkpoint.

The registry is constructed before execution from the captured documents' host-curated facts and admission metadata. It does not use gold labels, old answers, reviewer annotations or model outputs. An unacquired candidate stays absent from the final citations. Acquisition alone is insufficient: an exposed citation needs the selected proposition, actual admitted evidence, supported claim and matching verification receipt. Missing or altered referenced evidence fails closed.

The adapter validates the actual renderer/recovery → answer-safety → source-context → acquisition-coverage chain. It binds displayed fact and JSON-encoded quote spans as two distinct citation attachments to the same source. No broad span connects unrelated text. Full displayed bytes, including factual content within quotes, remain subject to independent semantic annotation and the unchanged scorer.

Acquisition selectors bind to the admitted target and capability, rather than assuming every requested facet equals the corpus's primary lookup key. Captured-origin manifest facts are reconstructed from the supplied snapshots and never treated as independent external citations. Absent citations continue to reduce citation coverage; the adapter does not invent them.

Absolute quality gates remain case and claim F1 ≥ 0.90, precision ≥ 0.95, recall ≥ 0.85; citation precision/coverage/fidelity 1.0; hard safety violations zero; no positive-control regression; net useful-answer gain ≥ 2 in each track/provider. Mean logical calls may increase by at most 1, aggregate tokens by at most 1.25× and p95 wall time by at most 1.5×. Incomplete or undefined metrics never pass. Consumed-set PASS is still not fresh acceptance.

The display adapter validates legal status transitions and preserves hard claim counts, coverage and uncovered propositions. A withheld hard result cannot gain source citations or factual text. Coverage may add only its exact diagnostic template while retaining the blocked status. The original hard result and both projection views remain in observations and independent review packets. The existing inert acquisition control now omits the planner key so both actual CLI hosts exercise the production single-key selector rule.
