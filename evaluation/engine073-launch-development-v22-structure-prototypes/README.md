# Structural repair development prototypes

All measurements use the same consumed frozen independent v22 inputs/labels. Original first FAIL, all551 predecessor files, scorer and published history are unchanged. These stages are development only, not six independent evaluations or ordinary-answer F1.

| Stage | v30 TP/FP/TN/FN | v30 F1 | Joint pass |
| --- | --- | --- | --- |
| c20 baseline | 9/0/30/21 | .461538 |39/60|
| appearance01 |20/0/30/10|.800000|50/60|
| printed02 REJECTED |24/1/29/6|.872727|53/60|
| global03 |24/0/30/6|.888889|54/60|
| structure04 |27/0/30/3|.947368|57/60|
| segmentation05 |29/0/30/1|.983051|59/60|
| name06 |30/0/30/0|1.000000|60/60|

FP1 in printed02 (li22-016) incorrectly ignored a collective withdrawal after a printed-property role. It is preserved and rejected. global03 checks collective event withdrawal before local borrowing; source structure04 separates quoted event/document owner, complete printed apposition and adjacent same-name foreign confirmation. segmentation05 retains maintained/parenthetical clauses and refuses an expressly unknown non-event owner. name06 bounds the negated clipping-note projection to existing canonical Clip/actual-target-fact prerequisites.

New unit-test design failures are retained separately, including an initial pronoun control replaced by an explicit unknown-owner control. The latter exposed a production boundary and required a guard. Strict all-target Clippy initially rejected clone_on_copy in the new test, before any full test stage; the final root source differs from name06 by only removing that redundant clone. The initial failing logs are not PASS evidence.

Each stage's identity was saved before its single development60 process and all source/library/input/label hashes were checked afterward. Scorer invocations0; provider/model/network/Oracle0. Before/after identity, full source snapshot, cargo artifact metadata, raw output and metrics are retained. original artifact-sha256.json also hashes a private tmp probe executable: executable bytes and linked core/serde libraries remain external/tmp, not packaged here. Their digests are provenance, not a claim that binaries are durably archived by this directory. New durable-artifact index excludes only itself. First6 stages =6 development processes/360 inputs. Additional preliminary old1080 =18 processes/1080, captured summary/log, not fresh independent acceptance. Final pinned actual-worktree validation is separate.
