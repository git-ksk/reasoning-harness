# リファレンスの信頼済みコマンド検証器

Issue #177では、既存の`TrustedResolutionVerifier`境界に、参照用のauthority-bearing adapterとして`trusted_command_verifier_v1`を追加します。これは`external_command_v1`や`mcp_readonly_v1`のような取得adapterとは意図的に分離されています。

外部commandは`VerificationReceipt`を返しません。正確なtyped resolution requestと、現在のHarnessのevidence/qualification policyを受け取り、参照するevidence IDとともに`supported`、`contradicted`、`no_result`のいずれかだけを返します。receiptはReasoning Harness自身が構築し、正確に要求されたpropositionへ束縛し、operatorが設定したverifier identityを記録します。

`resolution.trusted_command.trusted`は明示的に`true`でなければなりません。これはoperatorによるtrust decisionです。LLM、retriever、任意のshell script、未レビューのserviceをtrustedとして設定するとcorrectnessが弱まり、安全なdeployment patternとしてはサポートされません。deterministic compiler/test/schema validator/signature verifier/policy engine、または明示的にtrustedとした別のoracleを優先してください。

一致する`EvidenceRequirement`がある場合、trusted commandが返すすべてのevidence IDは、現在のHarnessのtemporal/scope/authority policyのもとですでに`Qualified`でなければなりません。stale、scope違い、authority不明、その他のnon-qualified evidenceを、このreference adapter経由でreceiptの発行に利用することはできません。適用可能なrequirementがない場合でも、参照evidenceは現在のartifactに存在し、IDは一意かつ空でない必要があります。

response schemaはclosed（`reason-trusted-verifier-response-v1`）であり、verifier identity、proposition binding、receipt ID、verdict、final proseを忍び込ませることはできません。operational failureはtyped resolution failureとして扱います。timeoutとresponse-size limitには上限があり、config identityはhash化され、ReasoningThread replayでは記録済みのattemptを再実行せずに保持します。

設定例:

```json
{
  "schema_version": "reason-config-v1",
  "resolution": {
    "trusted_command": {
      "trusted": true,
      "verifier_id": "reference-policy-oracle",
      "program": "/usr/local/bin/policy-oracle",
      "args": ["--stdio"],
      "timeout_ms": 5000,
      "max_response_bytes": 65536
    }
  }
}
```

supported responseの形:

```json
{"schema_version":"reason-trusted-verifier-response-v1","result":{"conclusion":"supported","evidence_ids":["e1"]}}
```

取得の成功とtrusted verificationの成功は、引き続き別々のresolution attempt/metricです。resolverが返したdataは、後からtrusted verifierが検査できるというだけで自動的に昇格しません。
