# Product external-information v4

`product-external-info-v4`はIssue #206向けの次のpre-provider successorである。v3はmandatory acquisition-only preflight（run `33999853866`）でprovider credentialを使う前に停止した。preflight自体の安全性・conflict検証は正常だったが、旧`encode/starlette` repositoryが現在`Kludex/starlette`へresolveする外部identity driftを1件検出したためである。v3のprovider/model観測は0回のまま。

v4はv3 evaluatorの設計と安全境界をそのまま維持し、case/target identityをすべて新しくし、driftしたgrounded entityだけ`pytest-dev/pytest`へ置き換える。

## freezeする設計

- 21ケース / 7 capability family / 各3ケース。
- v1/v2/v3のcase identity・target proposition pairは再利用しない。
- `mcp_readonly_v1`は変更しない。protocol `2026-07-28`、stateless single `tools/call`、stdio、generic content非昇格。session/negotiation compatibilityはIssue #204の責務。
- raw/Harnessの両armへ同じtask、exact target hypothesis key/value、evidence requirement、authority policyを渡す。supplied hypothesis自体は証拠ではなくauthorityも付与しない。
- arm 3/4は1回のreal decoded MCP acquisition observation setを共有する。rawはuntrusted contextとして受け取り、Harnessは通常のadmission / verification / conflict handling / finalizationへreplayする。
- npm caseはboundedな`/latest` endpointを使う。
- conflict caseはv3 preflightで実証済みの`pallets/click`構造を維持する。`/owner/login = pallets`と`/name = click`を同じfact keyへ出し、両方とも`/full_name = pallets/click`でidentity-validとする。
- irrelevant observation付きgrounded caseは`pytest-dev/pytest`を使い、`/description`は観測に含めるがtarget factとして選ぶのは`/full_name`だけ。

## 4 armとscoring

1. `raw_model_no_external`
2. `harness_no_external`
3. `raw_model_with_external`
4. `harness_with_mcp_external`

主比較はarm 3 vs arm 4。scoring identityは`product-external-info-scoring-v4`、comparison contractは`matched-target-context-four-arm-v4`。

Harness acceptanceはfail-closedのまま。unsupported grounded claims = 0、missed target insufficiency = 0、identity-unsafe admission = 0、MCP-output authority self-promotion = 0、expected-unknown preservation = 1.0を必須とする。coverage 1.0はutility目標であり安全境界を弱める理由にはしない。

costは各armのmodel attempts / latency / input-output-total tokens、shared external calls / latency、accounted end-to-end latency、arm 4対arm 3のoverhead ratioを記録する。

## mandatory acquisition-only preflight

provider credentialを利用可能にする前に、model callなしのv4 acquisition probeを全件通す。

- semantic 18ケースがすべてoperationally complete。
- synthetic grounded target coverage = 1.0。
- expected-unknown preservation = 1.0。
- Harness safety counter = 0。
- typed operational failure = 3。
- conflict caseが`click`と`pallets`の2値を生成し、conflictを検出し、targetをverify/public exposureしない。
- pytest grounded caseが`pytest-dev/pytest`を生成し、synthetic target probeで公開できる。

このgateが失敗した場合、v4をその場で修正せず次のsuccessor identityへ進む。

## 最初のprovider条件

- provider: `mistral`
- model: `ministral-8b-latest`
- seed: `28000` + deterministic case index
- max tokens: `1024`
- 最初のvalid provider runをcanonicalとする

provider観測後はv4のcase selection / expected outcome / target / scoringを編集しない。
