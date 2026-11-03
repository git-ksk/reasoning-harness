# #583 review and local acceptance — 2026-11-03 JST

The implementing agent re-reviewed `8832c74e307117fd0d0c3b2443cba65e55e45d68`,
reran its 111 existing controls, and added adversarial acceptance cases.
The original candidate failed two new cases. Both failures are corrected and
the final local suite passes 118/118 on fresh Python 3.13 and 3.14 environments.
This is local engineering acceptance, not independently authenticated reviewer
or real-runtime acceptance. #583 remains open and production GO remains false.

## Findings and correction

1. **P1, conditional on weak-key enrollment:** the policy accepted the identity
   Ed25519 public key (`01` followed by 31 zero bytes). A constant signature
   with identity R and zero S authenticated a complete attestation without any
   private signing key. This does not demonstrate a forgery against normally
   generated keys or an exploit of a real enrolled identity. Enrolled points now
   require canonical, non-small-order, prime-subgroup membership, and actual
   verification uses PyNaCl/libsodium.
2. **P2, conditional on a mutable verifier pathname:** hashing the OpenSSL file
   and subsequently executing its path did not bind execution to the hashed
   bytes. The local reproduction replaced that path after the hash read with
   an inert program returning zero, and a forged signature was accepted. There
   was no production-host exploit or model traffic. The correction removes
   caller-selected executable/hash parameters and all external-program
   verification. It does not try to fix the race with another pathname check.

The [initial failure transcript](acceptance-20261103-initial-failures.txt) retains
both failed tests, with checkout paths replaced by `<checkout>`. The
[machine-readable record](acceptance-20261103.json) retains original/sanitized
log hashes, conditions, source fingerprints, runtime results and open gates.
The original candidate commit and its earlier 111-test pass are not relabeled.

## Verification performed

| Check | Local result |
| --- | --- |
| Existing candidate controls before correction | 111/111 PASS; insufficient for new counterexamples |
| New unsigned-identity-key and external-program counterexamples before correction | 2 FAIL, recorded |
| Corrected full suite, clean Python 3.13.15 environment | 118/118 PASS |
| Corrected full suite, clean Python 3.14.8 environment | 118/118 PASS |
| Wheel-only hashed dependency install in both environments | PASS |
| Corrupted PyNaCl wheel hash, forced fresh dry-run resolution without networking | Rejected |
| Public RFC 8032 TEST 2, changed message and noncanonical S | Valid vector accepted; mutations rejected |
| Zero/identity/order-2/noncanonical/mixed-order public keys | Rejected |
| Simulated older point predicate false positive | Separate main-subgroup guard rejects |
| Authentic signature in another OS process; replay under a foreign evaluation | Accepted; replay rejected |
| Validly signed review with missing display segments | Rejected before score claims/output |
| 48 synthetic rows through actual #580 accounting/ordered-hash/first-score code on fake object store | PASS, unsigned and alternate-output score rejected |
| Python syntax, workflow YAML syntax and diff whitespace | PASS |

Signatures for generated fixtures are produced with OpenSSL and verified by a
different library, libsodium; public RFC test data provide another fixed
positive control. This is not a separately authored human review or an
independent semantic accuracy dataset. The original first-score safety and
quality metrics remain unchanged, including `quality_go=false`.

The dependency lock pins PyNaCl 1.6.2, cffi 2.1.1 and pycparser 3.12 to official
PyPI wheel SHA-256 values. A new CI-local virtual environment installs that lock
before running v3 controls. GitHub CI must be checked on the actual PR head;
its result is not inferred from the local result in this file.

## Open external acceptance

- No independently authenticated external source reviewer or real reviewer
  enrollment/key-custody/revocation evidence is supplied by this local run.
- #580's real immutable GCS/IAM authority, isolated two-host conflict and
  host-crash/descendant/billing evidence remain open.
- #581's independent authenticated HTTP/billable journal and #582's actual
  display/source/image/pre-advance safety admission remain open.
- No real GCS/model API requests, paid generation, new genuine first score,
  new accuracy F1, Engine publication or CLI adoption occurred.
- Historical v1/v2 assets, Core code and published artifacts remain unchanged.

The scope permits a reviewed engineering draft. It does not authorize merging
the blocked parent PR #587, closing #583, a paid 48-job run or production GO.
