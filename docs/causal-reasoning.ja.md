# エビデンスを踏まえた因果診断

Issue #4 は、モデルやヒューリスティックを因果的な権威にすることなく、Five Whys を単なる語彙的な言い換えチェックの先へ拡張します。

## 正規の関係

因果関係は、1つ以上の cause propositions と1つの effect proposition からなる空でない集合として表現します。proposition keys は scope を持ちます。Five Whys trace は effect-premise から proposed-cause conclusion として保存されますが、causal inspector は evidence との照合前に cause -> effect へ正規化します。

## 権限の境界

`CausalEvidence` は harness が所有し、provenance と typed conclusion（`supports`、`refutes`、`association_only`）を持ちます。Candidate/model output は claims と inference edges を提案できますが、trusted causal evidence や hard causal findings を作成することはできません。

Malformed harness-owned causal records は causal input boundary で失敗します。evidence IDs と sources は空でないこと、ID は一意であること、relations は少なくとも1つの一意で空でない cause proposition を含むこと、effect proposition は空でないことが必要です。無効な oracle input を `unknown` edge result に変換してはなりません。

`CausalInspector` は observational です。edge ごとの assessments と findings を出力しますが、claim epistemic state を変更したり、verification receipts を作成したり、最終的な `accept | reject | unknown` verdict を直接変更したりしません。現在の最終 verdict は claim-oriented のままであり、artifact 全体の causal gating は意図的に保留されています。

## ハード / ソフト診断

exact scoped support record は `supported` になります。exact trusted refutation は `refuted` と hard `explicit_refutation` finding を返します。deterministic authority を欠くものはすべて保守的に扱います。

- exact causal evidence がない -> `unknown` + soft `missing_causal_evidence`;
- association-only evidence -> `unknown` + soft `association_only`;
- multi-cause relation の一部だけを support -> `unknown` + soft `partial_support`;
- reverse direction の support -> refutation ではなく `unknown` + soft `direction_mismatch`;
- exact support と refutation の衝突 -> `unknown` + soft `conflicting_evidence`;
- proposition binding が不完全 -> `unknown` + soft `missing_proposition_binding`.

既存の lexical restatement heuristic は、狭い deterministic fast path のままです。cleanup は正確に問題のある inference edge に限定され、独立して `supported` された claim を downgrade できません。

## 決定論的因果コーパス

`fixtures/causal/` は、credentials を必要としない別個の regression corpus です。元の20-fixture claim-verdict benchmark や Issue #6 correctness denominator は変更しません。初期コーパスには、exact support、exact refutation、association-only evidence、reverse-direction evidence、conflicting evidence、missing proposition binding、multi-cause partial support、scoped near-neighbor evidence の positive controls と adversarial controls が含まれます。

`causal_benchmark` は edge assessments を supported/refuted/unknown として報告し、hard と soft findings を別々に数えます。これらの diagnostics は process-regression measurements であり、一般的な model reasoning accuracy ではありません。

## 保留中の範囲

この実装では、general causal discovery、SCM/do-calculus、learned process reward models、LLM-judge final authority、provider-specific causal branches、semantic similarity を hard gate とすることを意図的に提供していません。将来の model-backed causal critics も、deterministic または external trusted oracle により独立検証されない限り soft のままでなければなりません。

#4 からは、candidate-supplied causal-evidence reference hints と general temporal/domain-constraint reasoner も保留されています。Issue #11 は現在、causal finding/reason observations を集計できる provider-neutral repeated-trial report を提供しますが、それらは Issue #6 の correctness/availability denominators の外に置かれます。live causal-generation/input contract は依然として保留中です。現在の candidate schema に causal-evidence authority fields はなく、将来の reporting interface が candidate または model output に hard authority を与えてはなりません。

## 一般的なエビデンス適格性評価との連携

Issue #16 は structured claim verification で使う generic `Evidence` records を qualification します。`CausalEvidence` は、独自の provenance と conclusion semantics を持つ、別個の harness-owned relation-evidence contract のままです。generic `EvidenceMetadata` を causal records に暗黙に投影してはなりません。Domain adapters は両方の入力に共通の external source policy を適用できますが、claim qualification が意図せず causal proof にならないよう、core は authority types を分離します。
