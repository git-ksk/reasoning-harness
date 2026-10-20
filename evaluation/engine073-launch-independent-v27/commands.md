# v27 preparation commands

Working directory: `/Users/sawadakousuke/.codex/worktrees/engine073-source27-freeze/reasoning-harness/evaluation/engine073-launch-independent-v27`.

The original one-time pure evidence writer has completed; its exclusive outputs refuse overwrite:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -B selftest.py
```

Parent can reproduce pure59 without candidate calls or rewriting saved artifacts:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest test_evaluator
```

Read-only source/runtime/parent30/old829/M0 integrity verifier:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -B verify_preparation.py
```

Do not run future static/freeze/execute-first/recorder/scorer main or create authoring-start/author/cases/labels/inputs during preparation. They require parent review and a new explicit notice. Policy/helper/budget/first-once behaviors are inherited unchanged.
