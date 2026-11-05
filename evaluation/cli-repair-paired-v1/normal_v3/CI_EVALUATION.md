# CI measurement for the consumed real24 cohort

The `cli-normal-real24-development-v3` workflow uses one `macos-15` job and
the existing `MISTRAL_API_KEY` Actions Secret. It builds the two pinned cores
with the common CLI, materializes the immutable public source captures and
freezes the native bundle, 48 first requests, current metadata lookup, gold,
plan and receipt. The fixed provider is `ministral-8b-2512`, with the existing
[HTTP limits](NATIVE_LIMITS.md). The operator confirmed free mode with additional
charges disabled on 2026-11-05. `ci-operator.json` registers the public reviewer
policy, transport recipient and that account condition.

## Preparation and first execution

A manual workflow dispatch prepares and uploads encrypted evidence with zero
provider requests or generations. A push of the single new tag
`cli-normal-real24-development-v3` admits the live path. The complete repository
push-history guard checks that tag and workflow before the metadata GET. Attempt
two is refused. A new run uses an explicitly enrolled authority directory on
its own CI host, retaining the profile through final scoring in that same job.

The `native-v3-bootstrap-RUN_ID` artifact contains only `bootstrap.sealed`.
The appointed reviewer decrypts it with their separate Curve25519 key using
`ci_operator.py archive --key PRIVATE_KEY --input bootstrap.sealed --output PRIVATE_DIR`.
The reviewer checks the artifact's GitHub run/source identity, actual binary and
source hashes, runtime, full source captures, paired prompts and exact frozen
conditions. Encryption alone does not authenticate the CI host. The reviewer
signs `start-request.json` plus `decision=approve` under the Ed25519 domain
`reason-cli-normal-v3-ci-approval-v1` followed by NUL and canonical payload bytes.

The local operator publishes that signed JSON with
`ci_operator.py send --run-id RUN_ID --stage start --since SESSION_SINCE
--session SESSION_FILE --input SIGNED_APPROVAL`. This helper uses local GitHub
authentication and the CI session's measured inbox public key. The CI controller
checks the registered independent signer and exact session/bundle/plan/receipt
before it issues the authority receipt, claims the run or loads the model key
into a native CLI child.

## Review and scoring in the same job

The controller publishes encrypted packets into #574 using stage names
`job-00` through `job-47`. `ci_operator.py receive` reconstructs bounded chunks
and decrypts a packet for the reviewer. Replies use `job-00-review` through
`job-47-review`, retaining the existing native pre-advance signature format.
Unregistered, changed, stale or foreign replies cannot advance the run. A valid
signed unsafe decision stops it. All source/display bytes remain in the packet.

After 48 admitted observations, the encrypted `final` packet contains the
observation and review-packet indexes plus frozen gold. The reviewer returns
`final-review` with all 48 semantic review objects and the existing final review
attestation. The controller validates every review and signature before invoking
the authenticated once-only scorer against the same authority profile.

GitHub-hosted jobs have a six-hour limit. The controller uses a 330-minute
whole-job deadline starting before setup, reserves room for a complete 360-second
job before each spawn, and bounds every review wait by that deadline. The remaining
time is for sealing interrupted evidence. Cancellation and ambiguous publication
stop advancement and retain reservations. No resumption or automatic retry occurs.

## Public repository transport

Bootstrap and final archives use the pinned upload-artifact action. Per-question
packets and replies use compressed PyNaCl SealedBox ciphertext in bounded Issue
comments, avoiding a separate artifact for every question. Routing metadata is
limited to run, stage, chunk order and ciphertext digest. Private keys and provider
credentials are excluded from archives. The terminal artifact preserves partial
captures and the actual authority records, including failures.

The transport authenticates the registered agent's decision through the existing
Ed25519 gate. Semantic judgment still belongs to that reviewer. The resulting
scores describe consumed development questions. New independent accuracy and
Engine/CLI adoption continue under #531.
