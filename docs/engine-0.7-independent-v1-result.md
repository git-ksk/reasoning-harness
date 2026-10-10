# Harness Engine 0.7.0 — Independent first-observation v1

**Frozen synthetic source-reconciliation gate PASS across Mistral, Google and Groq. This is not Engine 0.7.0's final release acceptance.**

- [GitHub Actions #38013006323](https://github.com/git-ksk/reasoning-harness/actions/runs/38013006323): all five jobs successful, covering preflight, each model and frozen final score.
- Frozen candidate commit `4cae9326bcc3e210c3250f05772d84f00e41b645` compared against unchanged Engine 0.6.1 original source finalization.
- Runner tag `engine-0.7-independent-v1-runner-freeze` at `1124e5a7c34e42c088d81908c3cdc75aae2db309`.
- Corpus tag `engine-0.7-independent-v1-corpus-freeze` at `78f99559bf044013e66025bde1a612895f2d41e6`, corpus SHA-256 `d3ce38ee2ef06310afb6a247d3762fc408b93cb49bd602a52d8ce1b8bcf7fa0b`.
- Twelve never-before-evaluated fictional-source cases, fifteen target relations: six compatible, eight negative/context-different and one identical.

| Metric | Mistral | Google | Groq |
| --- | ---: | ---: | ---: |
| Additional attributed compatible targets | **3** | **6** | **6** |
| Compatible source wording missed | 3 | 0 | 0 |
| Incorrect model equivalent advice | 0 | 1 | 2 |
| Incorrect compatibility actually authorized | **0** | **0** | **0** |
| Model calls / provider attempts | 14 / 14 | 14 / 14 | 14 / 14 |
| Input / output tokens | 3,922 / 84 | 3,989 / 70 | 4,653 / 786 |
| Operational failures | **0** | **0** | **0** |
| Hard gate violations | **0** | **0** | **0** |

Google and Groq's mistaken affirmative advice occurred when the underlying source scope or version differed. The source-authority boundary rejected those proposals and preserved the legacy original `Conflict` and source citations. Mistral withheld three of the six compatible targets. No model's advice alone created reviewer authority or Known/Supported external truth.

**Scope limitation:** All inputs are fictional quotations. These observations measure controlled source-qualified compatibility, **not externally verified truth, real-world answer utility, or published CLI adoption**. The new target-local output is opt-in development functionality. The Reason CLI 0.5.4 release remains immutable.

## Archived first observation

Complete first provider observations, per-case synced checkpoints, dry preflights and exact official final cross-provider scorecard have been copied from that GitHub run to `evaluation/engine-0.7-independent-v1-first-observation/`. The `archive_manifest.json` records the exact source run, commit identities and SHA-256 of each stored file. Re-executing the **already frozen** original score script reproduced the archived final scorecard byte-for-byte.

The legacy serialized Thread, Artifact, source-attribution contract, CLI main and managed-session source are unchanged from `engine-v0.6.1`. Local Thread replay 11/11, CLI resume/fork 1/1, source reconciliation 5/5 and target-local development 1/1 PASS. PR #500's no-Rust fresh-install consumers across four OS/arch distributions all passed.

## Remaining source release scope

Proposed #488 source-lineage, #489 temporal supersession and #491 new answerability/acquisition semantics were **not adopted**: the frozen 0.6.1 baseline did not demonstrate a concrete unsafe existing transition for these extensions; conservative legacy boundaries stay active. Do not imply these additions ship in Engine 0.7.0.

The opt-in source presentation #490 demonstrates measured controlled synthetic gain, while complete Engine-only versioning and source-release checks continue under [#492](https://github.com/git-ksk/reasoning-harness/issues/492).