# Native #582 local acceptance — 2026-11-03 JST

Baseline: `76cd4f61dfc5c1fcc949920802ba281b2aed5461`. Implementing-agent source
review and local engineering acceptance of the
[native admission profile](NATIVE_ADMISSION.md); no independent acceptance or
new model-quality result is claimed.

- Mac Python 3.13 and 3.14 each pass **167/167**, including **22 new controls**.
- Nine new actual CLI controls include prospective first-request planning,
  full 48-child loopback execution, every pre-advance record, missing-record
  first-score refusal and once-only first-score storage. Safe, fully accounted
  local decode failures can advance only after a signed stream/source decision.
- Actual stderr cannot be changed after review. Unsafe decisions, omitted/
  changed streams, source changes, stale/revoked/conflicting keys and forged
  JSON safety acknowledgments halt before another child. Mutating returned
  packet data cannot change private pending state.
- Source/binary/runtime content and inventory replacement, source version/
  evidence/quote mismatches, unbound facts, incomplete UTF-8 byte review,
  stream overflow and a child lingering after closing its pipes are rejected.
- CLI unit tests **276 passed / 4 existing ignored**; strict Clippy,
  formatting, syntax and diff checks pass. Core contracts and historical
  v1/v2 raw/receipt/tag/first-score data are unchanged.

The [machine-readable record](native-admission-acceptance-20261103.json) binds
tested implementation and test-log/executable hashes. Private bundle manifests,
catalogs, raw streams, signatures and source packets are not published.

## Findings corrected

Real CLI structured-output fallback exposed a collector error hidden by the
synthetic fixture: `UsageReport.model_calls` counts a generation/correlation
once, while fallback adds another adapter/HTTP request under that correlation.
Previously the collector required equality with all `model_request` events and
rejected fully accounted decode failures and recovered trailing-output cases.
It now requires equality with distinct correlations. Physical HTTP identities,
counts and usage are still checked exactly; the counterexample of two distinct
correlations falsely reporting one call still fails. Only prospective v3
synthetic fixture metadata was corrected; old observations are not rescored.

Native admission is required by a **dedicated native receipt**, preventing an
alternate legacy reservation/scoring path from silently skipping the gate.
Verified admission capabilities carry exact immutable execution bindings.
Full stdout/stderr packets, source/native identities and ordered admission
records are rechecked before advancement and first-score creation.

## What these controls do not prove

The 48 children use one already-known synthetic question, local HTTP responses
and test reviewer keys. Their repeated execution tests ordering, capture,
accounting, signatures and replay protection, not independent answer accuracy.
They do not perform real reviewer enrollment, a source-reviewed production
bundle admission, arbitrary Web-source capture or detached-child termination.

No provider API calls, cloud resource operations, new real F1, model-quality GO,
merge, release or CLI adoption were performed. The real operator catalog,
reviewer/source enrollment and semantic acceptance remain separate from this
engineering result. `quality_go=false`.
