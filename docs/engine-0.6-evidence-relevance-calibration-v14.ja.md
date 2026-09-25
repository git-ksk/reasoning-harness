# Engine 0.6 candidate: evidence-target relevance calibration v14

状態: immutable v13 FAIL の pre-freeze successor。v14 live model observation はまだ0回。independent holdout authoring は引き続き禁止。

## v14 の目的

v13 は open-world 3-state risk を binary observable cue に変えたが canonical は FAIL。

Required Mistral は 81/81 完走・provider failure 0 だった一方、materialized exact 61/81。blocking-cue miss 7、spurious cue block 5、wrong-target / false Relevant 1、false relevance rejection 2、Relevant -> Ambiguous 6、utility miss 19。

Google replication は 81/81 完走、materialized exact 65/81、blocking-cue miss 3、spurious cue block 4、Relevant -> Ambiguous 3、utility miss 16。

Required Groq canonical は strict-JSON Text local qualification が192-token completion budgetでtruncateし、3/81で停止した。

別IDのnoncanonical full-case postmortemは512-token Groq transport floor・consecutive-failure circuitなしで81/81まで完走した。このdiagnosticはnoncanonicalのままでv13 resultを変更しない。運用結果は成功1/81、`assessment_timeout` 80/81。ログではHTTP 429と数百秒のprovider `retry-after` が繰り返し観測され、adapterがrate-limit retry待機中にrunnerのshared 60秒case deadlineが先に切れていた。したがって192-token truncation修正後にも別のGroq課題があり、provider指定の長いrate-limit waitをsemantic/model timeoutと混同しない必要がある。

Groq公式は `retry-after` を秒単位、Freeの `openai/gpt-oss-120b` を8K TPM / 200K TPDとしている。adapterは明示的なdaily quota文言を `Quota` に分類済みで、v14では429 loopもhardeningし、daily quota exhaustionはtransient retryを消費せずfail-fast、一時的429は従来のbounded retryを維持する。

## Successor hypothesis

v13 guard は依然として仕事を持ちすぎていた。target support、relation support、blocking cue 3種、explicit local absenceを独立に再判定していた。

target/relation support はprimary proposalと重複し、freshness、relation mismatch、identity wordingなどrelevance以外の要因で不一致を作っていた。

fixed 48-case successor coreに対し、既観測のv13 proposal/cueをそのまま使い、materializationから `target_support` / `relation_support` だけを外すcounterfactual replayを行った。新しいmodel callは行っていない。

結果:
- Mistral: 32/48 -> 36/48 exact。既存正解caseの新規回帰0。
- Google: 37/48 -> 40/48 exact。既存正解caseの新規回帰0。

これはv14 PASSの証明ではないが、redundant support voteが実際のdisagreement sourceだったことを示し、field追加ではなくguard縮小を支持する。

## Fixed calibration core

v14 は `evidence-relevance-fixed-core-v1` を使用。

48 cases固定:
- Relevant 14
- Irrelevant 18
- Ambiguous 16

v14 live observation前にv13からsemantic coverageで圧縮した。model missを理由にcaseを追加しない。

compact blocker label:
- `none`: 32
- `identity_mapping`: 5
- `ownership_scope`: 1
- `context_gap`: 5
- `multiple`: 5

explicit local absenceは4件。

同じfixed coreを旧v13 cue表現で見ると、identity-mapping cue 6、ownership-scope cue 5、context-gap cue 10、explicit local absence 4を維持する。

production motivating contentは除外。

## Binding proposal v4

primary model:
- `target_binding`: `exact | different | unresolved`
- `relation_binding`: `exact | different | unresolved`

両軸は独立。

`identity_requirement=allow_semantic_equivalent` の場合、canonical product nameがなくてもlocally specificなsemantic descriptionがtargetとsemantic equivalentなら `target_binding=exact` を許す。

また、local text自身がidentity uncertaintyを明示する場合は `unresolved` を維持する。“may be an alias”、“does not establish whether X succeeds Y”、“does not state whether X replaces Y” を `different` にhardenしてはならない。

relationはownerと独立したcoarse relation kind。sibling productのpricingもpricing questionに対してrelation `exact`。generic landing、navigation-only target mention、explicit local absence、relation content欠落は `unresolved`。

freshness、truth disagreement、authority、sufficiency、untrusted page instructionはbinding labelを変えない。

## Compact local guard v4

independent guardは2 fieldだけ:

