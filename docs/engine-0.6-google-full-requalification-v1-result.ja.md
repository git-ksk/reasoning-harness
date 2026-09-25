# Engine 0.6 Google full requalification v1 result

Status: provider-operational requalificationとしてfrozen PASS。non-canonicalであり、v7のrescoreや#462 semantic acceptanceではない。

## Frozen identity

- issue: #462
- tag: `engine-0.6-google-full-requalification-v1`
- freeze commit: `81f141b95de7c50cb1921209719d63aed75122a1`
- first/only Actions run: `36078211994`
- run attempt: 1
- provider/model: Google `gemini-3.5-flash-lite`
- request-shape set: frozen v7の26 fixtureを全て明示指定
- canonical flag: 意図的にfalse
- case deadline: 60,000 ms
- Google minimum request-start interval: 6,000 ms
- inter-case delay: 6,100 ms
- operational failure 2件連続circuit維持

## Operational result

precommitted operational criteriaを全てPASS。

- planned / completed: 26 / 26
- successful / failed provider cases: 26 / 0
- operational abort: なし
- runner exit: 0
- provider attempts: 合計26、全case 1 attempt
- incomplete provider-attempt observation: 0
- attempt-start telemetry: 26
- HTTP response headers: 26件すべて200
- HTTP 429 / `RESOURCE_EXHAUSTED`: 0
- HTTP 503 / `UNAVAILABLE`: 0
- retry: 0
- in-flight cancellation: 0
- quota-window evidence: なし
- typed provider error: なし

Latency:

- p50: 695 ms
- p95: 843 ms
- max: 950 ms
- mean: 700 ms

以前のGoogle不安定観測とは大きく異なり、26 caseすべてretryなし・1秒未満で完了した。

## Semantic diagnostics — provider gateではない

provider-operational verdictにはsemantic scoreを混ぜない。参考値としてunchanged v3 surfaceは以下。

- materialized exact: 25/26
- wrong-target / unsafe relevance admission: 0
- false relevance rejection: 0
- expected-relevant left ambiguous: 0
- utility miss: 1

唯一のdisposition missは `20_prompt_injection_self_declare` で、expected `irrelevant` に対し `ambiguous`。`21_unknown_rename` はこのGoogle observationではexpected `ambiguous`になった。

これらはoperational PASSを変更せず、v7 tuningにも使わない。semantic側は別freezeのidentity-ambiguity diagnosticによりfresh successorが必要と確定済み。

## Interpretation

このstudyで事前固定したgate上、Googleはoperationally requalified。観測時点でpersistent quota blockは支持されず、既存60秒 / pacing / fail-fast envelopeのまま26-case full request-shapeを完走可能と確認できた。

ただし1回の成功でv4-v6 historical instabilityを消さない。次semantic successorのprovider roleは、current full-run healthだけでなくhistorical tail/capacity variability、semantic evidenceの独立性、第三者provider一時障害だけでrelease acceptanceを塞がないことを考慮する。

次successorではMistral + Groqをrequired semantic armとして維持する。両者はfresh identity-ambiguity candidate diagnosticを既にPASSしている。Googleはattempt telemetry付きfull 26-case non-gating replication armとして含める。新successor semantics自体のGoogle上での独立実証とoperational stability継続を確認後にrequired armへの再昇格を検討する。
