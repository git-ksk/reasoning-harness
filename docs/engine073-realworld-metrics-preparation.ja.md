# Engine 0.7.3 実資料評価の指標と実行準備

日本語 | [English](engine073-realworld-metrics-preparation.md)

2026-10-17 JST時点で **準備のみ／未固定／未採点／採用NO-GO**。採点契約の提案と既存runnerの責任範囲を整理した文書です。採点器の実装、新評価の開始、Engine選定、既存freeze/protocol/初回結果の変更は行いません。新規24問・60件、gold回答、model/provider/network/Oracle呼び出しは0です。Engine実装、現候補tests、現在の独立ケース本文、親の診断・開発資料は未読を維持しています。[読んだファイルのhashとレビュー範囲](../evaluation/engine073-realworld-metrics-preparation-v1/review.json)を別保存します。

[既存protocol](../evaluation/engine-07-realworld-protocol-v1.json)はdraftのままです。元の0.6.1/0.7.2識別、改善・効率ゲート、論理操作6回という予算を、v3の12回や新F1で黙って置き換えません。後継は新しい完全freezeが必要です。製品baselineは0.6.1、0.7.2とv2/v3の初回比較は過去の証拠として保持します。[通常経路とhelperの契約](engine-07-realworld-evaluation.ja.md#機能単体と製品全体を別々に測る)の件数を混ぜません。

## 主張の単位・gold・支持の範囲

利用者に見える事実主張を、対象・関係/値・断定の強さ・適用範囲・時点を保った原子的命題に分けます。独立に真偽が分かれる並列文は分割します。最終文章に加え、表示artifact・claimカード・引用captionの事実、質問に不要な追加断定も対象です。同じ回答内の同一命題の繰返しは主張P/Rでは1件ですが、安全性監査では全出現と全引用edgeを保持します。pとnot-pは別命題です。疑問文、保留、非事実的な会話を機械的に事実主張へ数えず、保留理由として断定した事実は数えます。この分割規則も独立著作前に固定します。

goldには、回答に必要な最小のsupported命題/facet、同義の許容表現、明示的な選択肢、未解決facet、必要な矛盾・訂正の開示、**取得済み・適用可能な資料集合の下での**回答可能性（全部／一部／なし）を記録します。同義の選択肢は1要件groupで最大1点。goldの言い換え数で分母を増やさず、一つの曖昧な文で別々の要件を満たした扱いにしません。主指標の要件重みは全て1とし、文字列だけでなくgroupも固定します。重み付き診断を使う場合は別名・別固定式が必要です。groupを回答内容・矛盾開示・保留説明に分類し、支持された説明事実から質問事実が回答可能とは扱いません。正しく支持された余分な主張はprecision対象ですが、質問にない要件のrecall点を作りません。

goldは制御された参照注釈であり、外部の普遍的truthや人間の署名ではありません。「回答不能」は今回の許可資料に必要な支持がない意味で、資料がないことから実世界の出来事が起きなかったとは断定しません。「資料Aはp、資料Bはnot-pと述べる」は支持可能でも、p自体は未解決という区別を保ちます。注釈者の識別・役割、全文文脈での理由、意見相違の解消、全source bindingを記録します。モデルの同義判定は承認権限を持たず、goldの権威を作りません。人間の確認が未実施ならエージェント／ホスト作成と明記します。

重大な真の矛盾は、質問as-ofの下で同じ命題・対象・範囲・版の適用時点が重なる場合です。取得/公開時刻が新しいだけで以前の主張を捨てません。supersessionには権限あるpublisher/ownerの訂正関係と適用期間の根拠が必要で、系譜不明なら矛盾保持や保留が必要です。過去時点の質問は当時適用された版を使い、自動で最新版にしません。scope/version違いは真偽の矛盾ではなく条件付き回答を要する場合があります。必要な開示と未解決truth facetをgoldに記録します。

source bindingには、原文全体のraw hash、正規化text hashと正規化版、URL/document ID/publisher、引用textとUTF-8 hash、取得全文内locator、取得時刻、分かる範囲の公開/改訂時刻、有効期間、質問as-of、対象/product/version/platform/region/scope、origin/lineage groupと根拠またはunknownが必要です。取得時刻を公開時刻や適用開始と見なしません。適用情報の欠測は推定で埋めず、publisherの事実とharness所有のmanifest計数を区別します。同一origin・転載・系譜不明から独立裏付けを認定しません。正当に異なる取得raw bytesのdocument hashは異なるまま保持し、引用の同義性で同一identityへ強制しません。

**grounded**は、実観測の取得/admission/evidence経路で特定できる適用可能な資料が、回答の文面と断定の強さを支持し、重大な反対資料も処理されていることです。取得journal→admission/evidence ID→実content/facts/metadata→claim evidence IDs→表示をたどります。goldとの単語一致やモデルのもっともらしさだけでは支持を認めません。unsupported／contradicted／binding不明／対象・範囲・時点違い／過剰断定を別理由として残します。引用の否定・条件・例外を落としません。資料集合外の知識は一律に偽と決めませんが、このclosed-source評価のgroundedには加点しません。

取得modeを `host_curated_facts` / `captured_full_text_extraction` / `open_web` として固定し、別表示します。ホストが構造化事実を供給する評価は、その受入・推論であり、自動抽出や発見の評価ではありません。過去の開発評価はホスト注釈を使い、実際の訂正・矛盾・系譜不明を未検証としています（[既存範囲](engine07-realworld-development-results.ja.md#評価の範囲)）。

## Grounding precision・必要主張recall・F1

運用が完了し意味採点できる回答について、`A`を表示された重複なし事実主張、`S`をそのgrounded部分、`G`をgoldの必須supported要件group、`H`を適切な範囲・断定の強さでgrounded出力が満たしたgroupとします。Hには実際の支持された出力が必要で、unsupportedな単語一致ではrecallを加点しません。

- `P = |S| / |A|`：主張のgrounding precision。
- `R = |H| / |G|`：必要なsupported主張のrecall。
- P/Rが定義でき、`P + R > 0`なら `F1 = 2PR / (P + R)`。P=R=0ならF1=0。

支持された追加主張があるとP/Rの分子は同じ集合になりません。Relevant/non-Relevantの二値ソースF1、helper表示精度、運用完了率を「実回答F1」と呼びません。この準備で実回答F1は測定していません。

| 条件 | P | R | F1・扱い |
| --- | --- | --- | --- |
| Aあり、Gあり | 上式 | 上式 | 上式。P=R=0は0 |
| Aなし、Gあり：意味的な保留 | null（主張なし） | 0 | 空回答の明示規則として0。過剰保留も評価 |
| Aあり、Gなし | 上式 | null（必須supported主張なし） | null。unsupportedと正しい保留を別採点 |
| Aなし、Gなし | null | null | null。自動的な満点にせず保留・有用性を確認 |
| 運用失敗・抑止・finalization/必須trace欠測 | unavailable | unavailable | null。正しい意味的保留には数えない |

nullには理由を付けます。一部だけ回答可能な質問の完了回答は、支持可能な部分をGに置き、未解決facetを保留軸で測ります。P/R/F1が定義できても完全で有用な回答とは限りません。失敗runの表示済み部分主張も重大安全性監査へ残しますが、部分出力の診断を通常回答の有用性増加や完了回答の意味集計に混ぜません。

provider／track／arm／family／gold回答可能性の層ごとにmicro P（sum S / sum A）、micro R（sum H / sum G）とその調和F1、およびF1定義済みcaseのmacro F1を報告します。全分子・分母・null数・完了/予定件数を併記し、集計分母にも同じ分母0規則を適用して、同じ完了cohortだけで集計します。providerやarmの欠測cohortを黙って差し替えません。別途 `delivered_required_coverage = 実際に得たH / 全予定caseの必須G` を出し、未実行・抑止は得られた点なしとします。これは提供率込みの指標で、意味recall/F1ではありません。完了分だけのF1で欠測を隠さず、必須行欠測はNO-GO。case単位の対比較と両arm完了subset件数を保存し、確率的な対比較差だけでEngine原因を確定しません。

## 回答・引用・安全性・有用性の別軸

| 軸 | 事前に固定する評価と分母 |
| --- | --- |
| 回答可能性 | case/要求facetごとのgold全部・一部・不可と、出力の全部回答・一部・保留・誤回答。運用状態と別欄 |
| 過剰保留 | 完了回答で、supported要求facetの省略・明示保留数 / 全gold supported要求facet。省略はRも下げる。goldが許す条件付き表現を自動で保留としない |
| 正しい保留 | 未解決命題を断定せず適切に明示保留した要求facet / 全gold未解決facet。沈黙だけは有用な説明にしない。分母0はnull |
| 引用edge precision | 正しい表示claim→citation edge / 全表示edge。identity・URL・実source/version/hash・適用性・locator・支持内容を確認。実在しても無関係な引用は不正 |
| 引用coverage | 正しい表示引用が1件以上付く要求対象の事実主張 / 引用が必要な全表示事実主張。原則は全事実主張に引用を要求。主張0はnull。内部evidence bindingとは別軸 |
| 引用文忠実度 | locatorの固定正規化原文と一致するliteral引用 / 全literal引用。省略箇所、否定、文脈を保持。引用hashはbytesの確認でありpublisherの認証やtruth証明ではない。引用なしはnull |
| 矛盾・訂正・時系列 | 重大矛盾の開示、権限ある訂正系譜、current/historical as-ofと対象・範囲を、各適用case/facetで測る。分類別違反と未観測を残し、一つの率へ混ぜない |
| 有用回答 | 運用完了、全必須supported facet、適切な未解決facet処理、必要な矛盾・訂正・時点注記、正しい表示引用、重大違反0、固定した読みやすさ・行動可能な完結条件を満たすcaseだけ成功。分母は全予定適用task、失敗を別記。説明品質・簡潔さは固定ordinal rubricの別軸で、事実完成度の代わりにしない |
| 運用coverage | 完了 / 全予定row。開始・抑止・cancel・timeout・protocol・trace・会計失敗を別分類。運用完了は意味正解ではない |

supported回答の有用性には、保留理由の事実支持だけでなく、goldで回答可能な質問facetが最低1つ必要です。全部回答可能な完全回答、一部回答可能な支持・条件付き回答、noneの保留応答を別層にします。質問truthが回答不能でもGに出典別の矛盾報告や説明事実が含まれ得ます。そのcoverageは診断P/R/F1に加点できても、supported回答の増加や既存draftの完全有用回答増加へ加算しません。新しい増加閾値にどの回答可能層を含めるか先に固定し、保留の有用性は別記します。混合F1/有用性率で支持回答の改善0を隠しません。

通常E2Eとhelperは分母を分けます。helperの意味助言、制御ホスト承認、正しいtarget事前binding、実finalization、対象への表示を別欄にします。missing/wrong-target prebindingは、有効な陽性親を持つ別定義のtarget検出試験でなければinvalid/unproven診断です。helper表示増加を通常回答recallや有用回答に加算しません。

candidateは、unsupported truth/表示主張の昇格、捏造・誤binding引用、重大矛盾の隠蔽、対象・範囲・as-of漏洩、無権限review/supersession、未承認副作用を**各分類0**にする必要があります。失敗runの部分表示を含む全観測表示を監査し、未観測経路はunknownであり違反0としません。平均F1や有用性増加で重大違反や既知未修正の安全性欠陥を相殺しません。baseline違反と対比較の陽性退行も保持します。既存draftのprovider別増加・効率値は過去のdraft値として残し、新F1/有用性閾値と後継での用途は独立著作前に決めて固定します。新回答を見た後でこの文書から合格値を推定しません。

## Call・token・時間・費用・provider別集計

CLI論理操作、adapter生成、実HTTP試行（失敗・retry含む）、取得call/再取得round、input/output/total tokenとprovider固有reasoning/cache項目、child時間・credential/callback時間・全工程/lane時間、typed failure、完全使用量とlower-bound/unknown診断を分離します。生成回数を論理操作上限と比較しません。v2の単位ミスとv3の予算調整は元のまま保持します（[初回結果](engine07-realworld-development-results.ja.md#v2の初回結果--不合格を保持)）。

全寄与row/項目を完全に計測できたときだけtotalを数値にします。欠測時はtotal=nullとし、`observed_subtotal`、欠測数、lower-bound状態を別表示します。確認済みstart0はこのrunnerからのrequest0であり、共有組織の未知使用量0ではありません。baselineが0/欠測の効率比は利用不能とし、必須効率ゲートを満たした扱いにしません。価格根拠の版・日付、通貨、課金token分類の対応を固定し、不明ならcost=null。reserve token、観測token、推計金額、実請求額を混同しません。この作業で現行価格を取得していません。

必須provider/modelごとに、予定/完了、P/R/F1、facet保留、重大違反、有用回答の対比較増加/退行、call/token/time/costを個別報告します。モデル版、実request全設定、source順/as-of、arm順、host/adapters、予算を固定し、実request/response IDと設定を保持します。providerや通常/helperを合算して失敗を隠しません。反復task数、unique文書数、origin group数も報告し、24シナリオを24独立文書と呼びません。必須providerが停止中なら未完了であり、黙って除外・モデル/tier交換しません。

## 実行準備：実装済み範囲と不足する証拠

許可された既存ファイルのレビューであり、新runnerを実行した結果ではありません。[読んだbytesのhash](../evaluation/engine073-realworld-metrics-preparation-v1/reviewed-input-sha256.json)を保存しています。古い開発結果文書は当時の単独guardを記述していますが、後の#540 schedulerは永続実行を実装済みです。履歴を消さず、実装済み永続化を丸ごと未実装とは扱いません。

| 境界 | 確認した証拠 | 実資料freeze前の残ゲート |
| --- | --- | --- |
| 永続実行・会計 | [runner.py 97–156、263–329行](../evaluation/engine07-realworld-runner-v1/runner.py#L97-L156)：planの分離、排他出力/restart拒否、auth/spawn前のfsync reserve/inflight、journal/atomic state、返金なし、型付き失敗・未知会計。[runner README](../evaluation/engine07-realworld-runner-v1/README.md#independent-review-repair)は47 offline controlsと限定独立再レビューPASSを記録 | その保存済み成功をlive意味受入にしない。今回はtests未再実行。summaryは327行で `semantic_acceptance: NOT_SCORED`。新完全scorerとtrace/citation検証が必要 |
| 実host callback | [runner.py 48–94、187–253、284–311行](../evaluation/engine07-realworld-runner-v1/runner.py#L48-L94)：宣言された生成/input/output/retry最大値からreserveを算出し、subprocess envelope/usageから運用結果を読む | 固定実host/adaptersのcontext増大、fallback/tool/resolver/retryまで上限を導出・強制する。envelope成功/usage既知だけでmodel trace、取得/admission、最終主張支持を証明しない。独自credential loaderはchild時計の前に走る（288/198行）ため、CLI main同様に無作用とするか別に上限・会計を持つ |
| dependency closure | `validate_plan`は渡されたartifactsとargv[0]、provider/modelを検査し、credential callback後に再hash（65–93、187–196行） | interpreter、script/import、dynamic library、host/provider/Engine binary、resolver/tool子、prompt/schema/正規化/static input、関係する環境/configをownerが網羅する必要がある。hashリストは網羅性を自動発見しない。協調的freeze/recheckは敵対的writerに対するatomic check/execではない |
| 数値live予算・headroom | [reserve式](../evaluation/engine07-realworld-runner-v1/runner.py#L48-L53)は生成数×（input+output）×attempt数。同provider/model laneでreserve返金なし。[README 5–9行](../evaluation/engine07-realworld-runner-v1/README.md#L5-L9)は実数値導出を未完了と明記 | 実host由来のcall/task/arm/lane正上限に、両Engine・helper・別承認されたjudgeも含める。input増大/retry/取得/wall/grace/金額自己上限を固定。現schedulerはtoken予算で独立通貨予算を強制せず、応答後の超過発見で請求を取り消せない。旧6/12回や過去headroomを根拠なしに流用しない |
| Groq・共有利用 | [groq-state.json](../evaluation/engine07-realworld-operations-v1/groq-state.json)は起動不可、回復false、共有使用量unknown。runner121–130行でauth前に抑止 | Groq追加起動なし。時間経過・TPM待機・小さいreadiness成功・モデル変更では禁止解除できない。別途検証した回復/headroom方針と共有組織の調整が必要。他プロセス/新出力identityを含むaccount全体の協調はschedulerの外 |
| 中断・restart | [runner.py 166–235行](../evaluation/engine07-realworld-runner-v1/runner.py#L166-L235)はgroup grace/hardkillと未完了扱い。[README 11/34行](../evaluation/engine07-realworld-runner-v1/README.md#L11)はPOSIX/協調範囲を記録 | parent SIGKILL後の残存子groupを外部controllerで封じ、検査してから別実験へ進む。実機署名binary・全platformのcontainment受入ではない。未知使用量はunknownのまま |
| 通常CLI接続 | [評価契約25–29行](engine-07-realworld-evaluation.ja.md#機能単体と製品全体を別々に測る)は質問→取得→実Engine→表示を要求し、旧静的検索だけでは足りないとする。[runner README28/34行](../evaluation/engine07-realworld-runner-v1/README.md#L28)は固定offlineCLI証拠へのリンクとactual paired host/path/live残件を記録 | 0.6.1と修正後の正確な後継を同hostで比較するplan、実runtime path/admission/claim/citation trace、対象機能が実行されるactive controlが必要。固定offlinefixtureやscheduler成功envelopeだけで現候補のlive接続・抽出・最終回答改善を認定しない。全Engine変更が無効とも断定しない |
| 意味的証拠 | [開発結果11、41–55行](engine07-realworld-development-results.ja.md#v3の開発用後継評価--不合格)：ホスト事実、引用表示欠測、部分使用量、provider失敗、新規独立0/24 | 実際の矛盾・改訂・系譜資料と、最終文章全体の支持採点が必要。source-blind合成試験、release CI、helper増加で代替しない |

履歴上の `repair-verification-v2.json` はレビュー前のPENDINGを含み、後のrunner READMEは独立再レビュー完了とanchorを記録しています。両方とも変更しません。これは証拠の時系列であり、限定修正レビューが今も未実施という意味ではありません。

## 責任とfreezeの順序

1. **実験owner・独立著作前**：candidate安全性修正、同host/adapters下の0.6.1/後継actual build、new意味scorerとrequest/trace/evidence/表示検査、callback/dependency/containment、数値予算/headroomと全閾値/provider設定を揃える。旧結果へ新指標を後付けしない。
2. **評価器freeze M0**：完全scorer/rubric、gold schema/主張分割、取得mode、runner/host/dependency closure、model全設定、pair順、timeout/retry/reserve/costと安全・効率・品質閾値をcommit/hashで固定する。残件は明記し、M0未完了の間は独立caseを作らない。本準備文書はM0ではない。
3. **独立著者・出典reviewer、M0後**：新しいdisjoint24問と全文snapshotを作り、必須/選択/unknown/矛盾/時点goldを全文・適用性から監査する。注釈者の権限と曖昧さの解消を実行前に記録し、モデルへ正解を渡さない。本作業では未着手。既知・未使用の古いdraftをfreshとしない。
4. **初回要求前のcorpus/gold freeze M1**：全質問、全文bytes、正規化/引用系譜、labels/rationale、適用性、task/origin分母、取得対応を固定し、offline schema/active controls/両build identityを確認する。active controlは別名の開発用対照であり、新規独立caseの事前意味実行にしません。新規独立caseはM1固定後の初回まで意味観測0を維持します。M1前の設計修正は元bytes/静的結果/理由/差分を残す。plan/引数/成果物に秘密credentialを置かず、private traceの保存権限・auth header除去を定める。
5. **実行owner、準備完了後のみ**：create-new初回identity、auth前永続reserve、両Engine/helper共通latch、first-only provider scheduleを使う。抑止/部分/失敗は未完了。実request/response/trace/最終表示、使用量・失敗を残し、外側retry、黙ったmodel/tier交換、意味救済をしない。Groqは別回復ゲート完了まで起動しない。
6. **採点reviewer**：固定した主張分割/支持rubricを1回適用し、span/binding、可能ならarm/providerを伏せたadjudicationを保存する。固定rubricの人間判断はgold変更権限ではない。実行後にgold/scorer欠陥が判明したら診断を追記して測定無効とし、元label/初回点を保持して別後継/calibration identityへ進む。後付け合格にしない。将来model judgeを提案するなら別固定の権限境界・request・予算が必要で、本準備では含めず承認もしない。
7. **受入owner**：全必須provider/track分母、重大分類各0、対比較退行/有用性/効率、新規独立受入、別のCLI統合・配布を確認する。文書準備、限定コードreview、provider運用、Engine source品質、最終回答品質、製品採用は別ゲート。

次の具体作業は、新しい意味採点器の契約・実装とoffline反例検証、実host callback/dependency/予算plan、現候補のpaired CLI呼び出し/evidence traceです。これらの完全freeze後にだけ独立実資料gold著作へ進みます。本レビューはその実装・実行を行っていません。
