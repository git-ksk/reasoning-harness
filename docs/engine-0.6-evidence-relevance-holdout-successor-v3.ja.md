# Engine 0.6 evidence relevance holdout successor v3

Status: immutable holdout v2 FAIL後のdesign / implementation candidate。

## Evidence boundary

independent holdout v2はimmutable FAILのまま保持する。

- freeze: `engine-0.6-evidence-relevance-holdout-v2-freeze`
- commit: `c38b5f7dbfcf8c01dcaa6f6cd7341e43d3b87325`
- canonical run: `36533340582`、attempt 1 only
- required 3 providerは全て26/26 operational完走
- Mistral / Groq / Googleすべてwrong-target Relevant retention 0
- Groqはauthority/materialization 26/26 PASS
- Mistral / Googleは `v2h13_negative_unrelated_launch` の1件だけexpected Irrelevant -> observed Ambiguous

successorはfrozen observationをpostmortem / regression用途でreplayできるが、holdout v2をrerun / rescore / relabel / mutateしない。

## Generic gap

v4/v17はstrict Harness target anchorが無い候補をpositive Relevantへ通さない安全境界を正しく維持した。一方、両model stageがidentityをover-bind / under-resolveした場合、Harness自身では次の2種類を分離できない。

1. URL-only、mapping、ownership、truncation、omitted referent等で本当にidentityがAmbiguousな候補
2. visibleなsibling subjectが複数の独立local signalで一貫して反復され、requested relationもそのsubjectへ属するlocally substantive candidate

mere target-anchor absenceだけではnegative identity evidenceとして不十分で、Ambiguous維持が必要。

## Successor rule: repeated sibling subject

次の全条件を満たす場合だけHarness-owned negative identity corroborationを許可する。

1. identity policyが `require_harness_anchor`;
2. identity-capable signalにcanonical / authorized alias target anchorが無い;
3. target-only canonical URL signalも無い;
4. deterministic local scope riskが `none`;
5. requested relationがlocal materialに存在する;
6. `SourceTitle` / `Heading` / `StructuredMetadata` のnormalized 2-4 token phraseが、
   - Harness target canonical/alias identity tokenを最低1 token共有し、
   - target外かつgeneric/relation語ではない追加tokenを最低1つ含み、
   - 別の `Excerpt` / `Fact` / `StructuredMetadata` signalでもtoken boundary上exactに反復される;
7. advisory model 2 stageが両方完了しており、deterministic ruleがoperationally missingなproposal/verifierを隠さない。

この反復phraseはlocally stableなsibling subjectのevidenceとしてのみ使う。positive identity / Relevant authorityは一切作らない。

## Explicit non-goals / abstention controls

以下では発火しない。

- `allow_semantic_equivalent` policy
- URL-only target identity
- single-signal near-name mention
- target token断片が別signalへ分散したpartial identity
- explicit alias / rename / successor uncertainty
- omitted/shared ownership
- clipped/truncated context
- pronoun-only / unnamed-subject relation passage
- 既存target-absence pathで処理するgeneric landing/catalog absence

これらは既存v17 semanticsへdelegateし、従来AmbiguousならAmbiguousを維持する。

## Versioning

- effective qualification: v5
- materialization: v18
- holdout v1 runnerはv3/v16維持
- holdout v2 runnerはv4/v17維持
- successor replayだけ別versionのv5/v18を使う

v18は、repeated-sibling ruleが成立し、v5 effective identity=`distinct_target`、relation=`requested_relation`、deterministic/effective riskがともにnone、model 2 stageが存在する場合だけIrrelevantをmaterialize可能。それ以外はimmutable v17へdelegateする。

## Regression contract before semantic freeze

successor-v3 semantics freeze前に以下を必須とする。

- trigger positiveと全abstention boundaryのgeneric property/unit control PASS
- immutable calibration v23 replayで3 providerとも48/48 authority/materialization維持
- immutable holdout v1 replayでwrong-target Relevant 0を維持し、historical result自体は変更しない
- immutable holdout v2 replayでGroq 26/26を維持し、Mistral/Google v2h13 terminal utility missを安全に解消しつつwrong-target Relevant / false rejectionを新規発生させない
- full core/providers/CLI、workspace all-target Clippy、fmt、diff、YAML、public-safety、production special-case scan PASS
- production branchingにprovider名、fixture ID、synthetic entity名、exact holdout textを入れない

successor semanticsを先にfreezeした後でのみfresh independent holdout v3 surfaceをauthorする。

## Pre-freeze validation

local validation:

- repeated-sibling property / abstention control: 8/8 PASS
- successor v3 replay: v23 48/48 x 3、holdout v1 26/26 x 3、holdout v2 26/26 x 3
- holdout v1 historical Google h14はrecorded Relevantのまま保持し、successor semantics上だけIrrelevant
- holdout v2 historical Mistral / Google v2h13はrecorded Ambiguousのまま保持し、successor semantics上だけIrrelevant
- core full suite: PASS
- providers: 153 passed / 1 ignored / 0 failed
- CLI: 205 passed / 3 ignored / 0 failed + integration suites green
- workspace all-target Clippy `-D warnings`: PASS
- fmt / diff / workflow YAML: PASS
- public-safety / production special-case scan: clean

intended semantics freeze tag: `engine-0.6-evidence-relevance-successor-v3-semantics-freeze`。fresh holdout v3はこのtag作成後にのみauthorする。
