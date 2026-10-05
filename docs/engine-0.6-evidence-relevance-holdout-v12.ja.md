# Engine 0.6 evidence relevance holdout v12

Status: fresh independent acceptance corpus prepared / still unobserved。successor-v11 semantics は `engine-0.6-evidence-relevance-successor-v11-semantics-freeze` / `d8d9459d9bd225a16442e0ce32de55aa7563a08f`、専用runnerは `engine-0.6-evidence-relevance-holdout-v12-runner-freeze` / `920e3d91d9831c622890a9e183c40815323bf6e2` で固定済み。holdout-v12 provider observation はまだ0。

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

## Fresh corpus / acceptance gate

26-case corpus は runner freeze 後にのみ独立authorした。分布は Relevant 8 / Irrelevant 9 / Ambiguous 9、authority mode は require_different 7 / require_requested 9 / forbid_different 3 / preserve_risk 6 / preserve_absence 1。observed holdout-v1-v11 / successor development に対する case ID / canonical entity / task / exact signal / 8-token signal-window reuse を禁止する。

リスク面では fresh omitted/clipped ownership 4件で v17 ExactTarget -> Unresolved floor、paired relation-only truncation 2件で ExactTarget 維持を要求する。さらに前回Groq failure classの generic target-owned / no-classifiable-relation control 2件、wrong-target requested-relation control、adversarial relation-label instruction control を含む。

Canonical acceptance は Mistral `ministral-8b-latest` + Google `gemini-3.5-flash-lite` + Groq `openai/gpt-oss-120b` を全requiredとし、各26/26 operational完走、Harness-owned identity/risk / authority contract exact、v30 materialization 26/26、wrong-target Relevant 0、false relevance rejection 0、Relevant-left-Ambiguous 0、utility miss 0を要求する。required armが1件でもmissすれば immutable FAIL。rerun / rescore / relabel / tag movementは禁止。

Groq admission は canonical holdout-v11 実測 63,218 tokens / completion `2026-10-04T08:36:18Z` から再anchor。同じ55K configured start headroom / pacing modelで次の保守的55K-headroom floorは `2026-10-04T14:02:09Z`。freeze準備時点ですでに経過済み。corpus freeze tagがone-shot workflowをtriggerするまでGroq requestは行わない。

Freeze coordinate: `engine-0.6-evidence-relevance-holdout-v12-freeze`。annotated tag push前に exact-head CI / checksum / validate-only / freshness / fmt / Clippy / offline v17/v30 tests / workflow parse を全てPASSさせる。
