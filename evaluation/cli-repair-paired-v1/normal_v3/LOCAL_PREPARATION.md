# Prepare the next paired evaluation on a Mac

`build_native_pair.py` and `prepare_local.py` prepare the consumed real24
development comparison without a model API call. They do not authorize a paid
run or publish an Engine version. Everything runs on this Mac.
The prepared configs/jobs also freeze [native HTTP limits](NATIVE_LIMITS.md).

The next candidate is the unchanged historical successor core `ab3c632` for
unpublished Engine 0.7.3. Its package manifest still says 0.7.2. The control is
`engine-v0.6.1` (`4abee90`). Both receive the same previously admitted #552
acquisition-coverage patch and use exactly the same current CLI/provider source.
The build record preserves original/patched core files, the patch, compiler,
source and executable hashes. This comparison is of these documented patched
cores, not the untouched distribution binaries.

Build from a full immutable CLI commit using locally cached dependencies:

```sh
DIR=evaluation/cli-repair-paired-v1/normal_v3
python3 "$DIR/build_native_pair.py" --repo . --cli-commit FULL_COMMIT_SHA \
  --output /absolute/private/build
python3 "$DIR/prepare_local.py" --inputs /absolute/private/consumed-real24 \
  --build /absolute/private/build --output /absolute/private/preparation
```

Use the pinned PyNaCl environment described in [REVIEW_AUTH.md](REVIEW_AUTH.md).
For durable enrollment, invoke `prepare_local.py` from the immutable paired
build's `source-baseline/evaluation/cli-repair-paired-v1/normal_v3` directory,
so runtime measurements refer to that private source snapshot rather than an
actively edited checkout. The Python environment itself remains measured too.
The input directory must contain the original `597e1488...` manifest from
`6165de77` and every byte listed in it, including full raw/normalized primary
documents. Preparation performs no download and never repairs changed input.
No old raw, receipt, tag, run or first score is opened for writing.

The old curated resolver launched Python once for every snapshot. That cannot
run inside the Mac no-fork profile. New `captured_evidence` investigation
capabilities hold the **same acquisition records in explicit configuration**
and return them in process, through the ordinary source/authority/time/scope
admission policy. They do not inject user facts or verification receipts.
Duplicate-snapshot admission, cost accounting, acquisition limits and answer
context remain in the normal CLI. The adapter name changes in model-visible
capabilities, so v3 is a newly frozen profile; it is not a rerun of v1/v2.
Both arms use the same capability identity and inputs.

The private output contains the actual source catalog, exact evidence/quote
bindings, 24 configs, 48 model-free first requests in alternating arm order,
the native bundle and a newly initialized fixed local authority profile. Engine
evidence IDs such as `snapshot:rw-01:0` are data identifiers, not paths; job/file
names retain their stricter path-safe identifier rules.

`status.json` explicitly records that no reviewer is enrolled, no run is
claimed, no tokens are reserved and provider execution is not authorized.
Partial preparation remains evidence of a failed attempt; rerunning at the
same output path fails instead of resetting it. No fake/test reviewer keys or
semantic decisions are created for actual enrollment.

Before the first actual call, complete operator enrollment and independent
source/runtime review, current provider readiness, frozen pacing and both
per-generation/aggregate budget enforcement, the new genuine receipt/plan and
one-shot tag. Then each of the 48 responses must be reviewed against all output
bytes and complete sources before the next job, followed by authenticated
semantic first scoring. A distinct review agent can serve as the reviewer;
cryptographic signatures record the registered decision and exact bytes,
not human presence or semantic truth. See [NATIVE_ADMISSION.md](NATIVE_ADMISSION.md)
and [REVIEW_AUTH.md](REVIEW_AUTH.md).

The 24 questions are already consumed development data. New independent
accuracy, provider quality GO, Engine release and ordinary CLI adoption remain
separate steps under #531. No GCS, VM, cloud signer or cloud project is needed.
