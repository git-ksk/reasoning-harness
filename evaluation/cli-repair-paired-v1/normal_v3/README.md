# Prospective ordinary-CLI v3 collector / annotator / scorer (#574)

**Status: code/fixture implementation only; not a runtime admission, scoring acceptance, or permission to call a model.** This code is based on PR #576's private typed CLI diagnostics. It must be reviewed and tested independently before merging. The old v1/v2 raw, receipt, tag, annotation and first scores remain immutable.

This module is deliberately separate from the historical `collect.py`, `score_development_first.py` and frozen v1/v2 operational bridge. It is **not connected to the live scheduler**. That work belongs to the next #574 implementation stage, together with independent actual-runtime and image admission.

## Inputs and data provenance

The caller must first create a new (prospective) **post-execution capture manifest** in a private working area, containing:

- `contract = reason-cli-normal-v3-capture-manifest-v1`, the exact provider/model, `executor_id`, and SHA-256 of a **new** genuine receipt (never a historical receipt).
- One unique `(case_id, arm)` per job, the frozen `question`, a safe `job_id` and exact SHA-256 hashes of `cli.json`, `trace.json`, `http.json`, `capture.json` and `sources.json`. The manifest should ultimately be tied to the frozen new runtime, plan, M1 and execution artifact. This module does not perform that admission.
- `cli.json` is the unmodified CLI `reason-cli-output-v1` ask envelope. `trace.json` is the actual diagnostic trace. `http.json` is the **independently normalized** ordered HTTP-attempt inventory containing `request_id, provider, model, http_status, input_tokens, output_tokens, total_tokens` per attempt. Never synthesize it from the CLI's own totals.
- `capture.json` contains `display_artifacts`, `display_citations` (byte ranges, id, source id/version), and `admitted_source_ids`, generated from an **independently verified source/admission capture**. `sources.json` supplies complete source text and version for every cited/admitted source. The v3 collector verifies referential consistency; it cannot on its own establish that external records are genuine.

