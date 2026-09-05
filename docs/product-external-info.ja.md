# Product MCP external-information 評価

Issue #203では、既存のread-only MCP境界を通して実際の外部情報を取得するfreeze済みproduct workloadを追加します。従来の6-case `product-dogfood-v1` smokeや、#193/#195/#196のentity-identity research holdoutとは別系統です。

## 何をfreezeしたか

workload identityは`product-external-info-v1`です。

- 21ケース
- 7 capability family × 3ケース
- expected-grounded 5ケース
- expected-unknown 13ケース
- typed operational-failure 3ケース
- SHA-256 manifest: `fixtures/product-external-info-v1.sha256`
- 作業開始前のmain: `aa0a8325ea4c3b53b38c8fe83cf3aae691a38599`
- freeze commit: `8aa7a9ed72ed80b186bde230078a45d6ba28141c`

既存の`fixtures/product-dogfood-v1` 6ケースは別manifestでhash-lockし、#203開始時点からbyte単位で変更していません。新workloadには`resolver_facts`を入れず、#193/#195/#196で観測済みのholdout entity/caseも再利用・変形していません。

scoring contractは`product-external-info-scoring-v1`です。次の3 armを比較します。

1. raw model
2. Harness without external acquisition
3. Harness + MCP external acquisition

typed operational failureはsemantic denominatorから除外し、acquisition successとverification successを別々に記録します。

## 外部取得の境界

repository-local fixture MCP serverは`scripts/product_external_info_mcp.py`です。固定したpublic-host allowlistに対してbounded HTTPS GETだけを行い、Harnessが固定したJSON fieldだけを抽出し、必要な場合だけ`mcp_readonly_v1`が理解する明示的なuntrusted `structuredContent.reasoning_harness` acquisition envelopeを返します。

これは取得・正規化componentであり、authorityでもverifierでもありません。URL、field、source identity、authority class、scope、identity assertionはすべてHarness-ownedの固定設定です。generic MCP `content`からfact candidateを作ることはありません。

取得後のevidenceも、通常のHarness-owned provenance / freshness / scope / authority admissionとverificationを通らなければgrounded targetとして公開できません。

## 最初の有効なlive 3-arm観測

事前freeze後の最初のworkflow runではevaluator clock wiring defectが見つかりました。このrunはinfrastructure-invalidatedな監査記録として残し、fixture、target、expected outcome、scoring ruleは変更していません。修正ではuntrusted model generationの後にHarness evaluation clockを取得し、2つのHarness armで同じevaluation inputを共有するようにしました。

最初の有効なpost-fix観測は次です。

- GitHub Actions run: `33974104359`
- branch head: `146e17a1bed3314d2827957949e6e98665ea9594`
- provider: Mistral
- model: `ministral-8b-latest`
- max output tokens: `1024`
- base seed: `15000`
- report schema: `reason-product-external-info-v1`
- comparison contract: `shared-candidate-canonical-finalization-v1`
- machine-readable report: [`observations/product-external-info-v1-mistral-ministral-8b-seed-15000-2026-09-05.json`](observations/product-external-info-v1-mistral-ministral-8b-seed-15000-2026-09-05.json)

### Semantic utility

| Arm | semantic scoring対象 | expected-grounded scoring対象 | 根拠付きtargetを公開 | target coverage | expected-unknown scoring対象 | unknownを維持 | unknown preservation | false target abstention |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Raw model | 18 | 5 | 0 | **0.00** | 13 | 13 | **1.00** | 5 |
| Harness without external acquisition | 18 | 5 | 0 | **0.00** | 13 | 13 | **1.00** | 5 |
| Harness + MCP external acquisition | 16 | 4 | 3 | **0.75** | 12 | 12 | **1.00** | 1 |

利用者目線で重要なのは「MCPを付ければ何でも答える」ことではありません。このfreeze済みrunでは、raw modelと外部取得なしHarnessが1件も出せなかったexpected-grounded targetのうち、MCP armはscoring対象4件中3件を**根拠付きで回復**しました。同時に、scoring対象のexpected-unknown 12件は**12/12で断言せず未確定のまま維持**しました。

MCP armではsemantic case 2件がoperationally incompleteとなり、事前に決めた規則どおりsemantic denominatorから除外しています。toolの失敗を意味上の誤答や架空のabstentionへ変換していません。

### Safety / admission結果

MCP armの実測は次です。

- external acquisition attempts: `21`
- external acquisition successes: `16`
- verification successes: `4`
- unsupported grounded claims: **0**
- missed target insufficiency: **0**
- identity-unsafe admission: **0**
- MCP-output authority self-promotion: **0**
- stale rejection: `1`
- authority rejection: `2`
- conflict rejection: `1`
- scope rejection: `0`
- typed tool/protocol/timeout/policy operational failures: `5`
- freeze済みsafety gate: **pass**

acquisition successとverification successは意図的に別物として扱います。外部取得は16件成功しましたが、target verification successは4件だけでした。つまり「toolが値を返した」ことをtruthへ昇格させていません。

## 保守的に残った1件

`authority-crates-serde-primary`は、evidence取得・admit・exact target verificationまで成功しました。ただしmodel由来の別stateが未解決のままだったためartifact全体は`unresolved`となり、そのtargetを外部公開しませんでした。これがMCP armで唯一のscored false target abstentionです。

これは危険な誤答ではなく保守的なutility lossとして記録します。観測後にworkloadやscoringを変更して`0.75`を`1.00`へ作り替えることはしません。

## Operationally incompleteだったsemantic case

次の2件はMCP acquisition pathがtyped `policy_denied`で終了したため、semantic denominatorから除外しました。

- `fresh-npm-typescript-name`（expected grounded）
- `identity-npm-vite-vs-vitest`（expected unknown）

この結果を見てcase timeoutやresponse allowanceを後から広げることはしていません。

## 公式GitHub MCPとのcompatibility boundary

別のdirect probeでは、公式`github/github-mcp-server` v1.12.0 containerをread-only modeで確認しました。現行`mcp_readonly_v1`はstateless / initialize-independentですが、公式serverはMCP session initializationを必要とし、clientが`2026-07-28`を提示したとき`2025-11-25`へprotocol negotiationしました。

標準の`initialize -> initialized -> tools/call` session後なら`get_file_contents`は成功しましたが、返却はgeneric `content` / `resultType`であり、`structuredContent.reasoning_harness` fact envelopeではありません。そのため現行Harnessでは安全にnon-promotingのままです。

session/protocol negotiation対応は#203内で`mcp_readonly_v1`を暗黙変更せず、Issue #204へ分離しています。

## 制約・読み方

- これはfreeze済みworkload 1 slice、Ministral 8B 1 runの実測であり、すべてのopen-world task、MCP server、modelに一般化する主張ではありません。
- semantic case 2件はoperationally incompleteで、MCP semantic denominatorから除外されています。
- live corpusで`scope_rejection`は0でした。scopeはcase設定とcredential-free deterministic external-resolution coverageではHarness-owned gateとして維持していますが、このlive sliceではscope mismatch rejectionの実測件数は0です。
- 公式GitHub MCPは現行stateless `mcp_readonly_v1` transport contractとはまだend-to-end互換ではなく、#204でsuccessor対応します。
- verifiedだが公開されなかった1 targetは、観測後にtuningして消さず、保守的な残差として残します。

## 再現性ルール

この観測結果を理由に`product-external-info-v1`を書き換えません。semantic case、expected outcome、scoringを修正する必要がある場合はsuccessor corpus identityを作ります。operational retry / transport fixもsemantic tuningとは別の変更として扱います。
