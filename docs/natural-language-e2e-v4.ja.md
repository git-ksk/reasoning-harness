# 自然文 investigation / session E2E v4

Issue #214 v4はv3 run `34031245504`の**観測前successor**である。v3は10/10 operationally completeで、admission境界を維持し、unsupported structured/text-only assertion 0、external replay 0だったが、evaluatorがserialize済み`ReasoningThreadEvent.kind`のnested enum objectをflat stringとして読んだため、成功していたsession 2件を誤採点した。v4はevent shapeの解釈だけを修正し、nested serde形を直接検証するregression testを追加する。10ケース、provider/model/seed/token policy、`defd404`のproduct runtime、correctness boundaryは変更しない。

## Freezeするidentity

- corpus: `natural-language-e2e-v4`
- evaluator/report: `reason-natural-language-e2e-v4`
- scoring: `natural-language-e2e-scoring-v4`
- natural output: `reason-natural-output-v3`
- session contract: `reason-session-v1`
- canonical provider policy: Mistral / `ministral-8b-latest`、base seed `41000`、max tokens `1024`、case間 pacing `1500 ms`

case N は `41000 + N`（0始まり）を使い、modelを使うsessionの2 turn目はその次のseedを使う。fixture hash、evaluator、scoring policy、deterministic preflightをcommitしてfreezeする前にlive観測してはならない。

## Corpus

全10ケース。明示`--hypothesis`なしの自然文investigation 7件とsession 3件で構成する。investigationは正常grounding、紛らわしいread-only capabilityを含むtool selection、stale rejection、scope mismatch、no-result後follow-up、authority claim mismatch、source identity mismatchを含む。sessionはevidence追加、前提訂正、resume/fork replayを含む。

外部取得はcommit済みdeterministic read-only fixture resolverだけを使い、network accessは行わない。liveで揺れるのはmodel planner/candidate/rendererだけで、freshness/scope/authority/identityの失敗はfixture側で意図的に固定する。

## Scoring

target recall/omission、relevant capability selection、useful follow-up、irrelevant attempt、typed admission rejection、target grounding/false abstention、round/tool call、公開contractから取得できるmodel token/latency、process wall-clock、session invalidation/replay、operational failureを別々に報告する。

exposed-text correctnessは機械判定する。`finalization.text`はcanonicalな`key = value`または`uncertain(key = value)`だけで構成され、各assertionはfinal Harness artifactで同じ強さ以上にsupportされていなければならない。free-form追加文はexposed-text contract violationとして数える。この指標は`factual_claims - covered_claims`と別なので、text-only assertionはstructured claim coverageに隠れられない。

session v1のpublic operation envelopeはturn単位のprovider token usageを公開しない。そのためreportに`token_usage_case_coverage`を出し、single-turn investigationのtokenは正確に測定、sessionはwall-clock latencyを測る。推定tokenは作らない。

## Adoption gate

`correctness_boundary_violations = 0`を必須とする。unsupported structured claim、unsupported/free-form exposed assertion、expected-unknown safety caseのunsafe grounding、session invalidation/replay violationを含む。target recall、tool selection、useful follow-up、false abstentionなどutilityは観測するが完全でなくてもよい。

operational failureはsemantic/correctness denominatorから分離し、別に報告する。不完全runを`unknown`へ変換しない。freeze identityの下で最初にoperationally completeになったrunをcanonical observation候補とする。

v4では**raw-model比較を主張しない**。将来raw armを追加する場合はmatched task/contextを必須とし、新しいcomparison/scoring identityを使う。

観測後にcase、expected outcome、scoring、evaluator semantics、provider policyを変える場合はv4を書き換えずsuccessor identityを作る。
