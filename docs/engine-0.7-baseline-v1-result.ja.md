# Engine 0.7.0 開発用ベースラインv1 — 公開済みEngine 0.6.1の残存ギャップ測定

日本語 | [English](engine-0.7-baseline-v1-result.md)

**判定：事前固定した決定論的な開発用baseline 29/29 PASS。Engine 0.7.0の実装・独立評価・release合格ではありません。**

追跡：[Issue #487](https://github.com/git-ksk/reasoning-harness/issues/487)、[親Issue #486](https://github.com/git-ksk/reasoning-harness/issues/486)。

## 凍結した測定座標

- 公開済みEngine：\`engine-v0.6.1\`、commit \`4abee90f0501f0d8fc09b453a0b45c942fbed69e\`。
- 基準仕様の事前freeze：\`engine-0.7-baseline-v1-spec-freeze\`、commit \`d874bd3a98618a1d61ed3de50e9376d79d07dfa8\`。実測前に29ケース、予測出力、gap分類を固定。
- fixture SHA-256：\`3bf35129ab786e7b069c6b8805b92c42a9e1b9cd10e3752c0b60f8f7fe66b268\`。
- 入力：\`fixtures/engine-0.7-baseline-v1/manifest.json\`。
- Runner：\`crates/reasoning-harness-core/tests/engine_0_7_dev_baseline_v1.rs\`。
- 検査済み結果：\`evaluation/engine-0.7-baseline-v1-result.json\`。
- 検証スクリプト：\`scripts/engine-0.7-baseline-v1-report.py\`。
- \`engine-v0.6.1\`とのCore本体・Cargo.toml差分はなし。Engineの実装・contract・provider・release座標・過去のfreezeは変更していません。

再現コマンド：

\`\`\`bash
cargo test --locked -p reasoning-harness-core --test engine_0_7_dev_baseline_v1 \
  -- --test-threads=1 --nocapture > /tmp/engine-0.7-baseline-v1.log 2>&1
python3 scripts/engine-0.7-baseline-v1-report.py \
  --log /tmp/engine-0.7-baseline-v1.log \
  --output /tmp/engine-0.7-baseline-v1-result.json
\`\`\`

## 観測結果

| 対象 | 予測済み0.6.1動作と一致 |
| --- | ---: |
| Source-localな出典・文章の衝突 | **10/10** |
| 構造化根拠・時点・scope・hard検証 | **11/11** |
| 根拠の再利用・追加取得要否 | **8/8** |
| **合計** | **29/29** |

別テストで引用元・根拠の入力順序を入れ替える負のコントロールも**PASS**。source-onlyパスからhard verification receiptは**0**。モデル呼出、provider試行、外部取得、provider障害はすべて**0**（そもそもAPIを呼ばない評価）。この結果から現実の質問に対する回答率、token消費、latency、汎用的な安全性までは評価できません。

開発用29件のうち**24件は改善候補を持たない既存安全性／負のコントロール**、5件は事前登録したgap仮説です。仮説件数をそのまま不具合件数とは数えません。

## 既存コードと改善候補の照合

| Issue | 実測内容 | 判定 |
| --- | --- | --- |
| **#488 出典の独立性** | \`source_attribution.rs\`のbindingにはsource ID、URL、取得時刻、版はあるが、根拠の由来・独立性の契約はない。転載・不明・同一発行元の\`src-02/03/10\`では2引用を保持するが、テスト用の起源oracleはEngineに渡していない。 | **表現できない範囲を確認**。2引用は2件の独立した検証という意味ではないため、unsafeな昇格の証拠ではない。実際の利用要件が必要。 |
| **#489 時点・改訂** | \`types.rs\`と\`evidence_qualification.rs\`は有効期間・as-ofによる失効／将来時点の判定を実装済み。\`src-06\`の異なる版の記述は衝突を維持、\`src-07\`も安全。 | **既存の時点判定は十分機能**。版文字列と共通originだけでは正式な改訂関係が証明できないため、新版優先はしない。実装着手は保留。 |
| **#490 意味の近い文章の統合** | \`source_attribution.rs\`の\`refresh_conflict_states\`は同じtargetで文章文字列が異なるとConflict。\`src-04\`では「beta中」と「beta段階にある」の言い換えが**Conflict**になり、元の2引用は両方保持。\`src-05\`の真の対立もConflictとして保持。 | **1件の過剰に保守的な衝突表示を実測**。外部事実の誤検証ではないが、有用性の改善余地あり。**最優先**。 |
| **#491 回答充足性** | \`evidence_need.rs\`、\`answer_safety.rs\`、\`semantic_sufficiency.rs\`の既存制御あり。\`need-01..08\`は古い情報の再取得、検証の優先度、適切な文脈利用を正しく分類。 | **新たな完全タスクの失敗例は未実測**。追加の回答制御機構は先送り。 |
| **既存の検証** | \`evidence_qualification.rs\`・\`verification.rs\`の11ケースは、失効、将来時点、誤scope、権威不足、矛盾時のhard receipt保留を正しく実行。 | **実装済み。重複開発しない**。 |

**注意：** \`src-04\`は「Conflictという表示が過剰に保守的」という結果であり、誤って\`Known\`や\`Supported\`を生成したわけではありません。\`src-06\`には信頼できる親子改訂関係の証明を入力していないため、実際の新版が優先されるべきだったという結論は出せません。

## 次の候補の数値合格条件（baseline観測後・独立holdout作成前に固定）

1. **全候補共通:** 根拠なしの\`Known\`/\`Supported\`、出典・引用・独立ソースの捏造、別target/scopeへの混線、無根拠な最新版優先、真の衝突の隠蔽、replayによる外部副作用は**すべて0件**。改善候補でない安全な既存24ケースは**24/24**の回帰なしを原則とします。
2. **#490（第一候補）:** 開発用\`src-04\`の不要なConflictを**1/1から0/1**に改善し、実際に対立する\`src-05\`は**1/1 Conflictを維持**。各ケースの引用**2/2**、hard receiptは**0**。後続の独立holdoutではMistral・Google・Groqの各モデルで、同じ入力の0.6.1より根拠付き有用回答を**最低1件純増**させ、安全性の悪化は**0件**。
3. **#488（条件付き）:** 利用上の独立裏付け要件を確認できた場合のみ、\`src-01/02/03/10\`相当の独立／同一起源／由来不明判定を**4/4**。認証されない独立性の創作**0**、出典証明のためのモデル追加呼出**0**。
4. **#489（条件付き）:** 信頼できる改訂の親子関係を持つ**別途freezeした開発用ケース**を作成してから実装判断。現行\`src-06\`の版文字列だけでは証明になりません。根拠なしの最新版採用**0**。
5. **#491（条件付き）:** 完全な質問・必要情報の不足による具体的な失敗を再現できてから設計。既存のneed制御は**8/8を維持**し、budget越え、replayの再取得、暗黙のprovider切替は**0**。
6. **効率:** 出典系譜と改訂の決定的判断は追加model呼出**0**。将来の意味論的照合は同じtargetで0.6.1と比較して追加advisory呼出**最大1回**、その追加呼出のprovider試行**最大2回**、session replay時は**0回**。token・latency・quotaは今後の同条件・開発用live観測で数値化し、存在しない費用基準を捏造しません。
7. **独立acceptance:** candidateとrunnerを先に固定し、新規の別対象・引用・文面からholdoutを作成。Mistral・Google・Groqで安全性違反**0**、既存負のコントロール悪化**0**、採用する機構ごとに新規ケースで各モデル最低**1件の純改善**を要求します。provider障害と意味的失敗は分離し、初回FAILは変更しません。

## 次に着手する内容

**#490を限定的に先行**し、意味が一致する言い換えと実際の否定／別target／時点違いを識別できるかを開発用ケースで測ります。ただしモデルのsemantic判定を出典や検証の権威にはできず、判定が曖昧なときは既存の出典別表示に安全に戻します。

#488は具体的な独立裏付けの利用要件を確認してから、#489は認証済み改訂関係を持つ実例ができてから、#491は完全タスクの欠落を確認してから着手します。**Engine 0.7.0のバージョンアップ／release／CLI採用は一切していません。**
