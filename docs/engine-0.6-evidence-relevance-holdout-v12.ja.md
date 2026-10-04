# Engine 0.6 evidence relevance holdout v12

Status: runner-only preparation。successor-v11 semantics は `engine-0.6-evidence-relevance-successor-v11-semantics-freeze` / `d8d9459d9bd225a16442e0ce32de55aa7563a08f` で固定済み。holdout-v12 corpus はまだ存在せず、provider observation は禁止する。

## Runner binding

専用 binary: `reason-evidence-relevance-holdout-v12-study`。

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

Historical v1-v11 / reusable development profile は元の semantic binding を維持する。runner regression では immutable development-v1 omitted-ownership case に対し V11 が旧 ExactTarget state を維持し、V12 だけが frozen v17 の Unresolved identity floor を適用し、v30 が fail-closed Ambiguous を維持することを明示的に検証する。

## Ordering constraint

Runner freeze coordinate: `engine-0.6-evidence-relevance-holdout-v12-runner-freeze`。

exact-head CI PASS と annotated tag push 後にのみ `fixtures/evidence-relevance-holdout-v12` を作成できる。fresh corpus は runner freeze 後に独立 author し、observed holdout-v1-v11 / successor-development の case ID / canonical entity / task / exact signal / 8-token window を再利用しない。

後続 one-shot acceptance surface は Mistral + Google + Groq を required とする。runner preparation / corpus authoring 中に Groq を呼び出さない。
