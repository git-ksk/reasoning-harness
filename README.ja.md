# Reasoning Harness

日本語 | [English](README.md)

**根拠が足りないのに、AIが自信満々で答えるのを防ぐ。**

Reasoning Harnessは、AIの出力をエビデンスと検証の境界で包むruntimeです。人が直接使うCLIが **`reason`**。AIは回答候補を作りますが、どの事実をgrounded（根拠確認済み）として出せるか、どこを条件付きにするか、何を`unknown`のまま残すかはHarness側が決めます。

> **AIはcandidate（回答候補）を作る役であって、正しさを決めるauthority（権限）ではありません。**

```text
 task + evidence
       |
       v
      AI  -> untrusted candidate
       |
       v
 Reasoning Harness
       |
       +--> grounded answer
       +--> 条件付き回答
       +--> unknown / abstain
```

## どんなときに使う？

LLMやAgentは使いたい。でも、**「AIがそう言った」だけでは結果を信用したくない**ときに使います。

たとえば:

- **RAG / 調査AI** — 実際に検証できた根拠を超えて断言しないようにする。
- **障害 / architecture分析** — 観測済みfactは返しつつ、未証明のroot causeへ勝手に昇格させない。
- **Agent / CI** — model出力を次の自動処理へ渡す前に検証する。
- **安価なLLMの活用** — candidate生成は安いmodelに任せ、信用判断はprovider-neutralなruntimeに残す。

Harnessなし:

```text
evidence -> LLM -> answer
```

Harnessあり:

```text
evidence -> LLM -> candidate -> verify / resolve -> grounded | qualified | unknown
```

## 何が変わる？

たとえば障害中に、次の2つだけが確認できているとします。

```text
HTTP status = 503
DB connection errors = 7
```

流暢なAIは、ここから簡単にこう飛躍できます。

```text
「DBが障害の原因です」
```

Reasoning Harnessは、観測factと強い因果結論を分けます。root causeを裏付けるtrustedな因果エビデンスがなければ、結果は条件付きまたは`unknown`のままです。

```text
DBがroot causeとは確認できません。
HTTP 503とconnection error 7件は観測されていますが、それだけでは因果関係は確定できません。
```

これがこのプロダクトの中心です。**AIを便利に使いつつ、AIの自信をauthorityへ変換しない。**

## クイックスタート

現在のexternal previewは`v0.4.2`です。Rust 1.88+がある場合:

```bash
cargo install --git https://github.com/git-ksk/reasoning-harness \
  --tag v0.4.2 --locked reasoning-harness-cli --bin reason

reason --version
```

