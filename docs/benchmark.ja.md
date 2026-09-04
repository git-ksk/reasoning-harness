# Benchmark 設計

## 目的

異なる model、provider、prompt budget を誤って測るのではなく、harness 自体が生む reliability の差を測定する。

初期比較では、両 arm に **同じ `ReasoningCandidate`** を使う。

- **naive baseline**: model が提案した epistemic state を authority として扱い、deterministic verification process を適用しない。
- **harness arm**: 同じ candidate に harness-owned evidence を組み合わせ、未検証の strong state を downgrade し、required pass を実行して harness acceptance policy を適用する。

これにより process effect を分離する。後の supplementary experiment で free-form direct answer と比較する場合は、output format と prompt の差が confounder になるため、別結果として報告する。

## 記録済みフィクスチャと live model run

commit 済み fixture は特定の failure mode を試す synthetic recorded candidate である。**deterministic regression test であり、model quality の empirical evidence ではない**。

live model study では同じ fixture input を使い、`recorded_candidate` を fresh provider output に置き換える。

```bash
reason eval fixtures --provider mistral --model ministral-8b-latest --trials 5
reason eval fixtures --provider google --model gemma-4-26b-a4b-it --trials 5
```

provider が対応する場合は `--seed` を使う。trial N は `base_seed + N` を使う。network availability、provider behavior、quota、cost は外部変数なので、live run は required CI gate に意図的に含めない。

### 反復試行の安定性の意味

`--trials N` は各 fixture/trial observation を個別 case として保持するが、repeated-trial analysis は pooled `N × fixtures` comparison だけから推論しない。各 trial は full-corpus pass として実行し、その pass 内に fixture concurrency を閉じ込め、現在の pass が終わるまで次 trial を開始しない。最終 case output は従来の fixture/trial order に戻す。live JSON には `stability` も含まれ、trial index ごとに case をまとめ、trial 別の correctness denominator と operational status を明示する。pooled top-level `comparison` は backward-compatible な case-level inspection 用に残る。stable model comparison では `stability.correctness` を使う。

全 expected fixture が生成され provider/model operational failure なしに評価された場合だけ trial は `operationally_complete` である。partial trial は `stability.per_trial` に successful correctness denominator と failure-class count とともに残すが、cross-trial correctness mean/min/max/stddev から除外する。provider availability が model correctness variance に見えないようにするためである。complete trial が1つもなければ、partial data から合成せず `stability.correctness` を省略する。

complete trial では baseline と harness の distribution に verdict accuracy、accept/reject/unknown recall、unsafe-accept case、deterministic verifier failure rate、contradiction detection、counterexample detection を含める。各 scalar distribution は observed complete trial 上の `count`、`mean`、`min`、`max`、**population** standard deviation（`sqrt(sum((x - mean)^2) / N)`）を報告する。観測した trial set の記述であり未観測母数の推定ではないため、Bessel/sample correction は行わない。

`stability.diagnostics` は correctness とは別の sibling report である。operationally complete trial の fixture ごとに、provider-neutral な adversarial contradiction/counterexample、assumption、evidence-qualification、candidate-normalization diagnostic code、caller が causal inspection output を渡した場合の causal finding/reason を集計する。incomplete trial は diagnostic frequency/count distribution 全体から除外し、`operational_failures` と `excluded_incomplete_trial_observations` で可視化する。per-fixture report は正確な occurrence count と denominator に加え、diagnostic count の mean/min/max/population-stddev を保持する。

fixture の complete observation が少なくとも5つある場合のみ、diagnostic proportion に **95% Wilson score interval** を付ける。小標本では正確な frequency/denominator を残し、statistical precision を示唆しないよう interval を省く。Wilson interval は記述的な uncertainty bound にすぎず、harness はそこから model ranking や significance を推論しない。

operational variability は別に報告する。`stability.operational` は successful request の total-token/latency distribution と、complete-trial の total-token/summed-request-latency distribution を含む。provider token metadata がない場合は zero に変換せず token distribution から除外する。provider failure に correctness verdict は決して割り当てない。

