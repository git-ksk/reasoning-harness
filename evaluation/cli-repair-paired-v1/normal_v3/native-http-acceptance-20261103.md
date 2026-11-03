# Mac native HTTP acceptance — 2026-11-03 JST

This is the next #581 engineering step after local-authority commit `ac10f1e`.
The real Mistral CLI transport now supplies the journal, instead of trusting a
caller-generated `http.json`. See [data path and limits](NATIVE_HTTP.md).

## Verified on Mac

- Python 3.13 and 3.14 each pass **145/145** tests with
  `REASON_V3_NATIVE_CLI` set to the freshly built CLI. The 11 new controls
  include **six actual CLI subprocess controls**, with a local HTTP server,
  fake key, isolated configuration/environment and no provider calls.
- Native subprocesses bind actual request/response hashes and trace identity
  to the fixed local authority. Changed CLI totals cannot override measured
  transport totals. Unknown usage and SIGKILL retain the full reservation and
  suppress advancement. Nonempty/read-only FDs and external control URLs send
  no requests; numeric loopback controls bypass ambient proxies.
- Four new Rust transport controls cover exact wire hashes/reserve exhaustion,
  missing/inconsistent usage, malformed/wrong-model/non-2xx responses, I/O
  failure before send, and cancellation after response during pacing.
- CLI unit suite: **276 passed, 4 existing ignored**. Provider unit suite:
  **170 passed, 1 existing ignored**; provider integration: **1 passed,
  1 existing ignored**. Strict Clippy, formatting and diff checks pass.
- Journal validation rejects changed/partial/reordered/duplicate/hash-chain
  evidence, stale first requests, wrong bindings, internal retries, missing
  response/completion and unknown usage. Every accepted capture freezes
  journal/trace/normalized HTTP hashes and measured totals in the local ledger.

[The machine-readable record](native-http-acceptance-20261103.json) identifies
the tested source, native executable and test-log hashes. Previous 118-test
signature and 134-test local-store acceptance records remain unchanged.

## Review findings retained

The test-server implementation initially inherited nonblocking accepted sockets
on Mac; explicitly using blocking accepted streams corrected the test harness.
A trailing response needs a subsequent completion marker to detect a modified
or missing final record. Cancellation after a response but before pacing ends
also needs to burn the call. Both controls are now part of the tested path.
Loopback controls explicitly disable ambient proxies.

## Remaining

This is implementing-agent review/local engineering acceptance, not an
independent source or semantic review. The test cohort uses a synthetic request
template from an earlier loopback component control. Its one-call CLI budget
intentionally stops before final rendering; HTTP acceptance never erases that
CLI operational stop.

Actual source/native-bundle admission and prospective first-request planning,
full display/stdout/stderr/source binding, pre-advance safety (#582), complete
native scheduler integration, descendant/cancellation accounting (#540), and
real enrolled semantic review (#583) remain. #581 stays open pending that
integrated acceptance. Other providers and invoiced charges are not verified.

No external model API calls, cloud resource operations, new real F1, product
release or merge were performed. `quality_go` remains false. Historical v1/v2
raw/receipt/tag/first-score evidence and Core contracts are unchanged.
