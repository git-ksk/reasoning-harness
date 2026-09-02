# Product dogfood capability matrix

Issue #147では、従来の6-case smoke sliceを、観測前に固定する広いmodel-fitness評価へ拡張します。これはproduct評価であり、新しいsemantic authority実験ではありません。採用済みD3 runtime、sufficiency policy、verifier boundary、frozen research holdoutは変更しません。

## 3段階設計

### Stage A — 24-case development/product matrix

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

### Stage B — 5-run replication

Stage Aでoperationally completeかつ有用な候補だけを進めます。frozen v2 corpusを事前固定した5 base seedで再実行し、case×seedでpaired比較します。aggregate mean target coverageだけでは採否を決めません。

最低限、provider/protocol completion、unsupported grounded claim、missed target insufficiency、target coverage / false abstention、resolution success、safe-partial retention、token/latency overhead、run間case disagreementを記録します。

provider/protocol failureはoperational evidenceのままで、semantic denominatorへ架空のabstainとして入れません。

### Stage C — fresh holdout

Stage Bのmodel/runtime選定後にのみ、新しい12〜16 case holdoutを作成します。fixture payload、model list、seed、acceptance gateをprovider観測前にfreezeします。development matrix結果を見てfresh holdoutを後から書き換えません。

## v2で必要になったevaluator hardening

従来のdogfood helperはraw support判定でstructured key/value一致しか見ておらず、temporal/scope caseではstale/out-of-scope evidenceを誤ってsupported扱いする余地がありました。v2ではruntimeと同じtrusted structured-fact verifier selectionを評価側でも再利用し、evidence qualification requirementを反映します。matching key/valueが入力内にあるだけではsupportedになりません。

これは評価accountingの修正だけで、modelやevaluatorへ新しいauthorityを付与せず、採用済みruntime policyも変更しません。

## Workflow

manual `product-dogfood` workflowで`product-dogfood-v1` / `product-dogfood-v2`を明示選択できます。新しいcapability matrix観測ではv2をdefaultにします。provider credentialを読む前に、workflowは次を検証します。

1. `reason-product-dogfood --validate-only`でfixture corpusをparse
2. v2 SHA-256 manifestを検証
3. 24 cases、8 capability families、各3 casesを要求

6-case v1は高速smoke/regressionと過去NL-5結果の解釈用に残します。

## Stage A結果とutility recoveryの中間フェーズ

Stage Aは、freeze済み`product-dogfood-v2`、base seed `12000`、max tokens `1024`で完了した。6モデルが24ケースを完走し、Harness armではunsupported exposed grounded claimとmissed task-target insufficiencyはいずれも0だった。successor target coverageは、Gemini 3.5 Flash-Lite `1.00`、Mistral Small `0.70`、Gemma 4 31B `0.60`、Ministral 8B `0.20`、Ministral 14B `0.20`、Ministral 3B `0.10`。Gemma 4 26BとNemotron 3.5 Lightningはprotocol-incomplete。Gemini 3.1 Flash-Liteはcase 18まで進んだ後、Google側HTTP 500 high-demandでoperationally incompleteとなったためStage A semantic scoreは付けない。

24ケース化により、6ケースでは見えなかったproduct portability上の問題が局在化した。複数のMinistral expected-grounded missでは`final_verdict=accept`まで到達しているのにstructured final claimを露出できず、Gemma 4 31Bでは人間には意味が通るrenderer claimでもharness-owned exact keyからずれたためfinalizationが正しくblockしていた。これはexact proposition identityを緩める理由ではなく、すでにverify済みのartifact stateからHarness自身が安全に回収する余地を示す。

そのためStage Bの前にIssue #150を明示的な中間フェーズとして挟む。対象は、(1) behavior-neutralなfailure provenance、(2) exact `Known` / `Supported` task targetのdeterministic canonical recovery、(3) harness-owned unresolved targetだけを起点にしたbounded resolver closureとmandatory admission/re-verification、(4) conservative safe-partial recovery、(5) provider-neutral structured-output resilienceの別lane、(6) before/after development comparisonの後にfresh Stage-B seeds `13000`, `13100`, `13200`, `13300`, `13400`でreplication、の6点。

v2 fixtureとhash manifestはこの間もfreezeしたまま変更しない。Stage Bは#150のcandidate behaviorがfreezeされるまで意図的に保留し、fresh 12〜16ケースholdoutはStage B selection後まで作成しない。
