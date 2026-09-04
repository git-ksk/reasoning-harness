# `reason` CLI 日本語ガイド

[English CLI guide](cli.md) | 日本語

`reason`はReasoning Harnessの最初のproduct surfaceです。LLM/Agentの出力をそのまま信用するのではなく、Harness管理のevidence・verification・decision boundaryへ通すための非対話型CLIです。

`--candidate`と`--provider`の2つの使い方、AIなしでも`accept | reject | unknown`を判定できる原理は[Reasoning Harnessの仕組み](how-it-works.ja.md)で詳しく説明しています。

## 自然文AI path

人が使うprimary pathは、taskをそのまま渡せます。

```bash
reason "この障害を分析して" --fact http.status_code=503
```

provider/modelは`reason-config-v1`または明示flagから取ります。defaultはhuman-readable出力で、自動化では`--format json`を使えます。

| input | trust上の意味 |
| --- | --- |
| positional `TASK` | ユーザー依頼。evidenceではない |
| `--file PATH` | modelが読めるuntrusted context。自動でhard factにはしない |
| piped stdin | `--file`と同じuntrusted context |
| `--fact KEY=VALUE` | deterministic verification対象にできるHarness-owned structured fact |
| `--hypothesis KEY=VALUE` | 評価/解決するHarness-owned proposition |
| `--resolver-fact KEY=VALUE` | bounded resolution → admission → 再verification経由でだけ使う明示local fact |
| `--resolver-command PROGRAM` + `--resolver-arg ARG` | stdio JSONで外部取得するadapter。取得結果はdefaultではuntrusted |
| `--resolver-timeout-ms N` / `--resolver-max-response-bytes N` | external resolver 1 processのtimeout / stdout上限。どちらも0不可 |

context + target例:

```bash
cat error.log | reason "DBがroot causeか確認して" \
  --hypothesis incident.root_cause=database
```

trustedなstructured supportがなければ、結果が条件付き/`unknown`のままになることがあります。これは仕様です。`--file`に文章があるだけでverification authorityへ昇格はしません。

`main`ではv0.3.0 #174/#175のexternal resolver laneが入っています。`--resolver-command PROGRAM`はshell補間なしで明示programを1つ起動し、stdioで`reason-external-resolver-request-v1` / `reason-external-resolver-response-v1` JSONを交換します。返せるのはacquired evidence / candidate revisionなど既存`ResolutionResolver` contributionだけで、trusted metadata・verification receipt・verdict・final proseは返せません。external evidenceは`resolution.external_command.admission`でsource allowlistとHarness-owned freshness/scope/authority policyを明示した場合だけadmission対象になり、resolverが返したacquisition metadataだけでauthorityを昇格できません。reject理由はresolution attempt上でtypedに観測でき、admit後も通常のqualification / verificationを再通過します。#178ではprocess timeout / response-size上限、typed operational terminal、実call数・latency・optional cost telemetry、raw secretを出さないhashed adapter/admission config identityも追加しています。authorization / policy / protocol failureをgeneric retryすることはありません。詳細は[External resolver adapters](external-resolvers.md)を参照してください。

最終自然文もmodelがrenderするだけでは信用しません。`finalize_answer`がfactual-claim coverageを確認し、新しい事実を勝手に混ぜた場合はblockします。明示resolverで確認できる場合のみbounded resolutionへ戻し、再verification後に再renderできます。最初からHarness-ownedだったrequested hypothesisがartifact上でexact `Known`/`Supported`なら、rendererだけがclaimを落とす・exact keyからずらす・同じexact targetを`grounded`ではなく`uncertain`へ弱めるケースをdeterministicに回収できます。downgrade recoveryはrendererがその**同一exact requested proposition**を`uncertain`で出した場合だけ起動し、authorityはartifact stateからしか取りません。artifact-global `Unknown`ではtarget-onlyの`QualifiedPartialAnswer`のままです。artifact-global `Reject`もglobal verdict自体は絶対に上書きしませんが、targetがevidence-boundなdirect trusted `Supported` receiptを持ち、problematicなnon-target stateとtyped artifact上で構造的に分離できる場合だけtarget-only `QualifiedPartialAnswer`を出せます。same-key blocker、untyped blocker、shared evidence、inference/dependency path、target-local contradiction/qualification/hard adversarial signalのどれかがあればfail-closeし、依存関係が曖昧な場合も出しません。model prose解析・fuzzy key matching・新authority生成は行わず、recovery後も通常のanswer-safety gateを通ります。

providerのtransport reliabilityはHarness authorityと分離します。Google/Geminiではtemporary 429を`Retry-After`込みでbounded retryしますが、quota判定された429はfail-fastです。HTTP 500/502/503/504は最大2回retryし、HTTP自体は成功してresponse shapeもvalidなのにmodel textだけ空だった場合は1回だけretryします。Googleの1 requestはretry種別が混ざってもprovider HTTP attemptを合計4回までに固定します。credential、quota、通常の4xx/provider error、malformed success response、unsupported capability、transport interruption、timeoutはこのpolicyではfail-fastのままです。`provider_attempts`はadapter内部を含む実HTTP attempt数で、Harnessが別のstructured-output fallback callを行った場合は両callのattemptを合算します。retry exhaustionはtyped operational failureのままで、semantic `unknown`・evidence・abstentionへ変換しません。

自然文pathでは、grounded factを外へ出す前にcurrent semantic + evidence-sufficiencyの追加安全チェックも走ります。このチェックは**制限する方向にしか働きません**。追加verification / bounded resolution / abstainを要求できますが、model confidenceをtrusted evidenceや`accept`へ昇格させることはできません。一方、support済みのpartial factへ「task全体を完答できること」は要求しないため、安全な条件付き回答は残せます。

