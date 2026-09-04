# セマンティック決定可能性とエビデンス充足性の研究

> Successor research: RSD0 residual evidence-sufficiency discovery の記録は
> [evidence-sufficiency-rsd0.md](evidence-sufficiency-rsd0.ja.md) にある。採用された D3 gate が、より広い
> answerability evidence が不十分または混在していても正しく `permit` を返せることを示している。

Issue #73 は、却下された R4 semantic successor study に続くものである。目的は model
agreement をより権威あるものにすることではない。assertive な soft semantic decision を進めては
ならない harness-owned conditions を特定することである。

`soft-semantic-v3` は引き続き runtime baseline である。本書は calibration-only research を記述するもので、runtime behavior は変更しない。

## 研究フェーズの命名

Phase label は各 research issue に固有であり、runtime version や release version ではない。

- `R1`–`R4` は Issue #59 の semantic-successor **research** stages である。`R4` は特に fourth stage、すなわち frozen independent successor/holdout-v4 evaluation を意味する。
- `D1`–`D3` は Issue #73 の **decidability** stages である。`D1` は deterministic gate contract と calibration surface、`D2` は provider-backed decidability calibration、`D3` は D2 が pass した場合の candidate freeze/adoption preparation である。

prefix は意図的に、新しい #73 calibration phase が過去の #59 successor sequence と混同されるのを防いでいる。`D2` は「version 2」を意味せず、runtime-version の意味で `R4` より新しいわけでもない。

## 研究課題

provider-neutral かつ harness-owned の decidability/evidence-sufficiency gate により、typed binding または evidence qualification が不十分なときに `abstain` を強制し、相関した semantic over-assertion を減らせるか。それは既存の deterministic authority boundary を弱めず、有用な decision coverage を崩さずに実現できるか。

gate は control-plane mechanism であり、truth estimator ではない。

## R4 の教訓とデータ境界

R4 は cross-model disagreement が有用な risk evidence である一方、cross-model agreement は correctness evidence ではないことを示した。却下された successor は、この phase が対象とする具体的な failure mode、つまり複数の model が同じ安定した assertive semantic error を出し得ることも示した。

Holdout-v4 は観測済みの immutable diagnostic history である。relabelling、copy、transform、gate rule の導出、calibration に使ってはならない。D1/D2 中に holdout-v5 は作成しない。future holdout-v5 は、calibration candidate を predeclared adoption criteria とともに freeze した後に限り許可される。

## 設計原則: 正しさではなく許可

deterministic D1 gate は、既知の harness-owned blocker が abstention を要求するかを報告する。 otherwise permitted な decision が correct だとは報告しない。

```text
DecidabilityDisposition
  permit
  force_abstain
```

`permit` が意味するのは、gate が自ら所有する typed information の中に deterministic blocker を見つけなかった、ということだけである。evidence、verification conclusion、confidence score、verdict ではない。

初期の deterministic reason vocabulary は意図的に小さくする。

```text
missing_target_binding
missing_proposition_binding
no_evidence_for_explicit_requirement
no_qualified_evidence_for_explicit_requirement
conflicting_qualified_evidence
```

詳細な evidence-qualification reasons は既存の `EvidenceQualificationInspection` が引き続き所有する。decidability layer は競合する qualification ontology を作るのではなく、それらの結果を参照または要約すべきである。

## 単調合成

gate は soft semantic decision を維持するか、より conservative にすることしかできない。

```text
(base finding,    permit)        -> finding
(base no_finding, permit)        -> no_finding
(base abstain,    permit)        -> abstain
(base finding,    force_abstain) -> abstain
(base no_finding, force_abstain) -> abstain
(base abstain,    force_abstain) -> abstain
```

Operational failure または protocol failure は semantic result ではなく、この composition table に入らない。Malformed semantic output は gate が修復しない。

gate は次のことができない。

- soft または hard finding を作る。
- `abstain` を assertive decision に変える。
- trusted evidence または `VerificationReceipt` を作る。
- epistemic state を promote する。
- final verdict を決定する。
- model agreement を truth として再解釈する。
- operational failure を `no_finding`、`finding`、`abstain` に変換する。

