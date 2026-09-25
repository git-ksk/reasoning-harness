# Engine 0.6 candidate: evidence-target relevance calibration v10

Status: immutable v9 FAILのpre-freeze successor。v1-v9のfixture / observation / tag / scoreは履歴証拠として変更しない。first/only frozen v10 canonical calibrationがPASSするまでindependent holdoutのauthoringは禁止する。

## 設計修正

v9は安全境界自体は維持したが、説明用taxonomyとraw Text transportまで厳密にしすぎた。v10はHarnessがcandidate-local boundaryで実際に必要なactionだけを判定する。

- negative safety decision: `safe_to_reject | abstain`
- positive safety decision: `safe_to_accept | abstain`

`safe_to_reject` は2つの製品名が世界全体で別entityだと証明する意味ではない。与えられたlocal candidateをexact Harness target/relation向け証拠として捨てても、plausibleなtarget supportを失わないことだけを意味する。別製品へ明確にscopeされたpassage、navigation/comparisonだけのtarget mention、target-local supportのないgeneric passageは、material自体がidentity equivalenceを示唆しない限りreject可能。一方、rename / alias / successor / cross-language / version-lineageの明示的不確実性、mixed/shared ownership、partial/truncated identityはabstainする。

`safe_to_accept` はlocal document unitがrequested relationをexact Harness targetへ明確にbindしていることを意味する。source title / heading / structured metadataがtarget scopeを作り、relationが隣接excerptに現れるdistributed-document構造を許容する。URL-only identity、shared tableのowner不明、identity mapping不明はabstain。事実の矛盾、staleness、authority、verification、answer sufficiencyはdownstream concernであり、それ自体をrelevance failureにしない。

Harnessは引き続きtarget identity、aliases、provenance、policy、final dispositionを所有する。candidate contentはuntrusted dataであり、自分自身をrelevant/trustedと宣言できない。

## Materialization policy v6

primary target/relation bindingはadvisoryのまま。

- `target=different|unresolved` -> negative safety decision。`safe_to_reject`なら`irrelevant`、それ以外は`ambiguous`。
- `target=exact, relation=different` -> secondary callなしで`irrelevant`。
- `target=exact, relation=unresolved` -> `ambiguous`。
- strict identity modeではHarness-owned anchor無しのpositive promotionを引き続き禁止。
- promotion可能な`target=exact, relation=exact` -> positive safety decision。`safe_to_accept`なら`relevant`、それ以外は`ambiguous`。

v8でexact/exactのmixed ownership誤昇格があったため、v10ではpositive boundary自体を先に削らない。まずsemanticsとtransportを直し、安全性を実測してからsecondary call削減は別途判断する。

## Structured transportとbudget

primary bindingは最大2 model call (`JsonSchema` -> 必要時のみ既存のprovider-neutral `JsonObject` fallback)。secondary safety stageも最大2 model callとする。

1. typed `JsonSchema`
2. schema capability unsupportedまたはprimary structured response malformed時のみ、同一契約の`JsonObject` transport fallbackを最大1回

fallbackはtask/system policy/seed/token budget/semanticsを維持し、semantic retryや別質問、fuzzy repair、substring extraction、3回目の試行はしない。fallbackもmalformedならtyped `protocol` failureとしてfail closedする。safety outputはsingle-field JSON object、max outputは既存calibration policyの192-token budget。primary/secondaryは従来通り60,000 ms case deadlineを共有し、2連続operational failureでrun circuitを開く。

Googleについては別のprovider-adapter defectも修正する。Harness共通の`u64` seedをGoogle adapter内でprovider対応のnon-negative signed-32-bit domainへ正規化してからserializeする。抽象seed契約自体はprovider-neutralのまま。

## Calibration corpus

v10は56 synthetic case。

- v9の47 caseをregression/comparabilityとして保持しaction-safety expectationへ変換。
- v10 fresh 9 case: title-to-body distributed scope、stale-but-target-local、sibling local scope、comparison/context-only mention、generic no-target support、alias mapping unresolved、shared-row ownership unresolved、明確なpositive evidence内のprompt injection。

expected dispositionはRelevant 16 / Irrelevant 21 / Ambiguous 19。negative safetyは`safe_to_reject` 18 / `abstain` 17。positive safetyは`safe_to_accept` 16 / `abstain` 1。production motivating productはfixtureへ含めない。

## Provider roles / canonical gate

Required:
- Mistral `ministral-8b-latest`
- Groq `openai/gpt-oss-120b`

Full non-gating replication:
- Google `gemini-3.5-flash-lite`（pacing + attempt telemetry）

first/only canonicalは`engine-0.6-evidence-relevance-calibration-v10-freeze` tag、run attempt 1、checksummed exact surfaceでのみ実行する。各required armは56/56 operational complete、provider failure 0に加え次を個別に満たす。

- wrong-target relevance retention 0
- false relevance rejection 0
- expected Relevant left ambiguous 0
- utility miss 0
- materialized disposition exact 56/56
- unsafe negative rejection 0
- unsafe positive acceptance 0

primary proposal accuracyとsafety-decision exact accuracyはdiagnostic。acceptance boundaryはHarness-owned final dispositionとunsafe action countである。

v10がFAILならimmutable FAILとしてrerun/rescoreしない。canonical PASS後にのみfresh independent holdoutをauthorし、holdout PASS後にruntime acceptanceへ進む。PR #466は全段階完了までDraftを維持する。
