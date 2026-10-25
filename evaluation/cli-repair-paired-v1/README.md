# CLI repair paired evaluation preparation

This is an **offline preparation checkpoint**, not a completed evaluation or a frozen execution runtime. PR552 is merged at `2c4fea29ac9ae7c0447766cd0d83200ae01ea9f1`. The common host includes the source-registry fix in PR555 at `73bbd18173b4dfcd6664a26241de0385f2eb2e3e`; PR555 passed its bounded independent review and was merged at `3ceed8ac122c8be51ddc2a97f8d1f689827dcd02`. The historical common-host pin remains fixed.

The next measurement uses the consumed 24-document-task development set. It can measure whether the repaired CLI improves those previously observed tasks. It cannot establish fresh generalization or authorize Engine adoption. The original scores, observations and published tags stay immutable; fresh independent acceptance follows separately.

## Comparison identity

| Component | Pin / scope |
| --- | --- |
| Common CLI | `73bbd18173b4dfcd6664a26241de0385f2eb2e3e` (unreleased, package 0.5.4) |
| Baseline core | `4abee90f0501f0d8fc09b453a0b45c942fbed69e` (0.6.1) |
| Candidate core | `ab3c6322921c8c738ec35bd018345917b5afa538` (unpublished 0.7.3 repair; package 0.7.2) |
| Common core change | PR552's opt-in pending-investigation method, identical patch on both core trees |
| Provider instrumentation | Identical audit, one HTTP attempt per generation, bounds and pacing on both arms |
| Upstream evaluator | `6165de77c1544a7fe58fc1e675e7cb3a8ec7cc98`, exact file inventory in `upstream-files.json` |

These are patched arms under one CLI, not untouched published binaries. Other core files must match their pinned trees. Numeric F1/P/R, citation, utility, safety and efficiency gates remain unchanged. Ordinary CLI and opt-in helper results remain separate.

## Reproduce offline preparation

Run from the repository root with the pinned Git objects available locally:

```sh
python3 evaluation/cli-repair-paired-v1/prepare.py --repo . --output target/cli-paired-preparation
python3 -m unittest discover -s target/cli-paired-preparation/repo/evaluation/cli-repair-paired-v1 -p 'test_*.py'
```

`prepare.py` imports only declared files and applies this overlay without changing historical sources. The destination must not exist. `preparation.json` records imported, rewritten and overlay file hashes; it is explicitly not an execution receipt.

The builder is at `target/cli-paired-preparation/repo/evaluation/cli-repair-paired-v1/build.py`. It produces five binaries per arm, including a registry generator linked to the actual core's Evidence serializer. The consolidated builder completed both Linux aarch64 arms and all ten binary hashes were checked. Source inventories show only the identical PR552 resolution patch in each core tree; provider patches match. The final runtime containment and independent review still precede any freeze. The older macOS checkpoint used separately compiled registry binaries.

The existing local raw controls can be rechecked without another model call:

```sh
python3 evaluation/cli-repair-paired-v1/verify_local.py \
  --runtime target/cli-paired-preparation/repo/evaluation/cli-repair-paired-v1 \
  --observations target/private-evaluation-pending/cli-repair-paired-preparation-v1 \
  --output target/cli-paired-local-verification.json
```

The observations are local ignored artifacts. No API keys, raw private corpus or old annotations are included here. The saved control is the existing inert engineering-value fixture, not a new accuracy case. The first patch-application failure and first capture-adapter failure remain in their original logs. Rechecking saved bytes is not rerunning a failed accuracy observation.

## Remaining gates

1. PR555 passed bounded independent review and is merged. The first PR556 review at `687fb04` found three defects: blocked-answer projection acceptance, keyless selector rejection, and an inherited preflight status assertion. Fixed source `552a6d724e1d7ebbfccade8d0448e529d033467f` passed bounded independent re-review with 13 controls; PR556 is merged. The review does not grant final runtime/API admission.
2. Both Linux aarch64 hosts and all ten binaries are built. The final local Linux runtime passed registry parity, 51 evaluator checks, 18 provider checks, 17 actual CLI paths and seven readiness-aware scheduler controls. See `linux-checkpoint.json`. The original inherited runner suite passed 19/21; two startup-sensitive controls signalled before fixture readiness. Those failures remain recorded; the separate readiness-aware verification passed without modifying the scheduler.
3. Complete the new development execution receipt and current provider readiness. Historical M0/runtime-reuse receipts cannot authorize this host. The inherited independent-M0 path still expects the old host; it must be separately migrated and reviewed before fresh acceptance.
4. Freeze input/config/registry generation, scoring arithmetic, reviewers and limits before any new paid observation. Review full output bytes and exact quote attachments; annotations must be bound to the new outputs.
5. Run the admitted consumed-set comparison once, record complete and incomplete rows, then decide on a fresh independent set. All three providers and both tracks remain required for the final adoption decision.

All providers are held in this preparation. The owner reported Mistral can resume on 2026-10-25. Its new execution readiness/receipt is still required. Groq's prior whole-cohort reservation did not fit the verified daily capacity. Google's new runtime/review/account readiness is incomplete. `run_lane` and receipt validation reject preparation mode, while `run_task` permits only inert numeric loopback with no real credentials. Enabling live execution requires an explicit implementation and review change, not toggling a JSON flag.

## GitHub runtime artifact

The manual `cli-repair-paired-runtime` workflow reconstructs both hosts on GitHub Linux, runs only inert controls with external networking disabled, and uploads the exact image plus identity and offline proof. It receives no model credentials. The proof asserts `API_execution_admitted: false`; it is not an execution receipt. This provides the artifact needed for the later secret-backed Mistral/Google execution path.

The additional workflow/proof review was interrupted by the independent reviewer's usage limit. The completed `552a6d7` source/capture PASS remains valid; no independent PASS is claimed for the final runtime, account readiness or paid execution. Groq remains held.
