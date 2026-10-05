# Engine 0.6 evidence-target relevance calibration v7 result

Status: frozen FAIL / operationally incomplete。rerun / rescore禁止。

## Frozen identity

- issue: #462
- freeze tag: `engine-0.6-evidence-relevance-calibration-v7-freeze`
- freeze commit: `965582b244630200e5a01632eab2312e5fc0214c`
- first/only Actions run: `36026148264`
- run attempt: 1
- required arm: Mistral `ministral-8b-latest` / Groq `openai/gpt-oss-120b`
- semantic contract: binding proposal v2 + target-first materialization v3
- case budget: 2 model calls / 192 output tokens / 60,000 ms
- run-level circuit: operational provider failure 2件連続でabort

Groq live arm実行中にworkflowがcancelされた。frozen identityはrerunしない。Mistral armは既に完走しており、その時点でutility miss 1件が存在したため、Groqが後続完走していてもv7はPASSできなかった。

## Mistral arm

- planned / completed: 26 / 26
- successful / failed provider cases: 26 / 0
- operational abort: なし
- materialized exact: 25/26
- wrong-target / unsafe relevance admission: 0
- false relevance rejection: 0
- expected-relevant left ambiguous: 0
- utility miss: 1
- provider-attempt telemetry incomplete observation: 0
- latency p50 / p95 / max: 642 / 956 / 975 ms

唯一のdisposition missは `21_unknown_rename`。

- expected proposal: `target=unresolved`, `relation=exact`
- observed proposal: `target=different`, `relation=different`
- expected disposition: `ambiguous`
- materialized disposition: `irrelevant`
- assessment path: `model_assisted`
- reasons: `harness_canonical_name_anchor`, `model_irrelevant`

これはunsafe relevance admissionではなくutility failure / stochastic false-rejection risk。candidateは新名称がtargetを置き換えたか supplied page では確定できない旨を明示しているため、意図したidentity stateはunresolvedのまま。

またv3 materializationの設計上のリスクも露呈した。candidate内にHarness target-name anchorが存在しidentity relationshipが不確実でも、advisory modelが `target_binding=different` を返すと `irrelevant` を確定できる。このasymmetryはseedを変えて再実行するのではなく、successor canonical前に見直す必要がある。

## Groq arm

Groq jobはlive observation途中でcancelされ、complete resultはないためv7ではnon-scorable。

保存checkpointではcancel前に先頭3 caseだけ完了した。

- `01_exact_name_availability`: exact / exact, 370 ms, HTTP 200
- `02_acronym_alias`: exact / exact, 2,736 ms, HTTP 200
- `03_expanded_alias`: exact / exact, 3,574 ms, HTTP 200

3件ともprovider attempt 1回、materialized exact。rate-limit telemetryでもrequest capacityは残っており、この3件で429は出ていない。ただし23件未観測のためGroq PASSとは扱わない。

## Decision

v7はimmutable failed/incomplete evidenceとして保持し、同tag / workflowはrerunしない。

fresh successor identityの前に以下を行う。

1. 後続Google attempt-telemetry diagnosticの「active quota signalなし」という結果を独立記録として保持する。
2. `21_unknown_rename`を1 case prompt tuningではなく一般的なidentity uncertainty設計問題として扱う。
3. Harness-owned provenanceでdistinct identityを確立していない状況でも、advisory modelの `different` だけでnegative target identityを `irrelevant` に確定してよいか見直す。
4. materialization policy変更前にfresh independently authored identity-ambiguity probeを使う。
5. v7 provider-role historyを明示し、今回をGroq failureとは扱わない。

independent holdout authoringは引き続き禁止。
