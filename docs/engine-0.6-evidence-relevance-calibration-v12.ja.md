# Engine 0.6 candidate: evidence-target relevance calibration v12

Status: immutable v11 canonical evidenceの後継pre-freeze。v1-v11のfixture/tag/observation/scoreは変更しない。first/only frozen v12 canonical calibrationがPASSするまでindependent holdoutのauthoringは禁止する。

## v12の目的

v11でownership boundary自体は正しくなった。全caseで独立local qualification guardを実行し、最終Relevant/Irrelevant/AmbiguousはHarnessだけがmaterializeする。一方required Mistral armではguard contractの意味論矛盾が露出した。v11 promptはsupplied materialだけでriskを完全否定できなければunresolvedとしていたため、通常のopen-world uncertaintyまでlocal blockerへ変換し、Ambiguousを過剰生成した。

特にHarness-owned aliasで矛盾が明確だった。fixture側はHarness policyに既に与えられたaliasをauthoritative local identity anchorとして扱う一方、prompt側はalias equivalenceが記載・示唆されるだけでもidentity riskと読める定義だった。v12はfail-closed materializerを緩和せず、このcontract矛盾を解消する。

またv11 Groq armでは別系統のtransport問題が発生した。server-side structured generationがschema-generation failureを繰り返し、60秒case budgetを消費した。v12ではJsonSchemaを1回目に維持し、1回だけ許すfallbackをprovider JSON-object modeではなくstrict raw-JSON Textへ変更する。response全体をtyped contractとして直接parseし、抽出・fuzzy repair・semantic retry・3回目callは許可しない。

## Local qualification v2

6 fieldの型は維持する: target_support、relation_support、identity_mapping_risk、ownership_scope_risk、context_completeness_risk、explicit_local_absence。

3つのrisk fieldは、仮説上あり得る全外部riskが存在しないことの証明要求ではなく、具体的なlocal ambiguity signalの検出器とする。

- present: supplied candidate内にrisk成立を示す具体的local cueがある。
- unresolved: 具体的risk-relevant cueはあるが、local materialだけでは確定できない。
- absent: そのriskを起動する具体的local triggerがない。

未知の外部事実があるかもしれない、またはcandidateがrisk不在を明示証明していない、という理由だけでunresolvedにしてはならない。

target policy内でHarnessが与えたcanonical name / aliasはauthoritative local identity anchorとする。そのaliasをcandidateが一貫して使うだけではidentity riskにならない。一方、possible rename/successorの明示、identity cueの衝突、shared unlabeled ownership、明示的clip/truncate/omitted contextは引き続きfail-closedとする。

事実の不一致、staleness、source authority、verification、answer sufficiencyはdownstream concernのまま。

## Materialization boundary

materialization policy v7は変更しない。hard Relevantにはprimary target/relation Exact、guard target/relation Supported、3 riskすべてAbsent、必要なHarness-owned identity floorを要求する。hard Irrelevantにも独立したrisk-free negative agreementを要求する。qualification riskがPresentまたはUnresolvedなら必ずAmbiguous。CanonicalUrl-only identityも引き続きAmbiguous hard floor。

したがってv12はlocal risk signalの定義だけを修正し、riskが存在するときのHarness動作は緩和しない。

## Transport

primary binding / local qualificationの両方で、1回目はJsonSchema。structured transport failureまたはmalformed response時のみ、同じschema/task/system/budget/seed semanticsを保持したstrict raw-JSON Text fallbackを1回実行する。fallback body全体がtyped contractへ直接parseできることを要求する。

JSON抽出、Markdown除去、field補完、fuzzy repair、semantic retry、3回目callは禁止。primary + qualificationは同じ60,000 ms case deadlineを共有し、2連続operational failure circuitも維持する。
Groq canonical armではserver-side structured generation failureを1回でtyped UnsupportedCapabilityとしてsurfaceさせるため、adapter内部のstructured-output retry limitを0に事前固定する。fallbackはその後の別model callとしてstrict raw-JSON Textで実行する。8k TPM制約に対し、schema failure responseがusage telemetryを返さない場合でも過剰送信しないようminimum request intervalは10,000 msに固定する。これらはprovider-specific operational pacingでありsemantic contractは変更しない。

## Calibration corpus

v12は73 synthetic calibration cases。v11の65 caseを全保持し、v12 live observation前にfresh 8 caseを追加する。fresh setはHarness-owned alias、exact canonical identity、explicit possible rename、single-owner structured value、shared unlabeled ownership、complete local context、clipped/omitted referent、clear other targetについてno-risk / concrete-risk境界を測る。

expected final dispositionはRelevant 23 / Irrelevant 25 / Ambiguous 25。25 caseが少なくとも1つのexpected fail-closed qualification riskを持つ。production motivating product contentは引き続き除外する。

## Canonical provider / acceptance gate

Required:
- Mistral ministral-8b-latest
- Groq openai/gpt-oss-120b

Full non-gating replication:
- Google gemini-3.5-flash-lite

各required armは独立に73/73 operational completion、provider failure 0、attempt telemetry complete、local qualification expected/invoked 73/73、qualification risk miss 0、wrong-target relevance retention 0、false relevance rejection 0、expected Relevant left ambiguous 0、utility miss 0、materialized exact 73/73を満たす必要がある。

primary proposal exactness、全qualification field exactness、spurious risk blockはdiagnosticのまま。acceptance boundaryはfinal dispositionとmissed safety risk。

first/only frozen v12 canonicalはrerun/rescoreしない。canonical PASS後のみfresh independent holdoutをauthorできる。holdout PASS後にruntime integration acceptanceへ進む。PR #466は全stage PASSまでDraft維持。
