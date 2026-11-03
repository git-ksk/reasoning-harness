# Current native/local authority — Issue #580

2026-11-03 user direction: Reasoning Harness runs locally. The current
implementation and acceptance are [LocalAtomicStore](LOCAL_AUTHORITY.md) and
[the local execution policy](../../../docs/local-execution-policy.ja.md).
GCS, VM, KMS/IAM, remote two-host evidence and Docker are optional future
extensions, not prerequisites for this Mac/native profile. Keep local replay,
accounting, native-runtime/source and real review acceptance intact.

The original external-backend prototype below is historical context. Its
cloud-specific gates do not define the current local-run acceptance scope.

<details>
<summary>Historical external-backend prototype and its original limitations</summary>

# Prospective v3 external receipt/run authority — Issue #580

**Status: offline implementation and mock-remote acceptance only. Not independently
admitted for real model API, paid inference, 48-job evaluation, first-score
publication, Engine 0.7.3 release, or CLI adoption.** This module operates only
on prospective v3. Never alter or replay historical v1/v2 assets.

## Threat model and trust boundary

The previous v3 local receipt named its own `ledger_dir` and
`score_ledger_dir`. An alternate receipt and fresh local directory could
bypass a local `O_EXCL` claim across hosts. A SHA-256 digest or private 0700
directory is not authentication of a run issuer.

`authority.py` instead requires a **separately operator-provisioned** object
store. The operator chooses the GCS bucket and authenticates by a
restricted workload principal; the receipt, execution plan, output path, and
fixture **never select or change the bucket or claim namespace**. There is no
fallback to local locks if the remote store fails.

An independent issuance authority must create one canonical JSON object at
`reason-cli-normal-v3/authority/v1/receipts/<evaluation_id>.json`.
The immutable object binds:
- One unique evaluation/run identity and exactly one canonical frozen plan SHA.
- The corresponding prospective local receipt bytes SHA, full source identity,
  image identity, CLI build and runtime SHA.
- Provider/model, original executor ID, distinct evaluator ID, all 24 frozen
  questions/cases and *both* arms' ordered 48 jobs; each job's frozen first
  request binding, question SHA and callback generation/input/output/retry
  reservation. Overall token budget is fixed.

All blob values use strict canonical JSON (sorted UTF-8 keys, compact JSON,
non-finite numbers rejected). Any plan, identity, budget, request, case/order or
source/image/runtime mismatch fails **before** creating a run claim.

## Globally atomic claims

The shared GCS adapter uses `upload_from_string(..., if_generation_match=0)`
for **every** write. The first claimant also atomically creates
`cohorts/<stable-cohort-fingerprint>.json`, before `claims/<evaluation_id>/run.json`.
The fingerprint binds trusted source/image/CLI/runtime, provider/model, and all
paired case/question identities, **excluding** receipt IDs, local ledger paths,
first-request hashes (which include receipt identity), job order and budgets.
Consequently reissuing the same evaluation target with a new evaluation ID,
local ledger, changed request hashes, or larger budget cannot create a new
run **in the same immutably provisioned operator bucket**. A lost cohort-create
ACK burns that cohort; no automatic retry. The first-score verifier requires
both matching cohort and run claims and rejects noncanonical run/completion JSON.
A crashed run remains burned: there is
**no resume**, automatic retry, ID relabeling, output-path override or
recreated claim after a lost ACK.

For every job the `RunSession` writes, in sequence:

1. `reserved/<job_id>.json` with full worst-case token charge and ordinal.
2. `inflight/<job_id>.json` before spawning a subprocess.
3. `observed/<job_id>.json` after accounting, including typed failure,
   known token/attempt counts, full charged reservation, and SHA-256 of the
   canonical captured observation if available.

A missing receipt, unavailable remote store, ambiguous write, missing
inflight marker, duplicate claim, out-of-order job, over-cap attempt/usage,
unknown usage or operational error latches the run. The 48-job complete
marker can be created only after 48 success observations. **Unknown costs
are not assumed zero**: their entire reservations remain charged and
no further job may start; an abandoned/SIGKILLed run is not automatically
recovered. The former local runner's process-group SIGINT/SIGKILL logic is
unchanged and is not, by itself, proof of crash-descendant containment.