defaultは`--safety-profile current`（`verified-target-answer-gate-v1`）です。直前のclaim-local gate（`d3-sufficiency-answer-gate-v2`）を再現する場合は`rollback`を使い、従来の`d3-sufficiency` / `d3-sufficiency-v2`はそのaliasとして残します。さらに古い`legacy-v1` / `d3-sufficiency-v1`と`baseline`もtesting/rollback用に維持します。exact identityは[semantic-check](#semantic-check)、[仕組みの日本語解説](how-it-works.ja.md)、[用語ガイド](terminology.ja.md)を参照してください。

自然文JSON出力は通常の`reason-cli-output-v1` envelope内で`output_contract: reason-natural-output-v2`を明示します。

詳しくは[仕組みの日本語解説](how-it-works.ja.md)と[product dogfood](product-dogfood.ja.md)を参照してください。

## まずどのコマンドを使う？

| やりたいこと | コマンド |
| --- | --- |
| 人が自然文でtaskを依頼したい | `reason "TASK"` |
| 既存LLM/Agentの候補回答をstructured evidenceでチェックしたい | `reason run` |
| 完成済みartifactが構造・根拠ルールを満たすか確認したい | `reason verify` |
| contradiction/counterexampleなどsemantic診断をしたい | `reason semantic-check` |
| 入出力JSON Schemaを確認したい | `reason schema` |

人が直接使うなら**`reason "TASK"`**から、アプリ/CI統合や外部candidateを持ち込むなら**`reason run`**から始めます。どちらも自由文をtrusted evidenceとして扱うshortcutではありません。

## インストール

### 現在のexternal preview (`v0.2.0`)

自然文first pathは現在のtagged previewに含まれています。Rust 1.88+がある場合:

```bash
cargo install --git https://github.com/git-ksk/reasoning-harness \
  --tag v0.2.0 --locked reasoning-harness-cli --bin reason
reason --version
```

research binaryは入らず、supported product binaryの`reason`だけをinstallします。Linux x86_64 / macOS arm64 / macOS x86_64 / Windows x86_64向けstandalone archiveと`SHA256SUMS`もv0.2.0 Releaseで配布します。`main`は未releaseの開発snapshotを意図的に使う場合だけ選んでください。

`v0.2.0`はv1.0 readiness gateを満たした後も、v0.x support policy上はexternal previewのままです。versionはproduct/distributionの座標であり、新しいfrozen research generationを意味しません。

## 最小サンプル

```bash
reason run \
  --input examples/input.json \
  --candidate examples/candidate.json \
  --no-config \
  --format json
```

`examples/input.json`がHarness管理のtask/evidence、`examples/candidate.json`がLLM/Agent相当のuntrusted candidateです。

結果ではまず次を確認します。

```text
result.outcome.verdict
result.outcome.artifact.claims
result.outcome.artifact.verification_receipts
```

`accept | reject | unknown`は推論結果です。`unknown`でもコマンド自体が正常に完了すればexit codeは`0`です。

## stdin / CI

`-`でstdinを使えます。

```bash
cat examples/input.json | reason run \
  --input - \
  --candidate examples/candidate.json \
  --no-config \
  --format json
```

artifact検証:

```bash
cat artifact.json | reason verify - --format json
```

exit code:

- `0`: 処理成功。`run`のverdictは`accept/reject/unknown`を別途JSONで確認する。
- `1`: input/provider/runtime/validationなどの処理失敗。
- `2`: CLIの引数/usageエラー。

JSON modeでは失敗時もmachine-readableなfailure envelopeを返します。

## live provider

Mistral例:

```bash
export MISTRAL_API_KEY='...'
reason run \
  --input evidence.json \
  --provider mistral \
  --model ministral-8b-latest \
  --format json
```

live providerは候補を生成するだけです。その出力はtrusted evidenceではなく、その後に通常のHarness検証が走ります。

対応adapter:

- Mistral: `MISTRAL_API_KEY`
- Google Gemini/AI Studio: `GEMINI_API_KEY`
- NVIDIA Hosted NIM: `NVIDIA_API_KEY`

secretは`reason-config-v1`へ保存する設計ではありません。

## config

優先順位:

1. CLI flag
2. `--config PATH`
3. project `.reason/config.json`
4. user config
5. built-in default

再現可能なCIでは`--no-config`を使うとambient configを無視できます。

Schema:

```bash
reason schema config
```

## semantic-check

current semantic runtimeを明示的に使うproduct commandです。

```bash
reason semantic-check \
  --input examples/semantic-check.json \
  --provider mistral \
  --model ministral-8b-latest \
  --format json
```

通常は`--profile current`を使います。machine identityは`semantic-decidability-d3-v1`のままです。rollback:

```bash
reason semantic-check \
  --input examples/semantic-check.json \
  --provider mistral \
  --model ministral-8b-latest \
  --profile rollback \
  --format json
```

semantic findingはsoft diagnosticであり、trusted evidenceやfinal verdict authorityにはなりません。

## Schema discovery

```bash
reason schema artifact
reason schema candidate
reason schema config
reason schema semantic-check
```

現在のmachine contract identity:

- `reason-cli-output-v1`
- `reasoning-artifact-v1`
- `reasoning-candidate-v1`
- `reason-config-v1`
- `semantic-check-input-v1`

## もう少し詳しく

- [日本語README](../README.ja.md)
- [English full CLI guide](cli.md)
- [Product support / compatibility](support.md)
- [Product roadmap](product-roadmap.md)
