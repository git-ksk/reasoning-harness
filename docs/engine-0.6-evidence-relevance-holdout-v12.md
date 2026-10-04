# Engine 0.6 evidence relevance holdout v12

Status: fresh independent acceptance corpus prepared and still unobserved. Successor-v11 semantics are frozen at `engine-0.6-evidence-relevance-successor-v11-semantics-freeze` / `d8d9459d9bd225a16442e0ce32de55aa7563a08f`; the dedicated runner is frozen at `engine-0.6-evidence-relevance-holdout-v12-runner-freeze` / `920e3d91d9831c622890a9e183c40815323bf6e2`. No holdout-v12 provider observation has occurred.

## Runner binding

Dedicated binary: `reason-evidence-relevance-holdout-v12-study`.

V12 profile:
- configuration: `evidence-relevance-live-holdout-v12`
- suite: `evidence-relevance-holdout-v12`
- annotation protocol: `evidence-relevance-effective-qualification-v17-materialization-v30`
- fixed core: `evidence-relevance-fixed-core-v12`
- expected directory: `fixtures/evidence-relevance-holdout-v12`
- expected cases: 26
- issue: #468
- effective qualification: v17
- materialization: v30

Historical v1-v11 and reusable development profiles retain their original semantic bindings. A runner regression explicitly verifies that V11 preserves the old ExactTarget state for the immutable development-v1 omitted-ownership case while V12 applies the frozen v17 Unresolved identity floor and v30 remains fail-closed Ambiguous.

## Ordering constraint

Runner freeze coordinate: `engine-0.6-evidence-relevance-holdout-v12-runner-freeze`.

Only after exact-head CI passes and that annotated tag is pushed may `fixtures/evidence-relevance-holdout-v12` be created. The fresh corpus must be independently authored after runner freeze, with no reuse of observed holdout-v1-v11 or successor-development case IDs, canonical entities, tasks, exact signals, or 8-token windows.

The later one-shot acceptance surface must require Mistral, Google, and Groq. Groq is not to be invoked during runner preparation or corpus authoring.

## Fresh corpus and acceptance gate

The 26-case corpus was independently authored only after runner freeze. Distribution is Relevant 8 / Irrelevant 9 / Ambiguous 9. Authority modes are require_different 7 / require_requested 9 / forbid_different 3 / preserve_risk 6 / preserve_absence 1. Freshness excludes case ID, canonical entity, task, exact signal, and 8-token signal-window reuse against observed holdout-v1-v11 and successor development surfaces.

The risk set includes four fresh omitted/clipped ownership cases that must trigger the v17 ExactTarget -> Unresolved floor, two paired relation-only truncation controls that must retain ExactTarget, two fresh generic target-owned/no-classifiable-relation controls covering the prior Groq failure class, a wrong-target requested-relation control, and an adversarial relation-label instruction control.

Canonical acceptance requires Mistral `ministral-8b-latest`, Google `gemini-3.5-flash-lite`, and Groq `openai/gpt-oss-120b`, all 26/26 operationally complete with exact Harness-owned identity/risk and authority contracts, v30 materialization 26/26, wrong-target Relevant 0, false relevance rejection 0, Relevant-left-Ambiguous 0, and utility miss 0. Any required-arm miss is immutable FAIL; no rerun, rescore, relabel, or tag movement.

Groq admission is re-anchored from canonical holdout-v11 actuals: 63,218 observed tokens and completion at `2026-10-04T08:36:18Z`. With the same 55K configured start headroom and pacing model, the conservative next 55K-headroom floor is `2026-10-04T14:02:09Z`, already before this freeze preparation. No Groq request is made before the corpus freeze tag triggers the one-shot workflow.

Freeze coordinate: `engine-0.6-evidence-relevance-holdout-v12-freeze`. Exact-head CI, checksum, validate-only, freshness, fmt, Clippy, offline v17/v30 tests, and workflow parse must pass before that annotated tag is pushed.
