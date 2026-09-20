# Harness Engine 0.5.0 release

Harness Engine 0.5.0 is the first independently versioned Engine source release after the final unified `v0.4.2` release.

## Coordinate

- package owner: `reasoning-harness-core`
- Engine version: `0.5.0`
- release tag: `engine-v0.5.0`
- release namespace: `engine-v*`
- Reason CLI package version at this source release: `0.5.2`
- provider implementation crate version: `0.4.2`

The already-published `reason-v0.5.2` binaries remain immutable and continue to report Harness Engine 0.4.2. Engine 0.5.0 is not retrofitted into those artifacts; a later Reason CLI release may adopt it explicitly.

## Accepted semantic delta

Engine 0.5.0 includes the accepted verified-investigation line plus final hardening:

- finalization/grounding bridge and authority-bound exposed factual output;
- repeated-trial planner reliability and deterministic executable-action ownership;
- planner target-recall telemetry separated from finalization correctness (#445);
- deterministic explicit-fact session correction continuity (#446);
- deterministic admitted exact-fact investigation materialization after validated read-only acquisition (#450).

No provider/model-specific correctness branch was added. Evidence admission, verification, answer-safety, fail-closed behavior, and external-call replay boundaries remain Harness-owned.

## Release evidence

The release gate is the immutable `engine-0.5-final-v3-freeze` surface:

- freeze commit: `063833f38c38225109586b3db92348563b3822f8`
- product candidate: `d60b9afdf0bb2a0c1986f8c8f7cb47e534a4cd90`
- canonical GitHub Actions run: `35457038163`
- required rows: Mistral 14B, Qwen 3.8 27B, Mistral 8B, Gemini 3.5 Flash-Lite, Gemma 4 31B, GPT-OSS 120B
- result: 6/6 rows, 18/18 cases, correctness-boundary violations 0, session external replay 0

See [Engine 0.5.0 final-v3 result](engine-0.5-final-v3-result.md) and the preserved raw evidence under `docs/observations/engine-0.5-final-v3/`.

## Historical note

The earlier `engine-0.5.1-hardening-v1-freeze` identity was created before confirming that Engine 0.5.0 had not yet been version-released. It remains immutable pre-release evaluation provenance only and is not an Engine 0.5.1 release or release candidate.
