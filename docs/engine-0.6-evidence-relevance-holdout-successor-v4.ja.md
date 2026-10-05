# Engine 0.6 evidence relevance holdout successor v4

Status: immutable holdout v3a FAIL後のdesign / implementation candidate。

## Versioning

- effective qualification: v6
- materialization: v19
- historical v3/v16、v4/v17、v5/v18 semanticsは変更しない
- v3a replayは新しいregression surfaceであり、historical artifactを書き換えない

## Successor rule

1. Repeated authorized positive identity:
   - strict Harness anchor policy
   - deterministic risk none
   - 同じcanonical nameまたはauthorized aliasがtitle/heading系signalと別excerpt/fact系signalに出現
   - target occurrenceがcomparison/context-onlyではない
   - requested relationがlocalに存在
   - 2 model stageが完了しrequested relationに独立一致
   - v6 effective identityがexact
   - model identity voteがdifferent/distinctでもv19はRelevantをmaterialize可能

2. Single-signal near sibling:
   - target anchorもURL-only target anchorもない
   - deterministic risk none
   - identity-capable signalが1つだけ
   - target tokenの一部共有と別non-generic tokenを持つ
   - requested relationがlocalに存在
   - explicit deterministic distinct-target cueなし
   - v6はidentityをunresolvedへ戻し、v19はAmbiguous

3. Context-only target + repeated sibling:
   - target occurrenceがすべてcomparison/context-only
   - stable sibling subjectが別local signalに反復
   - deterministic risk none
   - v6はdistinct-target ownershipを確立可能
   - relationをdifferentへ変えるのは、textがrequested relationを明示的に除外し、両model stageもdifferent relationへ独立一致する場合だけ
   - このruleからv19はIrrelevantを作れるがRelevantは作らない

4. URL-only unnamed ownership:
   - target identityを持つのがURLだけ
   - local textがowning productをunnamed/unidentifiedとする
   - v6はunresolved + context_gap
   - v19はAmbiguousを維持

## Regression contract

semantic freeze前に:
- focused v19 control PASS
- immutable v23 replay 48/48 x3
- immutable holdout v1 replay 26/26 x3
- immutable holdout v2 replay 26/26 x3
- immutable holdout v3a replayをsuccessor semantics上26/26 x3へ回復
- wrong-target Relevant 0維持
- workspace test、Clippy、rustfmt、workflow/YAML、special-case scan PASS
- production branchingにprovider名、fixture ID、synthetic entity名、exact holdout textを含めない

successor-v4 semantics surfaceをfreezeした後でのみfresh independent holdout v4をauthorする。

## Pre-freeze validation

- focused v19 control: 6/6 PASS
- successor-v4 replay: v23 48/48 x3、holdout v1 26/26 x3、holdout v2 26/26 x3、holdout v3a 26/26 x3
- core: 585 passed / 0 failed
- providers: 153 passed / 1 ignored / 0 failed
- CLI: 368 passed / 4 ignored / 0 failed
- workspace all-target Clippy -D warnings: PASS
- rustfmt check: PASS
- git diff check: PASS
- workflow YAML parse: 92/92
- production special-case scan: clean
- v3a replay provenance shape: 3 providers x 26 PASS
