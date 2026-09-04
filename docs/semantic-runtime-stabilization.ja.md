# セマンティックランタイムの安定化

Issue #73 は、意味契約、キャリブレーション用フィクスチャ、holdout-v5 のラベル、閾値、プロバイダー固有の挙動を変更せず、凍結済みの `semantic-decidability-d3-v1` 候補を実行時利用に耐える形へ強化する。

## 凍結された実行時識別子

コアは、モデル出力やプロバイダーメタデータから実行時識別子を受け取るのではなく、自身で所有する。安定化 API は次の識別子を凍結する。

| 座標 | 識別子 |
| --- | --- |
| 特性評価済みロールバック基準 | `soft-semantic-v3` |
| R2 モデル向け materialization 契約 | `materialization-r2-v1` |
| 決定論的 decidability 契約 | `deterministic-explicit-typed-preconditions-v1` |
| D3 実行時候補 | `semantic-decidability-d3-v1` |
| 識別子スキーマ | `semantic-runtime-identity-v1` |

`SemanticRuntimeProfile::SemanticDecidabilityD3V1` は、基準、R2 契約、決定論的ゲート、ロールバック設定をまとめて記録する。安定化 PR の期間中は `SemanticRuntimeProfile::SoftSemanticV3` がコンパイル時のデフォルトとして残る。その変更が CI を通過した後、別途レビュー可能な実行時採用 PR でデフォルトを D3 に切り替えた。特性評価済み v3 プロファイルは明示的なロールバック選択肢として残る。

`run_semantic_runtime` は、プロバイダーに依存しない単一 API の背後で両プロファイルを提供する。D3 ブランチは変更されていない R2 semantic materialization を実行し、Harness が所有する型付き decidability ゲートを評価し、基底判定を維持するか `abstain` を強制することだけができる。壊れたプロバイダー出力の修復、abstention の昇格、信頼済み証拠の生成、判定権限の変更はできない。

## R2/D3 能力の事前確認

`reason-semantic-preflight` は、すべてのキャリブレーションおよび holdout corpus から独立した、プロトコルのみの合成リクエストを上限付きで実行する。デフォルトは3 probeであり、全 probe の成功が全体の `compatible` 結果に必要となるため、幸運な1レスポンスで断続的な R2 materialization 失敗を隠すことはできない。preflight の成功が意味するのは、プロバイダー/モデルが `materialization-r2-v1` に受理される payload を返したことだけである。観測された semantic decision は報告するが、スコアリングは決して行わない。

```text
cargo run -q --locked -p reasoning-harness-cli --bin reason-semantic-preflight -- \
  --provider mistral \
  --model ministral-8b-latest \
  --probes 3
```

出力は次を分離する。

- `compatible`: 要求した R2 decision-only protocol probe がすべて正常に parse された。
- `incompatible`: model-owned materialization field の禁止など、protocol/capability failure が発生した。
- `operationally_incomplete`: credential、quota、rate limit、timeout、transport、provider availability、truncation、その他の operational failure により capability を結論づけられなかった。

手動の `semantic-d3-capability-preflight` workflow も同じ probe を公開する。これは過去の凍結済み D2 および holdout-v5 workflow とは意図的に分離されており、互換性確認によって採用研究の provider-call plan が書き換えられることはない。

## 型付き運用テレメトリ

R2 materialization failure は、CLI ローカルの文字列ではなく `MaterializationFailureClass` を使うようになった。serialized class は setup、credentials、transport、provider error、rate limit、quota、provider unavailable、timeout、provider protocol、unsupported capability、materialization protocol、truncation protocol、provider generation error を区別する。

`reason-decidability-study` はケースごとの型付き failure を保持し、`failure_counts` も出力する。provider または protocol の failure は operational evidence のままであり、`finding`、`no_finding`、`abstain` の observation には決してならない。

## 部分結果の保持

長時間の decidability study は `--checkpoint <path>` により、進捗を atomic に保持できる。checkpoint は各 provider call の直前と直後に、同一ディレクトリの一時ファイルと rename を使って直ちに書き換える。そこには不変の study/candidate identity、provider/model、started/completed/successful/failed count、call 中であれば現在の fixture/trial/seed、型付き case failure、すでに観測した usage/latency、完了済み全ケースが含まれる。

Checkpoint の semantic status は明示的である。

- `partial_do_not_score`: 実行がまだ進行中。
- `operationally_incomplete_do_not_score`: provider call の failure を伴って実行が正常終了。
- `full_study_complete`: 期待された provider call がすべて成功裏に完了。

これにより、部分的に完了した model run の後の timeout のような evidence を保持しつつ、partial row が凍結済み study の semantic denominator に紛れ込むことを防ぐ。既存の complete-trial metric rule は変更しない。

## 採用とロールバック

安定化 PR は CI gate が通るまでデフォルトを意図的に変更しなかった。続く runtime adoption は別の可逆な変更として行われ、現在は `DEFAULT_SEMANTIC_RUNTIME_PROFILE` が `SemanticDecidabilityD3V1` を選択するため、`run_default_semantic_runtime` は R2 materialization と決定論的 D3 gate を実行する。`SemanticRuntimeProfile::SoftSemanticV3` はロールバック用に直接選択でき、低レベルの v3 semantic-judge API は書き換えも削除もされていない。provider-specific な semantic branch は許可されない。

したがってロールバックに fixture、prompt、threshold、記録済み research の変更は不要である。caller は `SoftSemanticV3` を明示的に選択するか、1つの isolated runtime change で default constant をその profile に戻せる。

観測済み holdout-v4/v5 corpus は不変の research history のままである。stabilization も adoption も、その内容を prompt tuning、relabelling、threshold selection、calibration に使ってはならない。

## ライブ運用スモーク

採用済み runtime には、D2 および holdout-v5 research corpus の外側に、意図的に分離された synthetic smoke surface がある。`reason-semantic-runtime-smoke` は選択した provider/model に対して、上限付きの2つの operational case を実行する。

- 明確な counterexample を持つ決定論的な `permit` case。D3 は R2 の base decision を維持しなければならない。
- 同じ model-visible semantic context を保ちながら Harness-owned proposition binding を欠落させた、対応する決定論的 `force_abstain` case。D3 は R2 の base decision に関係なく `abstain` を返さなければならない。

同じ case は明示的な `soft-semantic-v3` rollback profile でも実行し、引き続き operational に実行可能であることを要求する。fixture disposition は provider initialization 前に再計算し、live failure は既存の型付き operational class で報告する。この smoke surface は semantic calibration ではなく、correctness を score せず、凍結 holdout を消費も変更もしない。

Actions run `33408032079` の強化済み live smoke は、`mistral/ministral-8b-latest` と `google/gemma-4-31b-it` の双方で成功した。各 provider は operational failure なしに 4/4 call を完了した。双方とも、対応する clear-counterexample pair で base `finding` を生成した。D3 は決定論的な `permit` case では `finding` を維持し、Harness-owned proposition binding だけを削除した場合は `finding -> abstain` を強制した。明示的な `soft-semantic-v3` rollback も実行可能なままで assertive な `finding` を維持した。この2-case smoke の latency/token 値は記述目的に限られ、performance benchmark ではない。
