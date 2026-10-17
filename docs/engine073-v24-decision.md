# v24 source decision and next work

Candidate3ccda6b, new independent firstFAIL 47/60. v17 TP17/FP0/TN30/FN13、Precision 1.000000・Recall 0.566667・F1 0.723404；v30 TP17/FP0/TN30/FN13、Precision 1.000000・Recall 0.566667・F1 0.723404

| Axis | TP | FP | TN | FN | Precision | Recall | F1 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| v17 |17|0|30|13|1.000|0.567|0.723|
| v30 |17|0|30|13|1.000|0.567|0.723|

This is the **historical v24 first observation** under the then-frozen strict 60/60 source gate: FAIL remains unchanged. **Future newly frozen #520 independent synthetic source gates** now require F1>=0.90, precision>=0.95 and recall>=0.85 for both API axes, zero unsafe negative admissions and 100% known regression tests. This is separate from #531 real24, and never retroactively passes v24. Original v23 F1 .500 and every first failure remain immutable. Development v23 F1=1 is not independent acceptance. Different fresh corpora are not a measured improvement trend. First Engine/scorer once;no retry/rescore/relabel.

Candidate `9505300` repairs the13 consumed v24 misses and four additional event-owner note controls. On the same frozen60 development cases, both APIs have TP30/FP0/TN30/FN0 and precision/recall/F1=1.000. Original independentv24 firstFAIL47/60/F1 .723404 remains unchanged. Pinned-source validation: workspace1582PASS/0FAIL/8existing ignored,strict Clippy/fmt,known1264/original14 PASS and historical597 whole-output byte equality. Fresh independentv25 is preparing an evaluator/candidate freeze in a dedicated copy before authoring; no acceptance result exists yet. Future #520 cohorts follow [#547 policy](../evaluation/engine073-source-gate-v1/README.md): each API F1>=0.90,precision>=0.95,recall>=0.85,zero false admissions on safety negatives and100% known regressions. Never rescore earlier first records. Actual-answer F1 is unmeasured;real24=0/24,M0/M1 incomplete,release/CLI adoption NO-GO. See [repair and next work](engine073-v24-decision.md).

Before any new real24 authorship: complete actual semantic scorer/adjudication, immutable evidence/claim/citation trace, host callback/dependency closure,numeric live/headroom/currency budgets and feature-active ordinary CLI path;freezeM0. Prior01dc paired controlled Node path has0 relevance API calls with working direct counters. Static preparation for current candidate/core and common CLIhost finds these two literal API names only in relevance module/exports/evaluation bins, not other ordinary source files;this is a symbol inventory, not a complete call graph or runtime all-path proof. Do not interpret helper API scores as ordinary final-answer improvement.

Real24=0/24,M0/M1 incomplete,adoption1/6;published CLI0.5.5/Engine0.6.1 and sourceEngine0.7.2 unchanged. Groq remains stopped;provider/model/network/Oracle additional0. Full release/adoption remainNO-GO.
