# Engine 0.6 evidence relevance holdout v11

Status: runner-only preparation。successor-v10 semantics は engine-0.6-evidence-relevance-successor-v10-semantics-freeze / 2d75c2d8f1710c0553c8b2993cad201566be7d5c で freeze 済み。holdout-v11 corpus はまだ存在せず、provider observation も禁止する。

## Runner binding

Dedicated binary: reason-evidence-relevance-holdout-v11-study

V11 profile:
- configuration: evidence-relevance-live-holdout-v11
- suite: evidence-relevance-holdout-v11
- annotation protocol: evidence-relevance-effective-qualification-v15-materialization-v28
- fixed core: evidence-relevance-fixed-core-v11
- expected directory: fixtures/evidence-relevance-holdout-v11
- expected cases: 26
- issue: #468
- effective qualification: v15
- materialization: v28

checkpoint/replay compatibility のため historical v1-v10 と reusable development profile は残すが、既存 profile の semantic binding は変更しない。

## Ordering constraint

Runner freeze coordinate: engine-0.6-evidence-relevance-holdout-v11-runner-freeze。

この annotated tag を push した後にだけ fixtures/evidence-relevance-holdout-v11 を作成できる。fresh corpus は runner freeze 後に独立 author し、observed holdout-v1-v10 / successor development の case ID / entity / task / exact signal / 8-token window を再利用しない。development-v4 は development evidence のみで acceptance evidence には使わない。

後続 one-shot acceptance surface では Mistral / Google / Groq の3 provider を復帰させる。
