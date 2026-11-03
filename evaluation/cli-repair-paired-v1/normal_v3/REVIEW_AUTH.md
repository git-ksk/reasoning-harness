# Prospective v3 authenticated semantic review — Issue #583

Status: implementing-agent source re-review and local acceptance, with the two
original acceptance failures retained in [the acceptance record](acceptance-20261103.md).
Separately authenticated source review, trusted enrollment and real runtime
acceptance remain pending. This does not close #583, admit PR #587, authorize
model calls, publish a first real score, or approve Engine/CLI adoption.

**Current local scope:** follow the [Mac/native policy](../../../docs/local-execution-policy.ja.md)
and [local authority](LOCAL_AUTHORITY.md). Enrollment is an operator-owned local
key/policy registration. GCS, IAM/KMS, cloud signers and VMs are not required.
The owner OS account is trusted; this is not a hostile same-UID/root boundary
or automatic proof of a person’s identity.

## Admission boundary

The previous reviewer fields (`id`, `role`, `independent_of_execution`,
`full_source_context_reviewed`) were self-assertions. `review_auth.py` adds an
operator-injected policy of public keys, canonical subjects, aliases, roles,
conflict groups and revoked keys. It does not load a trust policy or verifier
from a receipt, review document, envelope, environment variable or CLI flag.

One enrolled evaluator signs the entire cohort's review freeze. The evaluator
must match #580's frozen `evaluator_id`; every per-job reviewer must resolve to
that same enrolled subject and role. An execution alias of the same subject,
an overlapping conflict group, ambiguous aliases, a reused public key, an
unknown key, a revoked key and an invented human role are rejected. Public keys
must be canonical non-small-order points on the prime-order subgroup. Supporting
multiple independent signing subjects in one cohort requires a new explicitly
admitted assignment contract; there is no implicit multi-reviewer fallback.

The operator must independently authenticate enrollment, alias ownership,
roles, conflict declarations, key custody/revocation and the verifier runtime.
A locally invented policy is not a real identity credential. Signatures
authenticate the signer and exact bytes, not semantic truth, human presence,
actual source acquisition or a declaration's honesty. The scorer retains
`human_attestation=false`, `quality_go=false` and its existing semantic checks.

## Signed bytes and first-score binding

The canonical JSON envelope has exactly `contract`, `algorithm`, `key_id`,
`payload`, `signature_hex`. Contract is
`reason-cli-normal-v3-review-attestation-v1`; algorithm is strictly `Ed25519`.
Signature bytes cover this contract followed by a NUL byte and canonical JSON
of the payload. Canonical JSON uses the repository encoder, sorted keys,
compact UTF-8 and no non-finite numbers. Noncanonical envelope/policy bytes,
including duplicate keys, fail closed. Public keys are raw 32-byte lowercase
hex; signatures are 64-byte lowercase hex.

The payload has exactly:

- `contract`, `evaluation_id`, `executor_id`, `evaluator_id`.
- `policy_sha256`: exact operator policy bytes.
- `plan_sha256`: canonical frozen #580 execution plan.
- `execution_binding_sha256`: canonical `vars(AuthorityBinding)`, including
  source/image/CLI/runtime hashes and executor/evaluator identities.
- `receipt_sha256`, `observation_index_sha256`, `packet_index_sha256`,
  `gold_sha256`, `review_manifest_sha256`: exact file bytes.
- `complete_display_reviewed=true`, `full_sources_reviewed=true`: signed
  declarations, checked in addition to existing full display-byte segments,
  per-fact citation checks, source-complete packets and observation bindings.

The review manifest already freezes every review file's hash. Packet, gold and
observation indexes bind all consumed source/display bytes. Rewriting a review
and recomputing its unsigned manifest cannot reuse the old signature. A new
evaluation, plan, image, source identity, policy, gold or packet also requires a
new admitted attestation. Signature verification itself does not consume a
score claim; invalid inputs stop before remote/local claims and score output.