manual workflow の Mistral と Google stability study は model ごとに既定5 trial。5 trial の distribution がなお実質的に重なる model にだけ10-trial follow-up を行う。NVIDIA は別の trial-count input を持ち、Hosted NIM は Issue #6 の Mistral/Google study に対する supplementary で遅く capacity-sensitive なため既定1 trial のままである。

## バージョン管理されたコーパス契約

`fixtures/corpus/v1.json` は41個の active deterministic case（20 claim、8 causal、5 assumption、8 evidence-qualification）にわたり corpus `1.0.0` / score-compatibility ID `corpus-v1` を定義する。suite-prefixed ID、category、difficulty strata、scoring mode、provenance/redistribution metadata、contamination note、lifecycle status は manifest contract の一部である。metamorphic seed fixture は unscored evaluation control として残す。

recorded claim JSON は歴史的な top-level `comparison` を変更せず、`corpus.stratification.by_category` と `by_difficulty` を追加公開する。live run は `corpus_version` と `score_compatibility_id` を保持するが、pooled stratification は省略する。repeated/incomplete trial の処理は `stability.correctness` だけが所有するためである。

直接 score を比較するには、同じ `score_compatibility_id` と変更されていない metric case/scoring contract が必要である。変更規律、contamination posture、saturation warning は [corpus versioning](corpus-versioning.ja.md) を参照。

## 記録済みコーパス

claim/verdict suite は20 fixture（expected `accept` 5、`reject` 6、`unknown` 9）を含む。対象は次のとおり。

- boolean、count、version、region、HTTP status にわたる direct structured fact
- 欠落した fact と正しく保持される unknown
- environment、tenant、temporal、population scope の overreach
- contradictory structured observation
- universal health、policy、request-success claim への counterexample
- unsupported causal attribution
- Five Whys の symptom restatement

corpus は意図的に adversarial で、case-by-case に inspect できる程度に小さい。statistically representative な model benchmark ではなく、deterministic protocol regression suite である。

## 指標

| Metric | Basis | CI-safe |
| --- | --- | --- |
| unsupported accepted claims | golden fixture labels + typed epistemic state | yes |
| evidence coverage | deterministic structural measurement | yes |
| verdict accuracy / accept / reject / unknown recall | golden fixture verdict | yes |
| hidden assumption exposure | golden fixture labels + typed state | yes |
| contradiction detection | golden fixture labels + typed verdict/state | yes |
| counterexample detection | golden fixture labels + typed adversarial findings | yes |
| causal edge quality | golden bad-edge labels | yes |
| deterministic verifier failure rate | runtime result | yes |
| token usage | provider usage metadata | live only |
| latency | local wall-clock around provider call | live only |
| provider cost | token usage + explicit caller-supplied price rates | live only |
| model-backed semantic-judge calibration | soft evidence only; precision/recall/coverage/abstention plus ambiguous-case abstention, with operational completeness kept separate | no hard gate |

### Evidence-aware causal diagnostic のリグレッション

Issue #4 は `fixtures/causal/` に別の deterministic corpus を追加する。typed causal relation と edge ごとの support status を評価し、exact scoped support/refutation、association-only evidence、reverse-direction support、conflicting evidence、missing proposition binding、partial multi-cause support、scoped near-neighbor を扱う。regression は `supported | refuted | unknown` edge count と hard/soft causal finding count を報告する。

malformed harness-owned causal evidence は fixture/input error であり、scored unknown case ではない。壊れた oracle data によって conservative-edge count が水増しされないためである。

この corpus は20-case claim-verdict benchmark とは意図的に分ける。上記の `causal_edge_quality` は retained bad inference ID に対する従来の fixture-label metric であり、**evidence-grounded causal accuracy ではない**。causal diagnostic result は Issue #6 の verdict-correctness denominator や operational-completeness calculation に入れない。provider-neutral repeated-trial diagnostic report は causal support/refutation/unknown assessment と finding/reason observation を受け取るが、verdict-correctness denominator には入れない。live causal-generation/input contract も別であり、それを追加しても causal diagnostic に verdict authority を与えてはならない。

provider pricing は harness semantics と独立して変わるため runtime に hard-code しない。live run では明示的な rate を渡せる。

```bash
reason eval fixtures \
  --provider mistral \
  --input-cost-per-million <usd> \
  --output-cost-per-million <usd>
```

