# Issue 620 limited authorization/observation integration — partial implementation

Engineering branch only. Not an independent implementation acceptance, normative
ratification, production enrollment, live run, formal score or release. The goal
remains Engine 0.7.3, not a general authentication product.

## Reuse and delta

- `auth620.py`: RH-AUTH/1 restricted canonical wire, immutable operator-pinned
  per-evaluation registry, Ed25519 verification using existing v3
  `review_auth.py` and hash-locked PyNaCl 1.6.2. The profile is **not**
  general JCS. Legacy wire/code is unchanged. Verification returns a receipt
  hash, not a capability.
- `first_claim_620.py`: synthetic-only 24/48/8 global first-claim coordinator,
  using `LocalAtomicStore` create-only/owner/root/fsync protection and
  `check_first_attempts`; stable pinned dataset/cohort identity, sequential
  reservation/inflight/observed/closure records, no restart, conservative
  global failure handling. No model dispatch.
- `observation_620.py`: v3 native HTTP v2 normalization, `audit_http`, actual
  CLI finalization trace/display, source catalog and legacy signed safety
  review joined to new signed terminal/admission receipts. Incomplete capture
  is diagnostic-only with a null final hash and unknown usage.
- `blind_fixture_620.py`: complete invented A/B manifests, full-display/source
  basis digests, separate adjudication/ratification roles, immutable signed
  stage records, full blind-map lock/opening, re-verification of signatures and
  persisted records at use and projection. Fixture currently fixes one
  citation target per slot. No external process/access separation is asserted.
- `fixture_permissions_620.py`: distinct synthetic prospective/run/score
  transitions, one-use durable fixture stages, expiry recheck before each
  reservation, existing pure same-account billing checker exercised only on
  invented values (no real account proof). `approval_620.py` validates the
  proposed future signed approval wire and exact expected parents; even an
  authenticated PERMIT does not grant permission.
- `test_auth620.py`, `test_fixture_graph_620.py`: public invented fixtures
  and public signature vector; deterministic demonstration keys only, never
  enroll them. The fixed fixture runner is the unittest entrypoint, not a
  production scorer. No user-supplied dataset/Gold path is accepted by it.

## Verification boundary

The existing formal scorer candidate is unchanged. The synthetic graph uses
its existing arithmetic, asserts exact complete-result equality with the
existing synthetic scenario, and tests order invariance. The formal entrypoint
is a fail-closed stub: an arbitrary caller dictionary, receipt digest, purpose
or simulated approval cannot activate it. No real input is relabelled.

This is **PARTIAL**, not IMPLEMENTATION CANDIDATE READY. Remaining technical
work before independent implementation acceptance: bind the full generalized
formal graph to a protected frozen plan (rather than the synthetic harness's
trusted expected-input arguments), finalize complete multi-citation inventory
and adverse nonterminal signed judgment integration, connect the approval-wire
validator to the future formal preflight and durable full approval DAG, and
verify the fixed synthetic lane boundary independent of caller-relabelled
objects. Recheck external snapshot freshness/rollback and audit/redaction as
part of the formal boundary; do not claim same-UID isolation here. Keep
formal paths disabled until the independent acceptance and external gates.

## Local results and scope

Python 3.13.15 and 3.14.8, PyNaCl 1.6.2 in isolated, hash-locked environments.
For each runtime: public evaluation suite 168 discovered, 161 passed, seven
private-Gold opt-in tests intentionally skipped; 40 targeted v3 reuse tests
passed. This includes 21 new tests (the fixed integrated graph plus unit
and negative probes). No model/provider/billing requests, actual private Gold
scoring or formal F1. The 49 acceptance scenarios are mapped in the private
implementation record; the full set is not asserted to have passed.

No changes to the Core, normal CLI, prior scoring algorithm, fixed paired
conditions, thresholds, or any frozen receipt. All eight external gates remain
HOLD; policy UNRATIFIED, formal scorer identity UNSET, formal/live/quality/release
authority false. Keep #531/#616/#618/#620 OPEN and PR #619 Draft. One combined
independent implementation acceptance only after remaining implementation work,
not a new design review unless a material contradiction is discovered.
