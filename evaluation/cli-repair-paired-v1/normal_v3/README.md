# Prospective ordinary-CLI v3 collector / annotator / scorer (#574)

**Status: code/fixture implementation only; not a runtime admission, scoring acceptance, or permission to call a model.** This code is based on PR #576's private typed CLI diagnostics. It must be reviewed and tested independently before merging. The old v1/v2 raw, receipt, tag, annotation and first scores remain immutable.

This module is deliberately separate from the historical `collect.py`, `score_development_first.py` and frozen v1/v2 operational bridge. It is **not connected to the live scheduler**. That work belongs to the next #574 implementation stage, together with independent actual-runtime and image admission.

## Inputs and data provenance

The caller must first create a new (prospective) **post-execution capture manifest** in a private working area, containing:

- `contract = reason-cli-normal-v3-capture-manifest-v1`, the exact provider/model, `executor_id`, and SHA-256 of a **new** genuine receipt (never a historical receipt).
- One unique `(case_id, arm)` per job, the frozen `question`, a safe `job_id` and exact SHA-256 hashes of `cli.json`, `trace.json`, `http.json`, `capture.json` and `sources.json`. The manifest should ultimately be tied to the frozen new runtime, plan, M1 and execution artifact. This module does not perform that admission.
- `cli.json` is the unmodified CLI `reason-cli-output-v1` ask envelope. `trace.json` is the actual diagnostic trace. `http.json` is the **independently normalized** ordered HTTP-attempt inventory containing `request_id, provider, model, http_status, input_tokens, output_tokens, total_tokens` per attempt. Never synthesize it from the CLI's own totals.
- `capture.json` contains `display_artifacts`, `display_citations` (byte ranges, id, source id/version), and `admitted_source_ids`, generated from an **independently verified source/admission capture**. `sources.json` supplies complete source text and version for every cited/admitted source. The v3 collector verifies referential consistency; it cannot on its own establish that external records are genuine.

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

The reviewer also freezes a `review-index.json` with contract `reason-cli-normal-v3-review-index-v1`, exact receipt hash, observation index hash, packet index hash, the *original* executor ID, and `reviews: {job_id: sha256(review_file_bytes)}`. This freezing/independence must be externally established by the upcoming runtime admission; a self-declared Boolean is not an independent attestation.

```bash
python3 "$DIR/score.py" --observations /private/v3/observations --packets /private/v3/packets --gold /private/v3/gold.json --reviews /private/v3/reviews --receipt /private/v3/receipt.json --output /private/v3/first-score
```

**All three stages use exclusive create-only output directories.** They refuse to overwrite files *within the same output directory*. **This is NOT a durable, receipt-wide first-score replay lock:** a different output directory could still be used for a second scoring run. Before any genuine v3 first score, stage ③ must integrate a trusted, persistent receipt-bound score reservation/claim and reject changed output paths, alternative ledgers or replay. Until then even a synthetically successful score is not an accepted first score. Keep private source/packet materials out of public repository and issue comments. Only status summaries and reviewed, non-secret metadata should be published.

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

The **20-test** suite exercises all six typed phases, provider protocol discrimination, fallback recovery, 429/non-2xx operational stop, unknown/changed usage, missing trace, partial-success trace refusal, forged terminal display, wrong-request case identity, full-display factual-byte review, citation gaps across multiple assertions, changed source/citation bytes, case/gold binding, invalid negative/positive scoring, unsafe priority, 24-case denominator, receipt/packet/review tampering, and same-output-directory first-score replay rejection. These tests are **synthetic only** and cannot substitute for the independent real-runtime acceptance gate.
