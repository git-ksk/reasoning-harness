# Native #540 engineering acceptance — 2026-11-03 JST

Base: `78903ed085477a0d6805be85ac45cf8338d8e2d7`. Implementing-agent source
review and Mac local acceptance of [process supervision](PROCESS_SUPERVISION.md).
Real operator/source/reviewer/provider acceptance remains separate.

- Python 3.13 / 3.14: **190/190 each**, no skips, including **19 new controls**.
- CLI unit: **278 PASS / 4 existing ignored**, including two new supervision
  controls. The isolated Mac test proves both `fork` and `posix_spawn` denied.
- Strict workspace/all-target Clippy, formatting, Python AST, workflow YAML
  syntax and diff checks pass. The full 48 actual native loopback children and
  signed pre-advance/first-score path remain covered on both Python versions.
- Real SIGKILL of the controller stops its native CLI during an outstanding
  loopback HTTP request. Reserved/inflight records remain; no completed record
  or second HTTP request is created. SIGTERM exercises cooperative cancellation
  and retains partial streams and a reaped-process record.
- Timeout first requests SIGINT and retains output emitted during grace. A
  child ignoring it is killed after the bounded grace. Explicit cancellation
  during actual HTTP and pending review blocks further jobs without refunds.
- The native lifeline requires a private socket pair and the exact handshake.
  Controller loss before GO and invalid acknowledgment permit no request plan.
  Failure to persist supervision identity prevents GO and produces zero HTTP.
- Stop/reserve races are serialized, retaining the exact reservation charge.
  Stop-write failure latches locally. Durable stop markers reject first scores;
  completion is a terminal transition. Changed supervision evidence cannot
  authorize advancement.

## Reproduced defect and corrections

Before this change, a real child created a detached sleeping grandchild and
exited zero. The native capture reported `streams_complete=true` while that
grandchild was still alive; the control explicitly cleaned it up. Group signals
alone cannot establish descendant containment. The original uncontained
SIGKILL regression is preserved, not rewritten into a pass.

The new Mac profile prevents descendant creation and uses a controller-owned
lifeline, with launch identity persisted before GO. During source review a
separate reserve/stop race was found: cancellation could observe the in-memory
charge before an acknowledged reservation was reflected there. Serialized
state transitions and a delayed-store/two-thread regression close that gap.

The first SIGTERM control lost its test ledger because TemporaryDirectory
cleanup ran on graceful exit. That fixture was corrected to retain the
operator-owned ledger across controller exit; production ledger protections
were not weakened. The corrected control passes on both Python versions.

## Admission limits

This is new native bundle **v2**, requiring a newly enrolled bundle/plan/receipt
and common CLI builds supporting the lifeline on both Engine arms. The Mac
profile is single-process curated-source evaluation. External resolver/MCP
stdio `program` settings are rejected before planning. Arbitrary subprocess
workloads are not admitted. Linux controls do not prove the Mac no-fork policy.
Mac arm64 and x86_64 CI now run the actual OS restriction and full local suite.
If macOS's deprecated sandbox API is unavailable, execution is refused.

No remote provider request, cloud resource operation, real F1, quality GO,
Engine release or CLI adoption was performed. Local process termination does
not cancel work already accepted by a remote provider or prove billing. Unknown
usage retains the full reserve. Same-UID/root/runtime compromise, host power
loss and independent semantic acceptance are not proved. Historical v1/v2
raw/receipts/tags/first scores and Core are unchanged.

[Source/native/log hashes](native-supervision-acceptance-20261103.json).