必要な metadata がすべて揃う場合、report は token count、latency、計算済み cost を記録する。

## 現在の記録済みフィクスチャのベースライン

harness-owned structured fact と deterministic authority が利用できる箇所の trusted verification により、20 recorded fixture は現在次を生成する。

- naive baseline verdict accuracy: 8/20 (40%)
- harness verdict accuracy under deterministic fixture coverage: 20/20
- unsupported accepted claims: 8 → 0
- accept recall: 1.0 → 1.0
- reject recall: 0.0 → 1.0
- unknown recall: 0.333 → 1.0
- hidden assumption exposure: 0.1 → 1.0
- contradiction detection: 0.0 → 1.0 for labeled deterministic conflicts
- counterexample detection: 0.0 → 1.0 for labeled deterministic counterexamples
- known bad Five Whys edges retained: 1 → 0

これは **generic model reasoning accuracy 100% ではない**。明示的な structured-fact/oracle coverage 下の deterministic process regression result である。harness-owned fact と hard verifier output は model が生成できない。hard authority がない case は保守的に `unknown` へ解決する。soft semantic discovery は research gap として残る。

receipt-backed verification 導入前の最初の live Mistral run は seven generation、6,022 token、provider latency 約17.2秒だった。untrusted candidate state が unsupported claim を accept し得る一方、harness は unsupported acceptance を排除したが over-conservative だった、という core safety trade-off を確認した。

exact-statement-bound verification receipt 後の2回目は seven generation、6,870 token、provider latency 約19.8秒。naive arm は5/7、harness arm は3/7 verdict accuracy。harness は unsupported accepted claim を zero にしたが、live model paraphrase が fixture receipt statement と完全一致しないため4/7 case は fail closed した。これは safe だが brittle な natural-language string binding という有用な negative result である。その後 built-in verifier は harness-owned structured fact を backing とする typed `Proposition { key, value }` target に移行した。verified proposition claim は canonical に `key = value` と render し、exact prose binding は compatibility-only。migration 完了を扱うには fresh live run が必要である。

live run は stochastic research observation であり、commit 済み deterministic fixture baseline の代替ではない。

## リグレッションポリシー

`cargo test --workspace` には recorded fixture aggregate の snapshot-style regression test がある。意図的な semantic change では implementation と expected benchmark baseline の双方を更新する。live provider result が commit 済み baseline を黙って書き換えることはない。

### Typed-target live の結果

typed proposition verification と malformed-inference isolation 後の subsequent seven-case live Mistral run は harness verdict accuracy 6/7 (85.7%)、deterministic verifier failure zero、unsupported accepted claim zero だった。accept recall と unknown recall は1.0、reject recall は0.5。exact-statement receipt run（3/7、deterministic failure 4件）より大幅に改善し、verifier binding は dominant failure mode ではなくなった。残る reject miss は hard-verifier transport/binding ではなく generic contradiction/counterexample discovery に属する。常に、この seven sample は diagnostic であり statistically stable な model-quality estimate ではない。

### Live metric の堅牢化と cross-model matrix

live benchmark label は provider-generated claim ID に bind しない。unsupported/hidden-assumption label は typed proposition に bind し、contradiction/counterexample detection は fixture level で測る。`unsafe_accept_cases` は unsupported strong claim を含む final `Accept` verdict だけを数える。これにより internal strong claim と overall `Unknown` verdict を混同しない。以前の20-case report にあった harness unsafe accept 1件は、final verdict が `Unknown` だった `partial-population` の metric false positive だった。

`HarnessInput.hypotheses` は task が明示的に置いた hypothesis を形式化する harness-owned proposition を持つ。candidate は target を追加・変更できない。runtime は missing hypothesis を assumed claim として materialize するため、structured-fact verification と adversarial discovery は provider が同じ proposition key を選ぶかに依存しない。

