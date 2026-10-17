# Independent candidate quality thresholds (draft protocol; not a release result)

Issue [#531](https://github.com/git-ksk/reasoning-harness/issues/531).
Authoritative draft values are in [engine-07-realworld-protocol-v1.json](../engine-07-realworld-protocol-v1.json) under `gates.candidate_quality`.

For **each provider, each track**, and the **candidate arm only**, require simultaneously:

- positive-class Precision >= 0.95, Recall >= 0.85, F1 >= 0.90;
- a complete, independently labeled cohort of 24 with at least 8 positive and 1 negative clear examples;
- no operationally incomplete, missing, duplicate or unlabeled observations;
- no hard safety violation, no known unrepaired defect, no positive-control regression,
  and all *other* pre-existing paired 0.6.1, product-utility, budget and release gates.

F0.5 >= 0.95 is a **stretch diagnostic**, not an acceptance gate.
0.95 precision and 0.85 recall alone yield F1 ~= 0.8972, so **all three
thresholds must pass independently**. Precision undefined (no predicted accepts)
cannot pass. A correctly supported positive withheld as Unknown/abstain is FN.
Operational failures are incomplete, not TN or successful abstention.
Pre-labeled ambiguous cases are excluded from the clear binary confusion matrix,
but an unsafe assertion on one is a separate **hard failure**.
Do not collapse ambiguous into negative to inflate apparent performance.

Label authority is outside the candidate model and fixed before first observation.
For `e2e_normal`, positive means a *complete, source-grounded, actually answerable*
user-visible answer. For `helper_opt_in`, positive means a separately authorized,
source-scoped target display; no opt-in gain proves normal CLI usefulness.
Diagnostic-only prebinding cases are reported separately, **not** silently excluded
from a declared cohort. The scorer is a small pure, **future-run integration
building block**, not yet wired to the full live provider runner. Its
`metric_gate_pass` never authorizes a source release or CLI adoption on its own.

These are project-specific engineering thresholds, **not** a universal ML
quality standard or population-level statistical guarantee. The 24-case
denominator is small; publish exact counts and, where applicable, descriptive
uncertainty intervals, not a claim that real-world precision is at least 95%.
Never retroactively relabel, rescore, rerun, or overwrite v19-v22 or other
frozen first observations. A new candidate requires newly frozen evaluators,
a disjoint independent cohort and full existing product acceptance.

Offline check:
`python3 -m unittest discover -s evaluation/engine07-quality-gates-v1 -p 'test_*.py' -v`

Standard formula reference: https://scikit-learn.org/stable/modules/generated/sklearn.metrics.f1_score.html
