# セマンティック判定器のハーネス所有マテリアライゼーション研究

Issue #59 R2 は、モデルに追加の権限を与えずに、モデル所有のプロトコル面を縮小できるかを評価する。これはキャリブレーション専用の研究である。実行時の `soft-semantic-v3` は変更せず、過去の holdout v1/v2/v3 はチューニングデータとして扱わず、holdout-v4 は引き続きブロックする。

## R2 契約

R2 のモデル向け出力は次のものだけを含む。

- `decision`: `finding | no_finding | abstain`
- 任意の `advisory_note`

スキーマには、モデルが所有する `finding`、`kind`、`target` フィールドはない。v3 の種別固有の判定ガイダンスは変更せず再利用する。R2 では出力所有権に関する指示だけを意図的に変更し、ハーネスが finding の識別とバインディングを所有するとモデルに伝える。

パースされた decision が `finding` の場合、ハーネスはリクエストに既存の `kind` と `target` をそのままコピーして soft finding を構築する。任意の advisory note は soft finding の note フィールドにコピーできる。decision が `no_finding` または `abstain` の場合、advisory note が存在していても finding はマテリアライズしない。

生成された finding は引き続き soft かつ advisory である。ハーネス所有のマテリアライゼーションは、信頼済み証拠、hard finding、検証レシート、認識論的な昇格、判定権限を生み出さない。

## 正規化境界

R2 で許可されるのは、既存の構造化出力ポリシーと同等の構文だけの正規化である。次のことをしてはならない。

- 異なる意味の判定を推測する
- `kind` や `target` を発明または変更する
- `no_finding` や `abstain` を finding に変える
- 権限を示すような不正なフィールドを解釈する
- 複数の JSON 値を1つの意味的な回答に修復する

未知のフィールドは `deny_unknown_fields` により fail closed となる。研究用の成果物に記録するのは advisory note が存在したかどうかだけであり、研究スコアリング用に自由記述の advisory-note 本文は保存しない。

## 対応付けたベースライン比較

R2 の研究では、1つの provider/model 内で次の2つのアームを比較する。

1. 正確な v3 full-JSON の主要表現
2. ハーネスがマテリアライズする decision プロトコル

ケースは `(fixture_id, trial, seed)` で対応付ける。実行順は fixture/trial ごとに交互にし、一方のアームが常に先に実行されないようにする。運用上の失敗は decision-flip の分母から除外する。

研究では、プロトコル完了率、precision、recall、decision coverage、曖昧なケースでの abstention、トークン使用量、レイテンシ、advisory note の有無、対応付けた decision の遷移、および `decision_flip_rate` を報告する。

不一致は不安定性の証拠にすぎない。ベースラインは多数決によって真実にはならず、繰り返し出力によって権限が生まれることもない。

## キャリブレーション専用の実行

研究バイナリは target を canonicalize し、この checkout の正確な `fixtures/semantic-judges` ディレクトリだけを受け付ける。holdout ディレクトリ、名前を変更したコピー、holdout への symlink は、provider credentials を使用する前に拒否される。

```text
cargo run -p reasoning-harness-cli --bin reason-materialization-study -- \
  fixtures/semantic-judges \
  --provider google \
  --model gemini-3.5-flash-lite \
  --fixture 07_causal_positive \
  --fixture 08_causal_negative \
  --fixture 09_causal_ambiguous \
  --seed 2000 \
  --trials 1
```

`semantic-materialization-study` GitHub Actions workflow は、デフォルトで causal positive/negative/ambiguous の三つ組を実行する。そのため最初の live validation は6回の provider call、すなわち v3 baseline の3回と materialized-arm の3回となる。完全なキャリブレーションと反復試行は、後段で明示的に選択する。