## 再利用する既存の型付き情報

D1 は、model に harness がすでに所有する metadata を繰り返させるのではなく、既存の harness-owned contracts を再利用すべきである。

### 対象とバインディング

`SoftJudgeRequest.target` は proposition、causal-relation、claim、inference の target をすでに区別している。claim/inference target について、harness は参照された artifact objects が存在し、qualification に必要な propositions が実際に bind されていることを確認できる。

missing claim/inference または required missing proposition binding は structural blocker である。model に prose からその binding を推論させてはならない。

### エビデンス要件と適格性

core が所有するものは次のとおりである。

- `EvidenceRequirement { proposition, as_of_unix_seconds, scope, minimum_authority_class }`;
- `EvidenceMetadata { temporal, scope, provenance_class }`;
- `EvidenceAuthorityPolicy`;
- `EvidenceQualificationInspector`;
- `EvidenceQualificationAssessment { Qualified, Disqualified, Unknown }`;
- explicit temporal、scope、authority、metadata、conflict reason codes。

これらは、semantic question がその proposition requirement に直接 bind されている場合に限り、deterministic evidence-sufficiency signals の primary source となる。Endpoint requirement を inference により relation-level または applicability-level requirement に promote してはならない。

`EvidenceRequirement` がないこと自体は insufficiency の evidence ではない。deterministic D1 blocker が許されるのは、harness が explicit requirement/binding を作成し、その requirement が unsatisfied または unqualifiable であることを示せる場合だけである。

これにより、「すべての semantic fact が formalize されるまで abstain」という mechanism になることを防ぐ。

### 因果情報

existing causal inspector も harness-owned だが、通常の `CausalSupportStatus::Unknown` または `MissingCausalEvidence` は自動的に abstention を force してはならない。それらの state は、soft semantic judge がそもそも有用になる理由になり得る。

D1 は structural causal binding failure を使ってよいが、deterministic causal `Unknown` を semantic inspection が不可能である証明として再解釈すべきではない。D1 v1 では、cause または effect proposition に対する generic `EvidenceRequirement` を directional causal relation の requirement として扱わない。future causal sufficiency gate には、まず explicit typed relation-level binding が必要である。

## D1 決定論的アルゴリズム

first research implementation は、`SoftJudgeRequest` と relevant `ReasoningArtifact` に対する pure deterministic function とすべきである。

High-level behavior:

1. target が claim または inference の場合、artifact に対して target identity を validate する。
2. semantic question が proposition conflict/support に直接関係する diagnostic kind（`contradiction` と `unsupported_premise`）についてのみ、explicit proposition requirements を derive する。
3. required proposition binding が missing なら `force_abstain` を返す。
4. explicit relevant requirements に対して `EvidenceQualificationInspector` を run/reuse する。
5. explicit requirement に candidate evidence がなければ `force_abstain` を返す。
6. explicit requirement に candidates はあるが `Qualified` が一つもなければ `force_abstain` を返す。
7. explicit requirement の qualified evidence が conflicting なら `force_abstain` を返す。
8. それ以外は `permit` を返す。

exact target-to-proposition derivation は conservative に保つこと。`contradiction` と `unsupported_premise` では、proposition target は自身に map し、claim は explicit proposition binding を介して map できる。claim/inference target には、該当する場合、structural target/proposition bindings がなお必要である。`counterexample` は generic proposition evidence requirement を applicability rule として継承せず、`causal_gap` は cause/effect proposition requirements を relation-level sufficiency rule として継承しない。free-text semantic binding は発明しない。

target に explicit evidence requirement がない場合、またはその requirement が semantic question の precondition として explicit typed されていない場合、有用そうだからという理由だけで D1 は推論しない。

## D1/D2 キャリブレーション対象

すべての historical holdout から分離した、新しい calibration-only fixture family を作成する。fixture format には ordinary semantic request と concrete harness-owned artifact の両方を含め、provider なしで decidability result を再現できるようにする。

