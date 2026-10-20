# Reproducible pre-author commands

Working directory: `/Users/sawadakousuke/.codex/worktrees/engine073-source26-freeze/reasoning-harness/evaluation/engine073-launch-independent-v26`.

Original one-time evidence writer (already completed; exclusive saved-log barrier refuses overwrite):

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -B selftest.py
```

Parent can independently recheck the pure output fixtures without writing the saved artifacts or running any candidate:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest test_evaluator
```

Read-only M0/candidate/old730/parent36/runtime4 integrity verification, no Engine or scorer main:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -B verify_preparation.py
```

The recorded RED command ran only the three invalid-metadata tests listed in `pre-freeze-history/trace-type-red.json`. No actual cohort scorer main or binary command has run.

Do not run `static_preflight.py`, `freeze_corpus.py`, `execute_first.py`, `first-run.py` or `score.py` main during this pre-author stage. These frozen future-stage tools require separate parent authorization, newly authored data and a complete M1. Do not create author/cases/labels/inputs until instructed.
