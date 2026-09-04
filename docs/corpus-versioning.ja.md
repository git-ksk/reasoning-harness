# バージョン管理されたベンチマークコーパス

Reasoning Harness では、ベンチマークの構成を測定契約の一部として扱います。評価対象ケースの識別子、スコアリングの意味、コーパス互換性の境界が明示されて初めて、スコアは意味を持ちます。

## Corpus v1

`fixtures/corpus/v1.json` は、コーパスバージョン `1.0.0` とスコア互換性 ID `corpus-v1` の正規マニフェストです。

マニフェストには **41 active deterministic cases** が含まれます。

- 20 claim/verdict cases;
- 8 causal-diagnostic cases;
- 5 assumption-diagnostic cases;
- 8 evidence-qualification cases.

`fixtures/metamorphic/` のメタモルフィック・フィクスチャは変換用の対照であり、スコア対象のコーパスケースではないため、corpus v1 のメンバーではありません。

各ケースには、suite-prefixed `case_id`、元の fixture ID とパス、能力/診断カテゴリ、根拠付きの難易度層、scoring mode、provenance、redistribution status、contamination note、lifecycle status が含まれます。

Suite-prefixed ID は必須です。スイートごとの fixture-local ID は正当に重複し得るためです。たとえば claim、causal、evidence-qualification のケースは、同様のローカル用語を使っていても同一のベンチマークケースにはなりません。

## スコアの互換性

`corpus_version` は公開済みマニフェストのリビジョンを識別します。`score_compatibility_id` は集計スコアを直接比較できるかどうかを識別します。

2つの実行結果を直接比較できるのは、マニフェストの `score_compatibility_id` が同じで、かつ報告するメトリクスについて評価された active case set と scoring contract が同じ場合に限られます。ランタイムはバージョン文字列の順序から互換性を推測せず、この ID を公開します。

メタデータだけの訂正であれば、ケースの所属、fixture の意味、期待ラベル、スコアリングロジック、または報告メトリクスで使う層を変更しない限り、互換性 ID を維持したままコーパスの patch version を上げられます。active membership、期待結果、semantic fixture content、scoring mode、集計に関係する category/difficulty assignment のいずれかを変更する場合は、全コーパス比較のために新しい compatibility ID が必要です。

互換性のないコーパスバージョンを比較する必要がある場合、レポートは次のいずれかにしなければなりません。

- 変更されていない scoring semantics に基づく、明示的に特定した stable-case intersection だけを比較する。
- バージョンを別々の測定結果として提示する。

新しいバージョンが過去のスコアを暗黙に上書きしてはなりません。

## 変更の規律

安定した `case_id` を、意味の異なるケースに再利用してはなりません。

ケースを追加するときは、新しいマニフェストバージョンを作成し、新しい安定 ID を割り当てます。測定対象が変わる形でケースを変更するときは、新しい case ID を作成するか、旧エントリを `superseded` とし、置換先を `superseded_by` で指します。deprecated または superseded のエントリも履歴上のマニフェスト記録に残します。旧マニフェストは、ベンチマークの意味を変えない、明示的に文書化されたリポジトリ整合性修復を除き、公開後は不変です。

active case の削除、期待ラベルの変更、deterministic oracle semantics の変更、score-reported strata 間の移動は全コーパス測定を変更するため、新しい compatibility ID が必要です。

正確な意味内容を保つ fixture-path の移動では case ID を維持できますが、マニフェストのリビジョンに新しいパスを記録し、すべての active manifest entry が matching fixture ID を持つ fixture に解決されることをテストで証明しなければなりません。

## カテゴリと難易度の報告

記録された claim evaluation のレポートは、既存の `BenchmarkComparison` 全体を変更せず、そのマニフェストから category と difficulty のスライスを追加します。各スライスは同じベンチマーク集計ロジックを再利用し、2つ目の correctness 実装を定義しません。

Live の repeated-trial output は再現性のため `corpus_version` と `score_compatibility_id` を記録しますが、部分的または反復された観測から pooled category/difficulty accuracy を合成しません。反復 correctness は引き続き `stability.correctness` の既存の complete-trial semantics を使います。

現在の難易度名は `basic`、`standard`、`stress` です。これはベンチマークの層であり、すべてのモデルに対する普遍的な task difficulty の主張ではありません。根拠はケースごとに保存され、将来の変更を明示的にレビューできるようになっています。

## 汚染と再配布に関する方針

Corpus v1 はリポジトリが作成した合成素材で、redistributable とマークされています。公開後に public model がこれらの fixture に一度も遭遇していないことをプロジェクトは証明できないため、マニフェストには完全な decontamination を主張せず、その制限を記録します。

プロジェクトは proprietary training corpora をスクレイピングしたり、secret training membership を推測したりしません。将来取り込むベンチマークケースは、採用前に provenance と redistribution status を記録しなければなりません。restricted material を redistributable であるかのように export してはなりません。

反復された live results は、バージョン管理されたコーパス上のモデルに関する観測であり、コーパスが汚染されていないことの証拠ではありません。

## 飽和警告ポリシー

deterministic recorded fixture suite が 100% に達しても、それは回帰結果であって、model benchmark が飽和した証拠ではありません。

claim stratum は、少なくとも3つの独立した model family がそれぞれ、変更されていないその層で少なくとも5回の operationally complete live trials を通じて 95%以上の harness accuracy に達した場合に限り、**saturation candidate** になります。それでも、その層を saturated と呼ぶ前に unsafe accepts とクラス別の失敗モードを確認しなければなりません。

層が saturation candidate になったとき、スコアを下げるためだけに旧ケースを変更してはなりません。旧コーパスを保持し、より難しい、または識別力の高いケースを新しいコーパスバージョンに追加します。active membership または scoring meaning が変わる場合は、新しい score-compatibility ID を割り当てます。過去の結果は元の契約の範囲内で引き続き比較できます。

## Resolution-loop ベースラインの識別情報

将来の bounded-resolution research では、direct-generation、diagnose-only、resolution variants の間で同じ stable base `case_id` を再利用しなければなりません。resolution outcomes と costs はその base identity に対する追加の観測であり、以前は未知だったケースを解決できても、元の denominator を削除・置換してはなりません。

このルールは、resolution が grounded answerability を高めるか、unsafe final answers を増やさずに測定するための前提条件です。
