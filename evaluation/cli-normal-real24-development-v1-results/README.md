# Stopped ordinary v1: immutable first scoring record

Run40873570944 / artifact12896743028 stopped after2 completed,1 failed and45 suppressed jobs. It has only1 complete paired case out of24. The independently annotated, hash-bound first score reports `actual_answer_f1: null`; quality/efficiency gates fail for incomplete coverage. The diagnostic1.0 arithmetic on the single completed positive case or its claims is not full-cohort F1 or acceptance. No missing row is a successful answer, true negative or safety pass.

The original raw/receipt/tag/failure latch are unchanged. `first-score/report.json` and `rows.json` are the single frozen scoring output, after authenticated exact-artifact reconstruction and the persistent score reservation. Retain the original named Docker volume and do not rerun the score. `observations/` and `semantic-review/` retain the exact independently frozen analysis inputs, including both inventories and provenance.

Independent RCA and source-only CLI review are stored separately here; neither is a successor runtime or quality acceptance. PR570 fixes the observed representation-only request-size blocker. The unrelated historical48-request size probe is a root-authored diagnostic, not an independent verdict or a guarantee that a future run fits or improves. A new source/runtime/profile/receipt is required for a new measurement.
