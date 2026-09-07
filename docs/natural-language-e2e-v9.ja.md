# Natural-language E2E v9 — exercised-lane successor

Issue #245 は `natural-language-e2e-v9` を frozen v8 の fresh successor として定義します。v8 は `a262eba0b12bef569a631910c2fa7e80dae94010` / `natural-language-e2e-v8-freeze` / Actions `34078381222` の immutable historical evidence のまま保持します。

v8 は frozen hard-correctness gate と rejection-validity gate を通過しました。ただし観測後監査で、MCP case が configured GitHub MCP lane を実際には呼んでおらず、dedicated adaptive case も意図した `no_result -> registry follow-up` を実行していないことが分かりました。v9 は v8 を修正・再採点・再実行しません。

## Frozen product coordinate

測定対象productは exact shipped v0.4.0 のままです。

- tag: `v0.4.0`
- commit: `50c750d976be63b4e489ba5d7d7f3225bdd839b8`
- natural output: `reason-natural-output-v4`
- session contract: `reason-session-v1`
- MCP adapter: `mcp_readonly_v3`
- provider/model: Mistral / `ministral-8b-latest`
- base seed: `49000`
- max tokens: `1024`
- inter-case pacing: `1500ms`

measurement-only fileはrelease commitから追加されますが、`Cargo.toml`、`Cargo.lock`、`crates/` はreleased product coordinateとの差分 `0` を必須にします。

## Fresh corpus

v9はfresh investigation 8件 + session 3件です。case/task/target/source/session markerは観測済みv1-v8と機械的にdisjointであることを確認します。

investigation dimensionはunique-safe selection、ambiguous tool selection、freshness、scope、adaptive follow-up、authority、identity、MCP generic-output non-promotionです。

MCP live laneはdigest固定のofficial GitHub MCP imageを使い、`refs/tags/v0.4.0` の `CHANGELOG.md` を読みます。dedicated follow-up laneはcacheを先に使い、`no_result` 後にregistryへ進むことを明示します。

## Pre-observation integrity

credential/live実行前に次を必須にします。

- v1-v8 observed surface untouched;
- fresh-markerの双方向disjointness;
- resolver capability 10件のno-model/no-network protocol validation;
- evidence source 7件、allowlisted 6件、intentional identity-negative 1件;
- mechanically admissible positive fixture 3件;
- intended rejection contract 4件;
- `mcp_exercised` と `no_result_followup` coverage contractを各1件だけ持つこと;
- status-aware `RequiresVerification` scoring と `reason-session-v1` fork semantics のregression;
- rejection期待なしのpositive caseでは `expected_rejection_observed = null` とすること。

## Scoring

### Hard correctness

`hard_correctness_gate_passed` は correctness-boundary violation `0` を要求します。unsupported final structured claim、unsupported/contract-invalid exposed factual text、expected-unknownのunsafe grounding、identity/authority/scope/freshness bypass、MCP self-promotion、session invalidation/replay violationを含みます。

### Measurement validity

`measurement_validity_passed` はproduct utilityとは分離します。次を必須にします。

- 11/11 complete、operational failure `0`。process failureだけでなく、成功result envelope内のtyped investigation action failureとinvestigation generation failureも含めて `0`;
- freshness/scope/authority/identity rejection coverage `4/4`;
- MCP live coverage `1/1`: target recall済みでconfigured MCP capabilityが少なくとも1回non-operational actionとして観測されること;
- adaptive follow-up coverage `1/1`: configured cacheが`no_result`となり、その後configured registryを呼び、follow-upが`applied_evidence`または`verification_progress`になること;
- session persistence/fork reconstruction valid、external replay `0`;
- token-usage case coverage `>= 0.70`。

`adoption_gate_passed` は hard correctness と measurement validity の両方がpassした場合だけtrueです。

### Utility

target recall、grounded-target coverage、false abstention、tool-selection success、useful follow-up、harness unique selection、irrelevant acquisition attempt、stop reasonはutility telemetryとして別報告し、観測後にgateへ合わせてtuningしません。

## Freeze discipline

corpus、checksum、evaluator、tests、scoring identity、provider/model、seed、budget、MCP coordinate、docs、workflowは最初のlive observation前にfreezeします。観測後のv9はimmutable historical evidenceとなり、semantic/measurement変更はsuccessor identityでのみ行います。
