# ADR 0007: 出典帰属付き qualified prose

状態: #463 の Engine 0.6 候補。

## 決定

Harness 所有の独立 source-attribution lane を追加する。「出典 S が X と述べる」は「X が外部世界で真・最新・適用可能」を意味しない。

Harness は exact target policy、admitted evidence/source binding、bounded UTF-8 span、locator、取得/版時刻、authority ceiling、materialization policy identity、transform acceptance、conflict state、canonical exposed text、citation、validation、replay persistence を所有する。

provider/model は paraphrase/summary/translation と semantic-preservation assessment を提案できるが、target、evidence/source binding、authority、hard-verification policy、citation、最終 prose は決められない。

exact quote は bound span から Harness が生成する。変換は deterministic anti-strengthening と exact-binding preserved assessment を通す。翻訳は authority-neutral で原典 binding を維持する。

canonical finalization は free-form renderer 入力を持たない。compatible source は citation を全保持し、conflict は統合しない。source attribution は hard-verification target を満たしたり修復したりできない。

accepted state は ReasoningArtifact に永続化し、replay で外部 refetch しない。通常 provider telemetry は attempt/status/token/timing 等の構造情報に限定し、raw source/provider payload を要求しない。

## Hard gates

source-binding violation、external truth promotion、renderer-only unsupported factual exposure、paraphrase/translation strengthening、wrong-target attribution、missing mandatory citation、replay external refetch はすべて 0。

production code は fixture ID/entity/固定文言に分岐しない。#461/#462/#468 の既存 observation/tag は不変。

## 評価 freeze

live model observation 前に18ケースの development surface と scoring contract を固定する。development は Mistral + Google。Groq は semantics freeze 後に別途作成する fresh independent acceptance holdout 用に予約する。

各 required provider で useful attributed-answer retention >= 90%、avoidable abstention <= 10%、citation/source-binding coverage = 100%。exact quote は transform model call 0、transform case は最大2 model attempts。token/latency/provider overhead は診断値であり authority にならない。

development FAIL は immutable。rerun/rescore/relabel/result-driven case addition をしない。acceptance holdout は development observation と semantics freeze 後にのみ作成する。#463 PASS 単独では Engine 0.6 release を承認しない。
