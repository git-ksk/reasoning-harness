# Reference trusted command verifier

#177では既存`TrustedResolutionVerifier`境界に`trusted_command_verifier_v1`を追加します。取得adapterとは完全に別のauthority-bearing laneです。

外部command自身は`VerificationReceipt`を返せません。exact typed requestと現在のevidence/qualification policyを受け取り、`supported` / `contradicted` / `no_result`と参照evidence IDだけを返します。receipt ID、verifier identity、exact proposition bindingはHarness側が生成します。

`resolution.trusted_command.trusted:true`の明示が必須です。これはoperatorによるtrust decisionです。LLM、retriever、未監査scriptをtrusted verifierとして設定するのはcorrectness boundaryを弱めるため、安全な利用方法として扱いません。compiler/test/schema/signature/policy engine等のdeterministicまたは明示的にtrustするoracle向けです。

対象propositionに`EvidenceRequirement`がある場合、verifierが参照する全evidenceは現在のtemporal/scope/authority policyで`Qualified`でなければreceiptに束縛できません。staleやwrong-scope evidenceによる迂回は`PolicyDenied`になります。

response schemaはclosedで、verifier identity、proposition、receipt、verdict、final proseのsmugglingを拒否します。operational failureはsemantic outcomeと分離され、timeout/response-sizeもboundedです。
