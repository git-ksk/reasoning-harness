# セマンティック判定器の選択的棄権安定性研究

Issue #59 R3 は、bounded disagreement を **risk signal** として使い、soft semantic decision を保守的に `abstain` へエスカレーションできるかを調べる。これは calibration-only research であり、trusted evidence、hard findings、verification receipts、epistemic promotion、verdict authority を生み出せない。

## シード安定性だけでは不十分な理由

反復 R2 materialization study は2つの failure mode を分離した。

- Gemini 3.5 Flash-Lite は protocol-complete だったが、1つの ambiguous fixture が seed 間で変化した。
- Ministral 8B は R2 の decision-owned/model、binding-owned/harness contract の下で 90/90 protocol-complete かつ完全に seed-stable だった一方、ambiguous abstention は全 trial で 0.5714 だった。

つまり model は **stably assertive** になり得る。R3 は seed disagreement 以上のものを測らなければならない。

## プローブ軸

最初の R3 surface は provider-neutral な2軸を組み合わせる。

1. seed perturbation;
2. information-equivalent R2 output representations.

R2 ownership contract と semantic decision guidance は固定する。変えるのは model-facing representation だけである。

- `decision_note_object`;
- `compact_decision_note_object`;
- `nested_decision_note_object`.

decision labels は canonical のままとする。model-owned `kind`、`target`、provenance、evidence、authority fields は、どの representation でも禁止する。

## リスク評価

1つの fixture について、harness は設定されたすべての probe を valid soft decision または operationally incomplete observation として記録する。operational failure を `no_finding` その他の semantic claim に変換することは決してない。

評価では次を独立して記録する。

- decision disagreement;
- operational incompleteness;
- no successful observation.

vote count を truth と解釈しない。

## 選択候補

R3 は事前宣言した2つの candidate を報告する。

- `disagreement_only`: successful probes が食い違えば `abstain`。operational incompleteness は可視のままだが、successful decision が unanimous ならそれ自体では上書きしない。
- `complete_unanimity`: 設定した全 probe の成功と一致を要求し、disagreement または missing probes があれば保守的に `abstain`。

両 candidate は precision、recall、decision coverage、ambiguous abstention、risk-fixture count、abstention escalation count を報告する。有用な coverage を壊すことで abstention だけを増やす candidate は合格しない。

これらは research policies であり、runtime defaults ではない。

## 実行設計

`reason-stability-study` は各 fixture/trial 内で3つの R2 representations を interleave し、`(fixture_index + trial) mod 3` で順序を rotate する。これにより provider-time/order drift を抑える。workflow は、full calibration matrix より先に causal positive/negative/ambiguous triad から開始する。

受け付けるのは、この checkout の canonical `fixtures/semantic-judges` directory だけである。historical holdouts は R3 tuning から引き続きブロックする。

## 測定した R3 キャリブレーション結果

最初の18-fixture single-trial R3 representation study は、2つの異なる regime を示した。Gemini 3.5 Flash-Lite では cross-representation disagreement のある ambiguous fixtures が2つあり、unanimity-based selective abstention は両方を `abstain` にエスカレーションした。その結果、precision/recall 1.0、ambiguous abstention 1.0、decision coverage 0.6111 となった。Ministral 8B は3つすべての R2 representations で18/18 protocol-complete かつ同一 decisions を返し、ambiguous abstention は0.5714のままだった。

Mistral の結果は stable-miscalibration/self-consistent-error case である。seed と representation の一致は、correctness や十分な uncertainty handling を意味しない。したがって R3 は完全な reliability mechanism ではなく、有用だが bounded な detector として特徴づける。

## R3b モデル横断リスク

R3b は、複数の model/provider を利用できる deployment 向けに、optional な別の risk axis を加える。各 source には同じ R2 semantic/materialization contract と canonical `decision_note_object` representation を適用する。model identity が影響してよいのは adapter mechanics だけで、別の semantic prompt や decision rule を選ぶことはできない。

cross-model outputs は probes であり、votes ではない。successful sources が disagreement した場合、既存の unanimity evaluator は保守的に `abstain` を返す。すべての source が一致しても soft decision を維持できるだけで、権限は増えない。operationally missing sources は別の risk signal として残り、より厳しい complete-unanimity candidate で処理できる。

CLI は N 個の distinct `provider:model` sources を受け付ける。最初の GitHub Actions surface は、初回 calibration study では意図的に2 sources に限定する。この方針は Tan et al., *Too Consistent to Detect: A Study of Self-Consistent Errors in LLMs* (EMNLP 2025, DOI 10.18653/v1/2025.emnlp-main.238) に基づく。同研究は self-consistent errors が same-model consistency detectors では検出しにくく、cross-model evidence が直交する signal になり得ることを示す。

## R3b 再キャリブレーションと R4 引き継ぎ

five-seed all-calibration R3b run (`33368618724`) は 180/180 provider calls を完了した。cross-model disagreement は4つの ambiguous fixtures に限られ、3つの causal ambiguity cases は全 seed で disagreement し、1つの contradiction-binding ambiguity case は5 seed 中3 seed で disagreement した。positive または negative fixture は、どの seed でも disagreement しなかった。combined `disagreement_only` result は precision/recall 1.0、ambiguous abstention 1.0、decision coverage 0.6111、clear-case coverage 1.0 を維持した。

これは independent test へ進むには十分だが、general correctness を主張するには不十分である。R4 thresholds と candidate identity は、holdout-v4 の最初の provider observation より前に freeze した。`semantic-successor-r4.md` を参照。
