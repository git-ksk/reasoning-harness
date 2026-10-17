# v25 first result: FAIL / NO_GO

The sole first observation under candidate `95053007bba54c728eee428837b22eb9d52a4f61` and the pre-author frozen #547 evaluator is preserved. Both APIs have TP5 / FP0 / TN30 / FN25: precision1.0, recall0.1666666667, F1 0.2857142857. Both fail the recall .85 and F1 .90 floors. Safety-negative errors0; operational errors0; incomplete0. Joint35/60 is diagnostic only. The outcome is NO_GO under the fixed policy.

One Engine process consumed the60 frozen inputs; one scorer process invoked the frozen helper once with explicit config. Probe and scorer returned0, all relevant stderr is empty. The driver returned1 because the semantic gate failed. No retry, requalification, rescore, label change or threshold change occurred.

All25 misses are complete positive observations. `observation-review.json` retains each precommitted rationale and exact returned tuple; `risk-diagnostics.json` copies the input and output enums literally. These data show disagreement, not an internal cause. Returned `model_ambiguous` strings do not imply a real model call. No implementation or current parent diagnostics/tests/development metrics were read.

M0 SHA `b172872d29b19e4308bf86bbce99b570f8d0198a087973def644f679fcb75ac6` was closed before authorship and remains unchanged. M1 SHA `78da5084a379b3717b804794dea6c7b76dd6bab60df25d71db3221990d1baebf` froze all60 input/label/rationale/scope artifacts before observation. Original pre-author `README.md` and `pre-author-*` files retain their historical preparation state. Parent review was referenced by opaque hash/copy only. `authoring-start.json` records the separate authoring authorization. The static checker and scope review were completed before M1; static PASS was not semantic acceptance.

Old source632 files plus separate preparation5 remain hash-identical. Frozen source/core/serde/opaque/dependency/executable pins match before and after. All new artifacts are fsynced/read-only at closure, with a recursive self-excluding final manifest. The final manifest also retains the old cohort files and their original first FAIL criteria/results, never rescored by the new policy.

Parent controls (known1264/1264, original14/14, historical byte equality, no known unfixed critical source defect at M0) are pinned reported evidence, not independent re-execution. No external provider/model/network/Oracle call, Cargo/shared-target mutation, Git mutation, additional agent or real24 authorship occurred.

This finite synthetic #520 source gate does not measure actual answer grounding, #531 real24, ordinary CLI, provider/Groq recovery, human acceptance, release or adoption. Scores across different fresh cohorts are not a measured improvement/regression rate. Cooperative artifact/hash/receipt ordering is not hostile-writer atomic execution or a semantic proof of authorship independence.

Closure ends this identity. Parent may release the candidate freeze only after verifying the separately reported final manifest anchor. No v26 or additional observation is started.
