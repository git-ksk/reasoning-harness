# Fresh independent evidence-sufficiency holdout v1（新規独立 evidence-sufficiency holdout v1）

Tracking: #91, #125. Execution workflow: `evidence-sufficiency-holdout-v1`.

この holdout は RSD0-RSD2 の calibration/characterization 後、product-bridge adoption claim の前に作成された。RSD calibration corpus は再利用していない。commit された corpus は8つの新しい family にまたがる24ケースで、各 family には `sufficient`、`insufficient`、`mixed` がちょうど1ケースずつ含まれる。checksum manifest は provider call 前に検証される。固定された semantic holdout-v4/v5 は変更しない。

## 凍結した実行（固定実行）

- models: `ministral-8b-latest` and Google-hosted `gemma-4-31b-it`;
- 24 fixtures x 5 trials = 120 observations/model;
- seeds: 7000-7004;
- maximum output tokens: 128;
- surface: `holdout`;
- workflow input から corpus、model set、trial count、seeds、threshold を変更できない。

Promotion gate は観測前に固定された。

1. operational completion = 1.00
2. conservative binary accuracy >= 0.95
3. false-safe count = 0
4. false-abstain rate <= 0.05
5. sufficient recall >= 0.95
6. binary fixture unanimity = 1.00

## 観測された結果（観測結果）

GitHub Actions run `33568061693` は main commit `c59790891911f9e75b85ac9cd30eb07994bec707` 上で、両方の model arm に合格した。

| metric | Ministral 8B | Gemma 4 31B |
| --- | ---: | ---: |
| operational completion | 1.000 | 1.000 |
| conservative binary accuracy | 1.000 | 1.000 |
| false-safe count | 0 | 0 |
| false-abstain rate | 0.000 | 0.000 |
| sufficient recall | 1.000 | 1.000 |
| binary fixture unanimity | 1.000 | 1.000 |
| exact 3-class accuracy | 0.8833 | 1.000 |
| exact fixture unanimity | 0.9583 | 1.000 |

Ministral の14件の exact-label error はすべて `mixed -> insufficient` だった。そのため、いずれも事前宣言した `non_sufficient` 側の safety boundary に留まり、non-sufficient から sufficient、またはその逆に越境したケースはなかった。これは多数決や threshold change で隠さず、diagnostic drift として報告する。

固定 holdout は事前宣言した task-specific な `required_information` を供給するが、後続の product mechanism がそれらの requirement を生成または選択することを検証するものではない。最初の product bridge は `generic-answer-sufficiency-requirements-v1` を使った。後に NL-5 で、この policy が safe partial fact を過度に抑制し得ることが示されたため、product successor は `d3-sufficiency-answer-gate-v2` の下で別個の claim-local policy `claim-local-answer-sufficiency-requirements-v1` を version 管理する。どちらの product policy も、遡及的に holdout-validated として扱わない。

## 権限の解釈（権限の解釈）

この holdout に合格したことが示すのは、conservative product gate を promote するための証拠だけである。`sufficient` は non-authoritative のままであり、trusted evidence、verification receipt、hard finding、epistemic-state promotion、final verdict を生み出せない。verification、bounded resolution、abstention を強制できるのは `insufficient` / `mixed` だけである。product usefulness は NL-5 で別途評価し、固定 holdout は決して tuning data にしない。
