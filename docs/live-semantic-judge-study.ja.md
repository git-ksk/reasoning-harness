# soft semantic-judge のライブ研究

Issue #33 は、model-backed soft semantic-judge path を反復 live provider study で検証する。ただし model output には correctness authority がないという原則は維持する。

## 研究設計

study では `fixtures/semantic-judges/` に commit された9 calibration cases を使った。

- positive-labelled cases 3件;
- negative-labelled cases 3件;
- 意図的に ambiguous な cases 3件;
- contradiction、unsupported-premise、causal-gap の diagnostic families.

live model は既存の Mistral `ModelAdapter` 経由の `ministral-8b-latest`。反復 study はどちらも5 trials、seed base `1000`、request あたり最大256 output tokens。provider credentials は manual GitHub Actions workflow に隔離した。

これらの cases は **calibration set であり holdout set ではない**。v2 prompt は同じ corpus で v1 の挙動を観測した後に変更した。そのため v2 result は calibration 後の protocol behavior を測るもので、semantic-judge generalization の unbiased evidence ではない。

## v1: 汎用的な棄権ガイダンス

GitHub Actions run `33307653357` は、5 complete trials で45 fixture calls を実行した。

Operational result:

- 45/45 successful fixture runs;
- 0 operational failures;
- 35,826 total tokens;
- 64 successful provider-generation attempts.

Complete-trial semantic distributions:

| metric | mean | min | max | stddev |
|---|---:|---:|---:|---:|
| precision | 1.000 | 1.000 | 1.000 | 0.000 |
| recall | 0.400 | 0.333 | 0.667 | 0.133 |
| decision coverage | 0.156 | 0.111 | 0.222 | 0.054 |
| abstentions per 9 cases | 7.6 | 7 | 8 | 0.490 |
| ambiguous abstention rate | 1.000 | 1.000 | 1.000 | 0.000 |

model は operationally reliable だったが、保守的すぎた。positive contradiction は5/5で検出した一方、positive causal-gap case は0/5、positive unsupported-premise case は1/5だけだった。ambiguous cases はすべて abstain した。

## v2: 診断種別の判定セマンティクス

v2 configuration (`soft-semantic-v2`) は typed diagnostic contract から導いた provider-neutral decision semantics を追加した。contradiction、counterexample、unsupported-premise、causal-gap requests における `finding`、`no_finding`、`abstain` の意味を定義するが、fixture-specific facts や authority path は追加しない。

GitHub Actions run `33307898636` は同じ model/corpus/trial configuration を反復した。

Operational result:

- 45/45 successful fixture runs;
- 0 operational failures;
- 43,016 total tokens;
- 64 successful provider-generation attempts.

Complete-trial semantic distributions は5 trialsすべてで同一だった。

| metric | value |
|---|---:|
| precision | 1.000 |
| recall | 1.000 |
| decision coverage | 0.667 |
| abstentions per 9 cases | 3 |
| ambiguous abstention rate | 0.667 |

labelled positive/negative behavior は大幅に保守的でなくなった。3つすべての positive cases は全 trial で検出され、negative cases の3つ中2つは全 trial で `no_finding` になった。残る negative contradiction case は一貫して abstain した。

ただし causal ambiguous case は5 trialsすべてで `finding` と分類された。ambiguous cases は precision/recall の confusion counts から意図的に除外されるため、precision と recall だけではこの挙動が隠れる。そこで #33 は `ambiguous_abstentions` と `ambiguous_abstention_rate` を明示的な metrics に追加した。

## 解釈

live study から導ける結論は4つに限られる。

1. provider-neutral model-backed soft-judge path は operationally viable である。2つの five-trial runs はともに45/45 fixture callsを provider/protocol failures なしで完了した。
2. typed output/authority contract が同じでも、semantic behavior は prompt に大きく依存する。
3. precision/recall と広い decision coverage だけでは calibration metrics として不十分であり、ambiguous-case behavior を可視に保つ必要がある。
4. これらの観測が hard authority を作ることはない。live decision はすべて `SoftJudgeObservation` のままで、既存の policy boundaries を通じた追加 evidence acquisition、deterministic verification、review の trigger にしかなれない。

v2 numbers を **general model quality として提示してはならない**。measurement に使った同じ9 casesで prompt を calibration したためである。次の semantic-quality study では、models の比較や reliability claims の前に、paraphrases、mixed evidence、unseen cases を含む別 holdout/expanded ambiguity corpus を使うべきである。

## 独立ホールドアウト v1

