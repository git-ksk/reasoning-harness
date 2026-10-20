# Engine 0.7.3 successor: first real-document evaluation — NO_GO

2026-10-19 JST. M0 software/runtime and M1 fresh24/source/gold were frozen before the first model run. This compares baseline Engine0.6.1 with unreleased repair source `ab3c632` using the same CLI0.5.5 host. The candidate package still says0.7.2; these are not measurements of the published Engine0.7.2 tag.

**All six provider/track full-cohort `actual_answer_f1` values are null.** The current acceptance is **NO_GO**. The 24-question evaluation did not complete. Arithmetic zeros from empty case inventories are not measured whole-cohort F1. No source merge, Engine release or CLI adoption follows from this result.

## What actually ran

[First live run39250362193](https://github.com/git-ksk/reasoning-harness/actions/runs/39250362193), head `9558bad07e73ab14eeefb4f1a81102dc611ee221`. All original raw/log bytes are preserved and hash-bound. The earlier [v1 permission failure](../engine073-realworld-first-v1-operational-fail/manifest.json) made no model calls and remains unchanged. The independently reviewed v1a operational successor used the identical M0/runtime/M1/models/order/seed/budgets, with an external receipt for ephemeral host ownership/permission handling.

| Provider | Actual live jobs | First frozen capture | Full24 F1 |
| --- | --- | --- | --- |
| Mistral | First baseline failed at CLI logical-call limit6; remaining71 jobs suppressed | 0 complete / 1 incomplete / 95 not_run | null |
| Google | First question baseline/candidate/helper completed; next candidate failed at limit6, remaining68 jobs suppressed | 2 helper complete / 3 incomplete / 91 not_run | null |
| Groq | Deferred before credentials; full-cohort capacity not admitted | 96 planned observations missing | null |

There are72 scheduled jobs per provider but96 observation rows: each shared helper advice job is replayed through both actual Engine hosts. Across all three providers,288 observations remain in the scoring inventory. Of192 captured observations,2 are complete,4 incomplete and186 not_run. Missing or incomplete results never become successful negatives.

Mistral's verified journals record9 HTTP attempts/8956 tokens; Google's record17/11606. These are separate recovered audit totals. The original strict guard summaries remain respectively0/0 and11/7822 known attempts/tokens because failure envelopes omitted complete usage. Zero known usage is not zero actual cost. Logical operations and adapter HTTP attempts are different units. The budget failure is neither a provider quota failure nor proof of an Engine accuracy defect.

## First independent semantic score

The unchanged frozen collector was first invoked locally with original raw and M1 read-only because the CI host collector guard had silently exited before collection. No model or Engine execution occurred. The independently fixed192 annotations were then passed to the unchanged frozen scorer **once**. The scorer reports two `semantic_review_incomplete` failures for normal displays whose trace binding is unresolved; these failures are preserved, with no annotation repair or rescore.

Only **one Google helper question per arm** is binding-complete and semantically reviewed. This is a descriptive positive-only subset, not representative acceptance or a24-question F1.

| Unit, one completed question per arm | Baseline0.6.1 | Candidate repair source |
| --- | --- | --- |
| Case TP / FP / FN / TN | 0 / 0 / 1 / 0 | 0 / 0 / 1 / 0 |
| Case precision / recall / F1 | undefined / 0.00 / 0.00 | undefined / 0.00 / 0.00 |
| Claim precision / recall / F1 | 1.00 / 1.00 / 1.00 | 1.00 / 1.00 / 1.00 |
| Grounded unique claims / displayed unique claims | 2 / 2 | 2 / 2 |
| Satisfied required groups / required groups | 1 / 1 | 1 / 1 |
| Correct citation edges / total edges | 3 / 3 | 3 / 3 |

Both displays quote supported content, but both retain the actual `conflict` terminal status on a compatible-positive question. Grounded wording does not make that final case decision eligible. There is no paired utility gain. Claim metrics, case metrics and historical synthetic API metrics have different denominators. No broad safety claim follows from zero observed violations in this tiny subset. The review used an independent agent; operational work had exposed provider/arm associations, so it was not fully blinded or human adjudication.

## Reproduced blockers and next work

1. **Fixed logical-call profile fails early.** Both lanes exhaust the declared CLI logical6 budget. Preserve this efficiency/operability finding; do not raise the budget after seeing results merely to obtain a pass.
2. **CLI failure diagnostics omit usage.** Verified journals contain completed HTTP usage, but the CLI failure envelope has no usage or partial diagnostics. The strict unknown-usage latch correctly stops later calls. Repair diagnostic preservation in a new host identity while retaining the original failure and incomplete accounting.
3. **CI collector guard lacks directory access.** Root-created mode0700 lane directories cannot be traversed by the host runner. `test -f lane/plan.json || exit 0` silently skipped collection; the later ownership transfer allowed raw upload but came too late for collection. Move validated access handling before collection and make inaccessible evidence an explicit failure in a future workflow.
4. **Frozen normal binder rejects legitimate two-stage finalization.** The normal traces contain `renderer_candidate` and `answer_safety` finalization events at sequence17/19. Requiring exactly one rejects both successful raw normal displays. Preserve both events and validate their ordered binding chain in a new evaluator; do not drop an event or retroactively credit the old capture.

#540 tracks the runner/evaluator repairs; #531 tracks actual-answer evaluation; #538 tracks ordering toward adoption. First validate the real normal/helper control paths, then separately freeze the next evaluation identity and admission plan. These now-consumed24 questions can support diagnostics, not a new untouched acceptance claim. Groq remains deferred. All old first results, gold, budgets and published tags remain unchanged.

## Evidence layout and limits

- [Exact first score](first-score/report.json), [288 scored/inventory rows](first-score/rows.json), [summary](status.json).
- [Independent postmortem](independent-postmortem/postmortem.json) and [semantic annotation notes](independent-annotations/semantic-review-notes.json).
- Original private raw/logs365 files, capture packets/observations and192 first annotations remain in the private evidence root recorded by `status.json`; their complete manifests are included. Full acquired source text and credential-bearing runtime material are not published here.
- The annotation/postmortem manifests each describe the full private closure; the selected public notes in this directory are not that entire closure. `artifact-manifest.json` below binds only this public archive.
- Source scope is24 host-authored tasks from12 fixed official documents/six software products, with host-curated facts; this does not measure unrestricted Web discovery, automatic full-text extraction or population accuracy.

All original M0/M1/candidate/runtime/scorer bytes are unchanged. One first scoring, no retries, no relabel, no retrospective acceptance.
