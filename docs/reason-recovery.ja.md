# Operational failure recovery

**状態:** Reason CLI 0.5.0開発ラインで実装済み。product UX layerの変更であり、Harness Engine 0.4.2のreasoning / authority semanticsは変更しません。

Reasonはoperational failureを`unknown`などのsemantic outcomeと分離します。provider outage、invalid credential、壊れたMCP server、corrupt session、update integrity failureをsemantic answerへ変換しません。

## Human error

product commandのrecovery-oriented human errorは、次の4点を明示します。

1. **What failed** — 完了を妨げたproduct subsystem。
2. **Task execution** — executionが開始前だったか、開始した可能性があるか、interrupt / incompleteだったか。
3. **Result trust** — failed operationから得たresultをtrustしてよいか。
4. **Next** — security checkを弱めずに確認・復旧するためのsafe command。

表示例:

```text
Error: GROQ_API_KEY is set but empty; refusing OS-store fallback
What failed: provider credential or native credential-store readiness
Task execution: the task did not start
Result trust: no result was produced
Next: reason auth status
```

remediation mapはcredential、model/capability compatibility、provider rate limit/quota/outage、provider protocol / malformed-output failure、configuration/project trust、MCP config/authentication/negotiation/read-only readiness、session state、usage budget、cancellation、update/version/distribution integrityを対象にします。

recovery commandには`reason auth status`、`reason models`、`reason config sources`、`reason trust status`、`reason mcp list`、`reason session list`、`reason update --check`、`reason doctor [--live-check]`など既存のsafe surfaceだけを使います。TLS無効化、provenance/integrity checkのbypass、project trustの弱体化、provider/model identityのsilent switchは案内しません。

## JSON compatibility

既存のmachine-readableな`failure.failure_class`は変更しません。通常のproduct failure envelopeへadditiveに`remediation` objectを追加します。

```json
{
  "status": "failed",
  "failure": {
    "failure_class": "credentials",
    "message": "..."
  },
  "remediation": {
    "what_failed": "provider credential or native credential-store readiness",
    "task_execution": "not_started",
    "result_trust": "no_result",
    "next_command": "reason auth status"
  }
}
```

remediation fieldはoperational metadataだけで、evidence、verification、finalization、semantic verdictを変更しません。
