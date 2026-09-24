# Engine 0.6 evidence-target relevance calibration v6 result

Status: frozen FAIL。rerun / rescore禁止。

## Frozen identity

- issue: #462
- branch: `feat/462-evidence-target-relevance`
- freeze tag: `engine-0.6-evidence-relevance-calibration-v6-freeze`
- freeze commit: `c4c16dd4968b9418f0105fa42bf11895ef26bb1b`
- first/only Actions run: `36022978827`
- run attempt: 1
- suite: `evidence-relevance-calibration-v6`
- semantic contract: binding proposal v2 + target-first materialization v3
- case単位budget: 2 model calls / 192 output tokens / elapsed 60,000 ms
- run-level operational circuit: operational provider failure 2件連続でabort

v6はimmutable historical evidenceとしてFAILのまま保持する。operational hardening自体が設計どおり動作したことと、canonical acceptanceのFAILは分けて扱う。

## Mistral canonical arm

model: `ministral-8b-latest`

- planned / completed: 26 / 26
- operational abort: none
- successful / failed provider cases: 26 / 0
- materialized exact: 26/26
- wrong-target / unsafe relevance admission: 0
- false relevance rejection: 0
- expected-relevant left ambiguous: 0
- utility miss: 0
- provider-attempt telemetry incomplete observations: 0
- latency p50 / p95 / max: 554 / 863 / 944 ms

結果: PASS。

## Google canonical arm

model: `gemini-3.5-flash-lite`

- planned cases: 26
- circuit openまでのcompleted cases: 3
- successful / failed provider cases: 1 / 2
- failure: `assessment_timeout` x2
- operational abort: `consecutive_operational_failure_budget_exhausted`
- `03_expanded_alias` 後にcircuit open
- 次の未送信case: `04_semantic_paraphrase`
- 抑止した残りrequest: 23
- provider-attempt telemetry incomplete observations: 2

観測case:

1. `01_exact_name_availability`: success、47,242 ms、materialized exact。
2. `02_acronym_alias`: assessment timeout、60,001 ms。outer Harness deadlineがin-flight adapter futureをcancelしたためprovider-attempt countはincomplete。
3. `03_expanded_alias`: assessment timeout、60,001 ms。同様にprovider-attempt countはincomplete。

完了semantic observationでは以下を維持した。

- proposal exact accuracy: 1.0
- materialized exact accuracy: 1.0
- wrong-target / unsafe relevance admission: 0
- false relevance rejection: 0
- expected-relevant left ambiguous: 0
- utility miss: 0

結果: operational FAIL。完了caseからsemantic regressionは観測されていない。

## Hardening outcome

v6で導入したoperational hardeningは狙いどおり動作した。

- Google provider retryはboundedのまま、deterministic fallback delayをbounded equal jitterへ変更。
- runner側には第2のretry layerを追加していない。
- operational failure 2件連続でrun-level circuitが開き、残り23 requestを送らず停止。
- outer deadline cancellation時はprovider-attempt telemetry incompleteを明示。
- p50 / p95 / max latencyを追加。
- canonical workflow final gateはoperational incompleteで明示的にredとなり、artifact保存jobのgreenをacceptanceと誤認しない。

したがってhardeningのrollbackや60秒deadlineの単純延長はv6から支持されない。bounded retry jitterとload sheddingを入れてもGoogle 3.5 serving instabilityが継続したことを示す。

## v7前のgate-design implication

Googleをsemantic scoreが悪いから外すわけではない。完了Google observationはsemantic exactかつsafeを維持している。論点は、canonical runを繰り返しoperationally成立させられないproviderをrequired operational gateへ固定し続ける妥当性にある。

v7でrequired provider/modelを変更する場合、#462 calibration outcomeを見て候補を比較・選別してselection biasを入れない。既存repoの独立履歴ではGroq `openai/gpt-oss-120b` にrequired/referenceとしての完走実績があり、NVIDIA routine candidateにはprotocol / timeout incompleteの記録がある。v7 freeze前にcurrent provider capability / availabilityも再確認する。

Google v4/v5/v6は明示的なreplication / operational evidenceとして残し、gating role変更で過去結果を書き換えたり捨てたりしない。

fresh canonical calibrationがPASSするまでindependent holdout authoringは引き続き禁止。
