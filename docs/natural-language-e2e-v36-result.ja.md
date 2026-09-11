# Natural-language E2E v36 — v0.4.2 canonical release acceptance

Issue #263では、v0.4.2 release gateの最終fresh metric-v13 successorとしてv36をfreezeした。evaluation surfaceは意図的にproduct `main`へmergeせず、immutable freeze tagをevidence coordinateとして保持する。

## Freeze identity

- freeze tag: `natural-language-e2e-v36-freeze`
- freeze commit: `57bea659d472a103cc48d86ddee7dfe4a41de790`
- candidate product commit: `9497b563ad914fada13d33e0c1a7fee549a1f1de`
- released v0.4.1 control: `29a9e4be6273dbffeda324e15517dc64930ad315`
- seed: `738214`
- metric revision: `v13`
- operational bounds: `natural-language-e2e-operational-bounds-v13`
- acceptance中のCargo workspace version: `0.4.1`
- observation後のcanonical rerun: `0`
- post-freeze mutation: `0`

credentialを使う前に、pair validation、metric lock、fresh collision check、checksum、Google pacing validation、runner-level no-model integration validation、exact provider capability probe、full Python test、`cargo fmt`、`cargo clippy`、`cargo test`、通常PR CIをすべてgreenにした。

## Canonical結果

### Mistral — PASS

paired canonical Actions run `34564120392`、`mistral/ministral-8b-latest`。

released control / candidateとも13/13 complete、operational failure `0`、generation failure `0`、correctness-boundary violation `0`でpaired gate PASS。両rowともtarget recall `0.6`、tool-selection success `1.0`、false abstention `7`、avoidable follow-up stall `0`、trigger exposure `3`、eligible follow-upのmechanism conformance `1.0`。このfrozen Mistral sliceではreleased controlがfollow-up utilityの構造上限に到達していたため、predeclared gate上はexact preservationでrelease-validとなる。

### Groq — PASS

cross-model Actions run `34564672351`、candidate-only generic provider-parity row `groq/openai/gpt-oss-120b`。

candidateは13/13 complete、operational failure `0`、generation failure `0`、correctness-boundary violation `0`、target recall `1.0`、tool selection `1.0`、mechanism conformance `1.0`、trigger exposure `3`、avoidable stall `0`。released v0.4.1で欠けていたgeneric provider pathを、provider固有correctness/authority branchなしで閉じた。

### Gemini 3.5 Flash-Lite — PASS

cross-model Actions run `34564672351`、paired `google/gemini-3.5-flash-lite`。

control / candidateとも13/13 complete、operational failure `0`、generation failure `0`、correctness-boundary violation `0`でpaired gate PASS。

- control: target recall `1.0`、tool selection `0.6`、false abstention `7`、trigger exposure `0`、avoidable follow-up stall `3`;
- candidate: target recall `1.0`、tool selection `1.0`、false abstention `7`、trigger exposure `3`、avoidable follow-up stall `0`、mechanism conformance `1.0`。

final release gateのstrict utility improvementを担うrowである。v35 released controlで発生したstructured-planner JSON EOFは再発せず、frozen 6000ms Google request-start floorのもとでv34 free-tier 429 quota failureも再発しなかった。

### Gemma 4 31B — PASS

cross-model Actions run `34564672351`、`investigation_workers=2`を単一shared Google pacerの背後で使うpaired `google/gemma-4-31b-it`。

control / candidateとも13/13 complete、operational failure `0`、generation failure `0`、correctness-boundary violation `0`でpaired gate PASS。両rowともtarget recall `0.8`、tool selection `0.9`、false abstention `7`、avoidable follow-up stall `0`、trigger exposure `3`、mechanism conformance `1.0`。

v35のeval infrastructure failureは再発しなかった。Google shared request pacingは`6000ms`、inter-case delayは独立して`3000ms`を維持し、Gemma two-worker canonical pathは旧`pacing == inter_case_delay` invariantでpre-model exitせず、実model executionまで到達して完走した。

## Release判定

**v36 overall v0.4.2 release gate: PASS。**

cross-model averagingは使っていない。全required rowが独立にPASSし、candidate operational failureはhard gate、`INCONCLUSIVE`はnon-releasableのまま維持した。したがってv0.4.xのcorrectness、authority、admission、verification、finalization、answer-safety、MCP non-promotion、session replay boundaryを維持したままv0.4.2をreleaseできる。

より広いfinalization/grounding bridge（#248）、repeated-trial planner reliability（#282）、Harness-owned deterministic action materialization（#283）はv0.5.0のまま維持し、このpatch releaseには取り込んでいない。
## Release provenance

acceptance済みproduct coordinateはmain commit `d8940b4a98f11ec3e0968444fadc8bc90eae01ff`から`v0.4.2`としてreleaseした。release workflow `34570773900`はSUCCESSで、Linux x86_64 / macOS arm64 / macOS x86_64 / Windows x86_64をbuild・smoke・package公開し、`SHA256SUMS`も生成・公開した。GitHub Releaseはdocumented external-preview support policyどおりv0.x prereleaseとして維持する。
