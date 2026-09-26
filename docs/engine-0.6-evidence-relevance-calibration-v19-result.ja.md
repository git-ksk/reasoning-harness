# Engine 0.6 evidence-target relevance calibration v19 — immutable result

Status: FAIL。最初で唯一のv19 canonical結果としてimmutable。v19はrerun / rescore / relabel / retagしない。

Freeze:
- commit: `471bc11a83ba36dbaf5cb69a5fa4550b0e35b578`
- tag: `engine-0.6-evidence-relevance-calibration-v19-freeze`
- canonical run: `36247789205`
- attempt: `1`
- preflight: success
- final gate: failure
- fixed core: `evidence-relevance-fixed-core-v1`（48件）
- annotation protocol: `evidence-relevance-effective-qualification-v19`

この結果を理由にcase追加・削除・relabelは行わない。

## Required Mistral

`ministral-8b-latest` は48/48完走、provider success 48、failure 0、provider attempt 96/96、model call 96、total token 84,234。

Semantic結果:
- proposal exact: 36/48（75.00%）
- raw verifier exact: 26/48（54.17%）
- raw scope-risk miss / spurious: 9 / 1
- raw identity-scope miss: 13
- raw relation-scope miss: 14
- effective qualification: 48/48 exact
- effective risk / identity / relation miss: 0
- materialized exact: 48/48
- wrong-target Relevant: 0
- false relevance rejection: 0
- Relevant -> Ambiguous: 0
- utility miss: 0

v19 authority splitは機能した。raw verifier disagreementは観測可能なまま保持しつつ、Harness-owned effective qualificationと最終materializationはfrozen coreへ完全一致した。

## Required Groq

`openai/gpt-oss-120b` は10件success後、`63_fresh_stale_contradiction_relevant` のlocal qualification中にtyped daily quotaへ到達。provider armを即latchし、残り37件を抑止した。

Operational evidence:
- successful provider cases: 10
- failed provider cases: 38
- 最初の10 successful observation: effective qualification 10/10、materialization 10/10 exact
- provider attempts: 22/22 completed
- model calls: 22
- latch前total tokens: 25,505
- active execution: 12,460 ms
- pacing wait: 179,197 ms
- retry wait: 0 ms

quota circuitは設計どおり動作し、confirmed daily quota 1回でarm latch、retry stormなし、semantic failureへの変換なし。required Groq armはoperationally incomplete / non-scorableなのでv19はPASSできない。

## Google replication

`gemini-3.5-flash-lite` はnon-gating。46件success、timeout 2件（`05_title_identity_body_relation`、`27_explicit_not_rename_distinct`）。

Results:
- provider attempts: 107 started / 105 completed
- model calls: 95
- total tokens: 84,033
- proposal exact: 34/46
- effective qualification: 43/46 exact
- effective relation-scope miss: 3
- effective risk / identity miss: 0
- materialized exact: 45/46
- wrong-target Relevant: 0
- false relevance rejection: 0
- Relevant -> Ambiguous: 0
- utility miss: 1

唯一のmaterialization missは`13_same_service_different_feature`。expected Irrelevantに対してAmbiguous。effective qualificationは`exact_target + different_relation + no risk`を確定した一方、primary proposalは`target=exact, relation=unresolved`だった。これはv19 live前から候補として残していたexact-target / independently-established-different-relation境界そのもの。

3件のeffective relation-scope mismatchは`25_insufficient_local_passage`、`26_url_only_identity`、`80_v13_clipped_relation_context`。いずれもnon-none `context_gap` riskを保持し、最終dispositionはfrozen expectedどおりAmbiguous。unsafe terminal outcomeではなく、provider-sensitiveなnormalization disagreement。

## Cross-provider conclusion

v19の主要semantic設計は支持された。Mistralはeffective qualification / materialization 48/48、Groqはquota前の10件が全exact、Googleもunsafe Relevant 0。canonical FAILの主因はrequired providerのdaily quotaによるoperational incompleteness。

## v20 direction

v20をv19の偽装rerunにせず、PASS目的でgateも緩めない。successor方針:
1. fixed 48、labels、primary proposal v5、typed local-risk floor、raw/effective telemetry分離、positive-admission ruleを維持;
2. Mistral + Groq required、Google full non-gating replicationを維持;
3. typed quota fail-fast + immediate arm latchを維持。v19 rerunなし、manual quota probeをacceptance evidenceにしない;
4. semantic delta候補は事前宣言済みの1つだけ。exact target + independently established different relation + no safety riskなら、primary relationがunresolvedでもIrrelevant terminalを許可する;
5. context-gap relation normalizationは、case固有phraseやfail-closed弱体化なしでfrozen expectation全件を再現できるgeneric property/replay ruleが証明されるまでdefer;
6. v20 freeze前にimmutable v19 Mistral / Groq successful / Google successful replayでcorrectness regression 0を要求;
7. first/only frozen v20 canonical PASSまでindependent holdout authoringは禁止。

## Final decision

required operational completeness=falseのためtop-level acceptanceはFAIL。aggregate correctness / utility / materialization / qualification gateもfinal-gate上false。v19はimmutableで再実行禁止。
