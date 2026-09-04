# Live benchmark CI の実行

live benchmark workflow は手動の research workflow です。provider availability、trial quota、rate limits、model catalog state、stochastic output が外部変数であるため、required deterministic CI とは意図的に分離されています。

## 認証情報

Repository secrets は provider-level credentials です。

- `MISTRAL_API_KEY` は Mistral 用;
- `GEMINI_API_KEY` は Google Gemini/AI Studio 用;
- `NVIDIA_API_KEY` は NVIDIA Hosted NIM 用。

通常の `ci.yml` はこれらの secret を必要としません。Google と NVIDIA の jobs は secret がない場合 provider calls を skip します。Credentials は provider authentication headers にのみ送信し、benchmark JSON、logs、committed fixtures、issue comments に決して書き込んではなりません。

## モデルの手動選択

`.github/workflows/live-benchmark.yml` は provider ごとに1つの selector を公開します。routine NVIDIA selector は、2026-08-30 の Hosted NIM research 後、意図的に狭くしています。その `all` set に含まれるのは `nvidia/nemotron-3.5-lightning-30b-a3b` だけです。このモデルは、operational failure なしで full 20-case corpus を完了した唯一の NVIDIA research candidate でした。

他の NVIDIA Hosted NIM model IDs も、data-driven CLI adapter を通じた ad-hoc research では実行できます。ただし routine workflow choices ではありません。GPT-OSS 20B は 18/20 generations と2つの protocol failures、Gemma 4 31B は14/20と5つの timeoutsおよび1つの protocol failure、DeepSeek V4 Flash は ten-way probe の20 requestsすべてで timeout しました。記録済みの結果は [benchmark.ja.md](benchmark.ja.md#nvidia-hosted-nim-research-outcome) を参照してください。

NVIDIA jobs は model-job level で `max-parallel: 1` を使います。選択した model 内では、workflow の fixture concurrency の既定値は4です。これにより複数の NVIDIA models が account-level pressure を増幅することを避けつつ、Nemotron Lightning で安全と確認された遅い Hosted NIM requests は並行実行できます。どちらの値も asserted provider quota ではありません。

## 結果と失敗の扱い

Live benchmark JSON は要求された top-level provider と model を明示します。成功した generation ごとに、返された provider model ID、latency、provider attempt count、API が公開する場合は token usage を記録します。Generation failures は fixture ID、provider、requested model、latency、failure class、長さを制限した diagnostic message を持つ structured case records として保持します。Aggregate correctness metrics は successfully generated/evaluated cases のみを使い、`operational.attempted_runs`、`generated_runs`、`failed_runs` により reduced denominator を明示します。

model run 中、provider failures は後続 fixture の collection を中断しません。report 生成後、`operational.failed_runs` が non-zero の場合 workflow はその model job を failed とします。Deterministic harness failures は `result.harness.deterministic_failure` に分離されます。provider outage、quota、timeout、malformed provider output を harness correctness failure として報告してはなりません。

Committed deterministic fixture regression が required correctness gate です。Live results は diagnostic observations であり、deterministic verification authority を書き換えたり上書きしたりしません。

NVIDIA Hosted NIM calls は client-side の保守的な minimum interval 1.6 seconds（benchmark process あたり最大37.5 request starts/minute）を使います。これは pacing であり、claimed provider quota ではありません。NVIDIA limits は model/account により変わり得るため、HTTP `429` と `Retry-After` が authoritative です。

Mistral live jobs は、通常の Mistral benchmark と Mistral-backed semantic-judge run を含む、repository-level GitHub Actions concurrency group を共有します。同じ account-level rate limit を競合しないよう serialize します。Mistral adapter は HTTP `429` を `rate_limit` として別分類し、provider message が quota/billing exhaustion を明示する場合は `quota` とします。存在すれば `Retry-After`/provider reset metadata を尊重し、それ以外は最大60秒の bounded exponential backoff を最大5 retries 行います。Optional provider-safe telemetry は rate-limit headers の allowlist のみを記録し、credentials、prompts、response content は含めません。成功レスポンスが request/token headroom の critically low を示す場合、adapter は次の call 前に proactive cooldown を行うことがあります。

この挙動は universal provider contract ではなく、2026-09-03 の product-dogfood diagnosis に基づきます。その account/run では `mistral-small-latest` が `20,000` tokens/minute と `10` requests/minute を示し、観測された blocker は monthly-quota exhaustion ではなく request window でした。remaining request が1の時に60秒 cooldownすると、同じ frozen 16-case Stage-C evaluation を完了できました。`ministral-14b-latest` は異なる `937,500` tokens/minute / `30` requests/minute limit を示し、limits が model/account specific であることを補強しています。#126 は、semantic results を変更せずに長時間研究を operational に復旧できる、より広い transient 5xx handling と case-level resumable evaluation を追跡します。

## 反復試行

workflow は Mistral と Google 用に `trials` selector（`1`、`5`、`10`）を公開し、stability research の既定値を **5** とします。NVIDIA は別の `nvidia_trials` selector を使い、routine Hosted NIM diagnostic が自動的に増幅されないよう既定値を **1** とします。

`--trials > 1` では、pooled top-level `comparison` を stability estimate とみなさず `stability.correctness` を読みます。各 trial は full 20-fixture pass です。`--concurrency` はその pass 内の fixtures だけを重ね、次の trial は現在の trial 完了後に開始します。期待されるすべての fixture を generate/evaluate できた trial だけが correctness mean/min/max/population-stddev に寄与します。不完全な trial は explicit denominators と failure classes を伴って `stability.per_trial` に残ります。Token/latency distributions は `stability.operational` に分けて報告します。

`stability.diagnostics` は correctness から独立しています。typed diagnostic signals の complete-trial-only per-fixture frequencies、diagnostic-count distributions、exact denominators、fixture に complete observations が少なくとも5つある場合の95% Wilson intervalsを報告します。Operationally incomplete trials は diagnostic denominators から除外し、diagnostic absence とみなさず明示的に報告します。

one-trial live result は diagnostic point observation にとどまり、stable ranking を主張するには不十分です。5-trial distributions が materially overlap する models は、targeted 10-trial follow-up の候補です。

各 live model job は raw JSON を short-retention GitHub Actions artifact として保持します。これにより workflow 完了後も per-case/per-trial evidence を確認でき、console summaries だけに依存しません。

各 live fixture-suite JSON は、manifest が存在する場合、committed corpus identity（`corpus_version` と `score_compatibility_id`）も記録します。Live output は pooled category/difficulty stratification を意図的に省略します。complete-trial correctness は `stability.correctness` が所有するため、部分的な provider failures が stratum denominator を静かに変えることはありません。

## 同一モデル内の並行実行

1つの live model で独立した fixture generations を重ねるには `--concurrency N`（1-10）を使います。結果は aggregation 前に fixture/trial order へ戻され、1つの fixture failure は他の in-flight work から分離されます。すべての workers は同じ provider adapter を共有するため、NVIDIA の request-start pacing と429 `Retry-After` handling は run 全体で引き続き適用されます。NVIDIA workflow の既定値は、成功した20/20 Nemotron Lightning repeat run に基づく4です。

## Live soft semantic-judge のキャリブレーション

Issue #33 は manual workflow を optional な `semantic-judge` job で拡張します。`judge_provider` に `mistral`、`google`、`nvidia` のいずれかを設定し、compatible な `judge_model` と `judge_trials`（stability observation 前は通常5）を指定します。job は次を実行します。

```text
reason eval-judges fixtures/semantic-judges \
  --provider <provider> \
  --model <model> \
  --max-tokens <judge-max-tokens> \
  --seed 1000 \
  --trials <trials> \
  --concurrency <N> \
  --format json
```

Trials は sequential stability samples のままです。`--concurrency` は active trial 内の independent fixtures だけを重ねます。workflow は provider ごとに conservative な semantic fixture concurrency を既定とし、Mistral 1、Google 2、NVIDIA 4 とします。NVIDIA workers は1つの adapterを共有するため、既存の1.6秒 request-start pacing と `Retry-After` handling は in-flight calls 全体で authoritative です。Google は parallelism 増加前に rate-limit behavior を把握できるよう、3ではなく2から開始します。

`judge_max_tokens` の既定値は256で、reasoning-heavy models の calibration 時は512または1024へ上げられます。新しい独立 holdout measurement の前に、calibration corpus で token-budget changes を characterise してください。`finish_reason=length` を伴う structured parse failure は truncation evidence であり、semantic evidence ではありません。

Live semantic-judge requests は provider-neutral な `reasoning_preference=minimize` hint も送ります。これは semantic prompt tuning ではありません。adapter は provider が対応する場合に hint を native request control へ map できますが、そのような control のない adapter は既存挙動を維持します。NVIDIA は `chat_template_kwargs.enable_thinking=false` へ map します。通常の candidate generation は provider-default reasoning behavior を維持します。

workflow は full JSON を short-lived artifact として保存し、aggregate semantic metrics と、各 failed run の fixture ID、trial、failure class、latency、bounded provider-safe message を出力します。provider/protocol failure は `no_finding` になりません。影響を受けた trial は semantic stability distributions から除外します。Precision/recall には decision coverage と ambiguous-case abstention を併記し、意図的に不確かなケースへの aggressive decisions を可視化します。通常の Mistral と Google の live benchmark summaries も、failure count だけでなく failed-run details を出力します。これらの metrics は通常の live correctness benchmark とは独立しています。[live soft semantic-judge study](live-semantic-judge-study.ja.md) に、最初の repeated Mistral result と holdout caveat があります。

semantic-judge studies では、manual workflow が calibration corpus と versioned holdouts v1、v2、v3 を公開します。3つの holdout はすべて observation 後の historical/diagnostic です。holdout-v3 は rejected `soft-semantic-v4` matrix の前に frozen されており、successor の tune に使ってはなりません。runtime default は `soft-semantic-v3` に戻されています。将来 materially changed successor を作るには、新しい configuration identity と provider measurement 前に newly frozen holdout-v4 が必要です。Live JSON は corpus identity、complete-trial semantic stability、complete-trial family decision counts、operational fallback rate、successful-run fallback-reason counts、token usage、latency を報告します。`fallback_reason_counts` は `not_needed`、`primary_json_schema_unsupported`、`invalid_primary_structured_output` を区別しますが、provider-internal retry attempts は分類しません。Family semantic summaries は incomplete trials を除外し、operational metrics はすべての attempted calls を引き続き記述します。
