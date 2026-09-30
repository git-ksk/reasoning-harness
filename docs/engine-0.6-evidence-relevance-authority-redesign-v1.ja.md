# Engine 0.6 evidence relevance authority redesign v1

Status: independent holdoutでutility failureが反復したためのarchitecture redesign candidate。

## v3-v6 / v16-v19 lineをここで止める理由

historical lineはsafetyを強化し、recent independent holdoutではwrong-target Relevant 0を維持できている。一方、effective qualification / materialization chainはadvisory model voteの周囲へ補正ruleを順番に追加する構造になった。

反復している症状は、Harnessが「acceptしてはいけない」は判断できても、そのpropositionを誰が所有しているかを一貫した表現で持っていないこと。結果としてIrrelevant -> Ambiguousの保守的utility missと、弱いlocal evidenceからのnegative identity過信が交互に出ている。

v19へ新しいcase-shaped branchを追加してもsemantic debtが増えるだけなので、v6/v19はhistorical semanticsとしてfreezeしたまま、新しいauthority layerを開始する。

## Core invariant: mention != ownership

target名の出現は次のどれでもあり得る。

- propositionを所有するsubject
- comparison/context object
- navigation/footer
- URL routing metadata
- explicit absence内のtarget
- alias/rename/successor mapping uncertainty
- mixed/shared ownershipの一要素

positive target ownershipを作れるのは最初だけ。

different-looking nameも自動的にnegative ownershipにはしない。distinct subjectにはstructural corroborationを要求する。

## Pipeline

新pipeline:

1. Local feature extraction
   - target mention role
   - structural distinct-subject evidence
   - relation evidence
   - blocking risk
2. Harness-owned authority resolution
   - identity ownership
   - relation coverage
   - 各authority source
3. Materialization table
   - resolved authorityだけからRelevant / Irrelevant / Ambiguousを決定
4. Model advice
   - proposal/verifierはunresolved axisのcorroboration用途
   - model voteからfinal dispositionへ直接遷移しない

effective qualification vNがvN-1へdelegateして1 familyだけoverrideする構造を終了する。

## Authority type

Identity:
- exact_target
- distinct_target
- target_absent
- unresolved

Relation:
- requested_relation
- different_relation
- relation_absent
- unresolved

Authority source:
- harness_deterministic
- harness_corroborated
- model_consensus
- unresolved

Blocking risk:
- none
- identity_mapping
- ownership_scope
- context_gap
- multiple

resolved objectはidentity / relation / riskと各axisのsourceを分離保持する。これにより一方のaxisから他方へのauthority launderingを防ぐ。

## Identity precedence

fail-closedで解決する。

1. mapping / ownership / contextのblocking riskがあればterminal authorityを作らず最終Ambiguous。
2. explicit target absence + local verifier corroborationでtarget_absent。
3. direct target-owned subject occurrenceはcontext-only / navigation-only / URL-only / absence-onlyでない場合だけexact_target。
4. authorized canonical/alias subjectが独立local signalで反復する場合はadvisory identity disagreementを越えてexact_targetを回復可能。
5. explicit separation、またはcontext-only target + stable repeated non-target subjectでdistinct_target。
6. stable repeated sibling/non-target subjectはdistinct_targetを作れるがsingle-signal near siblingは不可。
7. allow_semantic_equivalent policyではHarness contradiction/riskがない場合のみmodel consensusでexact_targetを許可。
8. その他はunresolved。

同程度のpositive/negative ownership proofが競合する場合は任意優先せずunresolved。

## Relation precedence

relationはidentityから独立して解決する。

1. blocking riskがあればterminal materialization不可。
2. requested relationのexplicit local exclusion + model corroborationでdifferent_relation。
3. substantive local contentにrequested relationが存在しmodel supportがあればrequested_relation。
4. explicit relation absence + corroborationでrelation_absent。
5. その他unresolved。

distinct/absent identityはrelation terminalでなくてもIrrelevantにできる。exact-target Irrelevantにはdifferent/absent relationのterminal proofが必要。

## Materialization table

risk = noneの場合:

| Identity | Relation | Final |
|---|---|---|
| exact_target | requested_relation | Relevant |
| exact_target | different_relation | Irrelevant |
| exact_target | relation_absent | Irrelevant |
| distinct_target | any | Irrelevant |
| target_absent | any | Irrelevant |
| unresolved | any | Ambiguous |
| exact_target | unresolved | Ambiguous |

