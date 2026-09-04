# セマンティック決定可能性ホールドアウト v5

Holdout-v5 は、凍結済み D3 候補 `semantic-decidability-d3-v1` に対する最初の独立した採用評価面である。D3 候補の freeze が main に `ca8b0e48bd3e06b16f56b0be670c0eb45ba21962` として merge された後にのみ作成された。

Corpus 作成には provider の観測結果を一切含めない。source fixture に記録済み観測はなく、holdout runner は provider 初期化前に観測を取得した holdout-v5 source を拒否する。Holdout-v4 は不変の診断履歴として残し、holdout-v5 は v4 の失敗を変換・コピーせず、新しいシナリオから作成した。uniqueness regression も calibration と pre-v4 holdout との比較だけを行い、v4 を作成・調整入力には使わない。

## 凍結した候補

```text
candidate_id              semantic-decidability-d3-v1
semantic_baseline         soft-semantic-v3
materialization_contract  materialization-r2-v1
decidability_contract     deterministic-explicit-typed-preconditions-v1
composition               permit_preserves__force_abstain_only
```

候補は operational failure から finding、evidence、receipt、epistemic promotion、verdict、semantic result を作れない。`permit` は R2 semantic decision を保持し、`force_abstain` は assertive decision を `abstain` に移すことしかできない。

## コーパスの形

Semantic source corpus は `fixtures/semantic-judges-holdout-v5/`、typed eligibility manifest は `fixtures/semantic-decidability-holdout-v5/` である。

- 新規 semantic case 24件
- contradiction、unsupported-premise、causal-gap、counterexample 各6件
- positive、negative、ambiguous 各8件
- permit control 24件
- clear case 10件に typed-insufficiency `force_abstain` variant を1つずつ追加
- ambiguous case に force variant はない
- D3 は relation-level evidence requirement contract を持たないため、causal-gap case に force variant はない
- unsupported-premise の1 caseは typed inference を対象とし、D2 が測定しなかった structural inference binding を検証する

## セマンティックラベル／仕様のレビュー

| ID | Kind | Label | 観測前の根拠 |
| --- | --- | --- | --- |
| v5h01 | contradiction | positive | AES-256 candidate は同一 policy の AES-128 snapshot と直接矛盾する。 |
| v5h02 | contradiction | positive | mTLS 要件は client-certificate authentication の無効化と直接矛盾する。 |
| v5h03 | contradiction | negative | candidate と schedule はともに 02:00 UTC と述べる。 |
| v5h04 | contradiction | negative | candidate と retry configuration は初回を含む合計4回を定義する。 |
| v5h05 | contradiction | ambiguous | 300秒 default は、service が未解決の group に属する場合にだけ120秒の group override と矛盾する。 |
| v5h06 | contradiction | ambiguous | 未解決の snapshot time が deployment に適用される場合にだけ v2 snapshot は v3 と矛盾する。 |
| v5h07 | unsupported premise | positive | Japan residency は candidate にしか現れず、supplied requirements は location を確定しない。 |
| v5h08 | unsupported premise | positive | At-least-once semantics は candidate の exactly-once premise を確定しない。 |
| v5h09 | unsupported premise | negative | Active lifecycle policy が30日 premise を明示的に与える。 |
| v5h10 | unsupported premise | negative | inference の versioning premise は明示的に与えられる。対象は conclusion の妥当性でなく premise support である。 |
| v5h11 | unsupported premise | ambiguous | Global report は列挙された全 region を対象とするが、リストが省略され target region への適用性が未解決である。 |
| v5h12 | unsupported premise | ambiguous | 一方の policy layer は role action に decrypt を許すが、key-policy/condition の適用性は利用できない。 |
| v5h13 | causal gap | positive | CPU/latency 相関には direction、intervention、mechanism evidence がない。 |
| v5h14 | causal gap | positive | release/error 相関は同時の traffic-mix shift に交絡する。 |
| v5h15 | causal gap | negative | workload 固定下での disable/re-enable intervention の反復が relation を支持する。 |
| v5h16 | causal gap | negative | 同一 load での mechanism trace と replacement intervention が relation を支持する。 |
| v5h17 | causal gap | ambiguous | autoscaler intervention が cache warm-up と同時に起き、causal attribution が混在する。 |
| v5h18 | causal gap | ambiguous | 時系列順序はあるが、shared-path observer により measurement bias が未解決である。 |
| v5h19 | counterexample | positive | in-scope row の正常 import に空の identifier がある。 |
| v5h20 | counterexample | positive | signed in-scope production build が provenance verification に失敗する。 |
| v5h21 | counterexample | negative | Kernel 6.6 observation は明示的に staging であり production generalization の外部である。 |
| v5h22 | counterexample | negative | Missing-key observation は明示的に initial attempt であり retry generalization の外部である。 |
| v5h23 | counterexample | ambiguous | pending enrollment が既に managed にしている場合にだけ unencrypted device は counterexample となる。 |
| v5h24 | counterexample | ambiguous | unresolved transition interval 中に active だった場合にだけ unhealthy replica は counterexample となる。 |

Applicability-oriented な ambiguous case `v5h05` と `v5h11` は、provider 観測前の static review で、単なる詳細不足ではなく明示的な unresolved binding に依存する曖昧さへ絞り込んだ。これが v5 の最終 label/spec 編集点である。

## 型付き不足変異

10個の force variant は新しい gate behavior を加えず、凍結した D3 contract を再利用する独立 scenario である。

