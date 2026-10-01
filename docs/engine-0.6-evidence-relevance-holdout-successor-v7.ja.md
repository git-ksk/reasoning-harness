# Engine 0.6 evidence relevance holdout successor v7

状態: immutable holdout v7 FAIL 後の pre-freeze successor candidate。effective qualification v9 + materialization v22 を generic control、immutable replay、Mistral + Google development で検証中。successor-v7 semantics freeze を commit/tag するまで fresh holdout v8 の authoring は禁止する。

## 観測された境界

canonical holdout v7 run `36800959088` attempt 1 は Mistral / Google / Groq の全 provider が 26/26 operational 完走、provider failure 0、wrong-target Relevant 0。required correctness gate は PASS。effective authority qualification は3 provider共通で25/26。Mistral / Google は materialization 26/26、Groq は25/26かつ utility miss 1。

3 providerで共通した authority miss は、URL-only Harness target identity + substantive single near-sibling signal の fresh family 1件。relation は意味上 availability だが、deterministic availability の小さな lexical set に含まれない表現だった。Mistral / Google は別の conservative path により Ambiguous を維持した一方、Groq は `different + distinct_target + no risk` が揃い historical negative-target path まで fall-through した。

v7 run / tag / label / artifact は immutable。successorでは replay evidence としてのみ使う。

## Generic root cause

既存 single-near-sibling identity floor が `requested_relation_locally_present()` に依存していた。この predicate は relation axis 用の bounded lexical helper なので、同じ requested relation を lexical set 外で言い換えるだけで identity ownership rule 自体が消える。

これは cross-axis coupling である。

- single near-sibling が identity authority として十分かは identity axis の問題;
- substantive proposition が requested relation を表すかは別の relation axis の問題;
- provider/model disagreement は欠陥を露出させるが authority の根拠ではない。

production logic に holdout wording、provider、fixture ID、synthetic entity、case family を追加して修正してはいけない。

## Versioning decision

- effective qualification: v9 を追加
- materialization: v22 を追加
- v8/v21 implementation は変更しない
- local qualification model contract は v8 のまま。model stage は追加しない
- historical v1-v7 observation / result artifact は変更しない

## Successor-v7 identity floor

`require_harness_anchor` の strict policy では、content-bearing Harness target anchor が無い状態の substantive single near-sibling identity signal 1件だけでは distinct-target authority を作らない。

この floor は relation-orthogonal とする。requested relation が lexical に検出できるか、model が requested / different / unresolved のどれに分類するかに依存しない。v9 は identity axis のみ `unresolved` に落とし、prior semantics が得た relation scope は保持する。

明示的な separate/distinct/replacement/successor など、より強い Harness-owned distinct-identity evidence がある場合はこの floor を適用しない。

URL-only / navigation-footer-only target occurrence は引き続き non-owning context とする。

## Materialization boundary

v22 はまず v9 をderiveする。relation-orthogonal single-near-sibling identity floor が active の場合、v21へdelegateする前に Ambiguous を materializeする。これにより、Harness-owned identity authority が unresolved のままなのに advisory `different/distinct_target` agreementだけで historical negative path が Irrelevant を作ることを防ぐ。

それ以外は v21 にそのままdelegateする。

## Generic controls

pre-freeze controls では少なくとも以下を要求する。

- URL-only target + single near sibling + non-lexical relation paraphrase => identity Unresolved / final Ambiguous
- navigation-only target + single near sibling + non-lexical relation paraphrase => Unresolved / Ambiguous
- target anchor無し + single near sibling + non-lexical relation paraphrase => Unresolved / Ambiguous
- relation scope は独立保持し、identity floor が `requested_relation` を捏造しない
- explicit distinct identity evidence は DistinctTarget のまま
- content-bearing exact target anchor は downgrade しない
- prior positive / repeated-sibling negative / mapping・ownership risk / absence / injection / contradiction behavior は immutable replay で維持する

production branch は provider / fixture ID / synthetic entity / family / exact holdout text に依存してはならない。

## Immutable replay

successor-v7 replay の対象:

- v23 fixed regression surface
- holdout v1
- holdout v2
- holdout v3a
- holdout v4
- holdout v5
- immutable holdout v7 canonical observations
- captured successor-v6 Mistral + Google development observations

historical Groq v7h25 の disposition は replay fixture 内で Irrelevant のまま保存する。同じ immutable proposal/raw observation を v9/v22 に通した場合だけ frozen expected authority と Ambiguous に回復する。wrong-target Relevant は0を維持する。

## Development policy

freeze前は fresh reusable development profile を candidate shaping にのみ使う。

- Mistral `ministral-8b-latest`
- Google `gemini-3.5-flash-lite`
- Groq は iterative development から除外
- fresh identity を使う synthetic non-holdout 12 case。relation paraphrase single-sibling boundary と relation orthogonality control を含む
- 両providerで successor-owned identity authority + scope-risk / materialization / correctness / utility の convergence を要求する。relation scope exactness は診断値として残す。v9はv8のrelation軸を再調整せず保存する必要があるため。

