# Native local authority for one Mac/host — #580

The [local execution policy](../../../docs/local-execution-policy.ja.md) replaces
the earlier mandatory cloud/distributed acceptance scope. GCS, cloud projects,
VM, IAM, KMS and Docker images are not prerequisites for local evaluation.

## Operator-owned profile

`initialize_local_store(root)` explicitly creates a private authority root
and returns `LocalStoreProfile`. Existing roots are never reset. The profile
binds a random identity, canonical path and filesystem device/inode. Save its
`to_json()` value in private operator configuration outside captured receipts;
`from_json()` reopens the same configured identity.

`LocalAtomicStore(profile, role=...)` never initializes a replacement authority.
Missing/replaced/permissive/symlinked roots or changed identity files fail closed.
Changing an output directory or receipt cannot redirect its fixed namespace.

```python
from authority import TrustedAuthority
from local_store import initialize_local_store, LocalAtomicStore

# Explicit one-time setup; use a persistent private absolute path.
profile = initialize_local_store("/absolute/private/authority")
issuer = LocalAtomicStore(profile, role="issuer")
execution = TrustedAuthority(LocalAtomicStore(profile, role="executor"))
scoring = TrustedAuthority(LocalAtomicStore(profile, role="evaluator"))
```

Publish the canonical receipt at `PREFIX/receipts/<evaluation_id>.json`.
Execution uses `start()`; scoring receives its evaluator authority through
`ScoringContext` plus enrolled `ReviewAuthentication`. This store uses no network.

## Guarantees and limits

- Create-only objects use `O_EXCL | O_NOFOLLOW`, 0600 files, 0700 directories,
  directory-FD traversal and owner/mode/ACL checks. Reads reject symlinks,
  hardlinks and non-regular files; nonblocking opens reject FIFOs without hanging.
- Files and parent-directory changes are fsynced. Interrupted/ambiguous writes
  are not unlinked or refunded; claims remain burned after process death/SIGKILL.
- Actual local subprocesses contend on the same run/first-score records.
  Stable cohort claims prevent reissuing the evaluation under another ID/output.
- Issuer/executor/evaluator API roles constrain record creation. Executors cannot
  mint receipts or first scores. Scoring still checks ordered 48 observations,
  complete accounting/run evidence and the authenticated review freeze.

The owner OS account, private profile and selected runtime are trusted. API roles
guard integration mistakes, not hostile same-UID/root code. Deliberately replacing
both a ledger and its trusted profile creates a new operator authority; this is
not global tamper protection or a cross-host lock. Network filesystems and
synchronization folders are outside this local profile.

SIGKILL/process-race tests do not prove physical power-loss behavior or cleanup
of an unaudited detached child. The earlier descendant counterexample remains
valid. Local supervision and unknown usage remain #540/#581 work, rather than
a reason to provision a different cloud host.

## Native identity and remaining gates

New native runs freeze actual source, CLI/Engine, Python/verifier/runtime inputs
and settings. For existing wire compatibility, `image_sha256` carries the
explicit native execution-bundle digest in this new profile. This does not assert
a Docker build or reinterpret historical image records. #582 owns validating
that native-runtime/source/display chain.

Local acceptance does not authorize model calls, a real first F1, quality GO or
release. Native-runtime review/readiness, frozen budgets, #581 HTTP/usage,
#582 actual display/source safety and #583 enrolled reviewer decisions remain.
The GCS adapter is an optional future extension. Its cloud-specific acceptance
is not a mandatory gate for this single-host profile.

See the [134-test Mac acceptance record](local-acceptance-20261103.md) for the
verified local controls, native CLI smoke and remaining runtime gates.
