# Engine 0.6 evidence-target relevance calibration v9 結果

Status: **immutable FAIL**。canonical run `36106913331`、attempt 1、freeze tag `engine-0.6-evidence-relevance-calibration-v9-freeze`、freeze commit `79089d0f1981207e47055fede9b342a60cb6f9a7`。rerun / rescore / tag overwriteは禁止する。

## Required arm

Mistral `ministral-8b-latest` は47/47 operational完了、provider failure 0。wrong-target relevance retention 0、false relevance rejection 0、false-safe negative confirmation 0、false-positive target-local confirmation 0でhard safety boundaryは維持した。一方、materialized exactは38/47 (80.85%)、utility miss 9、expected Relevant left ambiguous 2でutility/materialization gateをFAIL。primary proposal exactは20/47 (42.55%)、negative confirmation exactは19/30 (63.33%)、positive confirmation exactは11/13 (84.62%)。v9は安全側だがabstain過多だった。

Groq `openai/gpt-oss-120b` はoperational incomplete。`01_exact_name_availability` と `02_acronym_alias` のpositive Text confirmationで24-token output budgetを使い切り、`finish_reason=length` のままmodel textが返らなかった。2件ともtyped `protocol` failureとなり、2連続operational failure circuitがcase 02後に開いて残り45件を抑止した。semantic missとしては採点しない。

## Replication arm

Google `gemini-3.5-flash-lite` はnon-gatingで2/47時点にoperational abort。v9のconfirmation seed (`case_seed XOR 0xa93c_2b41`) がproviderのsigned 32-bit integer範囲を超え、先頭2件のpositive confirmationがHTTP 400となった。これはsemantic failureではなくGoogle adapter側のseed-domain bugである。

## Successor方針

v10ではhard safety gateを緩めない。修正対象を次のように分離する。

- raw Text enum confirmationを廃止し、小さいtyped action-safety JSON contract + 最大1回のtransport-only JSON-object fallbackへ変更する。
- global identity subtypeを当てさせるのではなく、Harnessが必要なlocal action (`safe_to_reject` / `safe_to_accept` / abstain) を判定させる。
- rename / alias / successor / shared ownership等の明示的不確実性はabstainを維持する。
- Harnessの抽象seedをGoogle adapter内でprovider対応seed domainへ正規化する。
- shared 60秒case deadlineと2連続operational failure circuitは維持する。

このFAILからindependent holdoutは作らない。fresh successor canonical PASSまでholdout authoringは引き続き禁止する。
