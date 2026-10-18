# Claim arithmetic preparation for future real-document evaluation

This is a development component for #531. It implements the claim P/R/F1 and
zero-denominator arithmetic proposed in
`docs/engine073-realworld-metrics-preparation.ja.md`. It is **not** the complete
semantic scorer, a new evaluation M0, or an acceptance result. No existing first
observations are re-scored. No provider, model, network, Oracle, Engine or Groq
calls are made. Tests use constructed annotations rather than real documents.

`display_binding.score_bound_case` joins UTF-8 byte/span checks to the arithmetic.
It verifies a recorder-supplied display inventory and canonical bundle digest,
requires every byte to have a factual occurrence or justified nonfact span, and
rejects gaps, overlaps, omitted artifacts and swapped claim/proposition IDs.
This checks declared coverage; annotating a factual sentence as nonfact can still
be wrong. Semantic segmentation/support and recorder completeness remain external.

`claim_metrics.report(plans, reviews)` preserves all planned IDs. It reports
separate provider / track / arm / family / answerability strata; it does not pool
helper gains into ordinary answers or combine providers into a pass. Case quality
F1 under #542 is a different metric and remains in its existing module.

- A = distinct reviewer-declared factual propositions in displayed output.
- S = propositions grounded in every annotated occurrence.
- G = fixed required supported requirement groups; synonyms share one group.
- H = distinct required groups met by grounded output.
- P = S/A, R = H/G; claim F1 is their harmonic mean. The two numerators need
  not describe the same set. Supported extra statements cannot create recall.
- A=0, G>0 gives P=null, R=0, F1=0. G=0 gives R/F1=null; empty answers are never
  assigned a perfect F1. Counts and null/zero reasons are retained.

Semantic micro P/R/F1 use the same operationally complete, externally reviewed
subset. Macro F1 excludes null cases but reports their count. A separate
`delivered_required_coverage` divides actually satisfied groups from complete
reviewed rows by **all planned** groups. Missing, failed and unreviewed rows
remain unavailable, receive no delivered credit, and remain visible for safety
audit coverage. A missing review does not prove a request was never made: its
operation is `unobserved`, distinct from an explicit `not_run` record.
Known safety violations on failed/partial output remain in the report. Zero
known violations with unreviewed rows never proves zero violations.

Input authority and remaining work:

1. `plans` is a caller-created join envelope containing immutable gold IDs,
   strata, required group IDs, and the **later observed** bundle hash. It is not
   the frozen gold file. Future output hashes must never be inserted into frozen
   gold. The hash is null when no bundle exists.
2. The trusted caller must compute the observation bundle hash over all answer,
   claim-card and citation displays and pass the independently captured identity.
   Comparing equal digest fields here does not establish file, trace or citation
   integrity. Complete/incomplete observations require a digest; absent bundles
   cannot receive semantic credit.
3. External review supplies every factual claim occurrence, proposition identity
   including negation/scope/strength/time, grounding status, satisfied groups and
   known violations. Reviewer ID and human/agent/host role remain explicit.
   `review_complete` is an external assertion, not independently proven here.
   Supported explanations in an unanswerable task may have required groups; they
   do not establish that the question's truth facet was answerable.
4. This module rejects duplicate IDs, foreign gold groups, swapped observation
   hashes/strata, invalid booleans and credit from unsupported claims. It cannot
   discover omitted claims, verify semantic identity/entailment or citation
   lineage, establish reviewer independence, or prove annotation correctness.
5. Semantic segmentation review and retrieval→admission→evidence→claim
   trace binding, source applicability/correction/contradiction adjudication,
   citation measures, useful-answer classification, paired baseline comparison,
   runner/callback/dependency containment and numeric live budgets remain before
   a complete real-document M0. No new claim thresholds are adopted here.

Run: `python3 -B -m unittest discover -s evaluation/engine073-answer-metrics-development-v1 -p 'test_*.py' -v`

All report/join output has `acceptance: NOT_EVALUATED`.
Final combined checks:32 passed (18 arithmetic and14 byte/span/join checks). Real-document cases remain 0/24;
actual-answer F1 is unmeasured; Groq is stopped; source release and CLI adoption
remain separate gates.
