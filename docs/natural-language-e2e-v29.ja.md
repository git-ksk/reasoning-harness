# Natural-language E2E v29 — Investigation Utility & Provider Parity acceptance

Issue #263 では v29 を v0.4.2 release gate の fresh held-out successor として使う。freeze 済み v1〜v28 evidence は immutable で、再実行・再採点・調整・書き換えは禁止する。release closeout までは Cargo workspace version を `0.4.1` のまま維持する。

## v29 を作る理由

v28 (`natural-language-e2e-v28-freeze`, commit `ccce3e56b3093746450db05a07ac2dacc473fa1b`, seed `95100`) は VALID RELEASE FAIL として確定済みである。Mistral paired と Groq candidate-only は positive evidence だった一方、必須の Gemini / Gemma paired row は released v0.4.1 control の structured-generation protocol operational failure により incomplete になった。旧 workflow は control の nonzero exit で停止したため、その Google row では candidate coordinate の canonical observation を残せなかった。

その immutable result の後、独立に正当化された generic 改善を2件入れた。#323 / PR #325 の `scripts/paired_canonical_observation.py` は control を1回、candidate を1回だけ起動し、control が nonzero でも candidate を抑止しない。両 report が存在する場合だけ acceptance を1回実行し、control hard FAIL は最終 gate でも hard FAIL のまま保持する。semantic retry、whole-run retry、gate 緩和は追加しない。#324 / PR #326 は candidate 側 structured-generation failure に `structured_mode`、terminal status/finish 情報、byte count、`parse_class`、`status_class` を追加したが、provider request、fallback 回数、retry policy、planner/action budget、scoring、failure classification は変更していない。

v29 は released v0.4.1 control `29a9e4be6273dbffeda324e15517dc64930ad315` と candidate `64f6669b872094577a77bcb4137a161aadb66f6a` を pair 測定する。corpus は fresh seed `96231` と、新規 case ID / task / fact key / answer / source identity / fresh marker を使う。

## Measurement lock

v29 では **metric revision を増やさない**。`metric_revision` は v28 と同じ `v12` のまま。v29 metric-lock validator は immutable v28 を predecessor とし、scoring-relevant AST を比較する。正規化を許可するのは v28/v29 identity/path、fresh base seed、candidate coordinate だけで、runner の scoring function、acceptance scoring function、pair scrub、acceptance `ZERO`、pair `PRESERVED` / `V12` policy に semantic diff があれば失敗する。

v12 の定義は完全維持する。target recall、tool-selection success、false abstention、follow-up stall、trigger exposure、continuation eligibility、mechanism conformance、correctness/safety boundary、eligible 0件時の `inconclusive` は v28 と同じ。合法な continuation opportunity がある candidate は mechanism conformance 1.0 必須。released control の mechanism conformance は引き続き baseline observation であり row validity gate ではない。

## Freshness / pairing

case family は v28 と同じ logical coverage を維持する。13件 = investigation 10 + session 3、そのうち observational exact-target no-result follow-up 3件、read-only GitHub MCP nonpromotion 1件。freshness test は v1〜v11 repository root と immutable v12〜v28 freeze ref を対象にする。GitHub MCP case は exact paired coordinate の `Cargo.toml` を読み、control/candidate config の差分は coordinate ref と3組の follow-up に対する candidate-only `selection_priority` だけに限定する。

provider 条件も固定する。Mistral `ministral-8b-latest`、Google `gemini-3.5-flash-lite`、Google `gemma-4-31b-it`、Groq `openai/gpt-oss-120b` candidate-only。`planner_max_tokens = 256`、provider `max_tokens = 1024`、cross-model concurrency policy は変更しない。

## Pre-live gate

provider credential を使う前に、exact control/candidate coordinate、candidate からの `Cargo.toml` / `Cargo.lock` / `crates` runtime diff なし、workspace version `0.4.1`、corpus checksum / surface checksum、v12 metric-lock no-diff、pair validator、validate-only、no-model/no-network preflight、exact CLI capability probe (control 3/3, candidate 4/4)、full Rust tests、provider tests、deterministic Python eval tests、paired orchestration tests、concurrency policy、fmt、clippy `-D warnings`、通常 PR CI をすべて通す。

これらが PASS するまで `natural-language-e2e-v29-freeze` は作成しない。

## Canonical live 順序

1. Mistral paired canonical を1回。
2. Mistral gate PASS の場合だけ cross-model workflow へ進む。
3. Gemini paired canonical と Gemma paired canonical を各1回。どちらも `paired_canonical_observation.py` を使い、control FAIL でも candidate canonical 1回を必ず観測する。
4. Groq candidate-only canonical を1回。
5. report、orchestration record、candidate diagnostic sidecar を artifact 保存する。
6. cross-model averaging なしで、変更していない v29 release gate を評価する。

v29 が FAIL した場合は immutable evidence として確定し、rerun / rescore / tune はしない。fresh successor は独立に正当化できる product change の後だけ作る。
