# Issue #520 — future synthetic source gate (policy revision, 2026-10-17)

**Decision:** Drop unconditional **60/60 perfect classification** as the launch/source-qualification adoption criterion for **future independently frozen** synthetic cohorts. Retain at least 60 complete, source-blind, separately gold-labeled examples (at least 30 positives and 30 negatives), and require **both** public API axes v17 RequestedRelation and v30 Relevant independently to satisfy:

- **F1 >=0.90, precision >=0.95, recall >=0.85 simultaneously.**
- **False positive on a safety negative = 0** (retracted, explicit non-event, actual target/global denial, mixed/unknown owners): even an otherwise high F1 must not excuse one.
- **Known regression cases 100% PASS**, including original 14-case audit, and historic public API byte-for-byte compatibility; **no known unrepaired critical defect**.
- Complete first observation, source/labels/scorer frozen in advance, no operational gaps, relabel, canonical rerun or retrospective re-scoring; keep all earlier first observations immutable. The new protocol must be frozen *before* new independent case authorship.
- Report TP/FP/FN/TN, denominator, coverage, F1, precision/recall and separate operational failure for both API axes. Precision with zero predicted positives is undefined and never passes.

Unlike previous **60/60**, safe withheld positives (FN) are *bounded* by these thresholds, not an automatic failure. For 30 positive/30 negative cases, TP26/FP0/TN30/FN4 passes (F1 ~0.929, recall ~0.867), whereas v24 TP17/FP0/TN30/FN13 remains FAIL (F1 ~0.723). The v24 result is a **historical FAIL** under its *original* frozen protocol; this policy does not rewrite or retroactively pass it.

This **synthetic API** gate is distinct from #531's independently labeled **real-source 24-question final-answer quality gate** and per-provider/per-track F1/precision/recall, end-user utility, safety, usage, provenance and E2E. #540 runner safety is unchanged. The synthetic sample gives no model population guarantee, and passing this gate alone can never authorize an Engine release or CLI adoption.

Future scorer helper: `score.py`, with offline `test_score.py`. It does *not* load frozen historical archives and is not yet wired to a fully frozen evaluator.

`PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s evaluation/engine073-source-gate-v1 -p 'test_*.py' -v`
