# ADR-0002: 根拠に基づく解決と最終化のループ

- Status: Accepted
- Date: 2026-08-30

## 実装状況

この ADR が記述するプロバイダー中立の有界制御ループは、Issue #22 の時点で core に実装済みである。型付きリクエスト、resolver/admission/trusted-verifier の分離された境界、有界な再検証、最終主張の事実に基づくカバレッジが利用できる。具体的な open-world resolver 統合は core の範囲外であり、この ADR のステータスから実装済みとはみなされない。

## コンテキスト

ADR-0001 では、ネイティブの harness runtime が確率的モデルを取り巻く実行プロトコルを所有することを定めた。その後、実装には強固な検証・診断境界が整備された。すなわち、harness が所有するエビデンス、信頼された検証レシート、型付きの矛盾・反例の検出結果、エビデンスを考慮した因果診断、仮定診断、メタモルフィック頑健性、反復試行の安定性レポートである。

これらの機能により、harness は、根拠のない・矛盾した、または未解決の中間推論を特定するうえで有効になっている。しかし、未解決の検出結果から根拠に基づく最終回答へ至る、完全なプロダクトループはまだ定義されていない。

`accept | reject | unknown` を実行の終点として扱うと、このプロジェクトは主として評価器または事後診断ツールになるおそれがある。意図するプロダクトの方向性は、より広いものである。検証済みの中間状態を使って、確率的モデルが最終回答へ進んでよいかを制御し、根拠が不十分な場合には追加エビデンスを要求し、最終レンダリングが根拠のない主張を黙って追加しないことを保証する。

## 決定

Reasoning Harness は **evidence-grounded reasoning runtime** へ向けて進化する。

runtime は、タスクと harness が所有するエビデンスを、検証済みの推論状態へ変換し、ポリシーが許す場合には根拠に基づく最終回答へ変換するプロトコルを所有する。モデルは交換可能な候補生成器および任意の renderer にとどまる。エビデンス、検証、最終的な認識論的ステータスの権威になることはない。

概念上の目標ループは次のとおりである。

```text
task + harness-owned evidence
          |
          v
candidate generation
          |
          v
ground + verify + diagnose
          |
          +--> supported enough --------------------+
          |                                         |
          +--> unresolved / insufficient support    |
          |          |                              |
          |          v                              |
          |     resolution request                  |
          |          |                              |
          |     external evidence / verifier        |
          |          |                              |
          |          v                              |
          |     regenerate or revise                |
          |          |                              |
          |          +----> re-run harness ---------+
          |
          +--> refuted --> discard/revise --> re-run
                                                    |
                                                    v
                                               finalization
                                                    |
                                                    v
                                          grounded final answer
```

`unknown` は引き続き有効な認識論的結果である。解決予算を使い切った場合、信頼できる resolver がない場合、またはポリシーが追加試行を禁じている場合、runtime は `unknown` で停止するか abstain できる。

## 解決境界

診断による検出結果は、それ自体が任意の情報を取得したり、新しいモデルの主張を信頼したりする許可ではない。runtime は未解決状態を、何が不足しているかを示しつつ答えを知っているふりをしない、型付きの resolution request に変換すべきである。

将来のプロバイダー中立な resolution contract は、次のようなリクエストを表現できる必要がある。

- 命題を裏付ける信頼されたエビデンスを取得する。
- 因果関係を裏付ける、または反証するエビデンスを取得する。
- 時間、スコープ、または provenance の不一致を解消する。
- 外部の決定論的 oracle で命題を検証する。
- 強い反証を受けた推論を修正または再生成する。
- ポリシーが許す場合に、明示的な人手レビューを要求する。

core runtime は、リクエスト、予算、状態遷移、再検証のセマンティクスを所有する。retrieval system、web search、database、MCP server、compiler、test runner、人手レビューシステム、ドメイン固有ツールは、明示的に信頼された evidence/verifier 境界を通じてデータを返さない限り、trusted core の外側にある adapter のままである。

## エビデンス取得は権威ではない

resolver がデータを返したからといって、そのデータが自動的に信頼されるわけではない。

- Retrieval は取得メカニズムであり、verifier ではない。
- モデルが生成した引用は、出典を示しているだけでは信頼されたエビデンスではない。
- 候補が記述した provenance によって権威性が高まることはない。
- resolver は、harness が所有する入力または verifier policy によって定義された provenance と authority metadata を伴うエビデンスを返さなければならない。
- エビデンスが不足している場合は `unknown` のままであり、runtime は resolution request を満たすために完了を捏造してはならない。

