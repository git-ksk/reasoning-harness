# Native Mac HTTP journal — #581

The ordinary native CLI now has an explicit Mistral-only evaluation transport
observer. Normal CLI execution keeps its existing behavior. This is a local
engineering acceptance path; it does not authorize a model call or establish
complete v3 runtime/display/source/reviewer admission.

## Data path

1. The trusted parent reserves a job in the fixed local authority, then creates
   `NativeHttpCapture(session, job_id, question, new_private_directory)`.
   The session persists the exact authority receipt, plan, job/question,
   first-request binding, source/native bundle/CLI/runtime identities and limits
   **before spawn**. Neither a receipt nor model output selects the authority.
2. The capture creates a fresh 0600 journal under its 0700 directory using
   no-follow directory FDs, ACL checks, exclusive creation and parent fsync.
   The parent passes only that already-open write capability to the CLI using
   `capture.subprocess_options()` and `capture.cli_arguments()`.
3. The CLI requires the diagnostic trace, binding digest, attempt ceiling and
   reserved-token ceiling. It rejects non-regular, non-owned, permissive,
   hardlinked, nonempty or nonzero-offset capabilities. It marks the FD
   close-on-exec before any external resolver subprocess can inherit it.
4. The Mistral adapter observes the actual serialized bytes submitted to
   reqwest and the complete received body. Task-local context binds the same
   pre-redaction `ModelRequest` digest and logical correlation/attempt as the
   diagnostic trace. No Authorization headers, keys or input/output prose are
   written to the journal. The private binding/trace can contain source prose.
5. Each request intent is hash-chained, written and fsynced before send.
   Responses record actual HTTP status, raw-body digest and provider-reported
   usage. A completed marker is written only after parsing and pacing finish.
   Partial writes, missing completion, non-2xx, malformed/wrong-model responses,
   unknown/inconsistent/over-reserve usage, cancellation or I/O errors latch
   stop. Internal HTTP retries are disabled on this opt-in path. Existing
   prompts, schemas, normal caps and successful-response pacing are preserved.
6. After the child has exited and the parent closes its write FD,
   `capture.verify()` checks canonical JSON/duplicates, sequence/hash chain,
   frozen first request, ordered trace/HTTP identity, usage and limits. It
   creates normalized `http.json` for the existing collector and acknowledges
   exact journal/trace/HTTP hashes and totals in the fixed local authority.
   `observe()` refuses missing capture or totals differing from that record.

`verify()` only acknowledges HTTP evidence. It does not decide that a job is
safe or permit the next one. CLI operational errors still halt the run even
when its HTTP evidence is valid. Any uncertain reservation stays fully charged;
do not remove records, refund it or recreate a profile to resume.

## Local controls

`--evaluation-http-loopback-url` is an explicit, journal-required control
restricted to `http://127.0.0.1:<port>/...`; redirects and external/control
endpoints are rejected. Tests use a fake key, no ambient configuration and a
local HTTP server. They make no provider calls. Set `REASON_V3_NATIVE_CLI` to the
freshly built `reason` executable to include real OS-subprocess controls in the
Python suite; CI builds it and sets the variable explicitly.

The test cohort freezes its synthetic first-request template from a separate,
provider-free loopback component control before starting the cohort. This is
not a source-reviewed production request-plan preflight. Its one-call CLI budget
intentionally stops before final rendering: successful transport capture must
not erase that CLI stop or claim a complete user-answer evaluation.

## Trust limits and remaining gates

Subsequent [#582 native admission](NATIVE_ADMISSION.md) implements model-free
planning, fixed native artifacts, complete streams/source packets and signed
pre-advance decisions. The remaining-gate list below describes what HTTP
capture alone proves; real operator enrollment and semantic acceptance remain.

The operator, measured native runtime, private parent and OS account are trusted.
A local owner/root can rewrite evidence or replace the admitted runtime; the
hash chain and passed FD are not cryptographic authentication against that
owner. Provider-reported usage is distinct from invoiced charges. A durable
request intent may precede a request that was never physically sent; lack of a
response is uncertain usage, not zero usage. A crash during response delivery
can leave only that intent. Do not infer that a detached descendant terminated.

Actual source/native-bundle admission and first-request planning, full stdout/
stderr/display/source binding, pre-advance safety (#582), supervisor/descendant
accounting (#540), complete native scheduler wiring and real enrolled semantic
review (#583) remain. Other providers are not admitted through this journal.
No new real F1, independent acceptance, release or quality GO is claimed.

See the [native HTTP acceptance record](native-http-acceptance-20261103.md) and
the preserved [earlier local-store acceptance](local-acceptance-20261103.md).
