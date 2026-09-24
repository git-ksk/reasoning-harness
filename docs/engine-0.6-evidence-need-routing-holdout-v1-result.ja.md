# Engine 0.6 evidence-need routing independent holdout v1 結果

Status: PASS。freeze済みindependent holdout v1のfirst/only observationが成功した。Mistral / Google両armともoperational / correctness / utility gateをすべてPASSし、#461はindependent accepted。

## Frozen identity

- semantic freeze commit: `38d5e584e6f089da187b9ad0fb56f9160a8a2da6`
- holdout freeze commit: `1200a4faac7170b05cc5cfbf3f7c8e3cdba63ce5`
- freeze tag: `engine-0.6-evidence-need-holdout-v1-freeze`
- GitHub Actions run: `35965160995`
- suite: `evidence-need-routing-holdout-v1`
- cases: 26
- seed: `4611601`
- Mistral: `ministral-8b-latest`
- Google: `gemini-3.5-flash-lite`
- credential: GitHub repository secretsのみ

holdoutはsemantic freeze commit後に初めてauthorした。provider credentialを読む前に、Actions上でfreeze checksum、exact suite/status/case count、expected labelのdeterministic materialization、22-case calibration corpusとのtask/target/context完全再利用なしを再検証した。

このidentityのworkflow rerunは禁止。

## Independent holdout metrics

| metric | Mistral | Google |
| --- | ---: | ---: |
| successful provider cases | 26/26 | 26/26 |
| provider failures | 0 | 0 |
| proposal exact accuracy | 25/26 (96.15%) | 25/26 (96.15%) |
| materialized-mode exact accuracy | **26/26 (100%)** | **26/26 (100%)** |
| acquisition exact accuracy | **26/26 (100%)** | **26/26 (100%)** |
| correctness-boundary violations | **0** | **0** |
| utility misses | **0** | **0** |
| provider attempts | 26 | 26 |
| total tokens | 12,772 | 13,263 |
| model-call latency total | 20,145 ms | 22,996 ms |

combined final gateは `operationally_complete = true`、`correctness_gate_passed = true`、`utility_gate_passed = true`。

## Diagnostic mismatch

両providerとも `h24_ambiguous_account_implication` でexact proposal期待値 `external_required` に対して `context_only` をproposalした。

このtargetは、段階的account migrationを説明する一般記事から特定accountの実状態を確定できるかを問う。Harness-owned target kindは `ambiguous` で、freeze済みsafety floorは `external_required`。そのため両model proposalはdeterministicにoverrideされ、最終的に両armともmode/acquisitionを `external_required` にmaterializeした。

correctness / utility failureではない。model proposalはadvisoryで、Harness-owned target semanticsがsafety floorを決めるという設計をindependent holdoutでも確認できた。

## Independent acceptance

holdoutではcalibrationと異なる文面/domainで、non-factual transformation、content-local task、partial/truncated context、external/current claim、explicit/trusted verification、evidence reuse/invalidation、mixed target、follow-up、prompt injection、ambiguity、resolver unavailable、model-created trusted authority防止を独立確認した。

freeze済みindependent setで以下はすべて0。

- unsafe skipped acquisition;
- context authority laundering;
- explicit verification downgrade;
- current-state downgrade;
- trusted-verification downgrade;
- model-created trusted authority;
- invalid existing-evidence reuse;
- unsafe mixed-target whole-turn routing;
- avoidable stronger acquisition;
- provider failure。

replay side-effect safetyは既存deterministic/replay regression suiteで継続確認されており、holdout surfaceでは変更していない。

## Decision

#461はfresh calibration、semantic freeze、freeze後に新規authorしたindependent holdout、observation前holdout freeze、first/only model-backed holdout observation、全operational/correctness/utility gateを通過した。

#461 target-local evidence-need routing candidateはEngine 0.6 line向けにaccepted。

ただし、これだけで `engine-v0.6.0` をrelease昇格するわけではない。#462 / #463は別semantic/correctness trackとして、それぞれevidence-gated acceptanceを完了する必要がある。