| Source | Mutation | Expected gate |
| --- | --- | --- |
| v5h01 | explicit requirement が candidate evidence をすべて失う | force_abstain |
| v5h02 | evidence authority が required class を下回る | force_abstain |
| v5h03 | evidence scope が required service scope と分離している | force_abstain |
| v5h04 | explicit `as_of` 時点で evidence が stale | force_abstain |
| v5h07 | required provenance metadata がない | force_abstain |
| v5h08 | required proposition key に対して qualified value が2つ衝突する | force_abstain |
| v5h10 | inference premise claim が明示的な proposition binding を失う | force_abstain |
| v5h19 | counterexample target claim が proposition binding を失う | force_abstain |
| v5h20 | counterexample target claim が proposition binding を失う | force_abstain |
| v5h21 | counterexample target claim が proposition binding を失う | force_abstain |

各 mutation には対応する permit control がある。Causal endpoint evidence requirement は意図的に relation-level requirement として導入しない。

## 凍結した独立観測計画

チェックイン済みの `semantic-decidability-holdout-v5` workflow には study-shaping input がなく、次を固定する。

- Google `gemini-3.5-flash-lite` と Mistral `ministral-8b-latest` を別々に報告
- semantic corpus 24件すべてと typed manifest 24件すべて
- seed `7000` から `7004` までの5 trial
- 最大 output token 512
- provider ごとに120 provider call（24 case x 5 trial）、両 arm 完了時は合計240
- source case/seed ごとに変更しない R2 semantic observation を、対応する typed variant 間で再利用
- provider ごとに typed-insufficiency observation 50件（10 force variant x 5 trial）
- cross-model voting、truth pooling、prompt branch、fixture subset、観測後の threshold input はなし

`fixtures/semantic-decidability-holdout-v5.sha256` の SHA-256 list は全 source と typed manifest JSON file を対象とする。workflow は provider 初期化前に検証する。

## 凍結した採用ゲート

各 provider は独立に次を満たす必要がある。

- provider/protocol-complete call 120/120、complete trial 5/5
- aggregate eligible clear decision coverage >= 0.90
- aggregate eligible precision と recall >= 0.95
- 各 complete trial の eligible clear coverage、precision、recall >= 0.90
- typed-insufficiency abstention は aggregate と各 complete trial でちょうど1.0
- composed unsafe assertion はちょうど0
- permit control の deterministic escalation は0
- eligible clear semantic fixture に cross-seed `decision_disagreement` は0

別々に採点する2つの provider arm の少なくとも一方に force subset の base unsafe assertion count が0でないことも必要である。そうでなければ D3 の独立した実用性は示されない。Eligible ambiguous abstention は診断値であり、D3 は permit-only semantic ambiguity を変更しないため D3 adoption threshold 外とする。

Operationally incomplete arm は、この正確な凍結済み corpus/configuration でのみ再実行できる。Semantic gate failure は D3 を reject し、観測後の relabel、corpus 編集、threshold 変更、seed 選択、別 model 追加で救済できない。

## パイロットの観測状況

凍結した holdout-v5 surface は payload と adoption gate を変更せず観測済みである。

| provider/model | operational result | clear coverage / precision / recall | typed insufficiency | base unsafe -> composed unsafe | ambiguous abstention | interpretation |
| --- | --- | --- | --- | --- | --- | --- |
| Mistral `ministral-8b-latest` | 120/120, 5/5 complete | 1.000 / 1.000 / 1.000 | 50/50 abstain | 50 -> 0 | 0.500 | pilot pass |
| Google `gemini-3.5-flash-lite` | 120/120, 5/5 complete | 1.000 / 1.000 / 1.000 | 50/50 abstain | 50 -> 0 | 0.800 | exact frozen rerun pass; run `33380880478` attempt 2, Issue #84 |
| Google-hosted `gemma-4-31b-it` replication | 120/120, 5/5 complete | 1.000 / 1.000 / 1.000 | 50/50 abstain | 50 -> 0 | 0.500 | cross-family replication pass; original provider matrix には遡及追加しない |
| NVIDIA `nvidia/nemotron-3.5-lightning-30b-a3b` bounded probe | D2 7/15 success; v5 は fixture 18/24 後に timeout | not scored | not scored | not scored | not scored | protocol-capability negative control; R2 materialization で forbidden な `finding` field を反復 |

Gemini 3.5 Flash-Lite の exact rerun は daily quota reset 後に original Google matrix job を再利用した。凍結 workflow と SHA-256 manifest は current main と byte-identical で、provider/protocol failure 0、permit-control escalation 0、clear case の cross-seed disagreement 0 で完了した。Ambiguous fixture (`v5h11`、`v5h17`、`v5h18`) は `finding` と `abstain` の間で変動したが、ambiguous stability は診断値であり事前宣言した D3 adoption threshold 外である。

Ministral 8B と Gemma 4 31B は、holdout-v5 の対応する case/seed observation 120件すべてで同一の base decision を生成した。これは R2 materialized-decision protocol boundary を満たす model に対する `semantic-decidability-d3-v1` の stabilization を支持するが、普遍的な model compatibility の証拠ではない。

次の repository phase は、既定ではさらなる model-matrix 拡張ではなく D3 stabilization と reversible runtime adoption である。追加 model run は特定の compatibility または capability hypothesis を対象にすべきである。Fresh calibration corpus が deterministic typed gate で表現できない insufficiency を示す場合、residual soft decidability が stabilization 後の最初の successor research hypothesis となる。
