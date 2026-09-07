# Natural-language E2E v8 — 修正版 successor independent v0.4.0 measurement

Issue #243 は、観測済み v7 の corpus wiring defect を受けた fresh successor `natural-language-e2e-v8` を定義します。v7 は historical evidence として固定します。Actions `34077700963` は 11/11 case 完走、operational failure `0`、frozen-v7 correctness violation `0` でしたが、external-command 7系統で resolver は fresh `fixture:v7:*` source を返す一方、admission source-map key が historical `fixture:v6:*` のままでした。product は正しく `untrusted_source` で fail-closed しましたが、意図した acquisition/admission dimension は測れていません。v8 は v7 を修正・再採点・再実行しません。

## 固定する identity と product coordinate

- corpus: `natural-language-e2e-v8`
- evaluator/report: `reason-natural-language-e2e-v8`
- scoring: `natural-language-e2e-scoring-v8`
- product tag: `v0.4.0`
- product commit: `50c750d976be63b4e489ba5d7d7f3225bdd839b8`
- natural output: `reason-natural-output-v4`
- session contract: `reason-session-v1`
- MCP adapter: `mcp_readonly_v3`
- provider/model: Mistral / `ministral-8b-latest`
- base seed: `47000`
- max tokens: `1024`
- inter-case pacing: `1500 ms`

測定対象product sourceはreleaseから変更しません。v8の変更はmeasurement側だけです。

## successorで固定する修正

v7で直した次の2点はそのまま維持します。

- `RequiresVerification` の uncovered proposition は blocked diagnostic/control state として `blocked_unverified_propositions` に分離し、answer-emitting finalization の unsupported structured claim と混同しない。
- `reason-session-v1` fork は selected checkpoint reconstruction、independent lineage、source非破壊、external replay `0` を採点し、source turn finalization継承を要求しない。

v8ではさらにv7のcorpus construction gapをlive前に閉じます。

### 双方向 historical marker 排除

fresh ID/task/target keyがhistorical側に存在しないことだけでなく、観測済みv1-v7の `fresh_markers` がv8 corpus/config内に残っていないことも検査します。これによりfresh config内へhistorical source identityが混入する事故を検出します。

### Admission wiring preflight

no-model/no-network preflightで全fixture resolverを実行し、実際のresolver outputをconfigured admission contractへ突き合わせます。

- evidence source identityをresolver outputから直接確認
- positive/tool/follow-up evidenceはallowlistedかつfreshness/scope/authority上mechanically admissibleであること
- freshness negativeはsource allowlist成功後にstaleであること
- scope negativeはsource allowlist成功後にrequired scopeを外すこと
- authority negativeはsource allowlist成功後に意図したauthority mismatchを持つこと
- identity negativeだけは意図的にnon-allowlisted sourceを返すこと
- no-result capabilityはevidenceを捏造せずfollow-up入力として成立すること

frozen v8 preflightの期待値は resolver capability `10`、evidence source check `7`、allowlisted evidence source `6`、intentional identity-negative `1`、positive admissible fixture case `3`、intended rejection contract `4` です。

## Fresh corpus

v8はfresh investigation 8件 + session 3件です。unique-safe selection、ambiguous tool selection、freshness、scope、adaptive follow-up、authority、identity、MCP generic-output non-promotionを測ります。

MCP laneは観測済みv6の`Cargo.toml`、v7の`Cargo.lock`を避け、pinned `v0.4.0` の `README.md` を使います。official image digest pinとread-only proofは維持します。

## Scoring / live acceptance

hard correctness gateはunsupported final structured claim、unsupported/contract-invalid exposed factual text、expected-unknown unsafe grounding、missed insufficiency、identity/authority/scope/freshness bypass、MCP self-promotion、session invalidation、external replayを `0` に保ちます。

さらにv8 live gateでは、意図した admission rejection 4件がすべて観測されることを `4/4`、coverage `1.0` として必須にします。これはutility gateではなく、主張したrejection dimensionを実際に測れたことを保証するmeasurement-validity gateです。target recall、grounded target coverage、tool selection、useful follow-up、false abstention等のutilityは別報告し、観測後にtuningしません。

## Freeze / observation rule

初回liveより前にcorpus、SHA-256、evaluator、tests、scoring identity、provider/model、seed、token/pacing budget、MCP coordinate、docs、workflowをfreezeし、`natural-language-e2e-v8-freeze` tagを付けます。

最初のoperationally complete live observation後はv8もimmutable historical evidenceです。post-hoc rescore/tuningは行いません。v1-v7は変更しません。
