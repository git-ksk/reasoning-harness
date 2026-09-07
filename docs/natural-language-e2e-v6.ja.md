# Natural-language E2E v6 — v0.4.0 リリース後独立測定

Issue #239 の `natural-language-e2e-v6` は、正式リリース済み `v0.4.0` / `50c750d976be63b4e489ba5d7d7f3225bdd839b8` を、新しい観測前 surface で測るための評価です。natural-language E2E v1〜v5、product-external-info v1〜v4、Stage-C、RSD2、#193/#195/#196 の freeze 済み研究 surface は再実行・再採点・修正・tuning に使いません。

## freeze する identity と product 座標

- corpus: `natural-language-e2e-v6`
- evaluator/report: `reason-natural-language-e2e-v6`
- scoring: `natural-language-e2e-scoring-v6`
- product tag: `v0.4.0`
- product commit: `50c750d976be63b4e489ba5d7d7f3225bdd839b8`
- product version: `0.4.0`
- natural output: `reason-natural-output-v4`
- session contract: `reason-session-v1`
- MCP adapter: `mcp_readonly_v3`
- provider/model: Mistral / `ministral-8b-latest`
- base seed: `43000`
- max tokens: `1024`
- case 間 pacing: `1500 ms`

測定前 capability probe Actions `34073701804` は release commit 上で 3/3 成功し、Mistral の `observed_model` も `ministral-8b-latest` でした。provider 応答は内部 backend revision を公開していないため、v6 は requested/observed API model identifier を固定し、内部 revision は観測不能であることを provenance に明記します。

## 独立 corpus

v6 は investigation 8件、session 3件です。case ID、task 文面、target fact key、source identity、fixture entity を新規作成し、manifest が指定する historical corpus と freeze 済み ref に対して marker 非再利用を機械テストします。

investigation は次を含みます。

1. explicit fact key と explicit read-only capability が1対1になる #233 deterministic selection 候補
2. 2 capability が残るため model selection を維持する ambiguous case
3. stale rejection
4. scope mismatch rejection
5. no-result 後に remaining unique safe action となる follow-up
6. authority claim mismatch rejection
7. source identity mismatch rejection
8. 実 `mcp_readonly_v3` の generic-output non-promotion

session は `add`、`correct`、`resume`、`fork` を対象にし、typed invalidation、stale finalization suppression、external replay 0、fork 非破壊、persisted finalization 一致を確認します。

## #233 の測定

runtime telemetry をそのまま使い、以下を分離します。

- `harness_unique_selections`: untried target/capability pair が機械的に1組だけで、fact key が明示対応している場合に Harness が決定した action 数
- `model_selected_action_calls`: model action selector の呼び出し数（investigation telemetry の `planner_calls`）

結果の良し悪しから selector を推測しません。

## MCP v3 lane

v6 の独立 MCP case は次を固定します。

- image: `ghcr.io/github/github-mcp-server@sha256:46cdbbd810faf6f7aed1745ea04057443f5cb9fcadc15c7308add18cf9a83e33`
- historical image version reference: `v1.12.0`
- repository: `git-ksk/reasoning-harness`
- ref: `refs/tags/v0.4.0`
- path: `Cargo.toml`
- tool: `get_file_contents`
- read-only 必須
- requested protocol: `2026-07-28`
- accepted downlevel: `2025-11-25`

これは #204 の historical acceptance case ではありません。GitHub MCP が返す通常の file content は generic acquisition output なので、Harness fact へ自己昇格してはいけません。そのため v6 case の expected outcome は意図的に `unknown` です。model-backed observation 前に pinned server/tool contract を別 preflight で確認しますが、その結果を semantic score へ流用しません。

## scoring

hard gate は `correctness_boundary_violations == 0` です。unsupported structured claim、unsupported exposed factual assertion、exposed-text contract violation、missed insufficiency、identity unsafe admission、MCP self-promotion、session invalidation/replay failure、admission rejection telemetry を分離して出します。

utility は target recall/omission、grounded target coverage、relevant capability selection、useful follow-up、irrelevant attempt、false abstention、Harness deterministic selection、model action selection、round、tool call、stop reason を測ります。

operational failure は semantic failure と混ぜません。provider call/attempt、公開 contract から観測できる model token、provider latency、process wall-clock、observed model identifier、tool call、stop reason、failure class を記録します。session operation は per-turn token usage を公開していないため、推定せず `token_usage_case_coverage` で欠測を明示します。

## freeze と観測ルール

最初の live observation 前に fixture、SHA-256、evaluator、test、scoring、provider/model、seed、budget、MCP coordinate、docs、workflow を commit し、`natural-language-e2e-v6-freeze` tag で固定します。workflow は provider credential を読む前にその freeze を再検証します。

最初の live observation 後、この surface は immutable です。operationally incomplete な試行だけ、同一 frozen coordinate の retry を許容し、失敗自体も operational evidence として保持します。corpus/evaluator/scoring/semantic を変更する必要があれば、新しい successor identity と別 Issue に分離します。historical v5 の公表値は書き換えません。
