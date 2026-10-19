# Independent review of practical evaluation components

Result: **FAIL for complete real-document M0 readiness**. The current modules correctly remain preparation with `acceptance=NOT_EVALUATED` and actual-answer F1 unmeasured. No current or historical answer was rescored.

The 32 arithmetic/span constructed tests and four strictly in-memory trace tests pass. Saved-corpus-dependent trace tests were read but not run to preserve the independent source-author boundary. New adversarial fixtures reproduce:

1. One occurrence ID reused at two displayed fact spans is accepted and counted once. Fix location/occurrence cardinality before complete display safety review.
2. A model response that completes a request after finalization is accepted as `STRUCTURAL_MATCH`. Require the complete terminal event state machine.
3. The isolated renderer's guard accepts `None == None` for an invalid source span and absent/non-string quote. This is an expression-only std-Rust reproduction with no Engine linked; no forged final citation is claimed.
4. An empty model-response dict is counted as a complete pair. This demonstrates a missing full-M0 response/usage schema gate, beyond the current adapter's identity-only scope.

`review.json` records exact code locations, counterexamples, matched negative controls, limits and concrete remaining M0 gates. `independent_checks.py` and the input/raw logs preserve the first component observations; they do not author a new semantic holdout or new real-document gold. Pure constructed arithmetic F1 values must not be presented as actual-answer F1.

Existing output identities/byte spans and source-chain checks are useful preparation. They do not judge semantic support, source authenticity or applicability, citation correctness, usefulness, equal-condition paired gains, live budgets or complete callback/dependency containment. The active relevance Node control withholds in both arms; isolated manual renderer output does not rescue that result. No demonstrated quality gain or cross-cohort improvement is asserted.

All three original component inventories/indexes and all 928 v27/old/preparation references remain exact. The earlier post-copy v27 archive check passed for both copies' 99 readonly files; the managed source27-freeze is archive-safe. Root/Git/source/old scorer and artifacts are unchanged by this review.

Provider/model/network/Oracle/Groq, Engine, actual-cohort scorer, real24/source-case/gold authorship and additional agents: 0. Parent will repair in a new namespace and provide a separately frozen integrated stack for review. No fresh authorship is authorized by this result. The reviewer stops after readonly closure.
