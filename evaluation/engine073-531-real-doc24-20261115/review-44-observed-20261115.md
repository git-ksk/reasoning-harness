# Issue #531 — 44 observed real-document CLI runs: development-only review

**Date:** 2026-11-15 JST. **Original tagged workflow:** [`43667947455`](https://github.com/git-ksk/reasoning-harness/actions/runs/43667947455) (`eval-531-real-doc24-live-20261115-one-shot`, source `74749b6b02e94c7fc2be5d68fd3aa44e3830b0e8`). **Decision: HOLD 0.7.3 adoption/release.**

## Execution / preservation status

- GitHub Actions **cancelled** the first and only tag execution at 2026-11-15 17:21 JST: **job exceeded 5h50m maximum runtime**, not a bad billing gate. Do not restart/rerun this tag or overwrite any artifact.
- **Shards 01–03 SUCCESS**; twelve paired CLI outputs in each, six question IDs per shard. Shard 04 was cancelled after eight complete CLI outputs for `rw-19` through `rw-22` in both arms. The workflow *did* encrypt and upload the fourth partial raw archive during cancellation; it did **not** write a normal sanitized summary in that shard. There are four original missing/unknown outputs for `rw-23` and `rw-24` (both arms). An attempt for `rw-23` may have been in flight when cancelled: **unknown provider charge and partial effects must not be considered zero**.
- All four original encrypted artifact files were downloaded and locally decrypted using the existing reviewer private key. Source copies, raw CLI stdout and independent development gold remain **private on the user's Mac**. Raw answers and label gold are not included in Git, Issues, or CI annotations. The private full review packet SHA256 is `07f7b3a387952208894723b7fb8b5c3e5f3e9050ab344c73dd31c948223a3466`.
- **44/48 completed raw CLI outputs** are parseable, with known usage **328,453 total tokens**, **325 model calls**, **328 provider attempts**. This is a **known measured subtotal**, not a billing total; the interrupted attempt has unknown usage. Four shard-bound Mistral read-only checks of API pay-as-you-go OFF and >=$1 included budget succeeded. No paid API retry or new model request was made during review.

## Scoring definition and limitation

Only the text in the *actual finalization/display* counts as an answer. Internal model candidates, selected proposals, investigation questions, source quotations standing alone, or unexposed claims **do not satisfy a gold requirement**. The old, already-consumed `e2e_normal` 24-case developer gold was used **privately** to inspect all required factual groups, scope qualifiers, and safe-refusal requirements. The unit is an entire question answered fully and correctly; a missing half of a two-part answer is **partial, not a true positive**. An ungrounded assertion on a pre-labeled negative is an FP; correct refusal is a TN. This is a **post-hoc model-assisted manual reviewer**, not new independent blind scoring or a formal accepted first-score protocol. With two cases missing, the full-population accuracy claim is **invalid**.

On the **22 complete paired questions only**: **17 positive/answerable or partially answerable, 5 negative/unanswerable.**

| Development diagnostic (same 22 paired cases) | 0.6.1 baseline | Corrected 0.7.3 candidate |
| :-- | --: | --: |
| Full answer TP | **10** | **8** |
| Unwarranted asserted answer FP | **0** | **0** |
| Positive incomplete/withheld FN | **7** | **9** |
| Correct safe refusal TN | **5** | **5** |
| Precision, case-level | 1.0000 | 1.0000 |
| Recall, case-level | **0.5882** | 0.4706 |
| **F1, case-level (development-only)** | **0.7407** | **0.6400** |
| Supported but **partial** answer | 5 | 6 |
| Unnecessary refusal on positives | 2 | 3 |
| Model calls in complete outputs | 161 | 164 |
| Provider attempts in complete outputs | 163 | 165 |
| Observed input tokens | 132,432 | 130,727 |
| Observed output tokens | 33,289 | 32,005 |
| Observed total tokens | **165,721** | **162,732** |
| Verified displayed citation occurrences | 24 | 21 |

Candidate spent **2,989 fewer observed tokens (-1.80%)**, but required **3 more model calls and 2 more provider attempts**, and delivered **2 fewer fully answered cases** (F1 −0.1007). On the small paired sample, these results are indicative of regression, **not statistical proof of a universal model-quality decline**.

### Important pairwise differences

| Case | Baseline | Candidate | Developer review |
| :-- | :-- | :-- | :-- |
| `rw-02` curl transient errors vs HTTP404/`--fail` | Full | Abstained | Regression; candidate had unexposed proposals, but finalization covered 0 claims (`no_progress`) |
| `rw-08` Git `check-ignore` exit codes 0/1/128 | Abstained | Full | Improvement; candidate grounded all statuses to the two pinned source records |
| `rw-19` SQLite Tcl extension installation | Full | Abstained | Regression; candidate finalization covered 0 claims (`round_budget`) |
| `rw-21` Redis Linux allocator and `MALLOC=libc` | Full | Partial | Regression; candidate grounded `jemalloc` but did not **assert** the build setting, although the source quote mentioned it |

Other missed answers were mainly **under-answering**: `rw-06` (`--no-index` diagnostic not asserted), `rw-10` (`return()` from included files omitted), `rw-11` (CMake policy caveat withheld), `rw-13` (`SSL_VERIFY_PEER` omitted), `rw-16` (no explicit conclusion that handshake continuation is not authentication proof) and `rw-17` (Tcl9 Unix recipe contrast omitted). All five measured negative controls were correctly withheld by both. A source quote or internal model proposal is not an externally asserted answer and must never be counted as one.

**Citation audit:** The **45** citations actually displayed in 44 finalization texts were revalidated against exact original per-case frozen evidence IDs, source URLs/versions, source quotation text, SHA256 quotation digests, registered evidence SHA256/proposition links, and JSON-escaped quotation rendering. **45/45 PASS**. The pinned full upstream documents had separately passed 12/12 raw SHA256 and 48/48 whitespace-normalized quote existence. This is evidence-integrity validation, **not verification that every possible statement in the hidden model candidate was true**.

## Why not immediately run a 4-observation gap fill?

`rw-23` and `rw-24` are **both pre-labeled negative/unanswerable controls**. They cannot add a true positive to the 17 already-measured positives; at best, a fresh run would preserve the observed case-level F1 values (0.7407 baseline, 0.6400 candidate), and at worst a false positive would reduce them. The original `rw-23` invocation's external effects are ambiguous because the job was cancelled mid-step, so even a new run would be a separate development observation, *not recovery of the original first result*. The current candidate has 3 observed regressions versus 1 improvement and fails the intended quality gate; **the 4 gap calls are not justified before investigating the candidate's under-answering.**

A future **separate, one-shot, explicitly reviewed** gap evaluation can be considered after the candidate is fixed. It would require a fresh immutable tag, small bounded job timeout, current same-account billing gate, and marking old incomplete `rw-23` as unknown/possibly billed; it must **not** rerun the cancelled original tag or merge new observations into a purported original first-observation result.

**Next engineering action:** Investigate `rw-02`, `rw-19` and `rw-21` fact-key/source-binding / target coverage differences; keep positive controls `rw-08` and the zero-unsafe-assertion evidence. Then use new independent holdout material for adoption gates. No Core merge, CLI adoption, or Engine 0.7.3 release from this review.