manual live workflow は同じ20-case corpus を `ministral-3b-latest`、`ministral-8b-latest`、`ministral-14b-latest`、`mistral-small-latest` に対して実行する。`GEMINI_API_KEY` 設定時は Gemini Interactions API 経由の Google-hosted model（`gemma-4-26b-a4b-it`、`gemma-4-31b-it`、`gemini-3.1-flash-lite`、`gemini-3.5-flash-lite`）も対応する。Antigravity managed agent は equivalent candidate generator ではないため除外する。すべての provider は untrusted candidate generator であり、provider output は verification authority を付与したり final verdict を決めたりできない。この matrix は diagnostic で required CI gate ではない。NVIDIA Hosted NIM は下記の optional matrix である。

### 最初の20ケースの cross-model 結果

以下は hardened 20-case corpus に対する one-trial manual matrix の harness-arm result である。

| Model | Baseline accuracy | Harness accuracy | Accept recall | Reject recall | Unknown recall | Unsafe accept cases | Contradiction detection | Counterexample detection | Tokens | Provider latency |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `ministral-3b-latest` | 0.60 | 0.80 | 0.20 | 1.00 | 1.00 | 0 | 1.00 | 1.00 | 7,414 | 19.0s |
| `ministral-8b-latest` | 0.65 | 1.00 | 1.00 | 1.00 | 1.00 | 0 | 1.00 | 1.00 | 27,221 | 71.5s |
| `ministral-14b-latest` | 0.85 | 1.00 | 1.00 | 1.00 | 1.00 | 0 | 1.00 | 1.00 | 32,033 | 109.5s |
| `mistral-small-latest` | 0.75 | 1.00 | 1.00 | 1.00 | 1.00 | 0 | 1.00 | 1.00 | 7,319 | 24.2s |

4つの harness run はすべて typed benchmark target で deterministic verifier failure zero、unsupported strong claim zero。一 model 一 stochastic trial なので statistically stable ranking ではない。ただしこの structured corpus では harness が8B、14B、Small run を完全に回復し、3B run は reject/unknown safety を維持しつつ direct-accept case で over-conservative だったという境界を示す。

### Gemma 4 アダプター

Rust `GemmaAdapter` は Google Gemini Interactions REST API（`/v1beta/interactions`）と標準 `GEMINI_API_KEY` credential を使う。provider-neutral `ModelRequest` contract を `system_instruction`、`input`、`generation_config`、`response_format`（JSON Schema structured output を含む）へ map し、model text と token usage だけを `ModelResponse` に parse する。API key は `x-goog-api-key` request header にだけ送り diagnostics には含めない。

Gemma live CI は optional。repository secret がなければ notice を出して provider call を skip し、required CI を弱めない。

### 最初の Gemma 4 cross-family 結果

最初の live Gemma 4 run は同じ hardened 20-case corpus に対し provider-neutral Google adapter 経由で `gemma-4-31b-it` を使った。一 stochastic trial で baseline verdict accuracy 0.85、harness verdict accuracy 0.95 (19/20)。harness accept recall 0.80、reject recall 1.00、unknown recall 1.00、unsafe accept cases 0、contradiction detection 1.00、counterexample detection 1.00、deterministic verifier failure rate 0。total 8,005 token、provider latency 約103.2秒。

これは non-Mistral model family の初の成功 live result で、provider-neutral boundary の initial cross-family check となる。ただし single trial であり Mistral model との stable ranking は確立しない。

`gemma-4-26b-a4b-it` は experimental/allow-failure matrix entry。Issue #6 five-trial study では100 case 中98件を生成し、2 request は HTTP 400 copyright/recitation block で provider に拒否された。そのため operationally complete は3 trial、incomplete は2 trial。stability report は incomplete trial を correctness variance から除外し、両 failure は provider operational observation として保持する。

### 反復試行の安定性に関する結果

Issue #6 stability study は Mistral/Google model ごとに full-corpus を5 trial（trial あたり20 fixture）実行した。correctness statistic は operationally complete trial のみを使い、incomplete trial は accuracy distribution に入れず operational observation とする。