development runner は frozen v7 holdout runner と分離する。semantics freeze 前に fresh holdout v8 runner/corpus authoring は開始しない。

### Development observation 36889080034

candidate commit `57b29326f7499f62e24c9d01d16074c00b6d85cf` の最初のreusable runは両providerともoperationalには完走したが、当初のfull-qualification gateはFAILした。Mistralはfull effective-authority qualification 9/12、Googleは11/12。一方、両providerともmaterialization 12/12、wrong-target Relevant 0、utility miss 0、identity-scope miss 0、scope-risk miss / spurious risk 0だった。missはすべてrelation-scopeのみで、MistralとGoogleのmiss集合も一致しなかった。

この観測を理由にrelation lexiconやprovider固有の調整は行わない。Successor v7が変更するのはidentity authorityのみである。generic controlでは同一proposal/raw入力に対してv9がv8のrelation scopeとscope riskを完全に保存し、single-near-sibling identity軸だけを変更することを固定する。live development convergenceはsuccessor-owned axis + materialization / correctness / utilityをgateし、full relation-scope mismatchは診断値として記録し続ける。full qualificationはsemantics freeze後のfresh holdout v8でも引き続き観測する。

### Development convergence 36890934124

candidate commit `41d853080a991b3a9f3a976b758d8b338ce2c784` の2回目のreusable runはsuccessor-owned development gateをPASSした。両providerとも12/12完走、identity-scope miss 0、scope-risk miss / spurious risk 0、materialization 12/12、wrong-target Relevant 0、false relevance rejection 0、relevant-left-Ambiguous 0、utility miss 0。relation-scopeのみの診断missはMistral 3件（full qualification 9/12）、Google 1件（11/12）のままだが、これを理由とするproduction semantics変更は行っていない。

成功runのraw observationとv2 gate summaryは `fixtures/evidence-relevance-successor-v7-development/` にcaptureし、offline replayする。replayではcaptured provider inputに対してv9がv8のrelation scope / scope riskを保存すること、successor-owned identity軸がdevelopment labelと一致すること、v22が全caseを期待どおりmaterializeすることを確認する。capture後、live development workflowは再びmanual-onlyとする。

### Development surface independence audit

freeze前監査でdevelopment-v1はidentity-freshではないことが判明した。`Juniper Vault`、`Ruby Queue`、`Saffron Bridge` のcanonical entity名が過去holdoutから再利用され、Ruby Queueはtask textも再利用されていた。development corpusはimmutable holdout v7観測後に作成した非holdout evidenceなのでholdout v7を汚染しないが、docsのfresh identity主張とは不一致であり、最終pre-freeze development surfaceとしては採用しない。

live v2 provider観測前に、独立した `evidence-relevance-successor-v7-development-v2` surfaceを別途authorした。12 caseはholdout v1-v7、successor-v5/v6 development、development-v1に対してcase ID / canonical entity / task / exact signal / exact 8-token signal n-gram overlapがすべて0。5指標のzero-overlapをdeterministic CI testで固定する。v2は同じgeneric semantic familyとv9/v22 expected labelを保つsurface/provenance correctionであり、production semantic changeではない。

### Independent development-v2 convergence 36937880336

独立development-v2 surfaceの最初のlive observationはcandidate commit `8021a37b22da3d7176f641da2821417db93b08b4` で実行し、successor-owned two-provider gateをPASSした。両providerとも12/12完走、identity-scope miss 0、scope-risk miss / spurious risk 0、materialization 12/12、wrong-target Relevant 0、false relevance rejection 0、relevant-left-Ambiguous 0、utility miss 0。Mistralはfull effective-authority qualificationも12/12、Googleはrelation-scopeのみの診断miss 2件を残して10/12だった。この結果を理由とするproduction semantic changeは行っていない。

v2のraw observationとsummaryは `fixtures/evidence-relevance-successor-v7-development-v2/` にcaptureし、unchanged v9/v22 semanticsに対してoffline replayする。capture後、live development workflowは再びmanual-onlyとする。

## Freeze blocker

以下がすべてgreenになるまで successor-v7 semantics をfreezeしない。

- successor-v7 generic controls
- development label deterministic self-consistency
- holdout v7までの immutable replay
- 全replayで wrong-target Relevant 0
- Mistral + Google development convergence
- provider / fixture / entity / text special-case scan
- relation / identity orthogonality audit
- repeated-sibling / explicit-distinct regression audit
- mapping / ownership / truncation / context-gap audit
- prompt-injection / conflicting-local-evidence audit
- core と affected CLI tests
- all-target Clippy `-D warnings`
- rustfmt
- workflow YAML validation
- semantics checksum

semantics freeze tag をpushした後にのみ fresh independent holdout v8 authoringを開始できる。
