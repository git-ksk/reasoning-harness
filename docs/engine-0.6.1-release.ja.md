# Harness Engine 0.6.1 ソースリリース

## バージョン

- Engine package: reasoning-harness-core 0.6.1
- 独立ソースリリースタグ: engine-v0.6.1（受け入れ条件のPASS後にのみ作成）
- 管理: #475、milestone #9
- 修正・対策: #474、#476、#477、#479
- 公開済みReason CLI 0.5.3はEngine 0.5.0のまま。今回はCLI更新ではない。

## 安全性修正

最新v17/v30のChangeOrLaunch判定で、対象自身が所有する肯定的な提供開始根拠を必須にする。別対象・条件・否定・疑問・架空・未検証の文を根拠として誤昇格しない。モデルの提案はHarness側のauthority floorを上書きできない。

過去の凍結評価・旧版のsemantic contract（v11/v23等）は保持する。

## MCPの残存制約

歴史的なmcp_readonly_v1はstdinの書込期限に問題が残る。凍結済みコードを直接変更せず、期限管理されたv2/v3へ移行する。v2のblocked-stdin回帰はPASS。本リリースはv1自体の不具合解消を主張しない。

## 受け入れ条件

- PR #480のexact-head CIはすべてPASS。
- 事前固定した独立評価v1の3件FAILは歴史記録として保持し、再採点しない。
- 新しい事前固定の独立評価v2は30/30 deterministic PASS（肯定12、否定18）。SHA-256: dea6536caf02ba015b1beeb1b7a92989fb61143639bb10e6e8192cecb57de3a3。
- Mistral / Google / Groqの新規live acceptanceはActions run 37935245178（attempt 1）で**全PASS**。各モデル12/12、計36/36、誤Relevant 0、positive utility miss 0、provider failure 0。
- release候補の全workspace tests、fmt、Clippy、各OSのCI、CLI/Engine version identityを確認後に公開する。
- 公開後のtag・SHA・GitHub Release URLを#475に記録する。

Engine 0.6.0、過去のfreeze、公開済みCLIバイナリを変更しない。
