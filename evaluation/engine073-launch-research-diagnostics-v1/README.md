# Research-informed launch archive diagnostics v1

This is a read-only evaluation-development tool, not a runtime or an acceptance scorer. It applies contrast-pair inspection from the [research-informed repair plan](../../docs/engine-073-research-informed-repair.md) to the already consumed [v19 first observation](../engine073-launch-independent-v19/README.md).

`analyze.py` verifies the pinned v19 final manifest and exact inventory/hashes of all444 independent artifacts before reading preserved grades. It neither imports/runs the scorer nor executes an Engine/provider. It prints JSON and does not write to the archived evidence. Its fixed anchor is intentionally specific to v19; a different corpus requires a separately reviewed diagnostic identity.

```sh
PYTHONDONTWRITEBYTECODE=1 python3 evaluation/engine073-launch-research-diagnostics-v1/analyze.py
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s evaluation/engine073-launch-research-diagnostics-v1 -p test_analyze.py -v
```

Saved result: original FAIL49/60, positive19/30 retained, negative30/30 withheld. Of30 contrast pairs,19 pass both sides and11 fail only the positive. The failed positives share the coarse `local_qualification_disagreement` reason; the archive has no internal rule path or source-span/reference-edge trace. A root cause cannot be inferred from that aggregate code.

Five checks validate the report and refusal of changed first output, extra inventory, changed predecessor proof and a replaced manifest anchor. Tamper checks operate only on temporary copies. `report.json` and `tests.log` are analysis/check records, not new first semantic observations or replacement scores.

The first inputs, labels, scorer, results and FAIL remain unchanged. Analysis invokes Engine0/scorer0/provider0. Candidate source remains f3d9ad15ae2e4fc81dadaed87c4e55fa8bce1c7d; core code is unchanged by this work. #520/Draft PR539 stay open; #531 real24=0/24, adoption1/6 and Groq stopped. A semantic repair still requires a new source freeze and fresh acceptance.
