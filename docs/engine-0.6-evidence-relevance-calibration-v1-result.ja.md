# Engine 0.6 evidence-target relevance calibration v1 結果

Status: frozen v1 observationはoperationalには完走したが、correctness / utility gateはFAIL。この結果はimmutable historical calibration evidenceとして保持し、変更後semanticsでrerun / rescoreしない。

## Frozen identity

- freeze tag: `engine-0.6-evidence-relevance-calibration-v1-freeze`
- candidate commit: `6c7551ecb9abffca5e42efc852cd4222995d9cc5`
- GitHub Actions run: `35991268202`
- suite: `evidence-relevance-calibration-v1`
- cases: 26
- seed: `4621601`
- Mistral: `ministral-8b-latest`
- Google: `gemini-3.5-flash-lite`

preflightと両provider armは成功。final semantic gateは、両armがv1で`ambiguous`期待だった1 caseを`relevant`として保持し、さらにutility mismatchが残ったためFAILした。

## Raw v1 metrics

| metric | Mistral | Google |
| --- | ---: | ---: |
| successful provider cases | 26/26 | 26/26 |
| provider failures | 0 | 0 |
| proposal exact accuracy | 20/26 (76.92%) | 23/26 (88.46%) |
| materialized exact accuracy | 20/26 (76.92%) | 24/26 (92.31%) |
| reported wrong-target relevance retention | 1 | 1 |
| expected-relevantへのfalse relevance rejection | 0 | 0 |
| expected-relevant left ambiguous | 0 | 0 |
| utility misses | 5 | 1 |
| deterministic safety overrides | 0 | 2 |
| lexical baseline exact accuracy | 12/26 (46.15%) | 12/26 (46.15%) |
| lexical baseline wrong-target relevance retention | 8 | 8 |
| lexical baseline expected-relevant misses | 2 | 2 |
| model calls | 26 | 26 |
| provider attempts | 26 | 26 |
| total tokens | 9,783 | 10,310 |
| model-call latency total | 15,332 ms | 20,440 ms |

semantic pathは単純lexical baselineを大きく上回ったが、v1 release gateは満たさない。

## Shared correctness finding: case 24

両providerが`24_conflicting_sections`を`relevant`と判定し、v1 expectedは`ambiguous`だった。

frozen v1 caseは、2つのpassageがどちらもexact targetとavailability relationを明示的に扱いながら、事実回答だけが衝突している。一方はWest対応、もう一方はWest未対応としている。

これはcalibration fixtureが **semantic relevance** と **truth / contradiction** を混同していた。互いに矛盾したclaimを含んでいても、material自体はtargetへ直接relevantであり得る。矛盾は後段のcontradiction / qualification / verification / finalizationが扱う責務で、relevance gateが内容の不一致だけで除外すべきではない。

したがってv1の`wrong-target relevance retention` metricはこのcaseをcorrectness violationとして過剰計上している。frozen v1結果は書き換えず、successor calibrationではtruth-conflictではなく本当にrelevance/bindingが曖昧なconflict caseを新suite identityで作る。

## Utility finding: irrelevant vs ambiguous

Mistralはv1 ambiguity 5 case（unknown rename、partial identity、mixed multi-product binding、insufficient local passage、URL-only identity）を`irrelevant`へ潰した。Googleはunknown renameだけを`irrelevant`にし、partial identityでの`relevant` proposalはstrict Harness-owned identity floorが安全に`ambiguous`へoverrideした。

v1 model guidanceでは次の境界が不足している。

- `irrelevant`はmaterialが別target / 別relationだと肯定的に判断できる場合に限定する;
- missing/partial binding、rename/alias不確実、truncated local support、multi-product applicability未解決は`irrelevant`ではなく`ambiguous`;
- candidate insufficiencyをirrelevanceの根拠にしない。

これはutility上の区別で、false rejectionは不要な追加acquisitionや有用materialの破棄につながる。

## Confirmed v1 positives

- 両arm 26/26 provider call完走、operational failure 0;
- strict identity anchor不在時のunsafe model `relevant`をHarnessがblock;
- URL-only / weak signal identityはself-authorizeしない;
- 日英alias / semantic-equivalent caseをlexical overlap必須にせず処理;
- stale-but-relevantをfreshnessと混同しない;
- semantic pathはlexical baselineのwrong-target retention 8件から、v1 conflict-label finding 1件まで大幅に改善。

## v2 requirements

successor calibrationはfrozen v1 identityを書き換えない。

1. `irrelevant`はaffirmative wrong-target/wrong-relationに限定し、applicability/binding未解決は`ambiguous`とするdecision ruleを明確化;
2. truth-conflict caseを新suite identityのrelevance/binding-conflict caseへ置換;
3. strict Harness-owned identity floor / no-authority-promotion invariantを維持;
4. relevanceとfreshness / authority / verification / contradiction / truth / answer sufficiencyの分離を維持;
5. production motivating incidentはtuningから除外;
6. new freeze identityでのみMistral / Google canonical calibrationを再観測。

successor calibrationがfrozen acceptance criteriaへ到達するまでindependent holdoutはauthorしない。
