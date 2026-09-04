# セマンティック後継版 R4 の独立評価

Issue #59 R4 は、新しい observation-free holdout 上で凍結済み `cross-model-selective-abstention-r3b-v1` candidate を評価する。この文書と holdout-v4 corpus は、R4 の provider call 前に凍結された。

## 凍結した候補

主要な R4 candidate は、`cross-model-selective-abstention-r3b-v1` に実装された provider-neutral な cross-model unanimity mechanism である。この独立研究では source を正確に2つ使う。

- `google:gemini-3.5-flash-lite`
- `mistral:ministral-8b-latest`

両 source には同一の R2 `decision_note_object` semantic/materialization contract、同一 fixture、token budget、matched seed を与える。model identity が影響してよいのは adapter mechanics のみである。source disagreement は risk signal であり、combined soft decision を `abstain` に escalate することだけができる。agreement は truth、evidence、verification authority、hard finding、epistemic promotion、verdict authority を作らない。majority voting は禁止する。

R4 measurement は matched seed (`5000`〜`5004`) 5個と 512-token output budget を用いる。`disagreement_only` が primary candidate policy である。`complete_unanimity` は operational sensitivity analysis として報告するだけで、観測後に置き換えない。

## 凍結した採用ゲート

canonical gate は、Issue #59 の observation 前に宣言した2つの基準の strict conjunction である。provider run 前に整合させ、観測後に弱めてはならない。

candidate が R4 に pass するには、contract、source set、threshold、corpus、label を provider observation 後に変更せず、次をすべて満たすこと。

- 両 source が 140 call（28 fixture x 5 trial）をすべて完了し、protocol completion 100%、combined trial 5つすべてが完了する。
- aggregate combined precision >= 0.95、aggregate combined recall >= 0.95。
- 各 trial の precision >= 0.90、recall >= 0.90。
- aggregate ambiguous abstention >= 0.85、各 trial >= 0.80。
- aggregate overall decision coverage >= 0.50、各 trial >= 0.45。
- aggregate clear-case coverage（positive+negative fixture）>= 0.90、各 trial >= 0.85。
- positive/negative fixture が成功した source/seed probe 間で `finding` と `no_finding` の両方を assertive polarity として出さず、combined trial decision もその polarity 間で oscillate しない。
- disagreement は unanimous soft decision を保つか `abstain` に escalate するだけである。agreement や vote count は truth、trusted evidence、verification receipt、hard finding、epistemic promotion、verdict authority を作れない。
- 全 source が同じ semantic decision guidance、R2 ownership contract、canonical representation を受け、provider/model 固有 semantic prompt branch を持たない。
- deterministic hard-verifier、resolution、validation、authority regression がすべて green である。

`disagreement_only` は凍結 primary policy であり、`complete_unanimity` は sensitivity analysis にすぎない。provider の外部 unavailable または quota exhaustion は R4 を operationally incomplete にするだけで、semantic pass/failure score に変換しない。gate failure は candidate を reject し、holdout-v4 に対して tuning しない。

この gate の pass が検証するのは、凍結した2-source set に対する R3b の independently supported **optional configuration** としての性質だけである。single-model `soft-semantic-v3` default を自動的に置き換えず、任意の N-source equivalence も示さない。

## 独立ホールドアウト v4 の凍結

`fixtures/semantic-judges-holdout-v4/` は、最初の R4 provider call 前に作成した28件の observation-free case を含む。diagnostic kind ごとに7件、kind ごとに positive 2、negative 2、intentionally ambiguous 3（合計 positive 8、negative 8、ambiguous 12）である。

Fixture ID、request ID、exact request payload は calibration および historical holdout と重複してはならない。`recorded_observations` は空のままにする。holdout-v1/v2/v3 は historical diagnostic evidence であり、この successor の tuning data ではない。

## 凍結した R4 の結果: 棄却

Run `33371523453` は frozen main `55dbda5e71e83bdec95bf4495f65354ca301ef34` を評価した。canonical gate は Issue #59 に `08:08:47Z` に記録され、run は `08:08:54Z` に作成された。PR #71 は artifact inspection 前に同じ gate を repository に同期した。

両 source は operational failure なしに 140/140 call を完了した。primary `disagreement_only` policy では次の通り。

- precision: `1.000` — pass
- recall: `1.000` — pass
- fixture-collapsed ambiguous abstention: `0.8333` — `>=0.85` に対して **fail**
- decision coverage: `0.6071` — `>=0.50` に対して pass
- clear-case coverage: `0.9375` — `>=0.90` に対して pass

trial ごとの ambiguous abstention は `0.5833`、`0.7500`、`0.8333`、`0.7500`、`0.6667`。5 trial 中4つが凍結 threshold `>=0.80` に fail した。mean は `0.7167` だが、これは frozen fixture-collapsed aggregate metric ではない。trial ごとの precision/recall、overall coverage、clear-case-coverage threshold は pass した。

independent labelled-polarity gate も fail した。`v4h-03-contradiction-negative` では Gemini が5 seedすべてで `no_finding`、Ministral が5 seedすべてで `finding` を返した。cross-model disagreement により combined output は安全に `abstain` となるが、source/seed assertive-polarity stability 要件には違反する。

R3b には structural limitation も残る。両 source が同じ assertive decision をすると disagreement は risk を露出できない。特に ambiguity-labelled case の個別 trial は assertive であり、`v4h-13` と `v4h-20` は全 seed で両 source が `finding` だった。

### 観測後のホールドアウト仕様監査

凍結済み semantic decision guidance に対する static audit で、holdout-v4 に2つの label/spec conflict が見つかった。

- `v4h-13` は backup frequency が supplied でないと明記する。凍結 `unsupported_premise` rule では support の affirmative absence は `finding` であり、`abstain` は partial、unbound、uncertain support に限る。
- `v4h-20` は simultaneous garbage-collector change が isolated されていないと明記する。凍結 `causal_gap` rule では explicit confounding / directional isolation の欠如は `finding` condition である。

これらは provider observation 後にのみ発見された。従って corpus と label は変更せず、holdout-v4 を post-hoc repair、relabelling、corrected independent test として rerun してはならない。spec conflict により v4 は不完全な diagnostic evidence だが、predeclared per-trial uncertainty gate と labelled-polarity gate が独立に fail しているため candidate を救済しない。

**Decision:** R3b は independently validated successor として採用しない。runtime `soft-semantic-v3` は変更しない。holdout-v4 と run `33371523453` は frozen diagnostic history であり tuning data ではない。将来の successor には fresh calibration-only research、pre-observation fixture-label/spec review gate、新たに凍結した holdout-v5 が必要である。