The scorer reads each input snapshot once and uses those same bytes for parsing
and hashing. It authenticates the reviews before invoking #580's complete-run
and ordered-48-observation verifier. Its authentication receipt hash is written
into both the remote first-score claim and private local claim/report. Once a
remote first-score claim exists, an alternative output path cannot rescore.
The lower-level #580 claim API is an accounting/lock primitive, not a semantic
review verifier; callers must use the authenticated scorer for real scoring.

## Host integration

An independently admitted host constructs `ReviewAuthentication` with immutable
operator policy bytes and captured untrusted attestation bytes. It passes this as
`score(..., authoritative_score=context, review_authentication=authentication)`.
There is no private-key loading, signing or self-enrollment in production code.

Verification uses [PyNaCl 1.6.2 VerifyKey](https://pynacl.readthedocs.io/en/latest/signing/)
in process; it never executes a caller-selected verification program. There is
no subprocess exit code to trust, executable pathname race or signature backend
fallback. Every enrolled key is checked with libsodium's canonical-point check
and its [documented main-subgroup check](https://libsodium.gitbook.io/doc/advanced/point-arithmetic)
`(L - 1)P + P == identity`, covering the older mixed-order predicate defect too.
These are vetted library operations, not a local signature/curve implementation.
Missing or unadmitted PyNaCl versions and invalid signatures fail closed.

Install the [wheel-only dependency lock](review-requirements.txt) into a private
virtual environment. It pins PyNaCl 1.6.2, cffi 2.1.1 and pycparser 3.12, with
official PyPI wheel hashes and `--require-hashes`; no unpinned source build or
global install is required. The authentication receipt records the PyNaCl
version and dependency-lock digest. Python/native-library/source/image and
privileged host integrity still require independent runtime admission; local
package installation is not that admission. The unreleased `8832c74` constructor's
executable/hash parameters are deliberately removed by this security correction.

Unauthenticated `--offline-synthetic-only` scoring remains available only with
an explicit synthetic HTTP-zero receipt; it cannot claim authenticated review.
The standalone scorer CLI cannot select its own trust root or produce a real
authenticated first score.

## Local evidence and remaining work

Run all v3 controls with the existing CI command:

```sh
python3 -m venv /private/tmp/v3-review-acceptance
/private/tmp/v3-review-acceptance/bin/python -m pip install \
  -r evaluation/cli-repair-paired-v1/normal_v3/review-requirements.txt
PYTHONDONTWRITEBYTECODE=1 /private/tmp/v3-review-acceptance/bin/python -B -m unittest discover \
  -s evaluation/cli-repair-paired-v1/normal_v3 -p 'test_*.py' -v
```

Tests generate temporary Ed25519 keys/signatures with OpenSSL and exercise the
actual libsodium verifier, plus the independent public RFC 8032 TEST 2 vector.
They cover signature/payload/policy/identity tampering, alias and conflict
rejection, key revocation, full-display/source declarations, changed review
freezes, missing backends and single-read input snapshots. Acceptance also
covers unsigned identity-key forgeries, mixed-order keys, noncanonical scalars,
host-program substitution and verification in a separate OS process. The full
synthetic 24-case/two-arm capture→packet→review→score test now uses actual #580
run/reservation/inflight/observation/first-score code on its fake object store,
with all 48 canonical observation hashes and a real temporary signature. It
rejects unsigned scoring and alternate-output first-score replay.

These tests provide no real reviewer decision, provider billing, paid model
generation, fresh accuracy or acceptance F1. #580 local-ledger acceptance,
#581 actual HTTP/usage evidence and #582 native display/source/runtime and
pre-advance admission remain separate gates. GCS/remote-host acceptance is
an optional extension rather than a local-run prerequisite. Historical
v1/v2 observations, raw, reviews, receipts, tags and first scores are unchanged.
