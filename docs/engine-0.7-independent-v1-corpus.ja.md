# Engine 0.7.0 — 独立評価v1 新規コーパス

**事前固定runner：** `engine-0.7-independent-v1-runner-freeze`（`1124e5a7c34e42c088d81908c3cdc75aae2db309`）。

このコーパスはrunner・評価閾値・採点プログラムをmainへマージし、上記タグをpushした**後**に作成した。12件の新規ケース（15対象）は既存開発fixtureおよび観測済みholdoutから、ケースID・対象ID・出典ID・連続8語の引用窓を区別し、新規性検査 `FRESH_DISJOINT`を通過。

- レビュー可能な意味の互換性：**6対象**
- 矛盾・不明・scope/version不一致：**8対象**
- 同一の原文比較：**1対象**（追加モデル呼出不要）
- 出典の対立は有効化/無効化、正確な数値、オンライン/オフライン、限定条件の反例を含む。
- 混合ターンで互換性と矛盾を別々に保持する。
- Prompt injection文字列は無害な出典データとして扱う。

**採点閾値は既存の事前固定値を変更しない。** 3社とも候補と0.6.1の出典付き回答を同じケース・出典で比較し、各社最低1件の有用な互換性改善、安全性違反0、運用失敗0を要求する。評価ケースはすべて架空の出典で、外部事実の真偽や実ユーザーの成果を検証したものではない。モデルの助言を単独で承認扱いしない。

**事前検証：** Mistral・Google・Groq全社で `validated_no_model_observation`（各12ケース、positive=6、negative=8、呼出0）、新規性検査PASS。実モデルでの評価は未実行。

初回観測時はタグ `engine-0.7-independent-v1-corpus-freeze` をpushし、固定CIで結果を保存する。FAILの場合でも入力と初回結果を変更せず、必要なら別バージョンで改善する。

<https://github.com/git-ksk/reasoning-harness/issues/492>
