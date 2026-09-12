# Provider usage / budget guard

日本語 | [English](reason-usage-budget.md)

**Status:** Reason CLI 0.5.0 development lineで実装済み。Harness Engine 0.4.2のcorrectness / authority semanticsは変更しません。

Reasonはprovider/resolver消費量をoperational telemetryとして表示します。usageやbudget exhaustionがepistemic evidenceになったり、verdictへauthorityを与えたり、semantic `unknown`として扱われることはありません。provider quota/rate-limit failureも既存のtyped operational failure classのままで、semantic uncertaintyへ再分類しません。

## 表示するusage

natural-language JSON outputにはadditiveな`usage` objectを含め、human output末尾には簡潔な`Usage` sectionを表示します。interactive sessionでは`/usage`でも確認できます。

次を区別します。

- **model calls**: candidate generation、investigation plan/action selection/regeneration、final render、model-backed answer-safety checkなどReasonのlogical model operation数。
- **provider attempts**: bounded adapter retryを含む、adapterが報告した実provider HTTP attempt数。
- providerが報告したinput/output/total token数。
- resolver/MCP/trusted-verifierのcall数、added token、elapsed time、adapter報告external cost（利用可能な場合）。

失敗したprovider operationがusageを返さない場合、そのtoken totalは`unreported`へ落とします。直前までのpartial totalを完全な値として見せません。

## hard guard

natural-language pathではCLIまたは`run` configからprovider-neutral guardを設定できます。

```bash
reason "TASK" \
  --max-model-calls 6 \
  --max-output-tokens 4096 \
  --max-total-tokens 12000
```

`reason-config-v1`では`run.max_model_calls`、`run.max_output_tokens`、`run.max_total_tokens`です。

`max_model_calls`は次のlogical model operation前に確認します。`max_output_tokens`は次requestの`max_tokens`自体を残量まで縮めます。`max_total_tokens`はinput tokenをReasonが推測せずprovider reportを使うため、測定可能なresponseごとに事後確認します。token ceiling設定時にproviderがusageを返さず検証不能なら`usage_budget_unmeasurable`でfail-closed、超過は`usage_budget_exceeded`です。どちらもtyped operational failureで、semantic uncertaintyではありません。

resolver/action側は既存の`max_resolution_attempts`とinvestigationのtarget/round/action上限がhard boundのままです。

## managed session accounting

managed interactive sessionではturn/resumeをまたいでusageを累積します。`/usage`で現在値を確認でき、新turnでは過去のtracked usageを含めてbudgetを強制します。

usageは`reason-managed-session-v1`へfield追加せず、managed session root配下のprivateな`reason-managed-usage-v1` sidecarへ保存します。これにより#381の0.5.x rollback contractを維持し、旧CLIはsidecarを無視できます。usage tracking導入前のsessionや、session本体更新後・sidecar更新前のcrashで履歴が一致しない場合は0と仮定せず「historical accounting incomplete」と扱い、その履歴を必要とする累積hard guardはfail-closedします。

managed sessionのdelete/purgeでは対応sidecarも削除します。`reason uninstall --purge-data`はmanaged session rootごと削除するためsidecarも対象です。explicit-path low-level session fileは引き続き対象外です。

## 通貨cost estimate

Reasonには古くなるbuilt-in price tableを持たせません。通貨estimateはoperatorがinput/output双方のrateとprovenance labelを明示した場合だけ表示します。

```bash
reason "TASK" \
  --input-cost-per-million 0.50 \
  --output-cost-per-million 1.50 \
  --pricing-source "provider-price-sheet-2026-09-12"
```

表示は**estimated model cost**です。token usageが未報告ならestimateもしません。pricing telemetryはcorrectness/authorityへ影響しません。

## setup live check

`reason setup --live-check`は実provider requestを送るためquota消費や課金が発生し得ます。interactive setupは実行前に確認し、non-interactive help/JSON outputにもこのdisclosureを残します。check自体はminimal responseへboundedされています。