| Model | Complete trials | Incomplete | Harness accuracy mean | Min-max | Pop. stddev | Accept recall | Reject recall | Unknown recall | Unsafe accepts | Verifier failures |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `ministral-8b-latest` | 5/5 | 0 | **1.000** | 1.00-1.00 | 0.000 | 1.000 | 1.000 | 1.000 | 0 | 0 |
| `ministral-14b-latest` | 5/5 | 0 | **1.000** | 1.00-1.00 | 0.000 | 1.000 | 1.000 | 1.000 | 0 | 0 |
| `gemini-3.1-flash-lite` | 5/5 | 0 | **1.000** | 1.00-1.00 | 0.000 | 1.000 | 1.000 | 1.000 | 0 | 0 |
| `mistral-small-latest` | 5/5 | 0 | 0.990 | 0.95-1.00 | 0.020 | 1.000 | 1.000 | 0.978 | 0 | 0 |
| `gemini-3.5-flash-lite` | 5/5 | 0 | 0.980 | 0.95-1.00 | 0.024 | 1.000 | 1.000 | 0.956 | 0 | 0 |
| `gemma-4-31b-it` | 5/5 | 0 | 0.950 | 0.95-0.95 | 0.000 | 0.800 | 1.000 | 1.000 | 0 | 0 |
| `gemma-4-26b-a4b-it` | 3/5 | 2 | 0.867 | 0.85-0.90 | 0.024 | 0.800 | 1.000 | 0.815 | 0 | 0 |
| `ministral-3b-latest` | 5/5 | 0 | 0.750 | 0.75-0.75 | 0.000 | 0.000 | 1.000 | 1.000 | 0 | 0 |

5-trial result は earlier one-run matrix の解釈を変える。Ministral 3B は lower point estimate 周辺の noise ではなく、全 trial で harness accuracy 0.75、accept recall 0、reject/unknown recall と safety を保持した一貫した over-conservative だった。Mistral Small と Gemini 3.5 は強いが unknown class に小さな variance。Gemma 31B は0.95で安定するが accept-class 1 case を一貫して miss。Gemma 26B は correctness が低く provider-blocking issue もあるため、complete trial 5件のように3件を比較してはならない。

5 trial 後も primary harness correctness metric で tie だった `ministral-8b-latest`、`ministral-14b-latest`、`gemini-3.1-flash-lite` には research plan に従い targeted 10-trial follow-up を行った。

| Model | Complete trials | Incomplete | Harness accuracy | All class recalls | Unsafe accepts | Verifier failures | Mean summed request latency / trial | Mean tokens / trial |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `ministral-8b-latest` | **10/10** | 0 | **1.000 ± 0.000** | 1.000 | 0 | 0 | 80.9s | 29,356 |
| `ministral-14b-latest` | **10/10** | 0 | **1.000 ± 0.000** | 1.000 | 0 | 0 | 95.8s | 28,419 |
| `gemini-3.1-flash-lite` | **9/10** | 1 | **1.000 ± 0.000** over complete trials | 1.000 | 0 | 0 | 154.8s | 8,731 |

Gemini 3.1 follow-up は200 fixture generation をすべて試行した。trial index 4 の `contradictory-evidence` generation 1件は Gemini Interactions response に model text output がなく（`protocol`）、operational failure。残り199 generation は成功。9 complete trial は harness accuracy と全 class recall が完全だった。この failure を消すために retry はしない。operational instability も observation の一部であり、correctness と provider reliability を分けて報告する理由そのものだからである。

従って10-trial follow-up は complete-trial harness correctness では3 modelを分離しない。今回の観測では operationally 分離する。両 Mistral は generation failure なしで10/10、Gemini 3.1 は protocol failure 1件のため9/10。Mistral 2 model の間では8Bの trial あたり token がやや多いが14Bより summed request latency が低い。これは本 corpus/provider state における observation であり、universal model ranking ではない。

### NVIDIA Hosted NIM の調査結果

NVIDIA adapter は OpenAI-compatible Hosted NIM endpoint `https://integrate.api.nvidia.com/v1/chat/completions` と provider-level `NVIDIA_API_KEY` 1つを使う。model ID は adapter branch ではなく data のまま扱う。adapter は structured candidate generation に generic JSON mode を要求し、schema validation は harness-owned candidate parser/validator に任せる。NVIDIA output に verification/verdict authority は与えない。

2026-08-30 live research は同じ20-case corpus で複数 Hosted NIM model を試した。correctness metric は successfully generated case のみで計算するため、まず operational success rate を読む。