The `score.py` function accepts an operator-injected `ScoringContext` for
an authoritative first score. With no context, it **fails closed by default**;
only an explicit `--offline-synthetic-only` flag **and** a receipt marked
`synthetic-v3-HTTP-zero` permit an unauthenticated local diagnostic score.
This mode never admits a real-model score or release. Before touching the local first-score output, it calls
`TrustedAuthority.claim_score`, which validates the frozen receipt and all
48 remotely persisted run/reservation/inflight/observed entries, and
matches **each exact canonical observation hash and the exact 48-job order**
to the scoring inputs. The scorer supplies observation hashes in the ordered
observation index's insertion order; merely matching an unordered set is
insufficient.
The prospective [#583 review verifier](REVIEW_AUTH.md) now additionally requires
an operator-enrolled Ed25519 signer and freezes the exact review authentication
receipt hash in the remote/local first-score claims. This is locally tested
with ephemeral synthetic keys, not independently admitted real enrollment.
Only then is `claims/<evaluation_id>/first-score.json` exclusively created,
binding the observation index and review manifest to the original evaluator.
Even a later local write failure cannot retry first-score. The original
local first-score claim remains defense in depth and existing synthetic CLI
scoring remains **not externally admitted** without a trusted context.

`OfflineV3Runner(..., authority_session=session)` exercises the real POSIX
child scheduling in the existing HTTP-zero, byte-pinned fixture only. No
code path in `authority.py` launches a provider or loads credentials.

## Prerequisites still required before production use

These **cannot be asserted by a local mock or by receipt JSON**:

- Independently verify GCS bucket identity, strongly consistent conditional
  object creation, durable retention/locked policy, audit logging and IAM.
  Give receipt minting to a distinct issuer; executor may read receipts and
  create only claims in its allowed namespace, **never overwrite or delete**
  receipts/claims or mint a new run identity. Keep score principal separate
  from executor and freeze its role. Prevent privileged tenant switching.
  An arbitrary GCS bucket supplied by the caller is not an admitted authority.
- Prove **two real isolated hosts** against the exact same IAM-bound backend
  and receipt, including 409/412 conflict, timeout after an ambiguous ACK,
  different ledger/bucket attempts and object unavailability.
- Prove privileged, separately audited process supervision across
  SIGKILL/host crash, orphaned descendants, actual spent tokens, and
  resurrection under another host. A permanently burned claim prevents a
  *new authorized run*, but does **not** kill already detached processes or
  establish provider-side billing. Full reserved charges are conservative
  upper bounds, not verified billed usage.
- Add #581 authenticated HTTP/billable usage journal, #582 actual displayed
  source/image/runtime and pre-next-job admission, and #583 independently
  authenticated reviewer/semantic decision. Until these are in place,
  no external source/hash/receipt can certify the underlying actual traffic,
  displayed safety or evaluation correctness.
- Install and pin the separately audited `google-cloud-storage` Python
  dependency in the approved runtime; it is optional/not imported by
  synthetic tests, and this commit does **not** install or configure it.
- Audit independently that a privileged issuer cannot switch buckets or
  forge source/image/runtime identities to sidestep the stable-cohort claim.
  Changing these independently attested identities may legitimately require
  a new admission; the authority **cannot prove true cross-bucket uniqueness**.

Do **not** pass bucket configuration, signing credentials, or source material
through GitHub issues, plan fields, fixture paths, or public artifacts.
All real activation requires a **new, explicit review/GO** after the above
external admission. Do not use `score.py`'s normal standalone CLI as proof
of authenticated first-score.

## Tests

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest discover \
  -s evaluation/cli-repair-paired-v1/normal_v3 -p 'test_*.py' -v
```

`test_authority_580.py` uses a deterministic thread-safe in-memory
remote-object mock, **not** real GCS. It checks two concurrent claimant
clients, plan/receipt/image/role mutation, 48-job reservation/inflight,
observed accounting, injected storage failure and lost ACK, missing and
forged first-score records, one first-score only, GCS generation-zero
arguments, and two real *inert* subprocess-scheduler integrations:
48 synthetic children complete, or one unknown-usage child stops the
remaining 47. It does **not** attest two physical hosts or external
storage/IAM configuration.

## Independent review additions (not external acceptance)

- OS-process tests use two independently launched Python interpreters sharing
  a real filesystem O_EXCL store, racing the run and first-score claims.
  This checks process isolation of the test harness, **not GCS semantics**.
- Real OS SIGKILL demonstration confirms that a detached grandchild survives
  its parent's SIGKILL; the test explicitly kills that child afterward.
  Therefore **actual host/scheduler descendant isolation remains BLOCKED**,
  as do live billing and a shared GCS two-host deployment.
- Counterexamples reproduced missing stable cohort deduplication and accepted
  noncanonical run/completion evidence; both are now rejected. Local scoring
  without authenticated authority is disabled except explicit synthetic mode.

</details>
