# Reasoning Harnessの仕組み

日本語 | [English](how-it-works.md)

このドキュメントでは、`reason run`が何をしているか、特に**外部AI/Agentが`ReasoningCandidate`を作り、Reasoning Harness自身はAI endpointを呼ばないモード**でもなぜ判定できるのかを説明します。

> **Product direction:** ここで説明するstructured objectは、runtimeのinspectableな内部/高度なcontractとして残ります。Primary UXはAI-backedな自然文CLI ([Issue #107](https://github.com/git-ksk/reasoning-harness/issues/107)) へ進めますが、自然文経路もこのauthority boundaryを省略せず同じruntimeを通ります。

## 一番大事なのは「提案」と「権限」を分けること

Reasoning Harnessでは、AIの出力と「信用してよいという判断」を同じものとして扱いません。

- AI/Agentはclaim、epistemic state、inference edgeを**提案**できる。
- `HarnessInput`はtask、evidence、evidence requirement、assumption、authority policyをHarness側で管理する。
- trusted verifierだけが`VerificationReceipt`を作れる。
- 最終的な`accept | reject | unknown`はmodelではなくacceptance policyが決める。

```text
               信用しない側                         Harness管理 / trusted側

 外部AI / Agent
       |
       v
 ReasoningCandidate  -------------------+
  claim / edge                            |
                                         v
 HarnessInput --------------------> materialization
 task / evidence                          |
 requirement / policy                     v
                                   validation + passes
                                         |
              +--------------------------+-------------------------+
              |                          |                         |
              v                          v                         v
      verification receipt          diagnostics              artifact state
              |                          |                         |
              +--------------------------+-------------------------+
                                         |
                                         v
                                acceptance policy
                                         |
                               accept / reject / unknown
```

## `reason run`は2モード。でも検証部分は同じ

### 外部candidateを渡す: Harness内AI不要

```bash
reason run \
  --input evidence.json \
  --candidate model-candidate.json \
  --no-config \
  --format json
```

`model-candidate.json`は、自作アプリのLLM呼び出し、RAG、別Agent、ChatGPT/Claude/Codex的な外部システム、recorded output、あるいは決定論的なコードから作っても構いません。

`ReasoningCandidate` JSONになった後のこのpathでは、AI providerは必須ではありません。

### live providerを使う: AIがcandidateを生成

```bash
reason run \
  --input evidence.json \
  --provider mistral \
  --model ministral-8b-latest \
  --format json
```

この場合だけ、`reason`がproviderへ問い合わせてcandidateを生成します。ただし生成されたcandidateは、その直後に**外部candidateと同じuntrustedな検証path**へ入ります。

高性能モデルを使えばcandidate品質は上がり得ますが、そのモデルにverification authorityが付与されるわけではありません。

## Step 1: candidateをmaterializeするときに「自己認証」を消す

`ReasoningCandidate`はuntrusted producerが提案したsyntaxであり、確定済みのreasoning stateではありません。

現在のmaterializerは、candidateのstateを次のように保守的に扱います。

| Candidateの`proposed_state` | 最初のartifact state |
| --- | --- |
| `unknown` | `unknown` |
| `assumed` | `assumed` |
| `known` | `assumed` |
| `supported` | `assumed` |
| `inferred` | `assumed` |
| `contradicted` | `assumed` |

つまりAIが`"proposed_state": "supported"`と書いても、それだけではsupportedになりません。強いstateはHarness boundary内で再確立する必要があります。

同時に、duplicate claim IDや存在しないclaimを参照するinferenceなどもnormalizeされ、`candidate_diagnostics`へ記録されます。壊れた参照をそのまま信頼することはありません。

## Step 2: 各passの後にartifactを再validationする

`run_harness`はmaterialize直後のartifactをvalidationします。さらに**各passの実行後にも毎回`ReasoningArtifact`を再validation**します。

passが不正なstateを作れば、そのまま続行せずfail-closedのHarness errorになります。

現在の`reason run` product pathには主に次が入っています。

1. structured adversarial discovery
2. evidence qualification
3. structured-fact verification
4. 明示的に渡されたtrusted verification receiptの適用
5. Five Whys restatement check
6. assumption discovery

diagnosticsは調査に使えますが、diagnostic codeが勝手に最終verdict authorityを持つわけではありません。

## Step 3: structured evidenceならAIなしでhard verificationできる

たとえばcandidateに次のtyped propositionがあるとします。

```json
{
  "key": "service.region",
  "value": "us-east-1"
}
```

Harness管理のevidenceに次のstructured factがあるとします。

```json
{
  "id": "e1",
  "facts": {
    "service.region": "us-east-1"
  }
}
```

built-in structured verifierは、これを**文字通り決定論的にkey/value照合**できます。

### 値が一致

対象keyについて観測された値がcandidateのpropositionと一致すれば、Harness側のverifierが`VerificationReceipt`を`supported` conclusionで作ります。

receipt適用後、claim stateは`supported`へ進み、どのevidenceで確認したかもreceiptへbindされます。

### 別の値が確認された

qualification requirementがないcompatibility pathでは、同じkeyに別valueが観測されると`contradicted` conclusionのreceiptを作れます。claim stateも`contradicted`になり得ます。

### 根拠が存在しない

対象factがなければhard receiptは作りません。AIのclaimを雰囲気で`supported`にはせず、不確実性を残します。

### evidence qualificationがある場合

時間、scope、provenance、authorityなどのrequirementが設定されている場合は、qualification-aware verifierがその条件を満たすevidenceだけをhard verificationへ使います。

条件不足ならhard receiptを出さず、uncertaintyを維持します。qualified evidence同士で値が競合する場合も、曖昧なデータから無理にhard conclusionを作らずreceiptを保留します。

## Step 4: 外部のtrusted verifierもAIとは別の権限として接続できる

`reason run --receipts ...`では、trusted external verifierがすでに作った`VerificationReceipt`を明示的に渡すcompatibility pathもあります。

より強いintegrationでは、次のようなdeterministic/domain-authoritative systemをtyped verifier adapterにできます。

- test runner
- compiler / schema validator
- database / query result
- policy engine
- trusted human review
- domain-specific oracle

重要なのは、**claimを作った同じuntrusted AIを、そのclaimのhard verifierとして扱わないこと**です。

## Step 5: diagnosticは「問題を見つける」が「証拠を作る」わけではない

product pipelineはcontradiction/counterexample、evidence qualification、assumption、reasoning structureなどのsignalを記録できます。

ただしdiagnosticはtrusted verificationと分離されています。diagnosticが「怪しい」と言っただけでtrusted evidenceやverification receiptが発生するわけではありません。

D3 semantic runtimeはさらに明示的に別surfaceです。`reason semantic-check`として提供され、soft decisionを維持するか`abstain`へ保守化できますが、trusted evidence、hard receipt、epistemic promotion、final verdict authorityは作れません。

## Step 6: 最後にacceptance policyが全体verdictを決める

現在の`StrictAcceptancePolicy`は、materialization、validation、各passを通過したartifactに対して最後に実行されます。

現在の集約ルールは保守的です。

```text
claimが0件
   -> unknown

1つでも contradicted
   -> reject

それ以外で assumed / unknown が1つでも残る
   -> unknown

それ以外
   -> accept
```

そのため`unknown`はエラーではありません。「今ある根拠だけでは断言できない」と正常に判定できた場合、CLI process自体はexit code `0`で完了し、利用者は`result.outcome.verdict`を確認します。

一方、JSON不正、config不正、provider unavailable、timeout、invalid harness stateなどはepistemic resultではなく**operational failure**なのでnon-zeroになります。

## AIなしrunで分かること / 分からないこと

`--candidate`のAIなしrunは、truth conditionをtyped evidence、deterministic verifier、explicit assumption、conservative policyへ落とし込める領域で強いです。

逆に、自由文だけを見て何でも意味理解できるわけではありません。たとえば「この設計はたぶんレジリエントです」というclaimについて、それを確立できるtyped proposition / verifier / evidence relationがなければ、Harnessは勝手に意味を補完せずuncertaintyを残すのが正しい動きです。

semanticなsoft診断が必要なら`reason semantic-check`を使えますが、それはhard authorityとは分離されています。

## 実アプリへ組み込むときの形

多くの場合、raw dataからHarness contractへの変換はアプリ側が持ちます。

```text
raw document / API data / test output
                 |
                 v
          application integration
          |                  |
          v                  v
     HarnessInput       model prompt/schema
   Harness管理evidence         |
                              v
                     ReasoningCandidate
                              |
                  +-----------+
                  v
              reason run
                  |
        +---------+---------+
        |         |         |
      accept    reject    unknown
        |         |         |
     続行       block     再取得/review
```

RAGのretrieval結果も、取得しただけでtrustedにはなりません。source/provenanceをどうevidenceへ表現するか、qualificationを要求するか、外部trusted verifierを使うかをintegration側で明示します。

## 関連ドキュメント

- [日本語CLIガイド](cli.ja.md)
- [Architecture (English)](architecture.md)
- [Evidence qualification (English)](evidence-qualification.md)
- [Grounded resolution (English)](grounded-resolution.md)
- [ADR-0001: interface and packaging boundaries](adr/0001-interface-and-packaging-boundaries.md)
- [ADR-0002: grounded resolution and finalization](adr/0002-grounded-resolution-and-finalization.md)
