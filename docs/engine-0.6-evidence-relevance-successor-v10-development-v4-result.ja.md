# Engine 0.6 evidence relevance successor-v10 development v4 結果

Status: immutable development PASS.

- Freeze tag: engine-0.6-evidence-relevance-successor-v10-development-v4-freeze
- Freeze commit: d9a5bd49d3c6f3ee5e79f9ba6704f0dc502a00c0
- Run: 37111376390
- Candidate: effective qualification v15 / materialization v28
- Fresh surface: 24 cases
- Required development providers: Mistral + Google
- Groq: 未観測。後続の fresh independent holdout まで除外
- Result: PASS
- Holdout acceptance evidence: false

annotated tag と attempt-1 observation は immutable とする。rerun / rescore / relabel / tag 移動は禁止。この結果は development evidence であり holdout acceptance evidence ではない。

## Preflight

frozen surface は checksum、validate-only、fmt、Clippy `-D warnings`、core library tests、v4/v28 deterministic tests、historical replay、predecessor regression suites、runner tests をすべて PASS。validate-only は 24 planned / 0 completed で provider observation は発生していない。

## Mistral

`ministral-8b-latest` は 24/24 完走、runner rc 0。

- authority failures: 0
- identity/risk failures: 0
- materialization failures: 0
- proposal exact: 16/24
- raw local qualification exact: 14/24
- effective v15 local qualification exact: 24/24
- materialized v28 exact: 24/24
- total tokens: 41,237
- result: PASS

## Google

`gemini-3.5-flash-lite` は 24/24 完走、runner rc 0。

- authority failures: 0
- identity/risk failures: 0
- materialization failures: 0
- proposal exact: 16/24
- raw local qualification exact: 17/24
- effective v15 local qualification exact: 23/24
- materialized v28 exact: 24/24
- total tokens: 42,426
- result: PASS

effective qualification の 1 mismatch は authority / identity-risk / final materialization failure を発生させず、precommit 済みの final v28 contract は 24/24 exact を維持した。

## Adjudication

required provider 2本とも development gate を PASS。v15/v28 の bounded successor design を支持する結果になった。

- direct `defined as` Definition evidence は exact-target / no-risk 制約下で bounded な other-relation authority になれる
- instruction/control text は model-only negative authority になれない
- 別セグメントの clean factual negative evidence は unrelated instruction があっても保持される
- Harness-owned requested-relation authority は instruction/control text があっても保持される
- v28 は effective v15 state を合成し、stale model proposal authority を再導入しない

historical replay の変更は precommit 済み adjudicated correction のみに限定される。frozen v11/v14/v23/v27 と過去の immutable run は変更しない。

## Next step

この 24-case development corpus を acceptance evidence に使わない。v15/v28 successor semantics を別 tag で freeze した後、観測済み development surface を再利用しない fresh independent holdout を author する。Groq は successor semantics freeze 後の fresh independent holdout でのみ再参加させる。
