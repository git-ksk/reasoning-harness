# Engine 0.6 evidence-target relevance calibration v21 — immutable result

Status: FAIL。first/only frozen v21 canonicalとしてimmutable。v21はrerun / rescore / relabel / retagしない。

Freeze:
- commit: `061aa323a2d5037511546a2eba4117292fa9cf4f`
- tag: `engine-0.6-evidence-relevance-calibration-v21-freeze`
- annotated tag object: `759655b1db1e093552c95dc6badce04a784ee36f`
- canonical run: `36295631123`
- run attempt: `1`
- preflight: success
- final gate: failure
- fixed core: `evidence-relevance-fixed-core-v1`（48件）
- annotation protocol: `evidence-relevance-effective-qualification-v21`

この結果を理由にfrozen case / label / expected disposition / v21 semantic ruleは変更しない。

## Pre-canonical Groq readiness

v21 freeze直前のsynthetic readiness run `36295565985` はtiny transport probe 3/3をHTTP 200で完走した。
- probe 1: TPM remaining 7,840/8,000、RPD remaining 999/1,000
- probe 2: TPM remaining 7,883/8,000、RPD remaining 998/1,000
- probe 3: TPM remaining 7,883/8,000、RPD remaining 997/1,000

これは小さいrequestがその時点で通ることしか証明せず、48-case Groq arm完走に必要なTokens Per Day (TPD) headroomは証明していなかった。v21により旧readiness定義はcanonical admissionには不十分と確定した。

## Required Mistral arm

Model: `ministral-8b-latest`。

Operational:
- 48/48完走
- provider success 48 / failure 0
- provider attempts 96/96 completed
- model calls 96
- input/output/total tokens: 81,544 / 2,686 / 84,230
- active execution: 52,855 ms
- provider/retry wait: 0 / 0 ms
- provider-arm latchなし

Semantic/materialization:
- proposal exact: 37/48（77.08%）
- raw verifier exact: 25/48（52.08%）
- raw scope-risk miss / spurious: 11 / 0
- raw identity-scope miss: 14
- raw relation-scope miss: 15
- effective qualification: 48/48 exact
- authority-qualified effective qualification: 48/48 exact
- effective risk / authority identity / authority relation miss: 0 / 0 / 0
- materialized exact: 48/48
- wrong-target Relevant retention: 0
- false relevance rejection: 0
- Relevant -> Ambiguous: 0
- utility miss: 0

Mistralはfrozen v21 surfaceに対してcomplete semantic PASS。

## Required Groq arm

Model: `openai/gpt-oss-120b`。

Operational:
- successful provider cases: 13
- failed / suppressed provider cases: 35
- `74_v13_exact_target_same_relation_no_cue`でtyped quota
- confirmed quota直後にprovider arm latch
- 残り34件を抑止
- provider attempts: 27/27 completed
- model calls: 27
- latch前input/output/total tokens: 28,319 / 3,329 / 31,648
- active execution: 15,420 ms
- provider/pacing wait: 219,931 ms
- retry wait: 0 ms

新しいpublic-safe `provider_diagnostic`により、account identityをredactしたまま決定的なGroq原文相当を保存できた。

`Rate limit reached ... on tokens per day (TPD): Limit 200000, Used 199298, Requested 1295. Please try again in 4m16.176s.`

同じ観測で以下も保存:
- `retry-after=257`
- RPD remaining 974/1,000
- TPM remaining 8,000/8,000
- token reset 1 ms

よってterminal 429はTPM/RPDではなくTPDと確定。256.176秒のretry intervalも、593 token不足を`200000 / 86400` token/secで補充する時間と一致し、0時一括resetではなく連続補充型daily token bucketの挙動と整合する。

latch前successful observationのsemantics:
- proposal exact: 13/13
- raw local qualification exact: 13/13
- effective qualification: 13/13 exact
- authority-qualified effective qualification: 13/13 exact
- materialized exact: 13/13
- wrong-target Relevant retention: 0
- false relevance rejection: 0
- Relevant -> Ambiguous: 0
- utility miss: 0

完了した13件にsemantic missは無かった。ただしfrozen 35件にprovider observationがないためrequired arm全体はnon-scorable。

## Google replication

Model: `gemini-3.5-flash-lite`。non-gating。

Operational:
- 48/48完走
- provider success 48 / failure 0
- provider attempts 96/96 completed
- model calls 96
- input/output/total tokens: 83,892 / 2,950 / 86,842
- active execution: 129,720 ms
- provider/pacing wait: 237,291 ms
- retry wait: 0 ms

Semantic/materialization:
- proposal exact: 37/48（77.08%）
- raw verifier exact: 28/48（58.33%）
- effective qualification exact: 45/48
- authority-qualified effective qualification: 47/48
- effective risk miss / spurious: 0 / 0
- authority identity miss: 0
- authority relation miss: 1
- materialized exact: 48/48
- wrong-target Relevant retention: 0
- false relevance rejection: 0
- Relevant -> Ambiguous: 0
- utility miss: 0

zero-risk authority relation mismatch 1件は`76_v13_sibling_different_relation_no_cue`。
- expected: `distinct_target + different_relation + none`
- primary: `target=different, relation=exact`
- raw verifier: `distinct_target + different_relation + none`
- effective: `distinct_target + requested_relation + none`
- final disposition: expected Irrelevant、materialized Irrelevant

既存target-negative terminalがrelation disagreementより優先されるため、安全性/utility missにはならない。その他all-axis effective mismatchはcase `25` / `80`のblocking `context_gap` relation diagnosticで、最終Ambiguousを維持。v22ではこれらnon-gating observationを理由にsemantic tuningしない。

## Cross-provider conclusion

v21は観測できた範囲でv20のauthority-conflict semantic failureを解消した。
- Mistral: complete 48/48 semantic/materialization PASS
- Groq: TPD latch前13/13 exact、semantic miss 0
- Google: complete 48/48 materialization PASS、correctness/utility miss 0

それでもrequired Groq armがoperationally incompleteのためcanonicalはFAIL。原因はorganization-level TPD admission rejection from insufficient instantaneous headroomと直接確認でき、直前の3-probe readinessはfull-run TPD headroomを測っていなかったため不十分と確定した。

Groqのhistorical token demandもtiny readinessでは不足することを示す。
- v20: 39 successful casesまでに97,259 tokens
- v21: 13 successful casesまでに31,648 tokens
- 現runner shapeで48件完走には概ね117K–120K tokensが必要と推定される

したがってsuccessorではv21 semanticsを維持し、quota結果へlabel/promptを合わせずcanonical admissionを変更する。

## Final decision

required top-level acceptanceはFAIL:
- required operational completeness: false
- required correctness gate: required set incompleteのためaggregate final-gate false
- required utility gate: required set incompleteのためaggregate final-gate false
- required materialization gate: required set incompleteのためaggregate final-gate false
- required qualification gate: required set incompleteのためaggregate final-gate false

v21はimmutable canonical FAIL。independent holdout authoringは禁止継続。
