# Native source/display and pre-advance admission — #582

This single-host Mac profile extends [native HTTP capture](NATIVE_HTTP.md).
It freezes the real native artifacts, plans the first request without a model,
captures both complete non-TTY streams, binds full source material and requires
an enrolled safety decision **before the next job**. GCS, VM, KMS/IAM and Docker
are not prerequisites.

## Freeze before execution

`NativeBundle.enroll()` is explicit operator enrollment, outside receipts/model
output. It freezes baseline/candidate native binaries; workspace source,
fixtures, examples, scripts and v3 module inventories; configuration/source/
public-policy files; actual Python, verifier modules/native extension and lock.
The same selected paths, regular-file identity, bytes and inventories are
verified before/after planning and each child, and again before advancement.
Added/removed source inputs, changed contents, replaced executables and symlink
substitution fail closed. OS/kernel/system libraries and runtime administrators
are trusted. Homebrew's administrator-writable runtime parent is supported;
private evidence/source paths keep their stricter owner/mode/ACL policy.

The legacy `image_sha256` field explicitly carries the native bundle digest in
this **new** profile. Source, binary set and runtime digests are separate. This
does not relabel any historical image/receipt or prove reproducible builds.

`reason ... --evaluation-request-plan PATH` runs the same natural-input,
configuration, budget and candidate-request builders with a non-network
planning adapter. It neither resolves provider credentials nor produces model
responses, answers or usage. The private record contains provider/model,
question, redacted request and the original pre-redaction request hash. The
controller additionally binds the exact argv, arm and bundle. Resume,
structured-command and mixed journal/trace modes are rejected. The actual first
HTTP/diagnostic request must match this prospective plan; a previous loopback
response is no longer needed to create a request template.

## Full streams and full source context

`NativeLane` accepts a fixed plan, trusted bundle, enrolled public policy and
canonical source catalog. It requires a dedicated native authority receipt
binding bundle, source-catalog and policy digests. That receipt cannot reserve
work via the legacy session path before enabling native admission. Job argv,
JSON/plain mode, path inputs, caps and first-request plans are checked against
the frozen artifacts/receipt. Path options require canonical separate-token
spelling; `--file=...` cannot skip inventory checks. Explicit measured configs
use `--evaluation-isolated-config` in both planning and execution, without
ambient user/project layers; configured programs must also be frozen.
No shell, ambient loader/Python-path settings,
stdin or implicit retry is used.

Each child runs in a process group with bounded pipes and deadline. The parent
captures exact stdout and stderr bytes, not only `finalization.text`. Partial
streams, overflow, a child lingering after closing streams, unknown exits,
changed runtime, HTTP/usage inconsistency and malformed/ambiguous JSON cannot
produce an admissible packet. Failed runs keep reservations and halt.

For this curated-source profile the operator supplies **complete source texts
and versions**, plus exact admitted `Evidence` hashes and source bindings.
`--file` context must also have a frozen full-text mapping; path inputs and CA
bundles must be in the native artifact inventory. Actual artifact evidence is
matched across initial/final/resolution outcomes. Unknown/changed evidence,
old/missing sources, unbound grounded displays, quote bytes/hash/JSON encoding
and citation offsets are rejected. Grounded source displays use the existing
`answer_context` binding. The packet contains all original stdout JSON side
fields, stderr, actual final display/citations, collector observation and full
catalog context. Arbitrary dynamic Web sources are not automatically admitted
by a catalog or a native binary hash.

## Review before advancing

`run_next()` returns a private immutable packet and review request while the
job remains inflight. Calling it again cannot spawn another child. A registered
reviewer signs `reason-cli-normal-v3-pre-advance-review-v1`, bound to the exact
evaluation/job, receipt, bundle, HTTP binding, observation, source catalog,
packet and both raw stream hashes under a separate signature domain.

The verifier requires an enrolled active key distinct from the executor's
subject/conflict group, explicit `safe`, full-source review, notes, and a
contiguous complete UTF-8 byte inventory of **both** streams. Each factual span
must cite enrolled full sources. Unsafe/unknown decisions, omitted stderr,
gaps/overlaps, split UTF-8, unbound facts, wrong/stale signatures, aliases and
revoked/weak keys fail closed. Changing raw bytes after review also fails.
Returned packet objects cannot mutate the controller's private pending state.

`accept_review()` rechecks runtime and captured hashes, consumes a typed
verified capability rather than caller JSON/Booleans, writes the durable
pre-advance record, then records the observation. Only then can another job
start. The native first-score authority requires all 48 ordered pre-advance
records and binds their digest in its create-only claim. The collector remains
measurement-only; its observation never grants itself continuation authority.

CLI logical `model_calls` count diagnostic correlations. A structured fallback
adds another `generate()`/HTTP attempt under the same correlation. The collector
now checks that distinction against real native traces; HTTP attempts, exact
usage, budgets, invalid metrics and stop rules are unchanged. Historical v1/v2
records and scores are untouched.

## Acceptance boundary

Signatures authenticate the enrolled decision, not semantic truth or human
independence. The owner account, selected native code, operator catalog and
runtime administrators are trust roots. File/role checks and Python capability
types do not isolate hostile code at the same UID/root. Process-group tests do
not prove that detached descendants terminated (#540).

Loopback controls use a dedicated purpose, `fixture-model`, fake key and test
review signatures. They do not authorize provider use or establish a real
reviewer's enrollment/source or semantic acceptance. Production enrollment,
the actual source/catalog/bundle review, model/readiness/budget authorization,
real semantic grading (#583), a new real F1, Engine release and CLI adoption
remain distinct gates. `quality_go` remains false.

See [Mac acceptance and review findings](native-admission-acceptance-20261103.md).
