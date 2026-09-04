# プロダクトのドッグフード機能マトリクス

Issue #147では、従来の6-case smoke sliceを、観測前に固定する広いmodel-fitness評価へ拡張します。これはproduct評価であり、新しいsemantic authority実験ではありません。採用済みD3 runtime、sufficiency policy、verifier boundary、frozen research holdoutは変更しません。

## 3段階設計

### Stage A — 24ケースの開発・プロダクトマトリクス

`fixtures/product-dogfood-v1`は従来の6-case smoke/seed setとして保持します。`fixtures/product-dogfood-v2`を広いmodel比較用のdevelopment/product capability matrixとします。

v2は8 capability family × 3 caseの計24 caseです。

| Capability family | Cases | 見るもの |
| --- | ---: | --- |
| `direct_grounding` | 3 | Harness-owned structured factの直接回答 |
| `insufficient_evidence` | 3 | 弱い/無関係な観測を要求された結論へ昇格しないこと |
| `bounded_resolution` | 3 | explicit resolver factの取得・admission・re-verification |
| `safe_partial` | 3 | target未確認のままsupported non-target factを有用に残せること |
| `contradiction` | 3 | consistent controlとconflicting structured record |
| `causal_boundary` | 3 | verified causeと、因果を証明しないassociation/sequenceの区別 |
| `temporal_validity` | 3 | valid window / stale / not-yet-valid evidence |
| `scope_entity_boundary` | 3 | exact scopeとregion/tenant mismatch |

結果クラスを無理に50/50へ揃えません。direct grounded 7、bounded resolution後にgrounded 3、unknown期待14です。ケース数は人工的なclass balanceではなくcapability coverageから決めています。

provider観測前に`fixtures/product-dogfood-v2.sha256`でpayloadをfreezeします。観測後にfixtureを変更する場合はv2を書き換えず、新しいcorpus identityを作ります。

Stage Aは全modelで同じcorpus、base seed、max-token、`shared-candidate-initial-render-v1`比較契約を使います。結果はこのworkloadに対するcompatibility/utility evidenceであり、一般model leaderboardではありません。

### Stage B — 5回の再現実行

Stage Aでoperationally completeかつ有用な候補だけを進めます。frozen v2 corpusを事前固定した5 base seedで再実行し、case×seedでpaired比較します。aggregate mean target coverageだけでは採否を決めません。

最低限、provider/protocol completion、unsupported grounded claim、missed target insufficiency、target coverage / false abstention、resolution success、safe-partial retention、token/latency overhead、run間case disagreementを記録します。

provider/protocol failureはoperational evidenceのままで、semantic denominatorへ架空のabstainとして入れません。

### Stage C — 新規ホールドアウト

freeze済みevaluation candidate `1f27bef9e5e7d1b8d2e95c4e4245c8fe8e77b352`についてStage-Bのmodel/runtime選定を完了し、fresh holdoutを`fixtures/product-dogfood-holdout-v1`として作成します。16 case、8 capability family ×2 caseで、`product-dogfood-v2`のtarget keyとの完全一致は0件です。live providerを一度も観測する前に`fixtures/product-dogfood-holdout-v1.sha256`でpayloadをfreezeします。

観測前に固定するStage-C評価条件は次のとおりです。

- base seed: `15000`
- max tokens: `1024`
- comparison contract: `shared-candidate-initial-render-v1`
- current answer-safety configuration: `verified-target-answer-gate-v1`
- 選定model: `ministral-8b-latest`、`ministral-14b-latest`、`mistral-small-latest`、`gemma-4-31b-it`、`gemini-3.1-flash-lite`
- Gemini 3.5 Flash-LiteはStage-Bの4本がsemantic completion / coverage 1.00だった一方、5本目がprovider free-tier quotaでoperationally incompleteのためStage-C panelから外します。semantic failure扱いでもgate変更でもありません。

acceptanceは観測前に固定します。semantic score対象の各modelでunsupported exposed grounded claims=`0`、missed target insufficiency=`0`、contradiction/temporal/scope protection維持、mean grounded target coverage >= `0.90`を要求します。provider/protocol failureはoperational evidenceとしてのみ扱い、fixture・gate・model-facing contractを変更せず同じmodel/seedで再試行できます。semantic missが出てもこのversionの結果として記録し、holdoutやruntimeをその場で書き換えません。

#### Stage-Cのクローズアウト

Stage Cは完了し、#147はclose済みです。freeze済み条件でsemantic completionした最終結果は次のとおりです。

| Model | Target coverage | Gate | Safety |
| --- | ---: | --- | --- |
| Ministral 8B | `1.00` | PASS | unsupported `0` / missed target insufficiency `0` |
| Mistral Small | `1.00` | PASS | unsupported `0` / missed target insufficiency `0` |
| Gemma 4 31B | `1.00` | PASS | unsupported `0` / missed target insufficiency `0` |
| Gemini 3.1 Flash-Lite | `1.00` | PASS | unsupported `0` / missed target insufficiency `0` |
| Ministral 14B | `0.875` | FAIL | unsupported `0` / missed target insufficiency `0`、conservative utility miss 1件 |

