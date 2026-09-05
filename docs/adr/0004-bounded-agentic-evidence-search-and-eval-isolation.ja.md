# ADR 0004: 境界付きエージェント型エビデンス探索と評価分離

Status: experimental candidate

## 背景

外部エビデンス実験では、型付きMCP取得の上に、plannerが検索クエリを再構成できる探索ループを追加する。ただし、エビデンスの採用・十分性・真偽・最終停止を決める権限はHarnessに残す。

この構成には主に2つのリスクがある。

1. エージェントループが止まらず、認識上の進捗がないまま予算を消費すること。
2. liveの公開API障害をplannerやsemantic品質の悪化として誤って計測すること。

現行実験では、round数、tool call数、planner call数、model token、latency、stop reason、operational failureを記録し、複数trialとdev/holdout分離も行っている。

## 参照した先行事例

- OpenAI Codexは、modelがtool callを要求し、harnessが実行結果を観測として戻して再推論するagent loopを採用している。長いループではcontext管理とorchestrationがharness側の責務になる。
  - https://openai.com/index/unrolling-the-codex-agent-loop/
- OpenAI Harness Engineeringでは、agentに単に「もっと頑張らせる」のではなく、強制可能な不変条件、agentから観測可能なfeedback loop、eval infrastructureを整える方針が示されている。
  - https://openai.com/index/harness-engineering/
- LangGraphは、loopに明示的なtermination conditionを要求し、それとは別にrecursion/super-step上限を持つ。残りstep数を使ったgraceful degradationも提供する。
  - https://docs.langchain.com/oss/python/langgraph/use-graph-api
  - https://docs.langchain.com/oss/python/langgraph/graph-api
- Microsoft AutoGenはterminationをruntime側のstateful conditionとして扱い、message数、token、timeout、external signal、functional conditionなどを合成できる。
  - https://microsoft.github.io/autogen/stable/user-guide/agentchat-user-guide/tutorial/termination.html
- Anthropicはagent evalで複数trial、trajectory、outcome、tool-call数、token、latencyを保持し、可能な範囲でdeterministic graderを使うことを推奨している。
  - https://www.anthropic.com/engineering/demystifying-evals-for-ai-agents
- Anthropicはagentic benchmarkについて、infrastructureの違いだけでも数ポイント単位でスコアが変動し得ると報告している。したがってinfra noiseをmodel capabilityへ混ぜない。
  - https://www.anthropic.com/engineering/infrastructure-noise

## 決定

### 1. Plannerは行動を提案し、Harnessがループを制御する

plannerはuntrustedとし、型付きの次検索actionまたはstop提案だけを返す。エビデンスの真偽・十分性・採用可否・最終回答は決定しない。

Harness/controllerがdeterministicな上限とstop semanticsを所有する。実験時の初期値は以下を維持する。

- max rounds: 6
- max external tool calls: 10
- wall-clock budget: 30秒
- model-token budget: 8,000
- no-progress cutoff: 2 rounds
- normalized duplicate-query rejection
- targetがsupported/refutedになった時点で即時停止
- budget exhaustionや未解決ambiguityは推測せずsafe unknown

plannerがstopを要求しても、それは探索継続に関する提案に過ぎず、最終epistemic statusはHarnessが決める。

### 2. 進捗判定はmodelの自己申告ではなく型付きstateで行う

no-progress判定は、candidate ID、resolved entity、corroborated title、property valueなどの型付きsearch stateの変化で行う。plannerの「進捗した」という自然言語は進捗根拠にしない。

### 3. 評価信号を分離する

以下を1つのscoreへ潰さない。

- **semantic safety**: 特にfalse acceptance / unsupported acceptance
- **agent capability**: bounded plannerがplanner failureやbudget failureなしに期待結果へ到達できるか
- **infrastructure health**: live依存先のtransport/protocol/provider failure
- **efficiency**: rounds、tool calls、planner calls、tokens、latency

live trialがoperational failureで失敗すること自体は許容しないが、それをsemantic regressionへ分類しない。

### 4. CIではlive依存クラスをfresh runnerへ分離する

MCP knowledge probeは以下を別GitHub Actions runnerで実行する。

- deterministic control contracts
- live Layer A/B adapter・fixed-policy checks
- live Layer C agentic planner dev suite
- natural-language MCP smoke
- frozen holdout（明示的workflow dispatchのみ）

各live laneが個別reportと明示gateを持つ。combined attribution reportは各laneのreportが生成された後にのみ作る。

### 5. deterministic controlをlive結果の解釈より優先する

transport retry/accountingとprogress-controlはfixtureベースで再現可能なcontract testを持つ。live Wikipedia/Wikidataは重要なdogfoodだが、loop correctnessの唯一の根拠にはしない。

### 6. Holdoutは初回観測後に凍結する

holdoutはdispatch-onlyとする。一度結果を観測した後、その結果を理由にcase、budget、planner prompt、stop rule、expected outcomeをretuneしない。追加調整する場合は新しい独立holdoutを用意する。

## 影響

利点:

- 無限ループ対策をmodelの従順さではなくruntime policyで保証できる。
- 外部API障害をsemantic failureと誤認しにくくなる。
- high-loadな他laneを再実行せず、失敗したlive laneだけを切り分けやすくなる。
- agent evalの先行事例に近いmetrics構成になる。
- Wikipedia/Wikidata固有ではなく、他の知識系MCPへ拡張できる。

コスト:

- CI job数とcheckout/build回数が増える。
- aggregate reportは少し複雑になる。
- 公開APIが不調なときはfull clean live run自体は失敗するが、失敗原因は明示される。

## 今回決めないこと

- このADRは新しいsemantic runtime世代（例: D4）を作るものではない。
- 任意のMCP自然言語出力をtrusted factへ昇格しない。
- live公開APIをそのままauthoritative frozen benchmark環境とはしない。
- acquisition-timeとsemantic/as-of-timeの分離問題は別のproduct issueとして扱う。実験で使っているfuture evaluation-time workaroundを本修正にはしない。