| Model / run | Concurrency | Generated | Operational failures | Baseline accuracy | Harness accuracy | Total tokens | Sum of request latency | Approx. live wall time |
| --- | ---: | ---: | --- | ---: | ---: | ---: | ---: | ---: |
| `nvidia/nemotron-3.5-lightning-30b-a3b` first run | 1 | 19/20 | protocol 1 | 0.368 | 0.947 | 46,989 | 757s | 15m22s |
| `nvidia/nemotron-3.5-lightning-30b-a3b` repeat | 4 | **20/20** | **none** | 0.450 | **0.950** | 38,045 | 736s | **4m19s** |
| `openai/gpt-oss-20b` | 4 | 18/20 | protocol 2 | 0.667 | 0.944 | 40,462 | 244s | 2m02s |
| `google/gemma-4-31b-it` | 4 | 14/20 | timeout 5, protocol 1 | 0.857 | 0.929 | 19,045 | 780s | 10m53s |
| `deepseek-ai/deepseek-v4-flash-0731` | 10 | 0/20 | timeout 20 | n/a | n/a | n/a | n/a | ~6m40s |

DeepSeek は全 case で adapter の180秒 request timeout に達した。先行 serial probe も case あたり約3分を費やし、最終的に40分の GitHub Actions job limit に達した。Gemma は有用な result を返したが20件中5件で timeout。GPT-OSS は useful probe として最速だが structured-output/protocol failure が2件。Nemotron Lightning だけがこの research set で operational failure なしに20 caseを完了し、repeat run は harness accuracy を保ったまま four-way fixture concurrency により wall time を約3.5x短縮した。

そのため routine NVIDIA live matrix は **`nvidia/nemotron-3.5-lightning-30b-a3b` のみ**に狭め、workflow default は `--concurrency 4` とする。GPT-OSS、NVIDIA 経由の Gemma、DeepSeek、大型 Nemotron variant は data-driven CLI adapter で ad-hoc research に使えるが、diagnostic workflow を slow/flaky にしないため routine live CI から除外する。これは今回の observation に基づく repository policy であり、除外 model が universally unavailable/slow という主張ではない。

Hosted model availability、latency、capacity、trial quota は repository change なしに変わる external provider state。live workflow は manual/secret-gated のままで、固定 RPM や token quota を promise しない。

NVIDIA rate-limit handling は `Retry-After` の delay-seconds または HTTP-date を尊重し、それ以外は bounded exponential retry を使う。operational failure は correctness と分けて `credentials`、`rate_limit`、`quota`、`provider_unavailable`、`timeout`、`transport`、`provider_error`、`protocol`、`unsupported_capability` に分類する。failed live generation は harness correctness と誤って報告せず benchmark case として保持する。

adapter は conservative な1.6秒の request-start pacing（benchmark process あたり最大37.5 request starts/minute）も適用する。これは client-side guardrail であり、NVIDIA quota の主張ではない。

### Live フィクスチャの並行実行

live fixture suite は `--concurrency N`（1-10、CLI default 1）を受け付ける。独立した fixture generation は並行 in-flight にできるが、aggregation 前に final output を fixture/trial order に戻す。全 worker は同じ provider adapter を共有するため、NVIDIA pacing と `Retry-After` handling は run 全体に適用される。routine NVIDIA workflow は、Nemotron Lightning が concurrency 4 で timeout、rate-limit、protocol failure なしに20/20を完了したため concurrency 4 を default とする。

## Assumption diagnostic のリグレッション

Issue #12 は `fixtures/assumptions/` に別の deterministic corpus を追加する。final task correctness ではなく premise support classification を測る。初期5 case は trusted supported premise、harness-owned explicit input assumption、introduced typed unsupported premise、複数 inference edge にわたる1つの unsupported premise の semantic reuse、unbound premise を扱う。

aggregate は supported/explicit/unsupported/unbound premise count、hard/soft finding count、unsupported-premise detection rate、explicit-assumption recognition rate を報告する。これらは20-case verdict denominatorにも8-case causal denominatorにも入れない。`AssumptionFinding` は observational であり、hard unsupported-premise finding でも直接 `reject` を強制しない。final authority は verification と acceptance policy にある。assumption signal は #11 の provider-neutral repeated diagnostic report でも利用できる。

