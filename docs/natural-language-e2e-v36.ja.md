# Natural-language E2E v36

## 目的

v36 は immutable な v35 の fresh held-out successor。v35 freeze (`natural-language-e2e-v35-freeze`, `17fa551b5a2aea1100083274f7e542f8bf30ded6`, seed `776466`) は historical evidence として固定し、rerun / rescore / retune / reclassification は行わない。v35 の release disposition は **FAIL**。Mistral paired と Groq candidate-only は PASS、Gemini は candidate 13/13 clean に対して released control が structured-planner JSON EOF protocol failure 2件で operationally incomplete だったため INCONCLUSIVE、Gemma は control / candidate とも provider/model request 前に eval-runner invariant bug で終了したため INCONCLUSIVE / measurement not executed。さらに v35 では Google shared request-start floor 6000 ms により、v34 の free-tier 429 failure が消え、実効rateも約9〜9.7 calls/minに収まった。

v36 candidate は `9497b563ad914fada13d33e0c1a7fee549a1f1de`（#349/#350 merge後の main）。product/provider runtime crates は v35 candidate `dd66a4372cfb462f876ac3169ba91df8d4a7f436` から変更しない。prospective delta は eval infrastructure のみ。Gemma runner validation は Google request pacing を `parallel_execution_policy.google_request_start_interval_ms` と、inter-case delay を `provider_policy.inter_case_delay_ms` と独立に検証する。Gemma workers=2 では absolute shared pacer path 必須を維持する。

## Fresh surface

- predecessor: `natural-language-e2e-v35-freeze` -> `17fa551b5a2aea1100083274f7e542f8bf30ded6`
- corpus: `natural-language-e2e-v36`
- seed: `738214`
- fresh identity / source ref / fact key / marker を持つ13 synthetic cases。v35を含む全observed predecessorとのcollisionを検査する
- scoring: `natural-language-e2e-scoring-v36-metric-locked-v13`
- operational bounds: `natural-language-e2e-operational-bounds-v13`
- control: released v0.4.1 `29a9e4be6273dbffeda324e15517dc64930ad315`
- candidate: `9497b563ad914fada13d33e0c1a7fee549a1f1de`
- Cargo workspace version: `0.4.1`

## Prospective Google execution infrastructure

Google canonical request pacing は 6000 ms、inter-case delay は 3000 ms のまま。Gemini / Gemma model job は直列 (`max-parallel: 1`) を維持する。Gemmaのみ eligible stateless investigation case を2 workerで処理でき、adaptive follow-up、MCP non-promotion、session/stateful case は直列のまま。provider retryを含むrequest startはshared Google pacerを通す。

`validate_google_canonical_pacing.py` がrepository policyとworkflow/manifest wiringを検査し、さらに `google_parallel_runner_policy.py` をv36 runner自身がlive provider/model request前に実行する。`--runner-policy-only` により、`workers=2 + pacing=6000 + delay=3000 + absolute shared pacer` の組み合わせをmodel/network未使用でpre-live検証できる。

## Release discipline

pair validator、metric lock、pacing validator、runner-integration validation、fresh collision tests、checksum、no-model capability probe、full Python、cargo fmt/clippy/test、通常PR CIがすべてgreenになるまでfreeze/live launchは禁止。freeze後はMistral paired canonicalをexactly onceで起動し、PASSの場合のみcross-model canonicalへ進む。Groq / Gemini / Gemmaは独立必須row。candidate operational failureはhard gate、INCONCLUSIVEはrelease不可、cross-model averagingは禁止。観測後にv36設定を変えて同identityをrerunしてはならない。
