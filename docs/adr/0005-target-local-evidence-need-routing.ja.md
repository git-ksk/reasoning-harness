# ADR-0005: acquisition前のtarget-local evidence-need routing

Status: calibration向けcandidate implementationとして採用。final adoptionにはfresh live calibrationと、別途freezeしたindependent holdoutが必要。

## Context

release済みHarness Engine 0.5.0はevidence admission、qualification、verification、bounded resolution、finalization、answer safety、replay-safe reasoning threadを所有する。一方、exact targetごとに「そもそもexternal evidenceが必要か」をacquisition前に決める責務はまだない。

この不足により、supplied contentのsummary/explainのようなcontext-local taskが不要なexternal acquisitionへ入る可能性がある。逆方向も危険であり、current-state questionや明示的なverification requestを、関連contextがあるという理由だけで弱めてはならない。

Issue #461ではEngine 0.6 candidateにprovider-neutralなpre-acquisition boundaryを追加する。Engine 0.5.0 semanticsはimmutableのまま保持し、今回の変更はadditiveに扱う。

## Decision

acquisition前にnarrowなtarget-local evidence-need contractを追加する。

evidence modeは弱い順に次の5つとする。

1. no_factual_evidence
2. context_only
3. external_optional
4. external_required
5. trusted_verification_required

evidence needとacquisition dispositionは分離する。既存のHarness-owned evidenceが現在のfreshness、scope、authority、policy requirementを満たす場合、modeはexternal_requiredのままでもacquisitionをreuse_existingにできる。

model-facing objectはexisting target IDとproposed modeだけを持つEvidenceNeedProposalとする。evidence、authority、source、freshness、scope、tool、verdictは持たせない。

Harness-owned EvidenceNeedTargetPolicyが次を所有する。\n\ntarget kindは non_factual / content_local / external_world / ambiguous のtyped Harness-owned fieldとし、content_localは最低context_only、external_world / ambiguousは最低external_requiredをhard floorとして適用する。

- exact target identity / question
- deterministic baseline mode
- minimum mode
- model downgradeを許可するか、および許可時の最低floor
- explicit user verification intent
- current-state requirement
- trusted-verification requirement
- supplied-context stateとtarget-local sufficiency
- existing-evidence reuse status

model proposalによるescalationは許容できる。downgradeはHarness-owned downgrade floorが明示され、その範囲内でのみ受理し、その後すべてのhard floorを再適用する。明示許可がなければlower proposalは無視する。

## Responsibility boundaries

EvidenceNeedTargetPolicyはpre-acquisitionのepistemic jobを決める。truthは決めない。

EvidenceRequirementはevidence取得後にqualification / verificationが使うproposition-level freshness、scope、minimum-authority requirementのまま維持する。evidence-need routingで置き換えない。

InvestigationTargetはbounded acquisition targetのまま維持する。evidence-need routingはそのtargetをacquisition pathへ入れる必要があるかを決めるだけで、第二のresolver abstractionは作らない。

EvidenceAdmissionPolicy / EvidenceQualificationPassはacquisition後もauthority boundaryであり続ける。retrieve/reuseできたこと自体はsource内容のtruthを意味しない。

ReasoningPolicyは将来のintegrated policy builderへconservativeなrun/domain constraintを供給できるが、target-local decisionはstickyなwhole-run modeへ吸収せず、別のauditable stateとして保持する。

GroundedResolutionRuntimeは既存resolution authorityのまま。#461 candidateはevaluationによるsafe adoption確認前にrelease済みEngine 0.5.0 behaviorへ接続しない。

ReasoningThread replayはserializable decisionとvalidity identityだけを保持し、executable resolver callbackを保存・再実行しない。

## Context semantics

context completenessとtarget-local sufficiencyは別軸である。

partial / truncated contextを「情報が存在しない」証拠として扱わない。partial excerptでも、そのexcerptについてのexact questionには十分な場合がある。一方、full source全体についてのrequestにtruncated inputしかない場合はinsufficientであり、context-only world claimをsilent authorizationしない。

prompt injection風instructionを含むsupplied contextはdataであり、EvidenceNeedTargetPolicyやauthority ruleを変更できない。

claims-about-contentとclaims-about-worldは別propositionとして維持し、context_onlyがauthorizeするのは前者だけとする。

## Monotone floors

proposalの有無にかかわらず、materializerは次のHarness-owned floorを適用する。

- explicit external verification intent => 最低external_required
- current-state requirement => 最低external_required
- trusted exact verification => trusted_verification_required
- local/optional factual targetでcontext insufficient / unknown => external_required
- configured policy minimum => downgrade不可

resolver unavailableでもmodeは変更しない。acquisition requiredなのにresolverが使えない場合、downstreamはunknown/abstainまたはtyped operational terminalを維持し、context-only authorityへfallbackしない。

## Evidence reuse

prior Harness-owned stateがexact targetについてeffective requirementを現在も満たす場合だけreuseを許可する。

external_requiredはexternal requirementを満たすevidenceをreuseできる。trusted_verification_requiredはtrusted verificationを既に満たすevidenceだけをreuseできる。

stale、scope mismatch、policy mismatch、ambiguous evidenceはrequired acquisitionを抑止できない。

evidence-need module自身はadapter payloadからfreshness/scope/authorityを推論しない。reuse statusは既存のHarness-owned qualification / verification stateから供給する。

## Multi-target, follow-up, replay

targetごとに独立materializationする。したがってmixed request内でcontext_only targetとexternal_required targetを同時に持てる。

follow-upではevidence modeを再計算し、過去のcontext-only statusをstickyにしない。

serialized EvidenceNeedDecisionはexternal actionを含まない。replayはresolverを実行せずdecisionを再構成する。将来freshness、scope、policy identity、target identityが変わった場合は、新しいacquisition decision前にreuseをinvalidateする必要がある。

## Evaluation / release boundary

productionで観測したmotivating incidentはgap reportとしてのみ扱い、tuning fixtureには使わない。

fresh calibration identityはevidence-need-routing-calibration-v1。

independent holdout authoring前にacceptance criteriaをfreezeする。correctnessとutilityは別々に評価し、unsafe skipped acquisition、context authority laundering、不正なmodel weakening、不正なevidence reuse、replayed external side effectはそれぞれ0をhard gateとする。

candidateはHarness Engine 0.6.0 research lineとして扱う。fresh live calibrationと、別途freezeしたindependent holdoutがPASSするまでEngine release versionはpromotionしない。Engine 0.5.0はimmutableのまま保持する。
