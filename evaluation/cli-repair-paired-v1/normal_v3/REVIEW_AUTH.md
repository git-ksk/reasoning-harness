# Prospective v3 authenticated semantic review — Issue #583

Status: local implementation and synthetic signature/integration controls.
Independent source review, trusted enrollment and real runtime acceptance are
still pending. This does not close #583, admit PR #587, authorize model calls,
publish a first real score, or approve Engine/CLI adoption.

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
unknown key, a revoked key and an invented human role are rejected. Supporting
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
operator policy bytes, captured untrusted attestation bytes, an absolute
OpenSSL executable path and its independently pinned SHA-256. It passes this as
`score(..., authoritative_score=context, review_authentication=authentication)`.
There is no private-key loading, signing or self-enrollment in production code.

Verification uses [OpenSSL pkeyutl](https://docs.openssl.org/3.0/man1/openssl-pkeyutl/)
with `-verify -rawin -pubin` and the
[RFC 8410](https://www.rfc-editor.org/rfc/rfc8410.html) Ed25519 public-key encoding.
It checks the executable hash before each verification, invokes no shell, strips
the inherited environment, disables ambient OpenSSL configuration, uses private
temporary files and imposes a 10-second timeout. No signature backend fallback
exists. Missing/changed executables, process failures and invalid signatures
fail closed. Runtime dependency/library and privileged host isolation still
need independent admission; pinning one executable cannot prove those gates.

Unauthenticated `--offline-synthetic-only` scoring remains available only with
an explicit synthetic HTTP-zero receipt; it cannot claim authenticated review.
The standalone scorer CLI cannot select its own trust root or produce a real
authenticated first score.

## Local evidence and remaining work

Run all v3 controls with the existing CI command:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest discover \
  -s evaluation/cli-repair-paired-v1/normal_v3 -p 'test_*.py' -v
```

Tests generate temporary Ed25519 keys and exercise the actual OpenSSL verifier.
They cover signature/payload/policy/identity tampering, alias and conflict
rejection, key revocation, full-display/source declarations, changed review
freezes, unavailable verifiers and single-read input snapshots. The full
synthetic 24-case/two-arm capture→packet→review→score test now uses actual #580
run/reservation/inflight/observation/first-score code on its fake object store,
with all 48 canonical observation hashes and a real temporary signature. It
rejects unsigned scoring and alternate-output first-score replay.

These tests provide no real reviewer identity, external store/IAM, provider
billing, paid model generation, fresh accuracy or acceptance F1. #580's external
GCS/host gates, #581's independent HTTP journal and #582's full actual
display/source/image/pre-advance admission remain outstanding. Historical
v1/v2 observations, raw, reviews, receipts, tags and first scores are unchanged.