paired/metamorphic cases を使う。各 pair は semantic target/concern を保持し、typed sufficiency precondition だけを mutate する。

Required mutation families:

1. complete proposition binding -> binding を remove;
2. qualifying evidence -> explicit requirement key を満たすすべての evidence を remove;
3. sufficient authority -> provenance を minimum authority class 未満に lower;
4. applicable scope -> scope を narrow または disjoint にする;
5. temporally valid evidence -> required `as_of` 時点で stale または not-yet-valid にする;
6. complete required metadata -> temporal/scope/provenance metadata を remove;
7. one qualified value -> conflicting qualified values。

すべての insufficiency mutation には、`permit` のまま残る paired control を用意する。

deterministic D1 mutation corpus は、v1 が defensible typed blocker を持つ三つの kind、すなわち proposition support/conflict（`contradiction`、`unsupported_premise`）と structural claim binding（`counterexample`）を対象とする。`causal_gap` は、harness が explicit typed relation-level evidence requirement を所有するまで permit-only control とする。D1 label は gate eligibility であり、semantic truth ではない。Provider-backed D2 semantic label は四つすべての kind について別途報告する。

## 事前宣言した決定論的ゲート

provider-backed D2 run の前に、次を満たすこと。

- 100% insufficiency-mutation monotonicity: declared mutation が `permit -> force_abstain` に移る。
- 100% paired-control preservation: sufficient control が `permit` のまま残る。
- 100% composition invariants: gate は decision を preserve するか `abstain` に移すことしかできない。
- malformed/missing target reference は fail closed し、semantic output を作らない。
- existing deterministic tests に authority-boundary regression が zero。
- calibration runner が holdout path を accept しない。

これらは contract gate であり、model-quality claim ではない。

## D2 ラベル軸とプロバイダー由来の指標

D2 は semantic polarity と assert permission を一つの label に collapse してはならない。したがって fixture には、observation 前の独立した二つの label が必要である。

```text
semantic_label     = positive | negative | ambiguous
assertive_eligibility = permit | force_abstain
```

`semantic_label` は supplied semantic content における diagnostic concern を記述する。eligibility label は harness-owned typed precondition が assertive soft decision を許すかを記述する。どちらの label も provider output から derive しない。

この分離は metric bug を避ける。explicit evidence requirement が unsatisfied の positive/negative case は conservative に `abstain` を force されるべきであり、expected abstention を false negative と数えると、正しい gating によって semantic recall が構造的に低く見える。

matched D2 case では、unchanged R2 materialized semantic request を provider/seed ごとに一度実行する。その後、paired harness-owned artifact variant に deterministic gate を apply する。semantic request content が同一で harness-owned qualification metadata だけが異なる二つの variant は、model を二度 sample せず同じ provider observation を再利用しなければならない。これにより gate intervention と model-sampling noise を分離し、provider call を減らす。

checked-in D2 v1 calibration manifest はこの設計に従い、`fixtures/semantic-judges/` のみを source とする 15 semantic case、すなわち 11 eligible positive/negative case と four eligible ambiguous control を含む。clear case のうち seven には、evidence presence、scope、temporal validity、authority、required provenance metadata、claim binding、qualified-evidence conflict を対象とする一つの paired `force_abstain` variant もある。force subset は three diagnostic kind にまたがり、`causal_gap` は意図的に除外する。causal case は relation-level sufficiency が typed されるまで permit control のままである。Existing semantic label は D2 manifest に copy し、provider credentials を読む前に source fixture と deterministic に照合する。

`reason-decidability-study` は exact D2 path を validate し、typed gate expectation をすべて解決した上で、semantic case/seed ごとに unchanged R2 materialization call を exactly one 回実行する。同じ returned decision をすべての typed variant と compose する。Operational failure では全 variant decision を unset のままにし、abstention に変換しない。この段階では live D2 provider observation は repository に記録しない。

