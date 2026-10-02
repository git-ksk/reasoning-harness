# Engine 0.6 evidence relevance holdout v6 immutable result

Canonical run `36750505629` attempt 1 は immutable **FAIL**。

失敗は provider observation 前に発生した。freeze済み holdout-v6 tag / surface は変更しない。

- tag: `engine-0.6-evidence-relevance-holdout-v6-freeze`
- freeze commit: `a8782eefd2609385ed7e598552aced05ddee0660`
- semantics baseline: effective qualification v8 + materialization v21
- Mistral / Google / Groq: runner RC 1、completed cases 0
- 共通failure: `unexpected checkpoint suite id "evidence-relevance-holdout-v6"`

原因は v6 runner で `HoldoutProfile::V6` を追加した一方、独立した `checkpoint_profile()` mapping が holdout v5 までのままだったこと。validate-only は checkpoint write を通らないため、pre-observation validation で wiring defect を検出できなかった。

これは operational harness evidence であり semantic evidence ではない。v6 case outcome は1件も観測しておらず、provider output を successor semantics の shaping に使わない。

後続では v6 を rerun / rescore / relabel せず、v8/v21 をfreeze済みのまま維持し、checkpoint/profile completenessをgenericに修正・回帰testで固定した上で、holdout v1-v6 と successor-v5/v6 development corpus に対して overlap 0 の fresh holdout をauthorする。
