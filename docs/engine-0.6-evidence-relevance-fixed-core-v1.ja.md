# Engine 0.6 evidence relevance fixed calibration core v1

状態: v13 canonical の FAIL が確定した後、v14 live observation 前に選定した successor calibration core。holdout ではない。

これまで calibration は新しい仮説ごとに fresh case を追加し、65 -> 73 -> 81 件へ増えた。回帰履歴は残せた一方、過去の failure に応じて corpus が膨張する構造になっていた。次 successor からは v13 の81件を semantic coverage で48件へ圧縮し、iteration ごとの追加をやめる。

方針:
- case 数は48件で固定;
- in-place で case を追加しない;
- model が miss したことを理由に case を差し替えない;
- 本当に新しい semantic dimension が見つかった場合のみ、別記録を残したうえで明示的な新 core version を作る;
- independent holdout は canonical calibration PASS 後に別途 fresh authoring する。

Coverage:
- Relevant: 14
- Irrelevant: 18
- Ambiguous: 16
- identity-mapping cue present: 6
- ownership-scope cue present: 5
- context-gap cue present: 10
- explicit local absence present: 4

positive exact / alias / paraphrase / distributed / cross-lingual / metadata / freshness / injection、negative wrong-feature / sibling / navigation / broad / unrelated / comparison / relation / injection / explicit absence、ambiguous rename / partial / mixed / conflict / insufficient / URL-only / shared ownership / context gap の代表ケースを維持する。

次 successor の execution policy:
- canonical provider arm は fixed core 全ケースを最後まで attempt する;
- provider failure は operational completeness FAIL のままだが、診断のため残りcaseを継続する;
- per-case absolute deadline は維持する;
- failed canonical の rerun / rescore はしない。
