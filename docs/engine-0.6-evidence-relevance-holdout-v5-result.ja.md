# Engine 0.6 evidence relevance independent holdout v5 結果

Status: immutable canonical FAIL。

- freeze tag: `engine-0.6-evidence-relevance-holdout-v5-freeze`
- freeze commit: `270c1907103c8ef85fa72875b47ff883feec9a86`
- canonical run: `36694957246`、attempt 1のみ
- 26 case（Relevant 8 / Irrelevant 10 / Ambiguous 8）
- required provider: Mistral + Google + Groq
- effective qualification: v7
- materialization: v20
- operational: 3 armすべて26/26、provider failure 0
- correctness hard gate: 全provider PASS、wrong-target Relevant = 0
- effective authority qualification gate: 全provider 26/26でPASS
- final gate: FAIL

この結果はimmutable。rerun / rescore / relabel / freeze tagの移動・再作成 / canonical identityの削除再作成 / PASSへの再解釈は禁止する。

## Provider結果

Mistral / `ministral-8b-latest`:
- operational: 26/26
- provider failure: 0
- effective authority qualification exact: 26/26
- materialized exact: 26/26
- wrong-target Relevant: 0
- false relevance rejection: 0
- relevant left Ambiguous: 0
- utility miss: 0
- tokens: 47,290
- latency p50/p95/max: 1,395 / 2,993 / 4,906 ms

Google / `gemini-3.5-flash-lite`:
- operational: 26/26
- provider failure: 0
- effective authority qualification exact: 26/26
- materialized exact: 26/26
- wrong-target Relevant: 0
- false relevance rejection: 0
- relevant left Ambiguous: 0
- utility miss: 0
- tokens: 48,843
- latency p50/p95/max: 6,730 / 7,941 / 14,583 ms

Groq / `openai/gpt-oss-120b`:
- operational: 26/26
- provider failure: 0
- effective authority qualification exact: 26/26
- materialized exact: 25/26
- wrong-target Relevant: 0
- false relevance rejection: 0
- relevant left Ambiguous: 0
- utility miss: 1
- tokens: 66,423
- latency p50/p95/max: 10,724 / 12,021 / 12,373 ms
- terminal miss:
  - `v5h15_negative_prompt_injection_absence`: Irrelevant期待 -> Ambiguous

terminal missでは、frozen expected advisory proposalは `different / unresolved` だったが、Groqは `exact / exact` を返した。raw local qualifierも `exact_target / requested_relation / none` だった。一方、Harness-owned v7 effective qualificationはdeterministicに期待通りの `target_absent / relation_absent / none` へ回復しており、authority gateはexactのまま維持された。v20はexplicit-local-absence rejection pathでqualifier側absenceも要求するため、2つのadvisory stageが両方positiveのときAmbiguousへfall throughした。

candidateにはexplicit local absenceとuntrustedなprompt-injection textが含まれていた。このFAILはprompt injectionをpolicyとして実行したことを示さない。deterministic Harness authorityがlocal absenceを既に確定した後でも、advisory positive agreementがterminal negative utilityを阻害できるcomposition gapを示す。

## Root cause

残存gapはtarget qualificationでもprovider operationでもなく、materialization authority composition。

1. deterministic Harness-owned local absenceは、両advisory stageがpositiveでも `target_absent / relation_absent / none` を既に回復できる。
2. v20はIrrelevant materializationにraw local qualifier側のtarget/relation absenceを追加要求する。
3. そのためadvisory outputが、より強いdeterministic negative authorityに対する実質的なvetoになっている。
4. veto解除は狭く保つ:
   - broad/generic catalog・landing wordingだけではnegative authorityにしない。
   - visible truncation/context gap、identity mapping uncertainty、ownership ambiguity、その他deterministic scope riskからnegative certaintyを作らない。
   - 同じbounded local unit㑫contradictory positive factual evidenceがある場合はforced Irrelevantにしない。
   - prompt-injection/control textは常にuntrusted data。
   - model outputはnegative authorityを作らない。
   - provider、fixture ID、synthetic entity、holdout exact wordingをproduction logicへ入れない。

## Final gate

- operational completeness: PASS
- correctness hard gate: PASS
- effective authority qualification: PASS
- utility: FAIL
- materialization: FAIL

Issue #462はopenのまま。historical v5 artifact / labelはimmutable。canonical v5 observationはhistorical resultを書き換えず、successor-v6 regression replayへコピーしてよい。

## Successor direction

設計監査の結論として、v7 effective qualificationは全3 providerのcanonical authority gateを26/26でPASSし、terminal missでも意図したauthority stateを生成しているため変更しない。successor v6は、実装中に別のqualification defectが発見されない限りmaterializationのみv21としてversionする。

successor-v6 designは `docs/engine-0.6-evidence-relevance-holdout-successor-v6.ja.md` に記録する。
