# Progress / retry / cancellation

Reason CLI 0.5.0のhigh-level progressは、human outputの完全なinteractive TTYでだけ表示します。JSON、piped stdin/stdout、non-TTY automationは静かなままです。

表示するlifecycle vocabularyは`Planning`、`Acquiring`、`Verifying`、`Finalizing`だけです。これはHarness-owned execution phaseであり、model reasoningやhidden chain-of-thoughtではありません。`--verbose`で追加できるのもprovider/model identity、attempt count、latency、typed failure classまでで、prompt、model response body、credential、hidden reasoningは表示しません。

provider待ちは既存adapter policyの範囲でboundedです。human TTYでprovider callが3秒以上継続した場合、Reasonは低頻度のwaiting heartbeatを表示し、provider-sideのbounded retryが進行中の可能性を示します。ただし取得できない途中retry countは推測しません。adapterが最終的に複数attemptを報告した場合だけ、完了後のattempt countを表示します。

Ctrl+Cはtyped operational failure `cancelled`として扱います。epistemic `unknown`へ変換せず、partial authorityも与えません。in-flight async provider workはdropし、Reasonが所有するexternal resolver / MCP / trusted verifierのsubprocessも同じcancel flagをdeadline loopで検知してterminateします。managed interactive turnは成功完了後だけadvanceするため、通常turnの中断をsuccessful checkpointとしてcommitしません。低レベルsession mutationでは既存のtyped invalidation/revalidation ruleを維持します。interactive promptで何も実行していない時のCtrl+Cは通常のterminal挙動を維持し、そのまま終了します。
