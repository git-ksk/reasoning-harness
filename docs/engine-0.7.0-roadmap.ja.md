# Harness Engine 0.7.0 — 複数根拠の整合と回答充足性

日本語 | [English](engine-0.7.0-roadmap.md)

**状態: 計画策定済み。未実装・未リリース。** [マイルストーン #10](https://github.com/git-ksk/reasoning-harness/milestone/10) · [親Issue #486](https://github.com/git-ksk/reasoning-harness/issues/486)

これはReason CLIとは独立した**Harness Engineのsource release**計画です。現在公開済みのReason CLI 0.5.4 / Engine 0.6.1を変更したり、次のCLIバージョンやEngine 0.7.0の完成を意味したりするものではありません。

## 開発仮説

Engine 0.6.1は、個々の根拠のtarget関連性、時点・スコープ・権威の適合、検証、出典付き説明をすでに扱っています。一方、**1つの質問に対して複数根拠を突き合わせる**場面――同じ一次情報の転載、改訂版の時点差、部分的な矛盾、回答に必要な情報の一部欠落――では、まだ品質上の改善余地があるかもしれません。

目的は単なる回答数増加ではなく、**根拠のない結論を増やさず、有用な回答と根拠を明示できるケースを増やす**ことです。

これは未検証の仮説です。0.6.1との凍結ベースライン比較で具体的な欠落が見つからず、独立評価で効果も確認できない機能は、0.7.0へ入れません。

## 既存機能は再実装しない

| Engine 0.6.1に存在するもの | 0.7.0の検討境界 |
| --- | --- |
| `EvidenceQualificationInspector`の時点・範囲・権威判定、構造化事実の衝突検出 | 明示的な版の継承関係とas-ofでの整合。既存の適合判定は置き換えない |
| `QualifiedStructuredFactVerifier`の矛盾時のhard receipt保留 | 不明な矛盾を維持。多数決や「新しい方が正解」は禁止 |
| `EvidenceNeedMaterialization`のtarget別取得要否、古い/範囲違い根拠の再利用拒否 | 未充足要件に対する追加取得判断を限定的に統合 |
| `SourceAttributionState`のsource span、複数引用、帰属評価、衝突状態、replay | 出典の由来・独立性と限定的な複数ソース統合 |
| `EvidenceSufficiency`のadvisory sufficient/insufficient/mixed | Harness管理のtarget別回答可否。ただしmodelにhard authorityを与えない |

現在、同じtargetの出典付き文が異なる**文字列**なら衝突として印を付ける実装があります。しかし異なる表現が論理矛盾とは限らず、同じ文章が独立ソース由来だとも限りません。URL・source ID・取得時刻・版番号だけで独立性や真偽を確定しないことが重要です。

## 開発順序と担当Issue

### フェーズ0 / P0 — 0.6.1残差の事前固定・実測（[#487](https://github.com/git-ksk/reasoning-harness/issues/487)）

既存コード・テストを棚卸ししてから、転載と独立根拠、履歴の訂正、否定と単なる言い換え、必要事項の部分欠落、別対象・スコープ・プロンプト注入、混合質問、session replayを含む開発用fixtureと採点規則を先に固定します。

安全性、実際に答えられたケース、不必要な回答保留、取得コスト、provider障害を別々に記録します。**独立holdoutの問題を作る前**に、開発ベースラインから実用性・効率の数値基準を決めます。

成果物は「実測された欠落／既存実装済み／未証明／対象外」を区別するギャップ表。明確な改善余地がなければ0.7.0の計画を縮小・延期します。

### フェーズ1 / P1 — 出典の系譜・独立裏付け（[#488](https://github.com/git-ksk/reasoning-harness/issues/488)）

#487で実測した不足がある場合だけ着手。複数のevidenceや引用が、同一一次情報に由来するか、**実際に独立した出所**かを区別します。

独立性を裏付ける情報はHarness管理の検証済みprovenanceに限定し、モデルの推測、同じドメイン、URL数、転載記事数から認定しません。由来不明は独立裏付けとして数えず、同一起源でも元の引用は残します。出典の独立性が分かってもtrusted verificationには昇格しません。

### フェーズ2 / P1 — 版・時間軸の整合（[#489](https://github.com/git-ksk/reasoning-harness/issues/489)）

#487で実測した不足がある場合だけ着手。既存の有効期間、質問の`as_of`、情報の取得時刻、公開版、**明示的に証明された改訂・失効関係**を分離します。

後から取得した情報が必ず正しいとは扱いません。版の継承が証明できない、時点・対象範囲が重なる、時刻情報が欠ける場合は、根拠のある範囲だけを示して不一致や未確定を残します。

### フェーズ3 / P1 — 矛盾を保持する複数根拠の統合（[#490](https://github.com/git-ksk/reasoning-harness/issues/490)）

実測された不足、および採用された系譜・時間軸制約に依存します。対象・時点・適用範囲が合い、各sourceが個別に支持する文だけを出典付きで統合します。

単なる言い換えと真の対立を区別しつつ、**矛盾する主張は引用先ごと保持**します。semantic classifierは補助的な判断だけを返し、衝突の隠蔽、根拠や引用の捏造、主張の強化、`Known`/`Supported`への昇格はできません。

### フェーズ4 / P1 — 回答可否・上限付き再調査（[#491](https://github.com/git-ksk/reasoning-harness/issues/491)）

実測された不足がある場合だけ、既存のevidence need・検証・finalizationに接続します。targetごとの**回答に必要な事項の充足状況**で、(a) 出典付きの限定回答、(b) 予算内の追加読み取り調査、(c) 一部回答・保留・不明を選びます。

モデルの「十分」という提案が、hard verification、古い根拠、スコープ・対象の不一致、ソースの信頼境界や衝突状態を上書きしてはいけません。セッションreplayで外部処理を再実行せず、quotaやtransport障害を事実上の`unknown`へすり替えません。

### フェーズ5 / P0 — 独立評価・source release判定（[#492](https://github.com/git-ksk/reasoning-harness/issues/492)）

実装候補・採点・評価runner・合格基準を固定し、開発ケースと異なる対象・文面・引用を使う**新規の独立holdout**をその後に作成します。Mistral / Google / Groqをそれぞれ独立に評価し、意味的な誤りとAPI・quota・provider失敗を混ぜません。初回FAILは上書きせず、後続候補は別IDで検証します。

安全性・実用性の事前基準と、merge対象のexact-head CIすべてがPASSして初めて`reasoning-harness-core`を0.7.0へ更新し、`engine-v0.7.0`を独立source releaseとして公開できます。Reason CLIへの搭載は別のリリース判断・検証が必要です。

## リリース合格基準

| 観点 | 必須条件 |
| --- | --- |
| 根拠のない結論 | Trustedなverificationなしに`Known`/`Supported`へ昇格する件数 **0** |
| 出典・独立性 | 存在しない出典・引用・独立起源・証拠bindingを作る件数 **0** |
| 時点・範囲・対象 | 古い情報の不当再利用、未証明の新版優先、対象違い・scope混線 **0** |
| 矛盾の保持 | 重要な不一致を言い換え・統合時に隠す件数 **0** |
| replay・リソース | replayによる外部副作用、無制限再取得、暗黙のmodel切替 **0** |
| 実用性 | 回答成功・不要な保留・再取得・引用網羅率を0.6.1と比較。数値基準は#487後、独立評価前に固定 |
| 運用 | token・call・latency・quota/transport failureを別計測し、`unknown`へ変換しない |
| 互換性 | 既存のmachine/session IDを維持するか、別versionの契約とmigration/replayテストを用意 |

条件に合わない提案は無理に実装せず**対象外にして先へ進む**ことができます。

## 対象外・別Issue

自律coding agent、汎用browser/RAG crawler、ソース信頼度の自動ランキング、モデル多数決による真偽確定、出典の自動捏造、黙ってmodel/providerを切り替える設計は含めません。過去のEngineリリース・評価freeze・旧MCP v1のコードは維持します。

別管理の[#465 CIテストの不安定性](https://github.com/git-ksk/reasoning-harness/issues/465)と[#375 WinGet外部承認](https://github.com/git-ksk/reasoning-harness/issues/375)は、新たなrelease阻害要因が判明しない限り0.7.0のスコープには含めません。

参照: [製品ロードマップ](product-roadmap.ja.md)、[根拠の適合判定](evidence-qualification.ja.md)、[ADR-0005](adr/0005-target-local-evidence-need-routing.ja.md)、[ADR-0006](adr/0006-evidence-target-semantic-relevance.ja.md)、[ADR-0007](adr/0007-source-attributed-qualified-prose.ja.md)。