blocking riskがあれば常にAmbiguous。

このtableにはproposalのtarget/relation voteを直接入れない。

## Structural evidence requirement

### Positive ownership

canonical/authorized aliasがidentity-capable local signalでsubjectとして成立する場合だけHarness-owned exact identityを作る。context-only / navigation/footer / URL-only occurrenceは不可。

authorized identityがtitle/heading/metadataと別excerpt/factで反復すればより強いcorroborationとし、advisory identity disagreementを上書きできる。

### Negative ownership

negative ownership候補:

- explicit separate/distinct service/product structure
- targetがcomparison/context-onlyでstable repeated substantive subjectが別に存在
- stable repeated sibling/non-target subjectが独立local signalに存在
- explicit target absence

mere target-anchor absenceはnegative authorityではない。

single-signal near-nameだけではdistinct_targetにしない。

### Conflict handling

mixed multi-product、owner column欠落、mapping uncertainty、truncated referentはmodel confidenceに関係なくAmbiguous。

## Model role

最初のredesign iterationではimmutable observation replayを維持するため、現行binding proposal / local qualifierをadvisory inputとして再利用する。

役割は限定する。

- Harness-visible structural claimのcorroboration
- policyが明示許可したsemantic-equivalent identity
- relation coverage補助
- stronger Harness proofを直接上書きしない
- 2 model voteだけをHarness ownership proofと同一視しない

将来span-backed model contractへ移行可能だがauthority-v1の必須条件ではない。

## 新semantic freeze前の必須regression

- 各authority source / conflict boundaryのgeneric property test
- immutable v23 replay
- immutable holdout v1 replay
- immutable holdout v2 replay
- immutable holdout v3a replay
- one-shot v4完走後のimmutable v4 provider observation replay
- 全replayでwrong-target Relevant 0
- false relevance rejection増加なし
- full core/providers/CLI、all-target Clippy、fmt、YAML、special-case scan
- production branchingにprovider名 / fixture ID / synthetic entity / exact holdout stringなし

authority-v1でv4を完全replayできてもfrozen holdout v4自体はimmutable FAILのまま。

authority-v1 semantics freeze後にのみ次のfresh independent holdoutをauthorする。

## Current prototype evidence

最初のauthority-v1実装はevidence_relevance.rsへの追記ではなく、新しいcore moduleへ分離した。

prototype API:

- EvidenceLocalAuthorityV1
- EvidenceAuthoritySource
- resolve_evidence_local_authority_v1
- materialize_evidence_relevance_v20

current redesign branchでのvalidation:

- generic authority/property control: 6/6 PASS
- immutable v23 provider-observation replay: 48/48 x 3
- immutable holdout v1 provider-observation replay: 26/26 x 3
- immutable holdout v2 provider-observation replay: 26/26 x 3
- immutable holdout v3a provider-observation replay: 26/26 x 3
- completed v4 Mistral artifactをauthority-v1でpostmortem replay: 26/26
- completed v4 Google artifactをauthority-v1でpostmortem replay: 26/26
- core full suite: 597 passed / 0 failed
- providers: 153 passed / 1 ignored / 0 failed
- CLI: 368 passed / 4 ignored / 0 failed
- workspace all-target Clippy -D warnings: PASS
- rustfmt / diff check: PASS
- production special-case scan: clean

v4 Mistral/Google replayはpostmortem evidenceに限る。frozen v4 resultをrescoreしたりPASSへ変換したりしない。Groq v4完走後にv4 result recordとauthority-v1 semantic freeze可否を確定する。

## Prototypeから得た設計上の知見

replayで次の4 boundaryをcase-specific patchではなくauthority primitiveとして明示化した。

1. model-only blocking riskはstronger Harness-visible structureを自動的に上書きしない。
2. generic/local target absenceとtarget-specific requested-relation absenceを分離する。
3. semantic-equivalent policyではcanonical name不在だけでrepeated paraphrased subjectをnegative ownershipにしない。
4. target tokenを内包するlonger distinct entityでもexplicit structural distinctnessがあればexact-target ownershipへ昇格しない。

いずれもfixture例外ではなくauthority compositionのpropertyとして扱う。