これにより、既存の verification receipt と決定論的診断で用いているものと同じ権威境界が維持される。

## 修正と再生成の境界

runtime は診断後にモデルへ候補の修正または再生成を依頼できるが、新しい候補は untrusted の状態から始まる。

修正ループは過去の検出結果をガイダンスとして利用できるが、次のことはできない。

- soft finding を hard truth に昇格させる。
- モデルが主張を繰り返しただけで、以前に棄却された主張を検証済みとして保持する。
- モデルが信頼されたエビデンスや receipt を作成できるようにする。
- 修正後の validator、policy、再検証を迂回する。

修正された候補はすべて、元の候補と同じ normalization、validation、verification、diagnostic、policy の境界を通過する。

## 最終化境界

最終回答の生成は、推論の検証とは別のフェーズである。

目標とする finalization contract は次のとおりである。

```text
verified ReasoningArtifact
          |
          v
answer renderer / optional model
          |
          v
claim coverage check
          |
          v
grounded final answer | unknown/abstain
```

renderer は要約、順序変更、簡略化、またはスタイルへの適応を行ってよいが、認識論的ステータスを引き上げたり、根拠のない事実主張を導入したりしてはならない。

runtime は最終的に、レンダリングされた回答に含まれる事実命題が、supported artifact propositions によってカバーされているか、またはポリシーに従って仮定・不確実性として明示されているかを検証すべきである。renderer が新しい事実命題を導入した場合、その命題は、最終回答に黙って追加するのではなく、通常の推論・検証ループへ戻さなければならない。

## ポリシーと終了

解決には上限がある。runtime は次のような明示的な制限を所有する。

- 最大 resolution attempt 数。
- model/token/time budget。
- 許可された resolver class。
- 必須の authority level。
- 人手レビューを許可するかどうか。
- 未解決の主張によって abstention を強制するか、条件付き回答を許可するか。

予算の枯渇はエビデンスではない。ポリシーの範囲内で十分な根拠を取得できない場合、正しい結果は、明示的なポリシーに従った `unknown`、条件付きの部分回答、または abstention である。

## 研究上の要件

解決ループは、生の診断精度とは分けて評価しなければならない。

重要な測定項目には次が含まれる。

- answerable-case recovery: 当初は未解決だったケースが、解決後にどの程度 supported になるか。
- unsafe-final-answer rate: 根拠がない、または矛盾する事実主張が最終出力に到達する割合。
- evidence acquisition efficiency: 回復した 1 ケースあたりに追加で必要となる call/token/latency。
- resolution convergence: supported、refuted、または exhausted に至るまでに必要な試行回数。
- direct generation および diagnose-only baseline に対する regression。
- finalization coverage: 最終的な事実主張のうち、supported artifact proposition に結び付いている割合。

回答可能性の改善は、安全でない最終回答が増えない場合にのみ有用である。

## 結果

Positive:

- プロジェクトが benchmark や事後判定器以上のものとして存続する。
- 既存の診断が runtime loop 内の実行可能な制御シグナルになる。
- core にドメインロジックを移すことなく、ドメイン固有の retrieval を統合できる。
- 検証済みの推論状態が最終回答構築の source of truth になる。
- より小さく安価なモデルについて、周辺プロトコルが根拠に基づく回答を安全に回復できるかを評価できる。

Costs:

- runtime に明示的な解決状態、予算、終了セマンティクスが必要になる。
- finalization には prose-only rendering ではなく、命題のカバレッジが必要になる。
- 回復ループによってモデルやツールの call が増えるため、live research のコストが高くなる。
- プロダクトの主張では、diagnose-only capability と、実装済みの end-to-end grounded resolution を区別しなければならない。

## 非目標

- 任意の open-world claim を数学的に証明済みにすること。
- 一般的な web crawler や RAG system を core runtime に組み込むこと。
- LLM judge、retriever、または renderer を correctness authority として信頼すること。
- すべての `unknown` case を解決させること。
- 回答率を最大化するために不確実性を隠すこと。
- harness を汎用 agent framework に変えること。

## ADR-0001 との関係

ADR-0001 は interface と packaging の境界について引き続き権威を持つ。この ADR は、その境界の内側で native runtime が最終的に所有すべきものを明確にする。すなわち、diagnosis と `accept | reject | unknown` だけでなく、解決、再検証、根拠に基づく最終化のための有界プロトコルも所有する。