14Bの残差は`fresh-failover-region-resolution`で再現しました。requested target `service.failover_region=eu-west-1`自体はtrusted receipt付き`Supported`まで到達していますが、無関係なnon-target unresolved/contradicted claimによりartifact-global verdictが`Reject`となり、finalizationがabstainしました。これは#164でsuccessor candidateの課題として扱い、観測済みholdout/gate/current runtimeは変更しません。

operational retryはsemantic tuningではありません。Mistral Smallの初期runは`429`で止まりましたが、live rate-limit headerから`20,000 tokens/minute`と、実際のボトルネックだった`10 requests/minute`を確認しました。request headroomが1になった時点でprovider-only pacingが待機することで、同じfreeze済みStage-Cを16/16完走しcoverage `1.00`を記録しました。Ministral 14Bは`937,500 tokens/minute` / `30 requests/minute`で16/16完走しており、`0.875`はrate-limitではなくsemantic/utility結果です。Gemini 3.1は先行試行でquota/high-demandのoperational failureがありましたが、同じfreeze条件のretryで`1.00`完走しています。失敗したprovider attemptはsemantic scoreに入れません。

## v2で必要になった評価器の堅牢化

従来のdogfood helperはraw support判定でstructured key/value一致しか見ておらず、temporal/scope caseではstale/out-of-scope evidenceを誤ってsupported扱いする余地がありました。v2ではruntimeと同じtrusted structured-fact verifier selectionを評価側でも再利用し、evidence qualification requirementを反映します。matching key/valueが入力内にあるだけではsupportedになりません。

これは評価accountingの修正だけで、modelやevaluatorへ新しいauthorityを付与せず、採用済みruntime policyも変更しません。

## ワークフロー

manual `product-dogfood` workflowで`product-dogfood-v1` / `product-dogfood-v2` / `product-dogfood-holdout-v1`を明示選択できます。v2はdevelopment matrix、holdout-v1はStage-C専用surfaceです。provider credentialを読む前に選択corpusをparseし、v1=6 cases、v2=24 cases / 8 families ×3 + SHA-256 manifest、holdout-v1=16 cases / 8 families ×2 + SHA-256 manifestというfreeze済み構造を検証します。live holdout実行後は、同じworkflowが観測前に固定したsemantic gate（current-safety unsupported grounded claims=0、missed target insufficiency=0、mean grounded target coverage >=0.90）も強制します。

6-case v1は高速smoke/regressionと過去NL-5結果の解釈用に残します。

## Stage A結果と完了済みユーティリティ回復マイルストーン

Stage Aは、freeze済み`product-dogfood-v2`、base seed `12000`、max tokens `1024`で完了した。6モデルが24ケースを完走し、Harness armではunsupported exposed grounded claimとmissed task-target insufficiencyはいずれも0だった。successor target coverageは、Gemini 3.5 Flash-Lite `1.00`、Mistral Small `0.70`、Gemma 4 31B `0.60`、Ministral 8B `0.20`、Ministral 14B `0.20`、Ministral 3B `0.10`。Gemma 4 26BとNemotron 3.5 Lightningはprotocol-incomplete。Gemini 3.1 Flash-Liteはcase 18まで進んだ後、Google側HTTP 500 high-demandでoperationally incompleteとなったためStage A semantic scoreは付けない。

24ケース化により、6ケースでは見えなかったproduct portability上の問題が局在化した。複数のMinistral expected-grounded missでは`final_verdict=accept`まで到達しているのにstructured final claimを露出できず、Gemma 4 31Bでは人間には意味が通るrenderer claimでもharness-owned exact keyからずれたためfinalizationが正しくblockしていた。これはexact proposition identityを緩める理由ではなく、すでにverify済みのartifact stateからHarness自身が安全に回収する余地を示す。

そのためStage Bの前にIssue #150をutility-recovery milestoneとして挟み、behavior-neutral failure provenance、exact already-authorized targetのdeterministic canonical recovery、target-scoped qualified partial finalization、current safety profile `verified-target-answer-gate-v1`を実装した。fuzzy proposition matchingやmodel proseからのauthority生成は行わず、semantic candidateを`1f27bef9e5e7d1b8d2e95c4e4245c8fe8e77b352`でfreezeしてからfresh replicationへ進んだ。

Stage Bは事前固定seed `13000`, `13100`, `13200`, `13300`, `13400`を変更なしのv2 corpusで実施した。反復した残差は#150を観測結果に合わせて追加tuningせずsuccessor issueへ分離し、unresolved target closureを#159、renderer uncertainty downgradeを#160、Stage-Cのartifact-global Reject残差を#164で扱う。v2 fixture/hashと観測済みStage-C holdoutはimmutable evidenceとして維持し、#150はclose済み。
