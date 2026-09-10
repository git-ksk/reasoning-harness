# Natural-language E2E v30 — Investigation Utility & Provider Parity acceptance

Issue #263 では、#328 / PR #329 の prospective v13 evaluation-design change 後、最初の fresh held-out successor として v30 を使う。freeze 済み v1〜v29 evidence は immutable で、再実行・再採点・調整・書き換えは禁止する。release closeout までは Cargo workspace version を `0.4.1` のまま維持する。

## v30 を作る理由

v29 (`natural-language-e2e-v29-freeze`, commit `a91e16c017efcc14bdd698afab8c588af7e6cbac`, seed `96231`) は immutable な VALID RELEASE FAIL である。v0.4.2 candidate は全必須 model row で operational / correctness clean だった一方、released v0.4.1 の Gemini control では structured-generation protocol failure が再発した。#323 の evidence-preservation helper により control nonzero 後も canonical candidate observation は保存できたため、残った問題は evaluator design、つまり operationally incomplete な released control で既に観測済みの semantic evidence と未観測部分を区別できない v12 gate だった。

#328 / PR #329 では provider-neutral な operational observability frontier と conservative partial-identification bounds を追加した。これは prospective only で、v26 / v28 / v29 を再分類しない。v30 が metric revision `v13` を初めて使える surface である。

v30 は released v0.4.1 control `29a9e4be6273dbffeda324e15517dc64930ad315` と candidate `597f5ac8ffaef5c4e66f23f301bfc05c9b73ae3f` を pair 測定する。corpus は live observation 前に固定した fresh seed `97362` と、新規 case ID / task / fact key / answer / source identity / fresh marker を使う。

## Measurement lock

v30 は v11 の target/tool/finalization semantics と、v12 の continuation-opportunity semantics を完全維持する。唯一許可する scoring delta は v13 の operational observability と conservative bounds で、`config/natural-language-e2e-metric-v13.json` と `natural-language-e2e-operational-bounds-v13` に固定する。

各 scoring metric は case 単位で `observed` / `censored` / `not_applicable` に分類する。operational terminal より前に確定した positive monotone witness は observed のまま残す。一方、terminal 後に起こり得たイベントが「起きなかった」とは扱わない。censored case は complete-case deletion せず、都合の良い値も代入せず、control の論理的に許される区間をそのまま計算する。

higher-is-better の非悪化は exact candidate value が control upper bound 以上の場合だけ証明成立する。lower-is-better は candidate が control lower bound 以下の場合だけ成立する。strict improvement も candidate に不利な control endpoint に対して証明する。証明できない比較は `INCONCLUSIVE` であり、`INCONCLUSIVE` では release しない。

candidate の operational incompleteness は従来どおり hard `FAIL`。既存の correctness / safety zero fields もすべて hard gate のまま。v12 の continuation eligibility が true の candidate は mechanism conformance 1.0 必須。#324 candidate-only diagnostic sidecar は引き続き `scoring_input=false` で、v13 bounds module は scoring input として受け付けない。

v30 metric-lock validator は immutable v29 を predecessor にする。v11/v12 から継承する runner scoring functions と pair scrub が identity / seed / candidate coordinate の正規化以外で AST 同一であることを証明し、その上で v13 policy identity と acceptance boundary を別途検証する。live observation 後の semantic rewrite は禁止する。

## Freshness / pairing

case family は logical coverage を維持する。13件 = investigation 10 + session 3、そのうち observational exact-target typed-no-result follow-up 3件、read-only GitHub MCP nonpromotion 1件。freshness test は v1〜v11 repository root と immutable v12〜v29 freeze ref を対象にする。GitHub MCP case は exact paired coordinate の `Cargo.toml` を読み、control/candidate config の差分は coordinate ref と3組の follow-up に対する candidate-only `selection_priority` だけに限定する。

provider coordinate は live 前に固定する。Mistral `ministral-8b-latest`、Google `gemini-3.5-flash-lite`、Google `gemma-4-31b-it`、Groq `openai/gpt-oss-120b` candidate-only。`planner_max_tokens = 256` と provider `max_tokens = 1024` は変更しない。Google Gemini / Gemma は model-specific quota lane として matrix `max-parallel: 2` で別job実行するが、各 paired row 内部では control → candidate の canonical 順序を厳守する。

## Pre-live gate

provider credential を使う前に、exact control/candidate coordinate、immutable v29 predecessor、candidate からの `Cargo.toml` / `Cargo.lock` / `crates` runtime diff なし、workspace version `0.4.1`、corpus / surface checksum、v13 metric lock、pair validator、validate-only、no-model/no-network preflight、exact CLI capability probe、full deterministic Python tests、full Rust workspace tests、fmt、clippy `-D warnings`、pinned GitHub MCP contract、workflow-policy tests、通常 PR CI をすべて通す。

これらが green になるまで `natural-language-e2e-v30-freeze` は作成しない。provider credential を露出する前に freeze tag と checksum を確定させる。

## Canonical live 順序

1. immutable v30 freeze で Mistral paired canonical を1回だけ実行する。
2. Mistral paired gate が `PASS` の場合だけ、同一 freeze で cross-model workflow を開始する。
3. Gemini paired canonical と Gemma paired canonical を各1回実行する。各rowは control を先に、candidate を後に各1回だけ実行する。control が nonzero でも canonical evidence が保存されていれば、v13 acceptance comparator に判定を委譲し、自動的な成功扱い・失敗扱いはしない。
4. Groq candidate-only canonical を1回実行する。
5. raw canonical stdout、report、orchestration record、candidate diagnostic sidecar を保存する。paired orchestrator の45秒 heartbeat は stderr の non-scoring progress で、captured canonical stdout は変更しない。
6. 必須 model row はそれぞれ独立判定し、cross-model averaging は禁止する。

v30 が `FAIL` または `INCONCLUSIVE` なら immutable evidence として確定し、rerun / rescore / tune はしない。fresh successor には別途正当化された prospective change が必要である。
