# Engine 0.7.3 fresh independent v26 decision

2026-10-18 JST. Unpublished source9082365 / content SHA2564f82b2e41614394f1bd528f01c817f0e962f1c546d7212f75f15c2f9c5787999. Package version0.7.2 and published CLI0.5.5 / Engine0.6.1 stay unchanged.

Fresh independent v26 at candidate `9082365` is **FAIL / NO_GO**: each API TP22 / FP0 / TN30 / FN8, precision 1.000, recall 0.733333, F1 **0.846154**. Safety-negative admissions, operational failures and incomplete observations are zero, but fixed F1 0.90 and recall 0.85 floors fail. This is separate from consumed v25 development F1=1.000; different independent cohorts do not measure an improvement rate. First-only Engine 1 process / 60 inputs, scorer 1 / helper 1, retry/rescore/relabel0. Parent review reproduces59 pure fixtures; M0/M1 precede observation, all13 provenance checks pass, and parent verifies99 original new files plus730 old references /829 total corpus references. Original v25 FAIL and all older firsts stay unchanged. Actual consumed diagnostics show7 typed_reference_not_excluded blocks and057 unresolved_adverse_scope. Known controls are now1392/1400; original14, historical597 and workspace1594PASS are separate pinned908 checks. Next: repair shared property/ownership/local-reference handling, pass all1400 known controls, then freeze a new candidate/evaluator before fresh authorship. v27 is not authored. Real24=0/24, actual-answer F1 unmeasured, Groq stopped, source merge/Engine publication/CLI adoption NO_GO. See [decision](engine073-v26-decision.md),[original first](../evaluation/engine073-launch-independent-v26/README-results.md),[actual routes](../evaluation/engine073-launch-trace-development-v10/README.md).

## Fixed first gate

| Axis | TP | FP | TN | FN | Precision | Recall | F1 | Decision |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| v17 requested_relation |22|0|30|8|1.000|.733333|.846154|NO_GO|
| v30 Relevant |22|0|30|8|1.000|.733333|.846154|NO_GO|

The adopted #547 policy/helper remain unchanged: each axis F1>=.90 / P>=.95 / R>=.85 with FP0,100% known regressions,complete first observation/pre-author freeze. Joint52/60 is diagnostic; this FAIL does not impose retrospective exact60 correctness. Recall and F1 fail. Driver exit1 is semantic NO_GO, recorder/scorer exit0, stderr empty and operational errors0. Original first observations are never retried to manufacture acceptance.

## Eight actual paths

001/027/033/041/043/047/055 first block at typed_reference_not_excluded;057 at unresolved_adverse_scope. Separate-copy cfg(test) observers match all8 original effective tuples. Diagnostics expose an uncollected prefixed role declaration001; excluded foreign-role reference027; printed-property values classified Launch033/055; foreign claim classified Launch041; unresolved material property043; prior foreign-product reference047; and prior foreign-claim reference057. These are actual instrumented paths, not internal-risk-loss inference from enums. Repair shared complete declarations/ownership/local references while preserving target/global denials,unknown/conflicting owners,real gaps and raw-risk boundaries.

Known controls are1340+new60=1400,with1392 passing. Repair8 abstentions and all known regressions before consuming freshv27,then recheck original14,historical byte compatibility and full checks at the new source. Preserve originalv26 FAIL/input/gold/evaluator/policy forever; consumed development gets a separate prefix.

## Preservation and next stage

M0 ad980eba8479eec16e78f5ef2c2d108ac949ed1cf4b6fe2277df4dc0e3742510,M1 7627332228e422c58b64890434f681066469d77ab3326a8037e9b8379508c366,final anchor8214db3607e32d707da797fb90bb4af77b1101db976b6f0d0a60f96d49cdb921. Parent review reproduces59 pure fixtures and verifies13 runtime-provenance checks/hash/readonly ordering plus first-result arithmetic. [Parent copy verification](../evaluation/engine073-launch-v26-parent-verification/final-copy-verification.json),[arithmetic report](../evaluation/engine073-launch-first-metrics-v26-report.json). Usage-limit interruption occurred after author-start only,before case save or semantic observation; the same start resumed,with no original-first retry.

Pinned908 full checks1594PASS/0FAIL/8existing ignored,strict Clippy/fmt/fixtures/archive,and evidence1d7019a CI19/19SUCCESS are separate source records,not acceptance of a future repair. [Current908 static CLI preparationv5](../evaluation/engine07-cli-path-preparation-v5/README.md) scans42 core/62 host files without ordinary CLI literal references to the evaluated APIs; this is not complete call-graph/all-path inactivity proof. Real-world M0 still requires an active ordinary path,acquisition/admission/claim/citation/display trace,complete semantic scorer,and actual callback/dependency/numeric-budget gates. Fresh real24 not authored/0 of24,Groq stopped. Continue repair→fresh source acceptance→real-world evaluation→CLI adoption.
