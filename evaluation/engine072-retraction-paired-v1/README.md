# Published-tag paired retraction audit — seen-data control

#520 / #531. These are the **same fourteen root-authored synthetic audit cases already observed** in `../engine-0.7.2-verification-audit-20261014/`, not fresh independent acceptance, real-source evidence, downstream final-answer correctness or a population error rate.

The same immutable compiled probe used by real-source pilot v1 is linked separately to actual locked/offline Engine0.6.1 and0.7.2 tag builds. `preflight.json` records exact source/lock/rlib/binary/probe identities and the frozen expanded input hash before both invocations. `inputs.json` excludes expected labels. The same input and each probe binary were checked immediately before each arm; stdout/stderr were directed to create-new files and flushed/synced, preserving partial output before any exit check. No provider/model calls occurred.

## Measured paired result

| Metric | Engine0.6.1 | Engine0.7.2 |
|---|---:|---:|
| Factual positive Relevant | 0/1 | 1/1 |
| Negative false Relevant | 0/13 | 10/13 |

The new affirmative launch recall admits one positive that0.6.1 withheld, but it also admits ten explicit retraction/non-event negatives. Positive regression0; observed v29 objects equal across all fourteen inputs. Candidate v17 and v30 dispositions exactly reproduce the original0.7.2 audit. This establishes a bounded **source-qualification regression versus0.6.1**, not a wrong final answer or verified truth promotion. Adoption remains NO-GO independently of the incomplete real-world/provider/E2E evaluation.

`paired.json` retains per-case effective qualifications and dispositions. `observation-*.jsonl` and stderr are the first observations of this paired seen-data control; the earlier root0.7.2 observation stays untouched.

## Reproduction

Build the same probe against both tags using the documented pilot v1 runner with absent work/output paths. Then run each compiled `probe-0.6.1` / `probe-0.7.2` against this directory's `inputs.json`, saving to **new private files**. Compare the v17 objects and v30 disposition with saved outputs; any replay is seen-data development evidence. Never overwrite these files or present the replay as a fresh holdout.
