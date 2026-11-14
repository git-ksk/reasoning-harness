# Issue #531 — new 24-case **development-only** preflight (2026-11-14 JST)

**NOT formal F1, NOT an independent cohort, NOT a GO.** This record reports only an offline comparison of the first planned model requests for the fixed native pair. No model/provider calls were made, no additional tokens were consumed and no final answers were scored.

## Fixed identities

- Baseline Core 0.6.1: `4abee90f0501f0d8fc09b453a0b45c942fbed69e`
- Corrected candidate Core (unreleased): `7466c6f90d420a98c99713b96d6a0623e2aeef52`
- Shared CLI: `988b69f086ac4a209fe93eb10cf1fa6786977567`
- Model planned: `ministral-8b-2512`
- Source repository main untouched by the 0.7.3 candidate; #609 was merged into candidate staging, not the published main engine.

## Private, original but **not independently authored** material

24 newly written, disjoint fictional case packets (12 clear support, 8 clear no-answer, 4 ambiguous) were authored **by ChatGPT in this evaluation conversation**. They include conflict, wrong target, correction, retraction, denied launch, invented source, scope and quotation-risk scenarios. They are *not* independent real-source holdout cases and must not be relabeled or reused as the formal #531 24-case cohort. The corpus, gold labels and supporting source snippets were frozen before the offline preflight and retained locally in a restricted user Mac directory, not added to this public repository or its artifacts.

- Private corpus SHA-256: `63da84b7176766c2fec9becf7ad631ff03b0b7c7264f4f120ac05816abd43e1c`
- Offline preflight report SHA-256 (private original): `7edb6915921ebcfeaab99fbb67073a67d12b37436cf3f14390d5b50d39e1f8e5`
- Same pinned pair as the completed four-case Dev Lite V5 experiment, using two pre-existing checked native binaries. First planned request content and SHA-256 matched for each of 24 pairs: **24/24 matches, 48/48 requests validated**; zero API calls.
- The sanitized `paired-preflight.json` includes request SHA-256 values and arm identity without questions, sources, gold labels or raw answers.

## Formal gating and incomplete measurements

- **Formal independent author/reviewer signoff: missing**; the same ChatGPT session created these provisional examples. An automated self-review, CI, or hashing does not establish independence.
- **Verified current Mistral billing/no-overage settings and monthly used credits: missing**, beyond the user's historical Free-plan screen observation of about $9.98 remaining on this date. Authentication/API connectivity were previously demonstrated, but authorization is not current quota evidence.
- Actual new24 model pair results: **0/48**; no substantive claim/quote audit for these cases. Old 4×2 developmental Mistral outputs must not enter this cohort.
- Baseline formal F1 / Candidate formal F1 = **null / null** (not 0); TP/FP/FN for new24 = not measured; quality GO=false; package/CLI adoption GO=false; release not authorized.
- Formal quality: candidate case and claim F1 >=0.90, Precision >=0.95, Recall >=0.85, no hard safety faults, citation fidelity/coverage=100%, no useful-positive regressions, all 24 paired observations; efficiency mean logical-call delta <=1, token ratio <=1.25, p95 time ratio <=1.5. A complete independent reviewer must see the full question, source, model output, asserted claims and citations in secure storage before any formal signoff.

## Safe continuation

1. Freeze **a different, independent, source-reviewed real-world 24-case holdout** and reviewer agreement, after scorer policy/gates are fixed. Do not promote this generated fictional packet to independent status.
2. Verify Mistral organization current usage/remaining included credit and pay-as-you-go/spending cap in the authenticated console **before any new API calls**, compute a capped max cost. Previous V5 paired jobs succeeded on macos-15 at around 305 seconds per CLI; 270 seconds was insufficient.
3. Use the pinned native pair and same per-case first-request plan, model, seed, config, budgets, no automatic retries and no public raw-source/artifact leaks. Store complete raw outputs encrypted/private for an independent claim/citation audit; record each completed/incomplete job, calls, input/output tokens and elapsed time.
4. Compute both case- and claim-level TP/FP/FN, precision/recall/F1, abstention, quote/source/version matching, scope/retraction safety and efficiency only for properly observed pairs. Any incomplete row or absent independent review prevents formal PASS.
5. Keep current main, historical v1-v6 receipts, previous evaluation results and all published Engine/CLI releases unchanged until #531 gates pass.
