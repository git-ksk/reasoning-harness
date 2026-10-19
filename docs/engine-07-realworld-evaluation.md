# Engine 0.7 real-world evaluation and CLI adoption

**Current checkpoint (2026-10-19, first live evaluation independently scored):** M0/M1 froze24 fresh tasks from12 official documents/six products before Mistral/Google execution. Both lanes stopped early at the CLI logical6-call limit with missing failure usage. The normal binder also rejects valid two-stage finalization. All full24 provider/track actual F1 values remain`null`; **adoption NO_GO**. Only one Google helper question per arm is eligible: baseline/candidate case F1=0.00 and claim F1=1.00, with actual conflict terminals and no paired gain. This is not overall performance. [First results, independent review and repair order](../evaluation/engine073-realworld-first-v1a-results/README.md). Repair #540 collection/binding/failure diagnostics and verify actual control paths before a separately frozen future evaluation. Groq stays deferred before credentials; original raw/gold/budgets/first failures are immutable, with no model retry or rescore. Published CLI0.5.5/Engine0.6.1 remains; Engine0.7.3 is unreleased. Dated preparation entries below are historical.

[日本語](engine-07-realworld-evaluation.ja.md) | English

Status: **in progress / adoption NO-GO**, 2026-10-15 JST. [Issue #531](https://github.com/git-ksk/reasoning-harness/issues/531), [milestone #12](https://github.com/git-ksk/reasoning-harness/milestone/12). The published product is Reason CLI0.5.5 / Engine0.6.1. Engine0.7.2 is an independent source release, not an adopted CLI Engine.

## Planned successor — Engine0.7.3

Follow [the0.7.3 ordered roadmap](engine-0.7.3-roadmap.md) / [#538](https://github.com/git-ksk/reasoning-harness/issues/538).0.6.1 remains the product baseline;0.7.2 results remain immutable historical development evidence. Repair #520 first, complete safe runner/wiring, then freeze and freshly evaluate the pinned successor. #535 is merged and the #537 bridge passes26 offline tests; these do not complete live runner or acceptance gates.0.7.3 is not published or selected for CLI adoption.

## First development comparisons completed; acceptance remains incomplete

[First v2/v3 results and limits](engine07-realworld-development-results.md) record both three-provider observations, exact source/build identities, preserved raw archives and verification. Both frozen runs FAIL; normal useful-answer gain was0 for Mistral/Google in v3 and Groq was quota-blocked. Opt-in helper display gain was2 for Mistral/Google, separately. No full safety, citation, fresh independent or adoption acceptance is claimed.

[PR #534](https://github.com/git-ksk/reasoning-harness/pull/534) contains the frozen evaluators and [pre-execution design review](../evaluation/engine07-realworld-v2-review.md). The practical corpus is agent/host-curated from8 official documents/4 publishers,24 reused scenarios/17 normal questions; automatic extraction/open-Web quality and actual correction/conflict/unknown-lineage coverage remain untested. [#535](https://github.com/git-ksk/reasoning-harness/issues/535) fixes failure diagnostic loss in new source. [Next-run operational controls](../evaluation/engine07-realworld-operations-v1/README.md) are offline-tested; they must be wired into a new runner. Known daily-quota exhaustion blocks additional Groq starts. #520 still requires a pinned successor and fresh acceptance. Adoption checklist1/6: development comparison and NO-GO record only.

## Ordered work

1. Measure immutable Engine0.6.1 (`4abee90f0501f0d8fc09b453a0b45c942fbed69e`) and0.7.2 (`296690a86e049b75ee693ae5032a71504366137c`) on identical real-source tasks. Separate development calibration, fresh independent acceptance, and model/provider failures.
2. Fix only measured defects. Preserve original first observations; a fix uses a newly pinned successor, followed by new acceptance. #488 lineage, #489 trusted supersession and #491 answerability/reacquisition require a supported residual and measured benefit. Sufficient existing behavior means record defer/no-op.
3. Select the accepted Engine explicitly; use a dedicated CLI integration PR and Mac/Windows/Linux, installation, distribution, upgrade and rollback gates. #465 infrastructure reliability and #506 WinGet maintenance acceptance remain separate.

## Two required tracks

`e2e_normal` measures the ordinary question → acquisition → Engine decisions → final user-visible answer, using identical CLI/provider host code with each separately built Engine. Capture actual invocation paths, acquired source IDs/versions, policy/admission decisions, every exposed claim and citation, operational failures, calls, attempts, tokens and latency. Building entire historical CLIs with different host code would confound an Engine-only comparison and must be labeled separately.

`helper_opt_in` measures explicitly enabled source attribution/reconciliation on controlled real primary-source snapshots. Actual0.6.1 finalization is the baseline; the candidate additionally exercises its opt-in host-review surface. Record semantic advice separately from review authority. Precommitted evaluator reference labels can authorize only a **controlled test-host review**, not external truth, a real human signature or model self-authorization. Measure attributed target presentation gains separately from complete useful answers.

Do not sum the denominators or call helper success product E2E. Static search at preparation commit `27737ead2b0dfeddcdcebff608e8957bd263c4e0` found no calls to the new relevance/reconciliation entrypoints in ordinary CLI/provider/runtime paths (the sole provider source-attribution occurrence initializes an empty artifact). This is a wiring concern requiring a runtime trace, not proof that every Engine change is inactive or an already completed CLI acceptance.

## Canonical protocol preparation

The [machine-readable protocol](../evaluation/engine-07-realworld-protocol-v1.json) proposes sample sizes, hard gates, useful-answer thresholds and task budgets to be precommitted at complete evaluator freeze. Its status is **draft pending complete evaluator/host freeze**. It remains the broader acceptance draft; both separate development evaluators above are frozen and their first failed observations are preserved. No fresh independent acceptance cases have been produced. The isolated pilot below remains historical preparation evidence.

Before independent authoring: commit complete evaluators, host/adapters, exact engine identities, model IDs/settings, source snapshot schema, paired ordering, provider pacing, timeouts, numeric thresholds and scoring rubric. Freeze SHA-256 manifests. Author 24 fresh disjoint tasks after freeze, separately from 24 development tasks; include at least eight supported positive tasks plus conflicts, revisions, same-origin duplication/missing lineage, missing information, wrong target/scope and control contamination. An independent reviewer must review full sources and expected proposition/scope/time labels before first observation. Models never receive expected labels. Preserve partial/failed first observations; operational repairs use new identities and never silently overwrite/rerun semantic failures.

Only perform expensive canonical runs once required stages and known source-safety controls are ready. A known uncorrected defect independently blocks adoption, even if the sampled tasks miss it.

## Candidate quality thresholds (new precommit policy; not acceptance)

For each required provider and for **each separate track**, the future independently source-reviewed candidate must meet all three positive-class floors **simultaneously**: **F1 >= 0.90, precision >= 0.95, recall >= 0.85**. These are project-specific operational acceptance targets, not a universal benchmark. F0.5 >= 0.95 is a diagnostic stretch goal, **not** an additional hard gate. Precision 0.95 and recall 0.85 yield F1 about 0.8972; the three minimums are not redundant. See the machine-readable `gates.candidate_quality` and [metric contract/tests](../evaluation/engine07-quality-gates-v1/README.md).

Only fully observed, independently labeled clear positive/negative cases enter the binary confusion matrix. Correctly withheld *answerable* cases count as FN, and unsafe accepts on ambiguous cases remain separate mandatory safety failures. Missing/timeout/quota cases are operationally incomplete, not TN; undefined precision or zero positive decisions cannot pass. Record exact TP/FP/FN/TN, ambiguous, coverage, F1, F0.5, per-class denominators and uncertainty. The small proposed 24-case sample provides a bounded gate result, **not** a population-level guarantee. Neither model advice nor a separately enabled helper result can supply normal CLI answer quality.

**All older hard gates are cumulative and unchanged:** zero observed critical safety/citation/authority violations, zero positive-control regressions, known defects closed, paired improvement over0.6.1, all required providers, runnable secure host and live budgets, complete independent full-source review, CLI/distribution acceptance. Freeze the final evaluator and these thresholds **before** authoring/executing the next new independent acceptance cohort; previous recorded first observations remain immutable. The small offline metric scorer is an integration building block, not itself a release gate or a completed live evaluator.

## #520 source qualification gate revision (2026-10-17)

The separate **synthetic** #520 source-qualification acceptance no longer requires an exact 60/60 first-pass score *for future independently frozen cohorts*. The [new explicit policy and offline tests](../evaluation/engine073-source-gate-v1/README.md) require both public API axes to achieve **F1 >=0.90, precision >=0.95, recall >=0.85**, with **zero false acceptance of negative/safety cases**, 100% of known safety regression controls and historical API compatibility, and complete immutable first-observation evidence. Legitimate positive abstention is measured as FN rather than a release-blocking error by itself. Past independent results (including v24 47/60) remain unmodified **FAIL under their original frozen gate**; no retrospective reclassification. The 60-case minimum and the future independent evaluator/source pre-freeze still apply.

This revision changes **only #520's future synthetic source gate**. #531's per-provider and per-track real-answer case-quality F1/precision/recall, factual/citation safety, baseline utility improvement, execution budget/host completeness and independent acceptance remain separate mandatory gates, as does #540. It neither authorizes Engine release nor CLI adoption.

## #520 source qualification gate revision (2026-10-17)

The separate **synthetic** #520 source-qualification acceptance no longer requires an exact 60/60 first-pass score *for future independently frozen cohorts*. The [new explicit policy and offline tests](../evaluation/engine073-source-gate-v1/README.md) require both public API axes to achieve **F1 >=0.90, precision >=0.95, recall >=0.85**, with **zero false acceptance of negative/safety cases**, 100% of known safety regression controls and historical API compatibility, and complete immutable first-observation evidence. Legitimate positive abstention is measured as FN rather than a release-blocking error by itself. Past independent results (including v24 47/60) remain unmodified **FAIL under their original frozen gate**; no retrospective reclassification. The 60-case minimum and the future independent evaluator/source pre-freeze still apply.

This revision changes **only #520's future synthetic source gate**. #531's per-provider and per-track real-answer case-quality F1/precision/recall, factual/citation safety, baseline utility improvement, execution budget/host completeness and independent acceptance remain separate mandatory gates, as does #540. It neither authorizes Engine release nor CLI adoption.

## Preparation evidence already collected

- [Published0.7.2 audit](../evaluation/engine-0.7.2-verification-audit-20261014/README.md): a root-authored synthetic14-input targeted matrix finds ten false-Relevant negatives for present-tense retractions/denials and non-event statements. #520 is reopened under milestone#12. Its original repair, acceptance and published tags remain preserved. No downstream wrong answer or Known/Supported promotion was demonstrated.
- [Actual-tag paired retraction control](../evaluation/engine072-retraction-paired-v1/README.md) replays those same fourteen synthetic audit inputs against both actual released builds.0.6.1 has0/13 false-Relevant negatives;0.7.2 has10/13, while positive recall rises0/1→1/1. This bounded source-qualification regression independently blocks adoption; it is seen development evidence, not a fresh holdout or wrong-final-answer claim.
- [Real-source pilot](../evaluation/engine-07-realworld-pilot-v1/README.md): actual locked/offline builds of both released tags, one identical public-API probe, five official short excerpt snapshots and eleven development inputs. Four of five author-labeled positive relevance cases were Ambiguous in both arms; observed gain0 and positive regression0. All six negative-labeled rows were also Ambiguous, but their parent/precondition limitations prevent safety acceptance. This is a **small, deliberately bounded relevance probe**, not model accuracy, complete answer usefulness, provider acceptance or a representative error rate. The [independent design review](../evaluation/engine-07-realworld-pilot-v1/design-review.md) found context/label, inactive negative-control and partial-journal limitations. Full-source/context and contract review is required before calling these four mismatches product bugs.
- Independent design review required separate tracks, isolated source identities, final-prose/citation scoring, hard gates per provider, and fresh authoring after evaluator freeze. No new independent acceptance set has been consumed.

The pilot uses optimistic fixed **untrusted** proposal/raw qualification to stress deterministic floors. It performs zero model/API calls; token/cost metrics are unavailable. It neither measures the new cross-source reconciliation surface nor authenticates an actual reviewer. Quote SHA-256 is explicitly the short UTF-8 excerpt hash, not the full live page or a publisher signature. Same-publisher documents do not become independent corroboration.

## Adoption checklist — 1/6

- [ ] Required real-data/provider evaluation complete.
- [x] Paired development baseline/candidate results, evidence limits and NO-GO decision recorded.
- [ ] Necessary repairs and regression/fresh reacceptance complete, including #520.
- [ ] Exact accepted Engine selected.
- [ ] Dedicated CLI integration PR, CI and applicable local/device E2E accepted.
- [ ] Formal CLI distribution, installation and update verification complete.

Source release CI and prior fictional-source12-case gains (+3/+6/+6) remain historical bounded evidence. Neither establishes these six adoption criteria. No automatic0.8.0 feature expansion or0.7.2 CLI adoption is authorized by a favorable helper score alone.
