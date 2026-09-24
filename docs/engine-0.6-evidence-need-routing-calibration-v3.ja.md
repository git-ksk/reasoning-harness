# Engine 0.6 evidence-need routing calibration v3

Status: frozen v2後のtuning successor準備済み。22-case calibration corpusは変更しない。

## v3の理由

frozen v2（GitHub Actions run `35956178160`）は両providerのcorrectness gateをPASSし、v1で見つかったGoogle mixed-target over-routingも解消した。一方Mistral `15_followup_escalation`でutility missが1件残り、Harness-owned policyがordinary external/current-state verificationだけを要求しているのに`trusted_verification_required`をproposalした。

これは単なるprompt varianceではなくauthority boundaryの問題。trusted verificationはHarness-owned authority classであり、untrusted model proposalが新規作成してはならない。

詳細は[v2 result](engine-0.6-evidence-need-routing-calibration-v2-result.ja.md)。

## v3 semantic change

calibration corpus / expected labelは変更しない。

- Harness-owned target policyの`trusted_verification_required == true`の場合だけ、model-facing enumに`trusted_verification_required`を含める;
- Harness flagがfalseなら手動で渡されたtrusted proposalも無視する;
- ordinary `external_required`までのmodel escalationは引き続き可能;
- explicit verification / current state / target kind / trusted exact verificationのHarness hard floorは変更しない;
- runner identityは`evidence-need-routing-live-calibration-v3`。

これにより「modelはroutingを提案できるがtrusted authority requirementは作れない」という既存invariantを型で固定する。

## Frozen v3 surface

- freeze tag: `engine-0.6-evidence-need-calibration-v3-freeze`
- workflow: `.github/workflows/engine-0.6-evidence-need-calibration-v3-live.yml`
- corpus: unchanged 22-case `evidence-need-routing-calibration-v1`
- seed: `4610600`
- Mistral: `ministral-8b-latest`
- Google: `gemini-3.5-flash-lite`
- credential: GitHub repository secretsのみ
- checksum: `fixtures/evidence-need-routing-calibration-v1/surface-v3.sha256`

## Acceptance gate

両provider armで以下をすべて満たすこと。

- 22/22 operationally complete;
- provider failure = 0;
- correctness-boundary violation = 0;
- utility miss = 0。

correctness/utility gateを侵害しないexact proposal/mode/acquisition mismatchはdiagnosticとして残ってよい。

v3 PASS後に#461 candidate semanticsをfreezeし、その後初めてseparate independent holdoutをauthorする。v3 freezeはrerunせず、追加tuningが必要なら新しいversioned identityを作る。
