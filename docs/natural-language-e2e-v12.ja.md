# Natural-language E2E v12 — v0.4.2 リリースゲート

v12 は、未リリース v0.4.2 candidate の product commit `2d53a27d5ea0e2eb28bba355496db1f2b513f6a7` を測る fresh successor である。ゲート通過までは Cargo version を意図的に `0.4.1` のまま維持する。v9-v11 の観測済み surface は再実行・再採点・修復せず、historical evidence として固定する。

## 仮説

v0.4.1 では、exact fact target を recall できていても model action selector が有効な capability を実行できず、round budget を消費する pre-trigger residual が観測された。v0.4.2 は authority を増やさない限定的な acquisition precedence を追加する。target identity が1つ、capability が exact fact-key-bound / read-only、かつ explicit `selection_priority` の最高値が一意である場合のみ、Harness が model action selector を呼ばず capability を選択する。その結果が typed `no_result` なら、次は既存 Issue #249 の exact-target continuation が担当する。

3件の fresh adaptive case は v11 の名前・fact key・答え・source identity・task text・seed を再利用せず、同じ residual *class* だけを測る。最初の cache/no-result capability を priority `20`、registry follow-up を `10` に固定する。

## Frozen coordinate

- corpus: `natural-language-e2e-v12`
- evaluator: `reason-natural-language-e2e-v12`
- scoring: `natural-language-e2e-scoring-v12`
- product commit: `2d53a27d5ea0e2eb28bba355496db1f2b513f6a7`
- release target: `v0.4.2`（未リリース）
- canonical: `mistral / ministral-8b-latest`
- cross-model: `google / gemma-4-31b-it`, `google / gemini-3.5-flash-lite`, `groq / openai/gpt-oss-120b`
- base seed: `61000`
- max tokens: `1024`
- inter-case delay: `1500 ms`
- freeze tag: `natural-language-e2e-v12-freeze-r2`

各targetの最初のlive case launchだけを canonical observation とする。live observation 後は corpus / evaluator / threshold / target / seed / token budget / workflow / checksum を変更しない。semantic change が必要なら別 successor identity を作る。

## 追加する観測

v11 の correctness / operational metrics に加えて以下を固定する。

- `harness_precedence_selections`
- `InvestigationTelemetry.rejected_actions` 由来の per-case `action_rejections`
- aggregate `action_rejection_count` / `duplicate_action_rejections`
- `precedence_selection_conformant`
- `deterministic_acquisition_ambiguity`
- `avoidable_followup_stall`
- aggregate `deterministic_acquisition_ambiguities` / `avoidable_followup_stalls`

v12 の trigger は、Harness precedence selector が高priority cacheを最初に選び、そのactionが typed `no_result` を返した時だけ exposed とする。Issue #249 conformance は exposed case のみを分母とし、分母がある場合は `1.0` 必須。

## Pre-live gate

provider credential を確認する前に、freeze tag と PR head の一致、product commit、checksum、historical surface 不変、workspace full test、clippy/format、provider-aware concurrency policy、v12 evaluator test、no-model/no-network resolver/admission/MCP-shape preflight、全 frozen provider/model の exact CLI probe を通す。CLI probe は provider credential を全て外し、provider/model parse failure ではなく typed `credentials` failure まで到達することを確認するため、model request/network call は発生しない。

## モデル別 release gate

cross-model average は禁止。各required rowが独立して operational complete、correctness/session/authority/identity safety 境界を維持し、deterministic acquisition ambiguity `0`、duplicate-action rejection `0`、exposed 時の Issue #249 conformance `1.0` を満たす必要がある。historical semantic baseline があるrowは target recall / tool-selection success を悪化させず、false abstention を増やしてはならない。

| Row | v0.4.1 reference | v12 必須条件 |
| --- | --- | --- |
| Mistral `ministral-8b-latest` | recall `0.6`, tool `0.8`, false abstention `6`, stall `2/3`, trigger `1/3` | stallを厳密に減少、triggerを厳密に増加、その他のlisted utilityは非悪化 |
| Google `gemini-3.5-flash-lite` | recall `1.0`, tool `0.6`, false abstention `4`, stall `3/3`, trigger `0/3` | stallを厳密に減少、triggerを厳密に増加、その他は非悪化 |
| Google `gemma-4-31b-it` | recall `0.8`, tool `0.9`, false abstention `4`, stall `0/3`, trigger `3/3`。historical row は operational incomplete | fresh row は operational complete、かつ semantic ceiling の stall `0/3` / trigger `3/3` を維持、その他は非悪化 |
| Groq `openai/gpt-oss-120b` | v11 は generic CLI が provider parse で停止したため semantic baseline なし | generic natural-language generation → investigation → action/tool → final report を完走し、同じ zero-regression safety boundary を満たす |

flat / mixed / worse / incomplete のrequired rowが1つでもあれば v0.4.2 はリリースしない。canonical result は保存し、同じ frozen successor を調整して再実行しない。

## Concurrency

Mistral canonical は1回だけ実行する。同じ freeze commit の canonical workflow 成功後にだけ cross-model replication を許可する。Google 2モデルは同一provider lane内で直列 (`max-parallel: 1`)。Groq は別laneとして Google と独立実行できる。providerをまたいだ repository-wide `max-parallel: 1` は置かない。