Optional model-backed residual decidability gate は後続の別 arm であり、D1 result と混ぜてはならない。

provider/model ごと、trial ごとに次を report する。

- unchanged R2 semantic call の provider/protocol completion;
- **eligible positive/negative** case の semantic precision/recall のみ;
- eligible clear-case decision coverage;
- eligible ambiguous abstention（typed insufficiency とは分けて report）;
- `force_abstain` variant の typed-insufficiency abstention rate;
- composition 前後の `force_abstain` variant における unsafe assertive rate;
- gate escalation count/rate と deterministic reason distribution;
- overall decision coverage は descriptive のみ（maximum は predeclared ineligible variant 比率に依存するため）;
- base semantic decision と composed result の cross-seed stability;
- token と latency cost（deterministic gate overhead は別途 report）。

複数 model を truth label に pool してはならない。`force_abstain` case を通常の positive/negative recall failure として score してはならない。

## 凍結した D2 v1 観測計画

first provider-backed D2 observation は、D2 provider call 前に freeze する。checked-in `semantic-decidability-d2` workflow には study-shaping input がなく、次を固定する。

- configuration: `semantic-decidability-d2-v1`;
- corpus: `fixtures/semantic-judges/` のみを source とする、checked-in D2 calibration manifest 15 件すべて;
- providers/models（separately evaluated）: Google `gemini-3.5-flash-lite` と Mistral `ministral-8b-latest`;
- seed `6000` から `6004` までの five sequential trials;
- `512` maximum output tokens;
- semantic case/seed ごとに unchanged R2 semantic observation を一つ実行し、その typed variant 間で reuse;
- cross-model pooling、voting、provider-specific semantic branch はなし。

provider study が semantically scorable になるのは、`5/5` complete trial にわたる全 `75/75` call が complete した場合だけである。Operational incompleteness はこの exact frozen configuration で rerun できるが、partial semantic denominator から rescore してはならず、fixture、seed、model、threshold、prompt semantics の変更も正当化しない。

predeclared D2 candidate gate は次のとおり。

- aggregate eligible clear-case decision coverage `>= 0.90` per provider;
- aggregate eligible precision と recall `>= 0.95` per provider;
- 各 complete trial の eligible clear-case decision coverage、precision、recall `>= 0.90`;
- typed-insufficiency abstention は aggregate と各 complete trial で exactly `1.0`;
- typed-insufficiency variant の composed unsafe assertion は exactly `0`;
- eligible clear semantic fixture に cross-seed `decision_disagreement` がない;
- deterministic preflight が全 declared `permit` control と authority/operational boundary を preserve する;
- typed-insufficiency subset に、少なくとも一つの provider で non-zero base unsafe-assertion count がある。なければ deterministic contract が正しくても D1 は empirical utility を demonstrate していない。

eligible ambiguous abstention は D2 adoption threshold ではなく、別途 report する diagnostic のままである。D1 v1 は permit-only semantic ambiguity を意図的に rewrite しないためである。この observation 後の threshold change には、新しい calibration configuration identity が必要であり、D2 v1 を in place で edit してはならない。

deterministic-gate calibration candidate を freeze する価値があるのは、同じ provider-neutral rule が次を満たす場合だけである。

- typed-insufficiency abstention が 1.0、composition 後の unsafe assertive decision が 0;
- 少なくとも一つの evaluated provider で typed-insufficiency subset の non-zero base unsafe-assertion rate を減らす（そうでなければ empirical benefit は unproven）;
- per provider の eligible clear-case decision coverage >= 0.90;
- 定義される範囲で eligible assertive precision と recall >= 0.95;
- predeclared `permit` control への gate escalation が zero;
- provider/model-specific semantic branch を追加しない;
- hard authority と operational-failure invariant を preserve する。

always-abstain mechanism は eligible clear-case coverage と permit-control preservation により fail する。高い insufficiency-abstention score に隠れることはできない。

deterministic gate が tautological metadata failure だけを捕捉し、over-assertion の provider-backed reduction を meaningful に提供しないなら、D1 は historical holdout に対して post hoc に拡張せず、insufficient として記録する。