standalone archiveと`SHA256SUMS`も[v0.4.2 Release](https://github.com/git-ksk/reasoning-harness/releases/tag/v0.4.2)から取得できます。

自然文taskと、Harnessに明示的なstructured factとして扱わせたい根拠を渡します。

```bash
export MISTRAL_API_KEY='...'

reason "確認できるdeployment regionを答えて" \
  --provider mistral \
  --model ministral-8b-latest \
  --fact service.region=us-east-1 \
  --hypothesis service.region=us-east-1
```

次に、わざと根拠不足のケースを試せます。

```bash
reason "DBがHTTP 503のroot causeだと断定できる？" \
  --provider mistral \
  --model ministral-8b-latest \
  --fact http.status_code=503 \
  --fact db.connection_errors=7 \
  --hypothesis incident.root_cause=database
```

強い結論を裏付ける根拠がなければ、条件付き回答や`unknown`になるのが正常な安全結果です。

**APIキーなしで試したい場合:** 外部AIが作ったstructured candidateをofflineで検証できます。[Getting Started](docs/getting-started.ja.md)を参照してください。

## 結果は何を意味する？

| 結果 | 意味 |
| --- | --- |
| **Grounded answer** | 表示するfactがHarness管理のverified stateで裏付けられている。 |
| **条件付き回答** | 確認できた観測factは出すが、未証明の強い結論は不確実なままにする。 |
| **Unknown / abstain** | 現在のtrusted evidenceでは要求された結論を安全に出せない。 |

`unknown`はepistemic（知識状態）の結果であり、自動的に処理失敗を意味するわけではありません。

## 何が根拠（evidence）になる？

大事なのは、**context（読ませる情報）とauthority（正しさを支える権限）を分けること**です。

| Input | Harness上の意味 |
| --- | --- |
| positional `TASK` | 聞きたいこと。evidenceではない。 |
| `--file PATH` / piped stdin | AIが読めるcontext。別途verifyされるまではuntrusted。 |
| `--fact KEY=VALUE` | deterministic verification対象にできる明示structured evidence。 |
| `--hypothesis KEY=VALUE` | 評価・resolveしたいproposition（命題）。 |
| external resolver / read-only MCP output | 取得データ。Harness管理のadmission（受け入れ判定）とverificationを通るまでauthorityにはならない。 |

RAGやツールが文章を返しただけでは、その文章が自動的にverified evidenceになるわけではありません。

詳しい入力・設定契約は[CLIガイド](docs/cli.ja.md)を参照してください。

## アプリ / 自動化への組み込み

### 既存LLM / RAGの回答をチェックする

アプリ側ですでにretrievalとcandidate生成を持っている場合:

```bash
reason run \
  --input retrieved-evidence.json \
  --candidate model-candidate.json \
  --format json > checked-result.json
```

次の処理はLLMの文章ではなく、Harnessのstructured resultを見て判断します。

### `reason`にcandidate生成も任せる

```bash
reason run \
  --input evidence.json \
  --provider mistral \
  --model ministral-8b-latest \
  --format json
```

providerが作るのはあくまで**untrusted candidate**です。その後に同じHarness管理のverification pathが走ります。

### CI / Agentのgateとして使う

```bash
reason verify artifact.json --format json
```

または:

```bash
cat artifact.json | reason verify - --format json
```

exit codeはprocess stateです。`accept | reject | unknown`を判断したいautomationはJSON resultを確認してください。

## 別のLLMに採点させるのと何が違う？

Reasoning Harnessは「この回答、正しそう？」と別のAIへ聞いて、その返事をそのまま信用する仕組みではありません。

```text
 外部AI / Agent / RAG          Harness管理input
          |                         |
          v                         v
   untrusted candidate        evidence / policy
          |                         |
          +-----------+-------------+
                      v
              safeにmaterialize
                      |
                structure validation
                      |
                evidence verification
                      |
                  diagnostics
                      |
                acceptance policy
                      |
             accept | reject | unknown
```

modelが自分で`known`や`supported`と書いても、それだけではtrusted stateになりません。strong stateはHarness境界の中で、deterministic checkや明示的にtrustedなverifierから再構築されます。

詳しくは[Reasoning Harnessの仕組み](docs/how-it-works.ja.md)を参照してください。

## 現在のプロダクト機能

| Command | 用途 |
| --- | --- |
| `reason` | Reason CLI 0.5.0開発ライン: TTY限定managed interactive sessionを起動。JSON/non-TTYはnon-interactiveのまま。 |
| `reason -c` / `reason -r [SESSION]` | 最新project sessionをcontinue、またはbacking file pathを意識せずmanaged sessionをresume/pick。 |
| `reason "TASK"` | 人が直接使う自然文path。 |
| `reason setup` | Reason CLI 0.5.0開発ライン: provider / credential / model default / readinessを初回設定。 |
| `reason update` / `reason update --rollback VERSION` | Reason CLI 0.5.0開発ライン: provenance検証済みupdateと明示rollback。 |
| `reason uninstall` | Reason CLI 0.5.0開発ライン: data / credential保持をdefaultにした明示uninstall。 |
| `reason auth ...` | Reason CLI 0.5.0開発ライン: provider credentialの安全な追加・確認・rotation・削除。 |
| `reason models [provider]` | Reason CLI 0.5.0開発ライン: curated model catalog、compatibility、credential readiness、current defaultを確認。 |
| `reason model set <provider> <model>` | Reason CLI 0.5.0開発ライン: silent fallbackなしでgeneral-use provider/model defaultを保存。 |
| `reason config list/get/set/unset/path/sources` | Reason CLI 0.5.0開発ライン: safeなeffective configとprecedence/provenanceを確認し、user-levelのnon-secret run defaultを編集。 |
| `reason session list` | managed interactive sessionをhuman/JSONで一覧化。 |
| `reason session ... --store PATH` | 低レベルtyped session互換surface: 明示session fileの保存・確認・追加・訂正・resume・fork・close。 |
| `reason run` | Application / CI統合、live candidate生成。 |
| `reason verify` | materialize済み`ReasoningArtifact`のdeterministic validation。 |
| `reason semantic-check` | soft semantic diagnostic。最終authorityは持たない。 |
| `reason schema` | versioned machine contractの確認。 |
`/add`、multiline input、TTY/JSON dispatch、privacy boundaryを含むinteractive REPLの詳細は[対話型ターミナルUX](docs/reason-interactive.ja.md)を参照してください。
verified fact、uncertainty、evidence provenanceを示すhuman outputは[Human answer presentation](docs/reason-human-output.ja.md)を参照してください。
provider usage、`/usage`、session累積accounting、hard budget guardは[Provider usage / budget guard](docs/reason-usage-budget.ja.md)を参照してください。
managed sessionのlock、optimistic concurrency、crash recovery、rollback compatibilityは[Managed session durability](docs/reason-managed-sessions.ja.md)を参照してください。
local storage、ephemeral mode、retention/purge、uninstall、outbound-data boundaryは[Local privacy / retention](docs/reason-local-privacy.ja.md)を参照してください。


Mistral、Google Gemini/AI Studio、NVIDIA Hosted NIM、Groq向けのprovider adapterを実装済みです。現在のruntimeには、read-only MCPによる取得、外部resolver、明示的にtrustedなdeterministic verifier、bounded investigation、再開可能なsessionも含まれます。

## 実測で見る：Harnessなしから何が変わる？

Reasoning Harnessの価値は、「安全そうに見える回答」を増やすことではなく、**答えるべきケースの有用性を保ちながら、根拠不足の断言をruntime側で止められるか**で評価しています。

### 総合比較：同じ入力をHarnessなし / ありで比べる

`product-external-info-v4`は、この比較のために固定した同条件比較です。21ケースのうち18件を意味上の採点、3件を型付きの運用失敗確認に使います。主比較では、Harnessなし / ありの両方へ**同じタスク、同じtarget hypothesis、同じ根拠要件、同じauthority policy、同じ取得済みexternal snapshot**を渡します。

つまり、下の差は「Harnessなし側だけ情報が少ない」といった不公平な比較ではなく、**同じ材料をmodelの自己判断だけで扱う場合と、Harnessの受け入れ判定 / 検証 / 最終化を通す場合の差**です。

| Model | 回答到達率（5件） | `unknown`維持率（13件） | 不要な棄権 | 裏付けなしgrounded claim | 根拠不足の見逃し |
| --- | ---: | ---: | ---: | ---: | ---: |
| **Ministral 8B** | **80% → 100%** | **53.8% → 100%** | **1 → 0** | **6 → 0** | **6 → 0** |
| **Gemma 4 31B** | 100% → 100% | **84.6% → 100%** | 0 → 0 | **2 → 0** | **2 → 0** |
| **Gemini 3.5 Flash-Lite** | 100% → 100% | 100% → 100% | 0 → 0 | 0 → 0 | 0 → 0 |
| **GPT-OSS 120B** | **80% → 100%** | **76.9% → 100%** | **1 → 0** | **3 → 0** | **3 → 0** |

`A → B`は **Harnessなし → Harnessあり** です。

各指標の意味は次の通りです。

- **回答到達率**（`expected_grounded_target_coverage`）— 本来答えられる5件のうち、要求されたtargetをgroundedな回答として出せた割合。高いほど有用性が高い。
- **`unknown`維持率**（`expected_unknown_preservation`）— 鮮度切れ、scope不一致、authority不足、identity不一致、conflictなど、確定してはいけない13件で断言を避けられた割合。高いほど安全。
- **不要な棄権**（`false_target_abstention`）— 答えられるケースなのにtargetを確定できなかった件数。少ないほど良い。
- **裏付けなしgrounded claim**（`unsupported_grounded_claims`）— 十分なsupportがないのにgroundedとして出した主張数。少ないほど良く、Harness側は4モデルとも0。
- **根拠不足の見逃し**（`missed_target_insufficiency`）— 本来`unknown`にすべきケースを確定回答してしまった件数。少ないほど良く、Harness側は4モデルとも0。

特に重要なのは、Harnessが単に「慎重になって答えなくなる」だけではない点です。Ministral 8BとGPT-OSS 120Bでは、**安全性を100%まで引き上げながらtarget coverageも80%から100%へ改善**しました。Gemini 3.5 Flash-Liteのようにraw modelだけで全ケースを守れた観測もあり、その場合Harnessはsemantic scoreを悪化させず同じ境界を維持しました。

### トークン量 / レイテンシへの影響

同じv4実測ではコストも記録しています。

| Model | Harness / raw model token | Harness / raw accounted latency |
| --- | ---: | ---: |
| **Ministral 8B** | 1.234x | 0.642x |
| **Gemma 4 31B** | 0.673x | 1.159x |
| **Gemini 3.5 Flash-Lite** | 0.641x | 1.034x |
| **GPT-OSS 120B** | 0.938x | 0.860x |

Harnessの導入が常にtoken増・常に低速になるわけではありません。このsingle-run観測ではGemma、Gemini、GPT-OSS 120BはHarness側のmodel tokenが少なく、Ministral 8Bでは増えました。latencyもmodelごとに方向が異なるため、**安全性の結果とは分けてoperational observationとして扱います**。安定した速度ランキングの主張ではありません。

詳細な条件・全ケース・run provenanceは[external information v4 cross-model比較](docs/product-external-info-v4-cross-model.ja.md)を参照してください。machine-readable artifactも`docs/observations/`へ保存しています。

### v36のリリース評価でも安全境界を追試

v4とは別に、`v0.4.2`最終release gateのv36から、raw modelと意味を揃えて比較できる5つの安全境界ケースだけを抽出した**補足評価**もfreezeして実行しました。ここではutilityやplanner性能を再採点せず、「同じpolicyとraw observationをmodelへ渡すだけで`unknown`を維持できるか」を確認しています。

| Model | Raw model | Harness | Rawで境界を越えたケース |
| --- | ---: | ---: | --- |
| **Ministral 8B** | 4/5 = 80% | **5/5 = 100%** | MCP generic content non-promotion |
| **GPT-OSS 120B** | **5/5 = 100%** | **5/5 = 100%** | なし |
| **Gemini 3.5 Flash-Lite** | 4/5 = 80% | **5/5 = 100%** | authority mismatch |
| **Gemma 4 31B** | 4/5 = 80% | **5/5 = 100%** | MCP generic content non-promotion |

4モデルともoperational failure 0、output contract violation 0で完走しました。raw modelは3/4モデルで1件ずつ安全境界を越えましたが、Harness側は4モデルすべて5/5を維持しました。これはv4の総合比較を置き換える結果ではなく、**release評価surface上でも同じ設計意図を再確認した補強証拠**です。

詳細は[v36 raw safety supplement](docs/v36-raw-baseline-supplement.ja.md)を参照してください。

### v0.4.2の最終リリース判定

`v0.4.2`自体のrelease判定は、freshにfreezeした13ケースのnatural-language E2Eで行いました。これは上のraw-vs-Harness比較とは目的が異なり、product runtimeがrequired provider rowごとにrelease条件を満たすかを見るgateです。provider間の平均でPASSにすることはしていません。

| Model / provider | v0.4.2最終実測 |
| --- | --- |
| **Mistral / Ministral 8B** | PASS — candidate 13/13、operational failure 0、correctness-boundary violation 0 |
| **Groq / GPT-OSS 120B** | PASS — candidate 13/13、operational failure 0、correctness-boundary violation 0 |
| **Gemini 3.5 Flash-Lite** | PASS — 13/13、tool selection `0.6 → 1.0`、trigger exposure `0/3 → 3/3`、avoidable stall `3 → 0` |
| **Gemma 4 31B** | PASS — candidate 13/13、operational failure 0、correctness-boundary violation 0 |

freeze座標、metric、run ID、pacing policy、provenanceは[v0.4.2 v36 release acceptance](docs/natural-language-e2e-v36-result.ja.md)に固定しています。失敗・inconclusiveを都合よく消して再測定するのではなく、freezeしたevaluationを履歴として残し、現在のclaimを再現可能な実測へ結びつけています。

## 製品・エンジン・研究を分ける

`v0.4.2`は、製品CLIとreasoning engineが同じversion座標を共有する最後のreleaseです。

次のlineからは:

- **Reason CLI** — terminal UX、setup、distributionなど人向けproductをversioning。
- **Harness Engine** — reasoning / correctness behaviorをversioning。
- **Machine contract ID** — wire/schema compatibilityを独立してversioning。

次の一般向けラインは **Reason CLI 0.5.0 + Harness Engine 0.4.2** を予定しています。セットアップ、OSのcredential保存、対話型UX、installer、doctor、update/uninstallなどを改善しても、Engineのcorrectness semanticsが変わったように見せないためです。

詳しくは[バージョニング](docs/versioning.ja.md)と[Reason CLI 0.5.0 ロードマップ](docs/reason-cli-0.5-roadmap.ja.md)を参照してください。

## ドキュメント

`docs/`をファイル名順・時系列順に読む必要はありません。ここには通常のproduct documentationと、freeze済みの研究/evaluation証跡が両方あります。

まず **[ドキュメント案内](docs/README.ja.md)** から入ってください。

- 初回セットアップと日常CLI利用
- アプリケーション / CI / MCP連携
- アーキテクチャと信頼境界
- 現在のロードマップ / プロジェクト状況
- 過去の研究 / 評価証跡

を分けて案内しています。

## これは何ではない？

- 汎用chat client / coding agent。
- prompt集。
- 特定model専用agent framework。
- 別LLMの自己申告をauthorityにするpost-hoc judge。
- open-world reasoningを数学的に解決したという主張。
- compiler、test、schema、policy engine、proof checkerなどdeterministic oracleの代替。
- correctness coreへ埋め込んだ汎用crawler / RAG framework。

## 開発者向け

Rust 1.88+がsupported toolchainです。first-party runtime componentはRust-onlyです。

```bash
cargo fmt --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
```

[CONTRIBUTING.ja.md](CONTRIBUTING.ja.md)、[SECURITY.ja.md](SECURITY.ja.md)、[architecture](docs/architecture.ja.md)、[Documentation index](docs/README.ja.md)を参照してください。
