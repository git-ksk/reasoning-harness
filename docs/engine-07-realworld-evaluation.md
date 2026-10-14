# Engine 0.7 real-world evaluation and CLI adoption

[日本語](engine-07-realworld-evaluation.ja.md) | English

Status: **in progress / adoption NO-GO**, 2026-10-15 JST. [Issue #531](https://github.com/git-ksk/reasoning-harness/issues/531), [milestone #12](https://github.com/git-ksk/reasoning-harness/milestone/12). The published product is Reason CLI0.5.5 / Engine0.6.1. Engine0.7.2 is an independent source release, not an adopted CLI Engine.

## Ordered work

1. Measure immutable Engine0.6.1 (`4abee90f0501f0d8fc09b453a0b45c942fbed69e`) and0.7.2 (`296690a86e049b75ee693ae5032a71504366137c`) on identical real-source tasks. Separate development calibration, fresh independent acceptance, and model/provider failures.
2. Fix only measured defects. Preserve original first observations; a fix uses a newly pinned successor, followed by new acceptance. #488 lineage, #489 trusted supersession and #491 answerability/reacquisition require a supported residual and measured benefit. Sufficient existing behavior means record defer/no-op.
3. Select the accepted Engine explicitly; use a dedicated CLI integration PR and Mac/Windows/Linux, installation, distribution, upgrade and rollback gates. #465 infrastructure reliability and #506 WinGet maintenance acceptance remain separate.

## Two required tracks

`e2e_normal` measures the ordinary question → acquisition → Engine decisions → final user-visible answer, using identical CLI/provider host code with each separately built Engine. Capture actual invocation paths, acquired source IDs/versions, policy/admission decisions, every exposed claim and citation, operational failures, calls, attempts, tokens and latency. Building entire historical CLIs with different host code would confound an Engine-only comparison and must be labeled separately.

`helper_opt_in` measures explicitly enabled source attribution/reconciliation on controlled real primary-source snapshots. Actual0.6.1 finalization is the baseline; the candidate additionally exercises its opt-in host-review surface. Record semantic advice separately from review authority. Precommitted evaluator reference labels can authorize only a **controlled test-host review**, not external truth, a real human signature or model self-authorization. Measure attributed target presentation gains separately from complete useful answers.

Do not sum the denominators or call helper success product E2E. Static search at preparation commit `27737ead2b0dfeddcdcebff608e8957bd263c4e0` found no calls to the new relevance/reconciliation entrypoints in ordinary CLI/provider/runtime paths (the sole provider source-attribution occurrence initializes an empty artifact). This is a wiring concern requiring a runtime trace, not proof that every Engine change is inactive or an already completed CLI acceptance.

## Canonical protocol preparation

The [machine-readable protocol](../evaluation/engine-07-realworld-protocol-v1.json) precommits sample sizes, hard gates, useful-answer thresholds and task budgets. Its status is **draft pending complete evaluator/host freeze**. No canonical provider observations or independent acceptance cases have been produced. The isolated, deterministic pilot below does not freeze an E2E evaluator.

Before independent authoring: commit complete evaluators, host/adapters, exact engine identities, model IDs/settings, source snapshot schema, paired ordering, provider pacing, timeouts, numeric thresholds and scoring rubric. Freeze SHA-256 manifests. Author 24 fresh disjoint tasks after freeze, separately from 24 development tasks; include at least eight supported positive tasks plus conflicts, revisions, same-origin duplication/missing lineage, missing information, wrong target/scope and control contamination. An independent reviewer must review full sources and expected proposition/scope/time labels before first observation. Models never receive expected labels. Preserve partial/failed first observations; operational repairs use new identities and never silently overwrite/rerun semantic failures.

Only perform expensive canonical runs once required stages and known source-safety controls are ready. A known uncorrected defect independently blocks adoption, even if the sampled tasks miss it.

## Preparation evidence already collected

- [Published0.7.2 audit](../evaluation/engine-0.7.2-verification-audit-20261014/README.md): a root-authored synthetic14-input targeted matrix finds ten false-Relevant negatives for present-tense retractions/denials and non-event statements. #520 is reopened under milestone#12. Its original repair, acceptance and published tags remain preserved. No downstream wrong answer or Known/Supported promotion was demonstrated.
- [Real-source pilot](../evaluation/engine-07-realworld-pilot-v1/README.md): actual locked/offline builds of both released tags, one identical public-API probe, five official short excerpt snapshots and eleven development inputs. Four of five author-labeled positive relevance cases were Ambiguous in both arms; observed gain0 and positive regression0. All six negative-labeled rows were also Ambiguous, but their parent/precondition limitations prevent safety acceptance. This is a **small, deliberately bounded relevance probe**, not model accuracy, complete answer usefulness, provider acceptance or a representative error rate. The [independent design review](../evaluation/engine-07-realworld-pilot-v1/design-review.md) found context/label, inactive negative-control and partial-journal limitations. Full-source/context and contract review is required before calling these four mismatches product bugs.
- Independent design review required separate tracks, isolated source identities, final-prose/citation scoring, hard gates per provider, and fresh authoring after evaluator freeze. No new independent acceptance set has been consumed.

The pilot uses optimistic fixed **untrusted** proposal/raw qualification to stress deterministic floors. It performs zero model/API calls; token/cost metrics are unavailable. It neither measures the new cross-source reconciliation surface nor authenticates an actual reviewer. Quote SHA-256 is explicitly the short UTF-8 excerpt hash, not the full live page or a publisher signature. Same-publisher documents do not become independent corroboration.

## Adoption checklist — 0/6

- [ ] Required real-data/provider evaluation complete.
- [ ] Paired baseline/candidate results and evidence limits recorded.
- [ ] Necessary repairs and regression/fresh reacceptance complete, including #520.
- [ ] Exact accepted Engine selected.
- [ ] Dedicated CLI integration PR, CI and applicable local/device E2E accepted.
- [ ] Formal CLI distribution, installation and update verification complete.

Source release CI and prior fictional-source12-case gains (+3/+6/+6) remain historical bounded evidence. Neither establishes these six adoption criteria. No automatic0.8.0 feature expansion or0.7.2 CLI adoption is authorized by a favorable helper score alone.
