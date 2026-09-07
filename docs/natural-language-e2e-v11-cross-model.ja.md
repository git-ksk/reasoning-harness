# Natural-language E2E v11 クロスモデル再現測定

Issue #256 は、planner/action-selection または downstream grounding の製品変更を行う前に、固定済み v0.4.1 `natural-language-e2e-v11` 観測を独立したモデル系列で再現測定する。

## 参照座標

- リリース製品: `v0.4.1` / `29a9e4be6273dbffeda324e15517dc64930ad315`
- v11 freeze: `natural-language-e2e-v11-freeze` / `a758af17a998493c1005702365b100e05b05f95d`
- canonical参照: Actions `34129798774`, artifact `10021729999`
- canonical参照モデル: Mistral `ministral-8b-latest`
- seed / max tokens: `57000` / `1024`
- 固定ケース: 13

canonical Mistral 観測は参照行としてのみ使用し、再実行しない。replication wrapper は固定済み v11 evaluator/scoring 実装をimportして再利用し、v11 freeze surface自体は変更しない。

## 固定する再現対象

- Google `gemma-4-31b-it`
- Google `gemini-3.5-flash-lite`
- Groq `openai/gpt-oss-120b`
- Groq `qwen/qwen3.8-27b`
- Groq `openai/gpt-oss-20b`

quota保護のためprovider固有のtransport pacing差は許容するが、製品・corpus・case・seed・max tokens・scoringの意味座標は固定する。

## レポート契約

各対象はcorrectness/operational gateとutilityを分離して報告する。3件のno-result follow-up caseでは次を独立に測る。

1. trigger reachability
2. predecessor typed `no_result` が露出したcaseだけを分母とするIssue #249 mechanism conformance
3. downstream useful follow-up と grounded-target utility

trigger missはplanner/action-selection utility dataでありmechanism failureではない。trigger分母0はinconclusiveとする。provider/protocol failureもsemantic failureと混同せず保存する。

## Freeze discipline

各対象で最初にlive case launch boundaryへ入った実行をcanonicalとし、launch済み対象を黙ってrerunしない。後続のsemantic変更は新しいreplication identityを必要とする。v9/v10/v11 freezeとv0.4.1製品sourceは不変のまま保持する。
