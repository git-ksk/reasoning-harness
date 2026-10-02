# Engine 0.6 evidence relevance calibration v22 result

Status: immutable canonical PASS。

- Freeze tag: `engine-0.6-evidence-relevance-calibration-v22-freeze`
- Candidate commit: `2e0268dc0fb565dab35d591e831aa5170ad75601`
- Canonical run: `36338187291`、attempt 1のみ
- Cases: 48
- Seed: `4626210`
- Required provider: Mistral + Groq
- Replication provider: Google
- Annotation protocol: `evidence-relevance-effective-qualification-v21`
- Materialization: v16

## Required Mistral

`ministral-8b-latest` は48/48完走し、operational abort / provider-arm latchなし。

- authority-qualified effective qualification: 48/48
- materialized disposition: 48/48
- wrong-target relevance retention: 0
- false relevance rejection: 0
- Relevant left Ambiguous: 0
- utility miss: 0
- provider attempt: 96/96 complete

## Required Groq

`openai/gpt-oss-120b` は48/48完走し、operational abort / quota latch / retry waitなし。

- authority-qualified effective qualification: 48/48
- materialized disposition: 48/48
- wrong-target relevance retention: 0
- false relevance rejection: 0
- Relevant left Ambiguous: 0
- utility miss: 0
- provider attempt: 96/96 complete
- observed total tokens: 120,426

意図的にslow pacingしたTPD planはoperationally成功した。v22はimmutableとして保持し、rerunしない。

## Google replication

`gemini-3.5-flash-lite` は48/48完走、materialization 48/48、correctness / utility miss 0。authority-qualified effective qualificationのみ47/48だった。

唯一のauthority missは `76_v13_sibling_different_relation_no_cue`。modelのraw local qualification自体は `distinct_target` / `different_relation` / `none` で正しかった一方、proposalが `target=different` / `relation=exact` だった。effective qualification v2がproposal側relationを優先し、正しかったraw relationを `requested_relation` に上書きした。target-negative terminalにより最終dispositionは正しく `irrelevant` を維持した。

したがってこれはGoogleの最終relevance判断失敗ではなく、Harness-owned effective-qualification compositionの欠陥。

## Final gate

freeze済みv22のrequired-provider契約ではtop-level gateを全てPASSした。

- required operational completeness: PASS
- required correctness: PASS
- required utility: PASS
- required materialization: PASS
- required qualification: PASS

v22は事前固定したMistral + Groq required定義の下でaccepted。

## Successor

v23ではprovider requirementを強化し、Mistral / Groq / Googleの3つを全てrequiredにする。48 frozen casesとmaterialization v16は維持し、conflicting proposal relationがlocal evidenceで裏付けられないときに、corroborated `distinct_target` + `different_relation` を保持するeffective-qualification compositionだけを変更する。Googleを通すためにstrict authority gateを緩和しない。
