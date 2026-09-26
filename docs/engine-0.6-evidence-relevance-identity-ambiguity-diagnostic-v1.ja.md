# Engine 0.6 evidence relevance: identity ambiguity diagnostic v1

Status: fresh calibration-only diagnostic。runtime relevance materializerは変更せず、v7のrescoreにも使わない。

## Motivation

frozen v7では `21_unknown_rename` だけMistral utility missとなった。advisory modelが `target=different` を返したためmaterialization v3が `irrelevant` を確定したが、supplied material自体は新名称がHarness targetのrename/replacementかを確立していなかった。

これは1 caseのprompt bugではなく、open-world entity identity / abstention問題として扱う。

先行研究では、対応先が確立できないentityを無理にmatchさせずNIL / no-match / abstainとして扱うこと、また高精度が必要なdecisionではone-sided evidenceがない場合にabstainする考え方が支持される。same-modelのseed一致だけではself-consistent errorを見逃し得るため、cross-model probeも独立risk signalとして使う。

References:
- https://aclanthology.org/2021.acl-long.278/
- https://aclanthology.org/2023.findings-acl.690/
- https://proceedings.mlr.press/v130/gangrade21a.html
- https://aclanthology.org/2025.emnlp-main.238/

## Fresh corpus

v7 case 21の名称・文言を使わないnew synthetic 12 caseをauthorする。

- open-world identityで `target=unresolved` を期待する6件
- 明示的distinct productで `target=different` を期待する4件
- canonical / registered aliasの `target=exact` control 2件

possible rename / successor、未登録acronym、未登録cross-language name、truncated mapping、version lineage不明、明示的separate product / comparison / not-a-rename / structured distinctness、canonical exact、registered aliasを含む。

## Runtime adoption前に測るcandidate

各observationで同じcase deadline内に2種類のbounded model outputを取る。

1. 現行binding proposal v2
2. one-sided distinctness verifier
   - `confirmed_different`: local materialが別entity/productであることをaffirmatively確立
   - `not_confirmed`: rename/alias/successor関係が不明、別名だけからの推測、mapping欠落/truncated、またはexact target

verifierはaliasやidentity factを新規作成できず、candidate textはuntrusted dataのまま。

診断計算上だけ、primaryが `target=different` でもone-sided verifierがdistinctnessをconfirmしない場合はeffective targetを `unresolved` へ落として、変更なしのmaterialization v3へ渡す。これはcandidate abstention policyの測定でありruntime semantics変更ではない。

## Execution

required diagnostic source:

- Mistral `ministral-8b-latest`
- Groq `openai/gpt-oss-120b`

各sourceは12 case x 3 matched seed trial = 36 observations。各observationはprimary + verifierの2 model requestを持つ。

## Precommitted diagnostic gate

各providerで36/36 operational completionを要求する。

one-sided candidateをruntime successor設計へ進める条件は各provider独立で以下すべて。

- unresolved/exactに対するfalse `confirmed_different`: 0
- explicit-different controlのconfirmation miss: 0
- gated disposition exact: 36/36
- provider failure: 0

primary binding accuracy / baseline materialized accuracyはdiagnostic。PASSしても#462 acceptanceではなく、fresh materialization successorを設計する根拠になるだけ。FAILした場合もこの12 caseを編集して合わせ込まない。