**Exact frozen first-request identity (#584):** Each prospective manifest
row must contain a `frozen_request_binding_sha256` computed *before
execution* as SHA-256 of canonical JSON
`{"question": <frozen question>, "request": <complete expected first CLI model request>}`.
The collector compares it to the actual first diagnostic `model_request`
for **failed and successful** jobs; a substring match against `task` is
not accepted. The inert scheduler similarly freezes all 48 per-job hashes in
its plan before spawning. This prevents accidental or malicious re-use of a
different question/request under the *same frozen manifest*, but only an
independently attested receipt/plan (#580) can certify the manifest itself.
It does not establish a provider-side signed request or authorize live use.

Do not use the collector to convert provider protocol errors, HTTP non-2xx or unreported usage into an eligible quality failure. It checks actual model-response trace counts, token accounting, reported HTTP statuses, phase/origin, job identity, frozen question in the first traced model request **for both successful and failed jobs**, displayed answer bytes, citation ranges, source versions, and the five captured-file hashes. A successful CLI result additionally requires a completed trace with the same last traced finalization (status, displayed text and uncovered propositions), known CLI exposure policy, canonical final-answer capture kind, and UTF-8-aligned citation boundaries; a partial trace cannot be silently promoted. It retains usage on local decoding failures but **never authorizes continuation**. Missing or contradictory evidence fails closed.

## Offline three-stage workflow

With private immutable inputs that have already passed independent capture/runtime controls:

```bash
DIR=evaluation/cli-repair-paired-v1/normal_v3
python3 "$DIR/collect.py" --capture-manifest /private/v3/capture-manifest.json --lane /private/v3/lane --output /private/v3/observations
python3 "$DIR/annotate.py" --observations /private/v3/observations --gold /private/v3/gold.json --output /private/v3/packets
```

The gold file is a **new** `reason-cli-normal-v3-gold-v1` record with the *same* receipt SHA-256 and 24 distinct fixed cases (`case_id, question, label, required_groups`). Cases are positive or negative. Positive groups are nonempty, negative groups must be empty. The reviewer packets preserve the exact captured display, citations, full sources, gold group obligations and typed failure diagnostics. They contain **no automated semantic answers**.

An **independent reviewer** must inspect the original full-source context and complete displayed bytes, including unsafe assertions, citations, quotes and incomplete/fallback outputs, then create one `<job_id>.review.json` per observation. Each review uses `reason-cli-normal-v3-independent-review-v1` and fields `job_id`, `observation_sha256`, `gold_sha256`, `reviewer` (`role`, `id`, `independent_of_execution=true`, `full_source_context_reviewed=true`), `decision` (`answer|withhold|invalid`), `correct_groups`, `incorrect_assertions`, `hard_violations`, `citation_checks` (`id, supported, literal_exact`), `segments` (a complete contiguous UTF-8 byte inventory of the displayed answer, each span labeled `fact` or `nonfact` and listing attached citation IDs), `readable_and_actionable`, `safety_audit_complete=true`, and `notes`. Each `fact` span should represent one independently adjudicated factual assertion, even when several assertions share one sentence or citation. The validator rejects gaps, overlaps, malformed byte boundaries, citations unattached to factual spans and incomplete citation audits. No gold, observation, source or terminal field can be replaced by a review.

The independently authenticated receipt must bind a canonical absolute `score_ledger_dir` ahead of the prospective first-score; do not reuse a historical receipt or casually redirect the ledger.

The reviewer also freezes a `review-index.json` with contract `reason-cli-normal-v3-review-index-v1`, exact receipt hash, observation index hash, packet index hash, the *original* executor ID, and `reviews: {job_id: sha256(review_file_bytes)}`. This freezing/independence must be externally established by the upcoming runtime admission; a self-declared Boolean is not an independent attestation.

```bash
python3 "$DIR/score.py" --observations /private/v3/observations --packets /private/v3/packets --gold /private/v3/gold.json --reviews /private/v3/reviews --receipt /private/v3/receipt.json --output /private/v3/first-score
```

**Private artifact hardening (Issue #579):** New v3 collector,
review packet and first-score output directories are created exclusively with
same-user mode 0700, regardless of the caller's umask. V3 JSON evidence and
receipt/first-score claims use exclusive O_NOFOLLOW/dirfd creation, 0600 mode,
file fsync and parent directory fsync. The receipt-bound score ledger and inert
run ledger must already be owned by the current process UID with mode exactly
0700; permissive, symlinked or replaced paths fail closed. Input receipts,
reviewer identities and upstream capture/source/HTTP evidence are **not** made
trustworthy by filesystem permissions; an independent external run authority
and cross-host receipts still block live admission. The scope is prospective
v3 only; no historical v1/v2 files are migrated or overwritten.

The private-path helpers check **every path component** via no-follow directory FDs, disallow untrusted writable ancestors, and reject macOS ACL *allow* entries (while tolerating restrictive deny entries). Native macOS root-managed `/var` and `/tmp` aliases are mapped to `/private/var` and `/private/tmp` for compatibility. They do not protect against processes already running as the same UID, external storage rollback, or forged receipt authorities. A failed file/directory fsync remains a failed operation; a partial claim is never silently retried.

**All three stages use exclusive create-only output directories.** The prospective first scorer also holds a durable O_EXCL claim keyed to the receipt SHA-256 in the score-ledger directory frozen inside that receipt, rejecting another output path for the same receipt. This remains **engineering-only** until a separately authenticated receipt, authoritative ledger and real runtime/image acceptance are proven; an untrusted receipt cannot self-certify an authoritative lock. Keep private source/packet materials out of public repository and issue comments. Only status summaries and reviewed, non-secret metadata should be published.

## Immutable scoring rules

- Observation completeness and semantic correctness are separate. For either arm, exactly **24 bound cases and no operational errors** are required for a full-cohort F1, otherwise that arm's F1/P/R and invalid rate remain `null`, never implicitly PASS.
- A fully observed terminal invalid of a **positive** case contributes case FN and misses every required claim group. A **negative** terminal invalid is an error/invalid, **not TN** or correct abstention. The invalid rate uses the fixed 24-case denominator, and counts are also grouped by typed origin/phase/failure class.
- A recovered canonical/fallback answer with a typed intermediate decode error can earn credit **only via the independent semantic review**. A failed/empty output cannot earn correct-withhold or source/claim credit. A displayed unsafe assertion overrides invalid: negative FP or positive FN, plus hard safety violation.
- The report distinguishes TP/FP/FN/TN, case/claim P/R/F1, invalid counts/rates, recovered-local-decode rates/classes, **valid answers versus valid finalizations**, task success, **per-annotated-factual-assertion** citation coverage (including exposed facts in invalid/unsafe outputs), citation fidelity and hard safety violations. A terminal invalid blocks semantic gate even when F1 stays 1.0. Baseline and candidate are independently reported. Semantic thresholds are case **and** claim F1 ≥ 0.90, precision ≥ 0.95, recall ≥ 0.85, citation fidelity/coverage = 1, hard safety violations = 0, invalid = 0.
- **`quality_go` is always false in this stage.** Source/image/runtime admission, durable 48-job scheduler controls, fresh receipt/tag, Mistral evaluation, non-regression/efficiency and new independent accuracy remain separate gates. No Google/Groq model calls.

## Offline controls

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s evaluation/cli-repair-paired-v1/normal_v3 -p 'test_*.py' -v
```

The baseline **20-test** stage-② suite exercises all six typed phases, provider protocol discrimination, fallback recovery, 429/non-2xx operational stop, unknown/changed usage, missing trace, partial-success trace refusal, forged terminal display, wrong-request case identity, full-display factual-byte review, citation gaps across multiple assertions, changed source/citation bytes, case/gold binding, invalid negative/positive scoring, unsafe priority, 24-case denominator, receipt/packet/review tampering, and same-output-directory first-score replay rejection. These tests are **synthetic only** and cannot substitute for the independent real-runtime acceptance gate.

## Stage ③: prospective dry-run scheduler verification

This directory additionally contains **`offline_lane.py`**, a versioned
integration of the actual preserved durable POSIX subprocess runner with the
ordinary-CLI nested-error bridge. Its only execution contract is
`SYNTHETIC_V3_HTTP_ZERO_CONTROL_NOT_MODEL_EXECUTION`, which enforces
`mistral` / `fixture-model`, the pinned inert fixture-driver SHA256, 24 fixed
cases × alternating 2 Engine arms, frozen fixture-file SHA256 entries, frozen
Python executable, a synthetic receipt that explicitly disallows real model
calls, and an exclusive filesystem receipt-bound run ledger. No live
credential callback is ever called; the full inherited environment
is removed before the child starts. The fixed fixture invocation uses Python
`-I -S` so unfrozen user-site startup hooks cannot run.

The offline subprocess fixture emits precomputed fake CLI JSON and trace plus
fake normalized HTTP records. **Fake HTTP200s are NOT provider communications,
billable tokens, or an independent HTTP provenance proof.** A typed fully
metered local decode failure with **no actual displayed assertion** becomes
`quality_invalid` while the durable runner accounts for the charge and
advances. This is *observed* not quality-pass. Actual non-2xx/quota/usage
unknown, a prior operational bridge failure, any unapproved extra retry,
budget reservation breach, missing/corrupt fixture, or displayed
unreviewed assertion latches the whole arm and suppresses all subsequent
spawns. A displayed factual assertion is conservatively held because a
retrospective semantic reviewer cannot attest it before the next paid call.

The code does not relax frozen real 48-job budgets, retry/pacing/schema, or
run credentials. It **does not authorize** the paid-model adapter or an
unadmitted image; wiring a genuine provider journal, stage-③ real-execution
pre-advance attestation with trusted signatures, complete full-display safety
validation, authoritative receipt/run identity and independent source/image
acceptance remains a separate blocker before live use. The synthetic O_EXCL
ledger is a replay regression control, not an external distributed lock.

The prospective first scorer now also requires that a **new genuine receipt**
contain a canonical absolute `score_ledger_dir` directory. It creates and
fsyncs a receipt-SHA-bound O_EXCL first-score claim before writing any output,
preventing an alternative output path from scoring the same receipt twice.
This replay guarantee assumes an *independently authenticated and frozen
receipt*, a trusted persistent filesystem and a single authoritative ledger.
The test suite exercises alternate-path refusal, but a future stage-④
independent verifier must validate these trust anchors. Historical v1/v2
receipts/score outputs are not migrated or changed.

### Independent adversarial review hardening (PR #578)

The v3 collector now reconciles CLI `model_calls` with each actual
`model_request` event (including multiple `generate()` attempts on one
correlation), checks contiguous chronological HTTP attempt groups, rejects
unknown trace kinds, and requires sequential per-adapter attempt identifiers.
A response's `provider_attempts` remains a count of actual HTTP calls,
including internal provider retries; it is **not** interchangeable with CLI
`model_calls` or with the number of correlation IDs.

The synthetic runner now invokes only the byte-pinned fixture executable via
`python -I -S` and provides an **empty child environment**. Previously,
inherited `PYTHONUSERBASE` could cause arbitrary `usercustomize.py` execution
before the pinned fixture code. The offline-only counterexample uses a local
marker file and no network call.

The independent tests also cover a local decode followed by HTTP 429, two
simultaneous OS processes competing for the exact same receipt, a strict
per-generation attempt cap, and suppressing all subsequent spawns on error.
The offline fixture's legitimate two `generate()` calls per adapter are
both counted toward `max_generations`; permitted provider-internal retries
are separately subject to `max_attempts_per_generation`.

**Remaining limitations:** The ledger is only a local filesystem O_EXCL
barrier. It is not a trusted, remote, distributed receipt authority; directory
ownership, permissions, tampering, time-of-check/time-of-use replacement,
process SIGKILL/crash descendant cleanup, and provider-side billed usage
reconciliation need independently admitted host controls before live use.
A local synthetic trace and its matching HTTP/CLI data cannot establish
that a provider really returned a response or that all user-visible output
was captured. No real provider credentials or requests are authorized.

Run all inert integration and existing scorer tests on the checked-out source:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover \
  -s evaluation/cli-repair-paired-v1/normal_v3 -p 'test_*.py' -v
```

The 39-test independently extended suite includes a full 48-job actual subprocess-runner pass in which a
metered fake initial JSON error continues; quota, unreviewed display,
unknown usage, unapproved internal provider retry, corrupted capture, and
aggregate reservation breach all suppress the following 47 jobs. No new
Mistral, Google or Groq generation, v3 freeze/tag, real billing or new F1
occurs from these tests.
