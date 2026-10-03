# Engine 0.6 evidence relevance successor-v10 development v2 result

Status: immutable development FAIL。

- Freeze tag: engine-0.6-evidence-relevance-successor-v10-development-v2-freeze
- Freeze commit: 44796d470fdbb359ea1356dad121aa9417a98cb7
- Run: 37100490212
- Candidate: effective qualification v14 / materialization v26
- Surface: fresh 18 case
- Required development providers: Mistral + Google
- Result: FAIL

v2 tag と observation は immutable development evidence。rerun / tag移動 / relabel / PASSへの再解釈は禁止。

## Provider result

Mistral / ministral-8b-latest:
- operational 18/18
- provider failure 0
- authority failure 0
- identity/risk failure 0
- materialized exact 17/18
- total tokens 30,796
- result FAIL

Google / gemini-3.5-flash-lite:
- operational 18/18
- provider failure 0
- authority failure 0
- identity/risk failure 0
- materialized exact 18/18
- total tokens 31,696
- result PASS

## Terminal miss

terminal miss は Mistral の sv10v2_18_requested_availability_with_feature 1件のみ。

Task: Where is Orchid Proxy deployable?

Candidate: Orchid Proxy deploys in Ridge zones and supports webhook retries.

Mistral:
- proposal: exact / exact
- raw local qualification: exact_target / different_relation / none
- v14 effective qualification: exact_target / requested_relation / none
- v26 materialized disposition: Irrelevant
- expected: Relevant

v14 の authority correction は正しい。失敗は v26 composition にある。

v26 は v14 を正しくderiveした後、baseline materializationを original raw local qualification のまま v23 に委譲する。そのため v23 が raw DifferentRelation を見て先に Irrelevant をterminal materializeし、v26 が corrected v14 RequestedRelation をfinal materializationへ反映できない。

Harness-owned v14 negative-relation cue のunsafe evidenceではない。両providerとも authority / identity-risk gate はclean。

## Next direction

v14 は変更しない。

次materializerは uncorrected raw qualification ではなく v14 effective qualification からcomposeする。

そのcandidateは次をdeterministic replayする:
- frozen successor-v9 expected contract
- frozen holdout-v10 expected contract
- canonical holdout-v10 全observations
- immutable v2 全observations

必須:
- known Groq v10h18 over-abstention -> Irrelevant
- Mistral sv10v2_18 -> Relevant
- Google v2 は全件不変
- その他 historical terminal disposition にunexpected change 0

これを通してから fresh independent v3 development surface を観測する。