Issue #36 は、provider result を一切観測する前に、別の28-case、observation-free holdout corpus を freeze した。corpus は contradiction、unsupported-premise、causal-gap、counterexample families にまたがる11 positive、8 negative、9 ambiguous casesで構成される。最初の live study は merged `main` commit `c50aa5b822307096b08dcdf63826cd3d40ad0f7d` から実行し、result 観測後に holdout fixture や prompt の変更は行っていない。

GitHub Actions run `33314808691` は `ministral-8b-latest` を5 trialsで評価し、140 fixture callsを生成した。

Operational result:

- 140/140 successful fixture runs and 5/5 complete trials;
- 0 operational failures;
- 151,699 total tokens;
- 276,440 ms aggregate fixture latency;
- 210 successful provider-generation attempts;
- 70/140 successful runs used the harness JSON-object fallback path (`fallback_rate = 0.500`).

Complete-trial semantic distributions:

| metric | mean | min | max | stddev |
|---|---:|---:|---:|---:|
| precision | 0.909 | 0.909 | 0.909 | 0.000 |
| recall | 0.909 | 0.909 | 0.909 | 0.000 |
| decision coverage | 0.664 | 0.643 | 0.679 | 0.017 |
| ambiguous abstention rate | 0.778 | 0.778 | 0.778 | 0.000 |
| abstentions per 28 cases | 9.4 | 9 | 10 | 0.490 |

independent corpus は、calibration-set score では見えなかった安定した generic error classes を示した。semantic-equivalent wording を contradiction と過剰判定し、reverse-causality uncertainty を current label contract における directional causal-gap finding ではなく abstention として扱い、partial または incompletely scoped causal evidence を意図的に ambiguous な2 casesで過剰判定した。一部の negative cases も保守的に未決定のままだった。

この result は frozen holdout version 上の `soft-semantic-v2` に関する evidence にすぎない。model を correctness authority に昇格させず、広い model ranking も正当化しない。Issue #38 は calibration corpus を用いた generic contract からの semantic calibration を追跡し、holdout v1 は freeze したままにする。Issue #39 は、successful calls の半数で JSON-object fallback path が必要だった理由を別に追跡する。broader model matrix はこれらの follow-ups を gate とする。

## soft-semantic-v3 キャリブレーション結果

generic decision contract と18-case calibration corpus の merge 後、GitHub Actions run `33316513051` は `ministral-8b-latest` を5 calibration trials (90 calls)で評価した。これは calibration result であり、generalization の独立 evidence ではない。

90 callsはすべて成功し、5/5 trialsが complete だった。run は80,646 total tokens、143,197 ms aggregate fixture latencyを使用した。successful provider-generation attempts は121。JSON-object fallback は31/90 calls (`0.3444`) で使われ、すべて `invalid_primary_structured_output` に分類され、`primary_json_schema_unsupported` は0だった。

calibration corpus 上の semantic stability は precision `1.000`、recall `1.000`、mean decision coverage `0.622` (range `0.611`–`0.667`)、mean ambiguous abstention `0.971` (range `0.857`–`1.000`)だった。clear semantic equivalence と paraphrased premise support は5 trialsすべてで `no_finding`、explicit undistinguished reverse-causal alternative は5 trialsすべてで `finding`、partial intervention と incomplete causal scope cases は5 trialsすべてで abstain、明確に out-of-scope の counterexample は5 trialsすべてで `no_finding` になった。以前からある mixed causal calibration case の1つは5 trials中1回 `finding` になった。

v3 contract はこの corpus 上で calibration されたため、これらの数字は reliability claim に使わない。別の holdout-v2 は最初の v3 provider evaluation 前に freeze する。

## soft-semantic-v3 独立ホールドアウト v2 の結果

holdout-v2 を provider observations なしで merge・freeze した後、GitHub Actions run `33318380199` は28-case independent corpus (140 calls) 上で `ministral-8b-latest` を5 trials評価した。140 callsすべて成功し、5/5 trialsが operationally complete だった。run は135,669 total tokens、192 successful provider-generation attempts、243,208 ms aggregate fixture latencyを使用した。JSON-object fallback は52/140 calls (`0.3714`)で使われ、すべて `invalid_primary_structured_output`、`primary_json_schema_unsupported` は0だった。

5つの complete trials で precision と recall はともに variance なしの `1.000`。mean decision coverage は `0.700` (range `0.679`–`0.714`)、mean ambiguous abstention は `0.933` (range `0.889`–`1.000`)だった。19 labelled cases はすべて、5 trialsの全回で evaluator-owned expected decision に解決した。ambiguous case で繰り返し non-abstain decision になった唯一のものは `v2h20_causal_partial_payload_scope` で、3/5 trials は `finding`、2/5 は `abstain` だった。これは frozen corpus を tuning せず、scoped-causal boundary として可視に保つ。

