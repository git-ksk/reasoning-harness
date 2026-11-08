# Normal Real24 v4 — prospective 24 paired shards (Issue #574)

## Scope and status

This is a **new**, fail-closed development evaluation protocol. It is
neither a continuation nor a replay of the spent v3
`cli-normal-real24-development-v3` tag / run 42438102599. No outcome from
the terminated v3 model run counts toward the new cohort. All 24 frozen
questions and their first 48 baseline/candidate requests remain prospective
for this *new* v4 run. Any published F1 requires all **48 actual** native
observations, two independently signed pre-advance reviews per case, 24
independent semantic score attestations, and full-cohort evidence verification.

At code authoring time the v4 tag has **not** been pushed, no v4 Mistral
requests have been sent and no new F1 exists. A successful offline test,
prepare-only workflow, or individual shard cannot be called cohort acceptance.

## Architecture

1. A new single-use GitHub tag launches the tag-admission job, which checks
   full calendar-windowed repository push history **once**, records the
   exact first run ID, source commit and workflow, and publishes a nonsecret
   immutable proof. Every shard validates the exact proof bytes against
   the GitHub same-run SHA-256 output; ambiguity stops before model use.
2. The matrix has 24 fixed, independently identified cases. GitHub
   `max-parallel: 1` and `fail-fast: true` keep one paired native workload
   at a time. Each case has two frozen arms, up to 1200 seconds/job, a
   separately sealed GH Actions bootstrap/final artifact, and a fresh
   two-job authority receipt. No one job crosses the hosted-runner 6-hour
   limit. This does not promise every provider call will succeed.
3. Each case validates the **entire** 24-case global manifest before its
   independently reviewed start. The registered reviewer receives encrypted
   stage packets addressed `rw-NN-start`, `rw-NN-job-00`,
   `rw-NN-job-01`, `rw-NN-final` and signs/sends the exact
   corresponding replies. Each signed pre-advance safety decision gates
   the next physical model job; there is no implicit unattended approval.
4. After all 24 first-score claims and 48 observations exist, the
   **offline** `aggregate.py` verifies the original source/manifest,
   bundle hashes, paired authority receipts, frozen gold, exactly two
   observations per case, independent semantic reviews and the original
   registered Ed25519 final attestation before computing cross-shard
   `score.report()` metrics. It does not authorize release or quality GO.
   Missing/duplicate/foreign/operational-stop shards yield **no F1**.

### Provider/financial/safety boundaries

Provider: frozen Mistral `ministral-8b-2512`, free-only and additional
charges disabled per preexisting owner-recorded operator confirmation;
**actual quota headroom is unknown until independently verified**. The
existing physical HTTP journal, exact source hashes, no-retry reservations,
Mac no-fork lifeline, strict CLI-native results, and signed source-complete
pre-advance gate remain mandatory. A real provider error/timeout halts a
shard and the matrix; it is not an incorrect-answer quality result.

Reviewer: separately enrolled Ed25519 key and own AI source/answer judgment
through a private host; a script merely signs the reviewer's explicit
decision and never manufactures a verdict. Credentials and source/answer
plaintext stay on the trusted private reviewer host. Published GitHub Issue
messages are encrypted. **Artifact retention is 30 days**, so old shards
must be reviewed/retrieved before then.

## Offline preparation and admission

Only run from an explicitly reviewed current `main` and after the tagged
source is ready. Provider-free read-only preparations may use Actions
`workflow_dispatch` with a chosen `case_id`. They must report
`model_generations: 0`, produce a decryptable bootstrap and freeze the
exact same full-global manifest. A prepare-only run is never accepted as a
live first attempt and may not be promoted to a scoring receipt.

A new real tag must **not** be issued until the registered independent
reviewer is reachable for every shard and able to process three signed
decisions per paired case within the 290-minute bound. A manual
misconfiguration/ambiguous publication burns that shard; do not rerun
the one-shot tag or bypass the signature. Stop and report if the 24-case
cohort cannot be completed, rather than silently substitute synthetic
output or reuse a score.

Local regression (no provider):
```sh
python3 -m unittest discover -s evaluation/cli-repair-paired-v1/normal_v4 -p 'test_*.py'
python3 -m unittest discover -s evaluation/cli-repair-paired-v1/normal_v3 -p 'test_*.py'
```

Offline global aggregation expects a trusted **private** directory containing
`rw-NN/bootstrap/` and `rw-NN/final/` for every case, decrypted under
strict 0700/0600 permissions by the registered transport-key holder. Never
commit or publish those plaintext directories or the signing key.
```sh
python3 evaluation/cli-repair-paired-v1/normal_v4/aggregate.py \
  --shards PRIVATE_ATTESTED_SHARDS --output PRIVATE_RESULT_DIR/v4-first-score.json
```