## D2 v1 の観測結果

Frozen GitHub Actions run `33377619803` は、merge commit `f7d99d80336c7854195dbd0f826dd9bcca3e3457` からの exact checked-in D2 v1 plan を測定した。observation 後に fixture、model、seed、token budget、threshold は変更していない。両 provider arm は operationally complete だった。

| provider/model | calls | complete trials | clear coverage | clear precision | clear recall | typed insufficiency abstention | base unsafe -> composed unsafe | ambiguous abstention | clear seed disagreement |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Google `gemini-3.5-flash-lite` | 75/75 | 5/5 | 1.000 | 1.000 | 1.000 | 35/35 = 1.000 | 35 -> 0 | 1.000 | 0 |
| Mistral `ministral-8b-latest` | 75/75 | 5/5 | 1.000 | 1.000 | 1.000 | 35/35 = 1.000 | 35 -> 0 | 0.750 | 0 |

両 provider とも、各 complete trial の clear coverage/precision/recall は個別に `1.000`、typed-insufficiency abstention は `1.000`、composed unsafe assertion は zero だった。したがって deterministic gate は predeclared D2 candidate gate をすべて pass し、両 provider で non-zero empirical utility も示した。unchanged R2 semantic decision は各 provider の 35 typed insufficiency variant すべてで assertive となり、D1 は 35 件すべてを `abstain` に移した。

Mistral の ambiguity diagnostic は意図的に visible のままとし、tune away していない。`14_causal_partial_intervention_ambiguous` は seed `6000` から `6004` まで `finding` を返し、他の three eligible ambiguous control は abstain した。これは D1 v1 の typed force surface 外であり、endpoint requirement を causal-relation requirement に拡張する根拠ではない。

Frozen run の operational cost は Ministral 8B で aggregate 31,820 tokens / 43,231 ms fixture latency、Gemini 3.5 Flash-Lite で 35,267 tokens / 291,945 ms だった。deterministic composition 自体に追加 provider call は不要である。

## D3 凍結候補

D2 は freeze には十分だが、provider-neutral successor candidate を一つ adopt するにはまだ不十分である。

```text
candidate_id              semantic-decidability-d3-v1
semantic_baseline         soft-semantic-v3
materialization_contract  materialization-r2-v1
decidability_contract     deterministic-explicit-typed-preconditions-v1
composition               permit_preserves__force_abstain_only
```

candidate scope は checked-in D1 contract と完全に同じである。semantic question がその proposition に直接 bind されている場合に限り、missing typed target/proposition binding と unsatisfied explicit proposition evidence requirement に対して abstention を force できる。counterexample applicability requirement は infer せず、cause/effect endpoint requirement を directional causal requirement に promote せず、model confidence/agreement を使わず、operational failure を決して repair しない。

この D3 candidate は runtime version ではなく、`soft-semantic-v3` を変更しない。independent adoption test は、この candidate freeze 後にのみ作成された newly authored observation-free holdout-v5 を使う。Provider result は separate のままであり、majority/pooled rescue はない。frozen corpus、label/spec review、typed mutation set、hash、execution plan、adoption gate は [semantic decidability holdout-v5](semantic-decidability-holdout-v5.ja.md) に記録されている。この freeze 時点で holdout-v5 provider observation は存在しない。

各 provider の predeclared holdout-v5 adoption gate は次のとおり。

- `5/5` operationally complete trial と 100% provider/protocol completion;
- aggregate eligible clear decision coverage `>= 0.90`;
- aggregate eligible precision と recall `>= 0.95`;
- 各 complete trial の eligible clear coverage、precision、recall `>= 0.90`;
- typed-insufficiency abstention exactly `1.0` と composed unsafe assertion exactly `0`;
- declared `permit` control に deterministic escalation が zero;
- eligible clear semantic fixture の cross-seed `decision_disagreement` が zero;
- typed-insufficiency subset に、少なくとも一つの provider で non-zero base unsafe assertion;
- provider-specific prompt/config branch がなく、post-observation relabelling、threshold change、corpus repair、selective rerun がない。