この independent result は、この model に対する generic `soft-semantic-v3` contract を支持するが、soft-only authority boundary は変更しない。これだけで model portability は確立しない。次の semantic-judge research step は、同じ frozen contract と holdout の下で cross-model conformance を測定し、majority-vote truth source を選ぶのではなく models を interchangeable implementations として扱うべきである。

## soft-semantic-v3 モデル横断適合性の追跡調査

その後の holdout-v2 matrix は、同じ semantic contract に対する実装が大きく異なることを示した。`mistral-small-latest` は140/140 callsを完了し、precision 約`0.982`、recall `1.000`、decision coverage `1.000`、ambiguous abstention `0.000`。`gemini-3.1-flash-lite` は140/140完了し、precision/recall `1.000`、coverage `0.800`、ambiguous abstention 約`0.622`。`ministral-14b-latest` は135/140 callsを完了したが、毎 trial で同じ typed-output failure、つまり non-finding decision と finding object の併記を繰り返した。

初回 Nemotron 3.5 Lightning holdout run (`33321109608`) は complete semantic trial がなく、有効な semantic score ではない。content-free diagnostics では final JSON 前の reasoning-token truncation が主因だった。provider-neutral reasoning minimization は calibration (`33340942700`: 256 output tokensで14/18 successful) でこの confound を除去したが、finding bias、non-finding-plus-finding protocol failures、schema-completeness failures は残った。bounded-reasoning experiments は悪化したため merge していない。

したがって Issue #46 は matrix を model ranking ではなく contract-portability study として扱う。Issue #53 は、compact global three-way decision rule と stricter discriminated model-facing output schema を備えた、v3-semantics-preserving successor を導入する。`soft-semantic-v4` は、v4 provider measurement 前に freeze した新しい observation-free 28-case holdout-v3 と組み合わせる。[cross-model semantic judge conformance](semantic-judge-conformance.ja.md) を参照。

## soft-semantic-v4 独立ホールドアウト v3 の行列と棄却

`soft-semantic-v4` は、28-case holdout-v3 corpus と compatibility thresholds を freeze した後にのみ測定した。各 model は provider-safe fixture concurrency の下、256 output tokens で full-corpus trialsを5回要求した。

| model | run | operational | precision | recall | coverage | ambiguous abstention | fallback | tier |
|---|---:|---|---:|---:|---:|---:|---:|---|
| `ministral-8b-latest` | `33342332130` | 140/140, 5/5 complete | 0.889 | 1.000 | 0.714 | 0.667 | 0.429 | non-conformant |
| `mistral-small-latest` | `33342547879` | 140/140, 5/5 complete | 1.000 | 1.000 | 1.000 | 0.000 | 0.050 | non-conformant |
| `gemini-3.1-flash-lite` | `33342334655` | 140/140, 5/5 complete | 1.000 | 1.000 | 0.821 | 0.417 | 0.000 | non-conformant |
| `ministral-14b-latest` | `33342335857` | 140/140, 5/5 complete | 0.800 | 1.000 | 0.786 | 0.500 | 0.543 | non-conformant |
| `nvidia/nemotron-3.5-lightning-30b-a3b` | `33342337031` | 71/140; 69 protocol failures; 0/5 complete | n/a | n/a | n/a | n/a | 1.000 on successes | non-conformant |

matrix には conformant model も usable-with-limitations model も0であり、事前宣言した successor adoption gate は failed になった。cross-family failure pattern として最も強いのは uncertainty collapse である。ambiguous unsupported-premise と scoped causal cases が複数の Mistral/Google models で assertive findings になった。Ministral 8B は semantic-equivalence case で stable labelled false positive も出した。Mistral Small は abstention が一度もなかった。

discriminated schema は、分離可能な protocol property の1つを改善した。Ministral 14B は v3 の135/140から v4 の140/140へ移り、反復していた non-finding-plus-finding protocol violation をなくした。この改善は research evidence として保持するが、combined v4 semantic/schema change の採用には不十分である。Nemotron は強く finding-biased のままで、71 successful callsはすべて `finding`、残り69は typed parsing に失敗した。

したがって experiment は holdout-v3 に対して tuning せず reject する。runtime defaults は従来特徴づけた `soft-semantic-v3` contract に戻す。将来の successor は protocol/schema experiments と semantic wording experiments を calibration-only data 上で分離し、その後 independent measurement 前に新しい holdout-v4 を freeze しなければならない。