[assumption diagnostics](assumption-diagnostics.ja.md) は explicit assumption、unknown claim、unsupported causal edge の境界を説明する。

## Evidence qualification のリグレッション

Issue #16 は `fixtures/evidence-qualification/` に別の deterministic corpus を追加する。初期8 case は exact qualification、stale/not-yet-valid evidence、disjoint scope、unsupported scope expansion、insufficient authority、qualified な structured value 同士の conflict、temporal/scope/provenance metadata 欠落を扱う。

aggregate は qualified/disqualified/unknown evidence count、hard/soft finding count、expected finding-reason detection rate を報告する。これらは20-case verdict denominator、8-case causal denominator、5-case assumption corpus に入れない。通常の benchmark input に `evidence_requirements` がある場合、built-in structured verifier は qualification-aware path を使う。missing/disqualified evidence と conflicting qualified value は hard receipt を生成できない。

正確な temporal、scope、provenance、trusted-receipt boundary は [evidence qualification](evidence-qualification.ja.md) を参照する。

## Metamorphic robustness のリグレッション

Issue #10 は、意味を保持した表現変更の下でも trusted outcome が不変であるかを測定する deterministic metamorphic layer を追加する。初期 suite は verdict、adversarial、causal behavior にまたがる6つの transform family を扱う。不変性 metric は raw 20-case claim accuracy および8-case causal corpusとは別であり、変換後の case はいずれの correctness denominator にも入れない。

semantic/non-semantic field contract と reporting rules の詳細は [metamorphic testing](metamorphic-testing.ja.md) を参照する。

## Bounded resolution と finalization のリグレッション

Issue #22 は `fixtures/resolution/` に別の deterministic suite を追加する。resolution scenario は新しい corpus-v1 primary case ではなく **variant** である。各初期 scenario は安定した `base_case_id` を記録し、`fixtures/corpus/v1.json` に既に commit 済みの path/fixture identity と一致しなければならない。初期9 variant は `claim:missing-evidence` を再利用するため、direct one-shot、diagnose-only、bounded-resolution の observation は同じ base-case identity を共有しつつ、20-case claim denominator は変更しない。

実行方法:

```bash
cargo run -p reasoning-harness-cli -- eval-resolution fixtures/resolution --format json
```

初期 deterministic aggregate は expected scenario 9/9 である。9件すべてが diagnose-only harness では `unknown` から開始する。bounded resolution は `resolved_supported` 1件、`resolved_refuted` 1件、stale、wrong-scope、insufficient-authority、conflicting、no-result、malformed、untrusted resolver condition に対する `exhausted` 7件を生成する。unsafe な emitted final answer は0件のままであり、blocked unverified finalization は別途報告され、typed factual-claim coverage の平均は1.0である。controlled resolver attempt は合計10件記録される。

これらの値は protocol regression のみを測定する。resolver output と trusted metadata は fixture により制御されるため、この recovery rate は web retrieval、model self-correction、external tool quality に関する empirical claim ではない。resolution metric は通常の `BenchmarkComparison` および `stability.diagnostics` とは別に serialize される。

acquisition/admission/verifier と finalization の境界は [bounded grounded resolution and finalization](grounded-resolution.ja.md) を参照する。

## Soft semantic-judge のキャリブレーションに関するリグレッション

Issue #13 は `fixtures/semantic-judges/` に別の deterministic calibration corpus を追加する。これは corpus-v1 の verdict accuracy、repeated diagnostic stability、resolution-recovery denominator の一部ではない。9つの labelled case は contradiction、unsupported-premise、causal-gap discovery にまたがる positive、negative、意図的に ambiguous な例を扱う。Issue #36 は live semantic generalization/ambiguity study 用に、observation-free の28-case holdout v1を別途追加する。これもすべての hard correctness denominator の外に置く。

offline report は3つの synthetic judge identity、judge ごとの precision/recall、decision coverage と abstention、pairwise categorical agreement、abstention を missing rating として扱う nominal Krippendorff alpha を保持する。synthetic recorded observation は aggregation fixture であり、empirical model-quality result ではない。live judge study は引き続き optional/manual である。

authority boundary と metric definition の詳細は [soft semantic-judge calibration](semantic-judge-calibration.ja.md) を参照する。