provider arm が operationally incomplete なら、exact frozen run は operational failure に限って repeat できる。partial semantic denominator は adoption result に数えない。frozen semantic gate が fail した場合、D3 は holdout-v5 に対して repair せず reject する。

## 凍結後のパイロット状況と安定化の方向性

上記の original D3 freeze と predeclared gate は historical protocol として残る。subsequent observation がその plan を書き換えることはない。current evidence は次のとおりである。

- Mistral `ministral-8b-latest`: holdout-v5 は 120/120 calls、5/5 complete trials、eligible clear coverage/precision/recall `1.000`、typed-insufficiency abstention `50/50`、base unsafe assertion `50 -> 0`、clear-case seed disagreement zero。
- Google-hosted `gemma-4-31b-it`: R2、D2、holdout-v5 の independent cross-family replication。v5 は 120/120、5/5 で完了し、同じ clear metrics と `50 -> 0` unsafe reduction を示した。120 base decision は、matched case/seed observation において Ministral 8B と完全一致した。
- Google `gemini-3.5-flash-lite`: Issue #84 は quota reset 後、Actions run `33380880478` attempt 2 で exact frozen rerun を完了した。120/120 calls、5/5 complete trials、eligible clear coverage/precision/recall `1.000`、typed-insufficiency abstention `50/50`、base unsafe assertion `50 -> 0`、permit-control escalation zero、clear-case seed disagreement zero、provider/protocol failure zero。Ambiguous abstention は `32/40 = 0.800`、three ambiguous fixture に seed disagreement があった。これは diagnostic であり frozen adoption threshold 外である。
- NVIDIA `nvidia/nemotron-3.5-lightning-30b-a3b`: bounded negative-control probing が protocol-capability boundary を露呈させた。D2 は 7/15 successful observation と、forbidden model-owned `finding` field に起因する eight materialization-protocol failure を生成した。dependent v5 probe は 40-minute job timeout 前に 18/24 fixture まで到達した。partial semantic score は adoption に使わない。

これらの observation により、`semantic-decidability-d3-v1` は R2 protocol capability boundary を満たす model に対する current stabilization/adoption candidate として扱える。ただし universal model compatibility を establish したわけではなく、provider-specific semantic tuning も正当化しない。

operational stabilization layer は semantic contract を変更せず、最初の four hardening requirement を実装している。

1. R2 materialized-decision protocol の corpus-independent capability/preflight reporting;
2. quota、rate-limit、timeout、provider、protocol failure の typed operational telemetry;
3. incomplete 中は semantic scoring を明示的に禁止する、atomic partial-result preservation;
4. immutable runtime/config identity と `soft-semantic-v3` への explicit rollback profile。

stabilization change は CI gate が pass するまで `soft-semantic-v3` を default に保った。その後、separate reversible adoption change により compiled default を `semantic-decidability-d3-v1` に切り替えた。`soft-semantic-v3` は explicit rollback profile として残る。[semantic runtime stabilization and adoption](semantic-runtime-stabilization.ja.md) を参照。

two independent model family が同じ v5 safety pattern を reproduce した後は、model-matrix expansion は secondary である。additional model は単に model count を増やすためではなく、specific capability boundary を test するときに追加すべきである。

D3 stabilization 後の first successor research hypothesis は、以下の residual soft-decidability arm である。deterministic typed gate が represent できない insufficiency を calibration corpus が示した場合にのみ開くべきである。Selective/conformal uncertainty は後続の risk-control candidate とし、causal relation-level sufficiency は directional relation evidence に explicit typed binding ができるまで deferred とする。

## 残余のソフト決定可能性は別個の仮説

D1 は missing decisive distinction のすべての形を検出できるとは意図的に主張しない。insufficiency 自体が semantic であり、current typed metadata では表現できないことがある。

