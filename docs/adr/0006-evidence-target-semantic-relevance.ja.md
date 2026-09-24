# ADR-0006: Harness-owned evidence-target semantic relevance

Status: fresh calibration向けcandidate contractを実装済み。runtime integrationとlive acceptanceは未完了。

## Context

#461はtargetがexternal acquisitionを必要とするかを担当する。acquisitionが必要と判断されcandidate materialを取得した後には、別の問いとして「そのmaterialが本当にexact Harness-owned targetとrequested relationについてのものか」を判定する必要がある。

既存のadmission / qualificationはprovenance、freshness、scope、authority、proposition supportを正しく担当しており、generic semantic target matchingまで混ぜるべきではない。product integrationごとにtoken比率、URL substring、model-authored search intent identityを独自実装するのも避ける。

#462はEngine 0.6 candidateへprovider-neutralなpre-admission relevance contractを追加する。productionで観測した事例はgap reportとしてのみ扱い、tuning fixtureには含めない。

## Decision

独立した `EvidenceRelevanceTargetPolicy`、`EvidenceRelevanceCandidate`、advisoryな`EvidenceRelevanceProposal`、Harness-materialized `EvidenceRelevanceAssessment`を導入する。

relevance dispositionは次の3つだけ。

- `relevant`
- `irrelevant`
- `ambiguous`

`relevant`はdownstream considerationへ進める資格があるという意味だけであり、Evidence、authority metadata、verification receipt、Supported claim、verdict、freshness、answer sufficiencyを作らない。

model-facing proposalはdispositionだけを持つ。target ID、evidence ID、source ID、entity identity、relation kind、authority、trust、freshness、scope、verification、verdictをmodelが返して上書きすることはできない。

## Harness-owned target identity

Harness policyが以下を所有する。

- stable policy / target ID
- exact target question
- optional canonical entity ID / canonical name
- approved alias / localized name
- typed relation kind
- identity requirement
- explicit model/token/time/attempt budget

identity requirementは `none` / `require_harness_anchor` / `allow_semantic_equivalent`。

`require_harness_anchor`では、Harness-owned canonical nameまたはaliasがcontent-bearing signal内に存在しない限り、modelが`relevant`を返しても通さない。URLだけ、navigation/footerだけの一致はanchorとして扱わない。required anchorが無いmodel-relevantは`ambiguous`へfail closedする。一方、modelが明確に別targetと判断した`irrelevant`は安全なrejectとして保持できる。

`allow_semantic_equivalent`は、lexical identityを必須にしないtargetに対する明示的なHarness policy。paraphrase / cross-lingual relevanceをmodel-assistedで許可しても、Harness-owned target identity自体をmodelが変更することはない。

## Candidate signals

candidate materialは未信頼データのまま。signalは次を区別する。

- source title
- canonical URL
- heading
- excerpt
- structured metadata
- navigation/footer
- fact text

いずれもauthorityを作らない。URLとnavigation/footerはstrict identityを単独では満たせない。titleやmetadataにidentityがあっても、単独token matchだけでself-authorizeせずsemantic proposal/materializationを通す。

signal内のprompt-like instructionはdataでありpolicyを変更できない。

## Relevanceはfreshness / truthではない

stale documentでもsemantic relevanceは成立し得る。relevant-yet-staleとして保持し、freshnessが必要なら後段の通常policyでrejectする。

同様に、trusted sourceはrelevanceを意味せず、relevanceもtrusted sourceを意味しない。relevanceはEvidenceRequirementを満たさず、EvidenceAdmissionPolicy / EvidenceQualificationPassをbypassせず、source-attributed proseも許可しない。最後は#463の責務。

## Bounded model assistance

`EvidenceRelevanceAssessmentBudget`はHarness-ownedで、max model attempts / max tokens / max elapsed millisecondsを保持する。v1 defaultは2 attempts / 192 tokens / 15,000 msで、1回のprimary structured callと最大1回のbounded JSON-object fallbackを収容する。0はinvalid。

calibrationのoperational lineageではsemantic contractを変えず、v4で30,000 ms、v5で60,000 msへbounded elapsed budgetのみを拡張した。max model calls=2 / max output=192は維持する。v5の60秒はGoogle adapterの最悪retry envelope全体を吸収する値ではなく、実測tail latencyにheadroomを与えつつ継続的provider instabilityをtyped operational failureとして残す上限である。

model proposal欠落やassessment失敗をimplicit relevantにしてはならない。proposal無しはtyped `ambiguous`へmaterializeする。provider / transport / protocol failureはlive runnerでoperational failureとしてsemantic outcomeと分離する。

serialized assessmentはstable policy/target/evidence/source ID、disposition、assessment path、typed reasonだけで診断可能とし、raw document payloadをtelemetry/replayへ要求しない。

## Evaluation

calibration identityはv1-v4をimmutable historical evidenceとして保持し、現在のfresh successorは `evidence-relevance-calibration-v5`。v5 PASS後にのみsemantic implementationをfreezeし、別authorのindependent holdoutへ進む。

26 synthetic caseでexact identity、alias/acronym、semantic paraphrase、distributed title/body support、structured metadata、日本語/英語cross-lingual identity、stale-but-relevant、same-service wrong feature、sibling product、navigation/footer-only、broad landing、comparison-only、relation mismatch、prompt injection、unknown rename、partial identity、mixed document、conflicting section、insufficient excerpt、URL-only identityを含む。

hard correctness gateはwrong-target relevance retention = 0。

utilityは別軸で、relevant materialのfalse rejection / avoidable ambiguityを評価する。always-relevantはcorrectness FAIL、always-rejectはutility FAIL。

live calibration後にsemantics/thresholdをfreezeするまでindependent holdoutはauthorしない。release済みEngine 0.5.0はimmutable。
