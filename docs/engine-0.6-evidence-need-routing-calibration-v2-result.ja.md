# Engine 0.6 evidence-need routing calibration v2 結果

Status: frozen v2 observationはSUCCESS。両provider armでcorrectness hard gateはPASSしたが、Mistralにutility over-escalationが1件残ったためsemantic freeze / holdout authoringはまだ行わない。

## Frozen identity

- freeze tag: `engine-0.6-evidence-need-calibration-v2-freeze`
- candidate commit: `e4f57c347ef8bfc9c15d6dc19dcf7f88e70337d5`
- GitHub Actions run: `35956178160`
- corpus: unchanged `evidence-need-routing-calibration-v1`
- cases: 22
- seed: `4610600`
- Mistral: `ministral-8b-latest`
- Google: `gemini-3.5-flash-lite`

preflight、両live job、combined final correctness gateはすべてSUCCESS。

## Raw v2 metrics

| metric | Mistral | Google |
| --- | ---: | ---: |
| successful provider cases | 22/22 | 22/22 |
| provider failures | 0 | 0 |
| proposal exact accuracy | 19/22 (86.36%) | 21/22 (95.45%) |
| materialized-mode exact accuracy | 20/22 (90.91%) | 22/22 (100%) |
| acquisition exact accuracy | 20/22 (90.91%) | 22/22 (100%) |
| correctness-boundary violations | 0 | 0 |
| utility misses | 1 | 0 |
| provider attempts | 22 | 22 |
| total tokens | 10,212 | 10,686 |
| model-call latency total | 15,968 ms | 23,770 ms |

## v2で確認できた改善

v1 scorerのfalse-positiveは解消。Mistral `17_external_optional`は再び`context_only`をproposalしたが、runnerはminimum Harness-permitted routeも`context_only`と記録するため、明示許可downgradeをcorrectness/utility failureにしない。

v1の実mixed-target over-routingも解消。Googleは`09_mixed_summary_target`をover-routeせず、materialized mode / acquisitionとも22/22 exact、utility miss 0。

## 残存utility finding

Mistral `15_followup_escalation`は次の状態。

- exact target: `Verify the current official status.`
- Harness flags: explicit verification = true、current state = true、trusted verification = false
- expected / minimum permitted mode: `external_required`
- model proposal: `trusted_verification_required`
- materialized mode: `trusted_verification_required`

correctness上は安全だが、必要以上に高コスト/高authorityのrouteになっている。さらに`trusted_verification_required`はHarness-owned trusted-authority requirementであり、untrusted model proposalがこれを新規作成できる状態は「model cannot create authority」というarchitecture invariantと衝突する。

## v3 requirement

calibration corpusは変更せず、trusted-verification proposal availabilityをHarness-ownedにする。

- `trusted_verification_required == false`ならmodel-facing schemaから`trusted_verification_required`を除外;
- `trusted_verification_required == true`ならmodeを許可し、Harness hard floorもtrusted verificationのまま;
- ordinary `external_required`までのmodel escalationはadvisory/safety-conservativeとして維持可能;
- model単独でtrusted authority classを新規導入できない。

両frozen calibration armでcorrectness-boundary violation 0かつutility miss 0になるまでindependent holdoutはauthorしない。