- `blocking_reason`: `none | identity_mapping | ownership_scope | context_gap | multiple`
- `explicit_local_absence`: `present | absent | unresolved`

target support / relation support は返さない。

`blocking_reason` はfinal bindingをAmbiguousに保つconcrete local ambiguityだけ:
- `identity_mapping`: alias / rename / successor / version / cross-language mappingがuncertain/conflicting
- `ownership_scope`: substantive row/value/sectionのownerがshared/unassigned
- `context_gap`: supplied material自身がclipped/truncated、referent omitted、または必要local contextが外部にあると明示
- `multiple`: 上記2種以上
- `none`: concrete ambiguityなし

wrong-target、wrong-relation、generic landing、navigation/footer、archived/stale value、factual disagreement、明示的 `distinct_from`、explicit target absence はそれ自体blockerにしない。

explicit local absenceはnegative evidenceでありcontext-gap blockerではない。

## Materialization v9

Harness final authorityはfail-closedのまま。

Ambiguous:
- blocking reasonが`none`以外
- policyがHarness-owned anchorを要求する場合のCanonicalUrl-only identity floor。semantic-equivalent policyでは incidental なURL identity stringだけを理由にblockしない
- missing primary proposal
- missing compact guard
- strict required identity anchorがないpositive exact/exact
- explicit local absenceなしでbindingがunresolved
- exact/exact proposal と explicit local absence `present` が矛盾する場合

Relevant:
- primary exact/exact
- compact blocker `none`
- strict policyならHarness-owned anchor、またはpolicyがsemantic equivalentを明示許可

Irrelevant:
- primary target `different` + blocker `none`
- primary exact target + relation `different` + blocker `none`
- exact/exact以外のnon-positive binding + explicit local absence `present` + blocker `none`

model fieldがHarness-owned strict identity floorを上書きすることはない。

## Provider transport / diagnostic completeness

Mistral / Google:
- JsonSchema primary
- structured capability / parse failure時だけstrict raw-JSON Text fallback最大1回

Groq:
- strict raw-JSON Text primary
- preceding JsonSchemaなし
- 両semantic stageでtransport completion floor 512 tokens
- malformed Textはtyped protocol failure。semantic repairなし
- transient HTTP 429は従来どおりbounded provider retry（rate-limit retry最大5回）
- `tokens per day` / daily limit・quotaの明示的枯渇はtyped `Quota` とし、transient retryを消費しない

全provider:
- JSON extractionなし
- Markdown repairなし
- field synthesisなし
- fuzzy repairなし
- semantic retryなし
- third model callなし
- 通常のmodel workに対するshared semantic per-case deadline 60秒は維持

v14 canonicalは `--continue-after-operational-failures` を使い、providerがoperationalな限り診断のためfixed core全caseをattemptする。ただしacceptanceは緩めない。provider failureが1件でもあればrequired operational completenessはFAIL。

v14 freeze前にはcalibration fixtureを一切読まないsynthetic Groq transport readiness workflowを別途実行する。shared organization/project quotaが既に枯れている状態でfirst/only canonical observationを消費しないためのpreflightであり、calibration observationではないためrepeat可能。

最初のv14 readiness observation（GitHub Actions run \`36172193572\`）はsynthetic probe 3/3 PASS。全call HTTP 200、\`Retry-After\`なし、\`x-ratelimit-limit-tokens=8000\`、\`x-ratelimit-remaining-tokens=7840\`、\`finish_reason=stop\`。artifactには \`calibration_observation=false\` / \`ready=true\` を明示しており、v13 postmortem後かつv14 calibration観測前にproviderがoperationalであることを確認した。

## Canonical roles / acceptance

Required:
- Mistral `ministral-8b-latest`
- Groq `openai/gpt-oss-120b`

Full non-gating replication:
- Google `gemini-3.5-flash-lite`

required armは個別に:
- 48/48 completed
- successful provider cases 48
- provider failures 0
- provider-attempt telemetry complete
- compact guard invoked 48/48
- blocking-reason miss 0
- wrong-target / false Relevant retention 0
- false relevance rejection 0
- expected Relevant left Ambiguous 0
- utility miss 0
- materialized exact 48/48

proposal exactness、compact guard full exactness、spurious blocker countはdiagnostic。user-visible final disposition exactnessとhard safety counterをauthorityとする。

first/only frozen v14 canonicalはrerun/rescoreしない。canonical PASSの場合のみfresh independent holdoutをauthorする。PR #466はcanonical / holdout / runtime acceptance全PASSまでDraft維持。
