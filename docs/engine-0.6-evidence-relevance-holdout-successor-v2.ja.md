# Engine 0.6 evidence relevance holdout successor v2 design

Status: successor semanticsは実装・replay validation済み。fresh independent holdoutはまだfreeze / observeしていない。

immutable independent holdout v1がzero-wrong-target hard gateをFAILしたため、このsuccessorを新しいsemantic identityとして作る。holdout v1はimmutable FAILのままで、tuning surfaceにはしない。

## Versioned successor

- effective qualification: **v4**
- materialization: **v17**
- predecessor calibration semantics: effective qualification v3 + materialization v16
- issue: #462

production ruleにprovider名 / fixture ID / synthetic entity名 / holdout固有case branchは入れない。

## Authority precedence

v1 failureはidentity-authority precedence defectを露出した。

Harness-owned canonical/alias occurrenceが証明するのはcandidate内に名前が存在することだけで、proposition ownershipではない。

v4:

1. deterministic scope-risk blockerは従来どおりfail-closed。
2. advisory target bindingがExactでもindependent raw verifierがscope riskなしの `distinct_target` なら、lexical Harness anchorだけで `exact_target` へ昇格させない。
3. Harness-owned target-local syntaxにより、substantive target-name occurrenceがすべてcontext-only / comparison frameだと確認できる場合だけ `distinct_target` としてcorroborateする。それ以外は `unresolved` へabstainする。
4. model outputが両方exactでも、target名がbounded context-only frameにしか現れないならexact-target authorityを作らず `unresolved`。
5. relation axisはorthogonal。pricing / availability / limit等の語があることは、そのrelationをどのentityが所有するかを証明しない。

現在のdeterministic context-only frameはtarget-localに限定する: `unlike TARGET`、`versus TARGET`、`vs TARGET`、`not TARGET`、`compared to/with TARGET`、`rather than TARGET`、`in contrast to TARGET`、`as opposed to TARGET`。canonical名とaliasをnormalized identity phraseとして扱い、長いspanから評価して完全に重なる短いaliasをdedupeする。一方、同じsignal後半などに重ならないsubstantive alias occurrenceがあればcontext-only判定にはしない。

このcueはnegative / abstention authorityだけで、positive relevanceは生成しない。

## Materialization v17

v17がinterceptするのはadvisory proposalがexact target identityを主張するpositive identity-authority conflictだけ。

- raw `distinct_target` + deterministic target-local context-only corroboration -> Irrelevant
- deterministic corroborationのないexact/raw-distinct conflict -> Ambiguous
- model 2出力がexactでもtarget occurrenceがcontext-only -> Ambiguous
- 既存proposal=Differentのnegative pathはv16へdelegateし、弱めない
- 通常のexact-target positive materialもv16へdelegate

hard gateは変更しない。

- wrong-target relevance retention = 0
- false relevance rejection
- Relevant utility
- materialization exactness
- authority qualification
- operational failureとsemantic failureを分離

## Regression evidence

canonical-artifact replay fixtureはimmutable observationからの派生物であり、新規observationでもfrozen runのrescoreでもない。

- v23 canonical run `36400085595`: v4/v17で144/144 provider observationがexpected authority/materializationを維持
- holdout v1 run `36495389012`: Groq 26/26維持、Mistral 24/26・wrong-target Relevant 0維持、Googleは25/26へ改善し旧h14 wrong-target Relevantを安全にreject
- generic focused unit testでcomparison-only corroborated conflict、uncorroborated exact/raw-distinct conflict、2-exact context-only conflict、通常positive preservation、無関係なcomparison marker非干渉を固定

## Fresh successor holdout rule

successor semanticsを固定した後にだけ、新しいindependent holdoutをauthorする。

v1 failure case / entity名 / candidate textをコピーしたり、h14の表層paraphraseを作ってrepair確認だけを行うことは禁止。新suite/surface identityを持ち、positive ownership、context-only mention、sibling/comparison structure、relation ownership、ambiguous identity、alias、distributed evidence、non-comparison controlを独立にcoverする。

holdout v1 observationはpostmortem replayだけに使う。fresh successor holdoutがoriginal zero-wrong-target hard gateを独立にPASSするまで#462はcloseしない。

## Groq operational admission

successor live freeze前に:

- holdout v1 Groq実消費65,244 tokensとobservation timestampからmodeled TPD headroomを再計算する;
- tiny readinessはtransport / credential / TPM / RPD evidenceだけで、TPD-headroom proofとしない;
- fail-closed TPD admission floor / pacing / observed-token cap / reserveを事前固定する;
- materialなorganization-level Groq利用が既知・疑わしい場合はmodelをinvalidate / re-anchorする;
- operational failureとsemantic failureを分離する。

このadmission計算、local validation、exact surface checksum、public-safety scan、PR CIがgreenになるまでsuccessor live freezeは行わない。
