# Natural-language E2E v32

## 目的

v32 は immutable な v31 r2 の次の fresh held-out successor である。v31 r2 (`natural-language-e2e-v31-freeze-r2`, `22a035ae7e3c88de679a950ab4429b391f9ebafa`, seed `98473`) は Mistral paired PASS、Gemma paired PASS、Groq candidate-only PASS、Gemini paired FAIL だった。Gemini candidate は Google free-tier quota 429 により 6/13 で operationally incomplete となったが、観測済み correctness-boundary violations は 0。v31 r2 は再実行・再採点しない。

v32 candidate は `eafa6015c0d092c629d286708cb745ac3c9d7859`。v31後に独立mergeした #335（Google structured quota-window classification）と #334（opt-in cross-process shared Google request pacer）だけを追加する。metric-v13、control `29a9e4be6273dbffeda324e15517dc64930ad315`、prompt/case semantics、correctness / authority / admission / verification / finalization boundary は変更しない。

## Fresh surface

- corpus: `natural-language-e2e-v32`
- seed: `99584`
- 13 fresh synthetic cases
- scoring: `natural-language-e2e-scoring-v32-metric-locked-v13`
- operational bounds: `natural-language-e2e-operational-bounds-v13`
- control: released v0.4.1 `29a9e4be6273dbffeda324e15517dc64930ad315`
- candidate: `eafa6015c0d092c629d286708cb745ac3c9d7859`

## Prospective execution infrastructure

Google lane は `REASON_GOOGLE_MIN_REQUEST_INTERVAL_MS=1500` と共有pacer pathを使い、複数 `reason` subprocess が同時に動いても request start を1本のlaneで直列化する。Gemma (`google/gemma-4-31b-it`) だけ、隣接するeligible stateless investigation caseを最大2 workerで実行する。adaptive follow-up、MCP non-promotion、session caseは直列のまま。reportはmanifest case index順にmaterializeする。

この並列化はthroughputだけを対象とし、provider retry count、semantic retry、scoring、case replacement、result-dependent rescheduling、control→candidate順序を変更しない。

## Release discipline

pre-live deterministic gatesと通常CIがすべてgreenになるまでfreeze/tagやlive labelを付けない。freeze後は Mistral pairedを最初に1回だけ実行し、PASS時のみcross-model gateへ進む。candidate operational failureはhard FAIL、cross-model averagingは禁止、whole-run retryは禁止。
