# Engine 0.6 evidence relevance fixed calibration core v1

Status: selected after v13 canonical had already failed, before any v14 live observation. This is a successor calibration core, not a holdout.

The previous calibration line grew 65 -> 73 -> 81 cases by appending fresh cases for each new hypothesis. That preserved regression history but increasingly coupled corpus size to prior failures. Starting with the next successor, calibration uses a fixed 48-case core selected from v13 by semantic coverage rather than by adding a new slice for every iteration.

Policy:
- case count is fixed at 48;
- do not append cases in-place;
- do not replace a case merely because a model misses it;
- if a genuinely new semantic dimension is discovered, record it separately and revise the fixed-core identity only through an explicit new core version;
- independent holdout remains separately authored only after canonical calibration PASS.

Coverage:
- Relevant: 14
- Irrelevant: 18
- Ambiguous: 16
- identity-mapping cue present: 6
- ownership-scope cue present: 5
- context-gap cue present: 10
- explicit local absence present: 4

The core retains representative positive exact/alias/paraphrase/distributed/cross-lingual/metadata/freshness/injection cases; wrong-feature/sibling/navigation/broad/unrelated/comparison/relation/injection/explicit-absence negatives; and rename/partial/mixed/conflict/insufficient/URL-only/shared-ownership/context-gap ambiguous cases.

Execution policy for the next successor:
- all fixed-core cases are attempted for every canonical provider arm;
- a provider failure still fails operational completeness, but the runner must continue through the remaining cases for diagnosis;
- per-case absolute deadline remains bounded;
- no failed canonical is rerun or rescored.