後続の calibration arm では、`sufficient | insufficient | mixed` のような narrow model-backed decidability output を test してよい。ただし deterministic surface を characterize した後に限る。そのような arm を test する場合は次の条件を満たす。

- model 間で同じ provider-neutral semantics を受ける;
- harness がすでに所有する authority field を見たり emit したりできない;
- `insufficient`/`mixed` は `abstain` のみを force できる;
- `sufficient` は correctness evidence を構成しない;
- operational failure は semantic decision と separate のままにする;
- model consensus または majority vote と combine せず、distinct coordinate として evaluate する。

## 文献上の根拠

これらの source は answerability/evidence sufficiency と answer generation の分離を motivate するものであり、harness authority semantics を定義するものではない。

- Rajpurkar, Jia, Liang, [*Know What You Don’t Know: Unanswerable Questions for SQuAD*](https://aclanthology.org/P18-2124/) (ACL 2018): supplied context が answer を support しない場合、guess を force せず answerability を detect すべきである。
- Thorne et al., [*FEVER: a Large-scale Dataset for Fact Extraction and VERification*](https://aclanthology.org/N18-1074/) (NAACL 2018): `NotEnoughInfo` は support/refutation と distinct であり、evidence は verification task の一部である。
- Xin et al., [*The Art of Abstention: Selective Prediction and Error Regularization for Natural Language Processing*](https://aclanthology.org/2021.acl-long.84/) (ACL 2021): abstention は accuracy だけでなく risk/coverage trade-off として evaluate すべきである。
- Joren et al., [*Sufficient Context: A New Lens on Retrieval Augmented Generation Systems*](https://research.google/pubs/sufficient-context-a-new-lens-on-retrieval-augmented-generation-systems-2/) (ICLR 2025): context sufficiency は generation quality とは別の variable であり、selective generation は sufficiency information を使って answered case の correctness を改善できる。
- Wang et al., [*SConU: Selective Conformal Uncertainty in Large Language Models*](https://aclanthology.org/2025.acl-long.934/) (ACL 2025): conformal uncertainty は risk-controlled selection を支援できるが、distribution/exchangeability assumption は verification authority ではなく明示的に扱う必要がある。
- Gu et al., [*Bridging the Detection-to-Abstention Gap in Reasoning Models under Insufficient Information*](https://arxiv.org/abs/2605.28070) (2026 preprint): solving 前の explicit answerability control decision により、missing information を検出しても assertively answer する case を対象にする。

## 採用と後継版の順序

D1 -> D2 -> D3 freeze -> holdout-v5 sequence は original provider arm について complete である。Ministral 8B は pass し、Gemini 3.5 Flash-Lite は Issue #84 で exact frozen rerun を完了し、Gemma 4 は pattern を independent に replicate し、Nemotron は protocol-capability boundary を document した。次の作業順は次のとおり。

1. [done] semantic retuning なしで frozen D3 contract と runtime/config identity を stabilize する。
2. [done] capability preflight、failure telemetry、partial-result preservation、rollback を harden する。
3. [done] `soft-semantic-v3` を rollback baseline として保持しつつ、separate reversible change として runtime adoption を行う。
4. [done #84] candidate を変更せず exact frozen Gemini v5 rerun を complete する。
5. [done #91] fresh calibration-only residual-gap corpus を作り、current typed D3 が measurable evidence-sufficiency distinction を miss することを prove する。
6. [done #91] monotone な `sufficient | insufficient | mixed` coordinate を evaluate し、seed/model across の risk/coverage を characterize し、`sufficient` に authority を与えず fresh independent frozen holdout を pass する。
7. [done #129/#134/#113] residual coordinate を versioned product wiring、explicit rollback、claim-local requirement policy v2、target-aware/shared-render NL-5 dogfood を通じて promote する。
8. selective/conformal abstention または causal relation-level sufficiency は、それぞれ固有の calibration/holdout identity と typed prerequisite を持つ新たな follow-on research としてのみ evaluate する。

Successor は observed holdout-v5 content を tuning、relabelling、threshold selection、corpus repair に使ってはならない。
