# Engine 0.6 evidence relevance: identity ambiguity diagnostic v1 result

Status: diagnostic candidate studyとしてfrozen PASS。runtime semantics変更や#462 acceptanceではない。

## Frozen identity

- issue: #462
- tag: `engine-0.6-evidence-relevance-identity-ambiguity-diagnostic-v1`
- freeze commit: `18a226e5ab0b7d022c587a6c4ee7e57b1c154bb1`
- first/only Actions run: `36029430165`
- run attempt: 1
- corpus: v7 case21の名称・文言を除外したfresh synthetic 12 case
- trial: providerごとにmatched seed 3回
- required diagnostic provider:
  - Mistral `ministral-8b-latest`
  - Groq `openai/gpt-oss-120b`

## Result

両providerとも36/36 observationを完了し、provider failure 0。

### Mistral

- primary target-binding exact: 25/36
- expected-unresolvedに対するprimary false `different`: 11
- baseline disposition exact: 25/36
- one-sided distinctness exact: 36/36
- unresolved/exactへのfalse `confirmed_different`: 0
- explicit-different controlのconfirmation miss: 0
- gated disposition exact: 36/36

現行primary assessorはpossible successor、未登録alias、cross-language name、truncated mappingなどのopen-world identity uncertaintyを繰り返し`different`へ寄せた。one-sided verifierは一貫して`not_confirmed`を返し、診断candidate policyは`irrelevant`ではなく`ambiguous`へabstainできた。

### Groq

- primary target-binding exact: 35/36
- expected-unresolvedに対するprimary false `different`: 1
- baseline disposition exact: 35/36
- one-sided distinctness exact: 36/36
- unresolved/exactへのfalse `confirmed_different`: 0
- explicit-different controlのconfirmation miss: 0
- gated disposition exact: 36/36

primary missはpossible-successor 1件のみ。one-sided verifierが`not_confirmed`を返し、診断candidate policyはexpected `ambiguous`へ回復した。

## Interpretation

v7 `21_unknown_rename` missは単発seed事故だけでは説明できない。fresh corpusでも、advisory binding modelがunknown identityを`different`へ過剰確定し、materialization v3がそのnegative bindingを`irrelevant`確定に使う構造的riskが再現した。

fresh one-sided confirmation candidateは両provider合計72 observationで全てexact、かつexplicit-different controlを落とさなかった。したがってnegative identity decisionはaffirmative distinctness evidenceを要求し、rename/alias/successor/lineage mappingが未確認ならunresolvedを維持する方向に根拠がある。

この結果だけでcase21をpatchしたりv3をin-place変更したりしない。explicit negative-identity confirmation semanticsを持つfresh materialization successorを設計し、新identityでcalibrationする根拠とする。

## Next steps

1. v7はimmutable FAIL/incompleteのまま。
2. materialization v3はhistorical evidenceとして変更しない。
3. `target=different`が`irrelevant`を確定する前にconfirmed distinctnessを要求するfresh successor policyを設計する。
4. operational failureとsemantic abstentionを分離し続ける。
5. 次のrequired-provider構成決定前に、別freezeのGoogle 26-case operational requalificationを完了する。
6. fresh successor calibration PASSまではindependent holdout authoring禁止。
