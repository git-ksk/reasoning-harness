# Metamorphic reasoning の堅牢性

一点の正解率だけでは、意味が変わらないはずの表現上の細部に reasoning harness が左右されるかどうかは分からない。metamorphic layer は commit 済み fixture に意味保存変換を決定的に適用し、変換後の trusted outcomes を比較する。

## 権限の境界

metamorphic transform は test operation であり、新しい verifier ではない。trusted evidence を作成したり、oracle conclusion を変更したり、model-authored statement を昇格させたりすることはできない。必須 CI では commit 済み fixture に対する決定的な transform だけを使い、provider credential も LLM judge も必要としない。

## 初期変換ファミリー

- `evidence_order`: 内容を変えずに harness-owned evidence の順序を入れ替える。
- `inference_order`: 独立した candidate inference edges の順序を入れ替える。
- `stable_id_remap`: すべての参照を維持したまま、evidence、claim、inference、互換性のある receipt ID を一貫して変更する。
- `irrelevant_evidence`: 無関係な proposition key の下に structured fact を追加する。
- `causal_cause_order`: candidate relation と harness-owned causal evidence の両方で、複数原因集合の順序を入れ替える。
- `causal_evidence_order`: conflicting support/refutation records を含む causal evidence records の順序を入れ替える。

自由形式の paraphrase 生成は意図的に除外する。自然言語の同値性には、別途 calibration された soft semantic layer が必要になる。

## 意味論的フィールドと非意味論的フィールド

初期の deterministic layer では、proposition `key`/`value`、evidence facts、verification conclusions、causal relation の membership/direction、inference connectivity、finding kind/reason/strength、final verdict を semantic とする。これらを変更する transform は、将来の transform が独立に検証済みの等価規則を定義しない限り有効ではない。

contract が set または独立 records として記述する箇所では collection order は non-semantic である。stable evidence/claim/inference/receipt identifiers は semantic ではなく参照用であり、変更できるのはすべての内部参照を一貫して remap する場合だけである。追加された evidence record は、その proposition key がテスト対象のすべての proposition と明示的に無関係な場合に限り non-semantic となる。

human-readable task、observation、source、claim prose は現在 non-semantic と宣言されていない。そのため deterministic transforms では書き換えない。

## 報告

各 base-case/transform pair について、final-verdict invariance、hard-finding invariance、soft-finding stability、diagnostic-status invariance、raw diagnostic-ID changes、diagnostic reason changes を報告する。stable-ID remapping では raw ID が変わることは正当であり、semantic finding signatures では意図的にこれらの ID を除外する。

`hard_outcome_invariance_rate` は final verdict、hard findings、typed diagnostic statuses を組み合わせた指標である。通常の benchmark accuracy とは分けて報告し、transform 後の cases が元の benchmark denominator を置き換えることはない。
