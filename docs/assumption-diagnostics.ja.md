# Assumption と unsupported-premise の診断

Assumption diagnostic layer が問うのは狭い process question、つまり「trusted support または harness input の明示的 permission なしに、どの proposition が inference premise として使われたか」である。General fact checker ではなく、任意の world knowledge を推測しない。

## 入力の権限

`HarnessInput.assumptions` は harness が premise として明示的に許可する proposition を含む。これは harness-owned で `ReasoningCandidate` には存在しないため、model が自分の assumption を承認することはできない。Assumption と `hypotheses` は別物で、hypothesis は task が評価を求める proposition、explicit assumption は独立検証を主張せず task が許す premise である。

## ステータス

- `supported`: premise claim が `known`/`supported`、または trusted supported/known claim か explicit input assumption に導出の底がある `inferred` claim
- `explicit_input_assumption`: premise proposition が harness-owned explicit assumption と完全一致
- `unsupported`: typed proposition はあるが trusted support も explicit assumption status もない。supplied context に対する hard **process** finding
- `unbound`: typed proposition がない。support は deterministic に検査できず、finding は soft

Candidate が書いた epistemic label は support を与えない。特に `inferred` を提案しても、上記のとおり derivation が独立に grounded されない限り premise は trusted にならない。

## `unknown` との違い

`unknown` は claim の epistemic state で、harness に accept/reject の十分な authority がないことを示す。`unsupported_premise` は **use** に関する diagnostic で、supplied context が support/explicit permission を与えない typed proposition を inference premise として実際に使ったことを示す。従って artifact は unsupported-premise finding を含みながら overall `unknown` verdict を保持できる。

## Unsupported causal edge との違い

Unsupported premise は inference input が grounded かを問う。Unsupported causal edge は特定の typed cause→effect relation に causal evidence があるかを問う。Premise が fact として supported でも、それを使う causal relation は unsupported のままであり、causal relation は別の ungrounded premise を修復できない。2つの diagnostic family は分離したままである。

## 報告と権限

`fixtures/assumptions/` の5-case deterministic corpus は final verdict accuracy と causal corpus から分けて報告する。Repeated-trial diagnostic aggregation には assumption finding を含められるが、finding が verification receipt を作成したり claim state を変更したり `accept | reject | unknown` を直接強制したりすることはない。Free-form prose からの semantic assumption extraction は将来の soft diagnostic 課題である。

## Evidence qualification との連携

Assumption inspection は standard runtime sequence で hard verification の後に実行する。Input が temporal/scope/provenance evidence requirement を指定する場合、built-in structured hard support を作れるのは requirement を通過した evidence だけである。従って unqualified structured fact が `inferred` premise を trusted-support closure に bootstrap することはない。
