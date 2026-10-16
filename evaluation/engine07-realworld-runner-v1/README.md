# Durable practical-evaluation runner (development)

This is a new first-observation subprocess scheduler for #540/#531. It executes every planned normal baseline/candidate and external helper job through one provider/model guard. It does not modify or rerun the frozen v2/v3 evaluators, score semantic answers, publish an Engine or adopt one in the CLI. No provider/model calls have been made with this runner.

A plan pins executable/static-input hashes, one provider/model, every row, the arm self-budget and full-callback generation/input/output/attempt reserves. Every command must select that exact provider/model. The scheduler records exclusive raw output, typed outcomes, incomplete/suppressed rows and an exclusive summary. Budget reservations cover all declared generations and internal attempts; they are conservative estimates, not proof of an exact billing cap. Before a live freeze, derive those numeric bounds from the pinned host/adapters, including accumulated context, fallback generations and retries. CLI post-response token limits alone cannot justify them.

Before credential lookup or process spawn, the runner fsyncs reserve/inflight and a journal event, then atomically replaces/fsyncs state. Known usage is a separate measured subtotal. Reservations are never refunded; unknown usage is unavailable, never a billed zero. Partial diagnostics are retained separately from complete totals. A killed/crashed/interrupted or already completed output path always refuses restart rather than creating a fresh guard. A new output/experiment requires a new explicitly reviewed freeze and must carry forward known provider prohibitions; this module does not establish account-wide accounting across unrelated processes or experiments.

The existing Groq prohibition is read before authentication, and blocks every Groq model/account here. Time passing, a tiny readiness success or changing a model cannot clear it. Recovery requires a separate verified policy/budget change. The recorded provider state is not edited by this runner.

Deadlines and parent SIGINT/SIGTERM request child-group SIGINT, wait the frozen grace, then use SIGKILL only if needed. Both remain incomplete and stop subsequent work. A parent SIGKILL cannot execute cleanup; durable inflight survives and restart refuses to spawn. The external controller must contain/inspect any still-running child group before any new experiment. No automatic retry, credential/model/tier escalation or semantic rescue occurs.

Run the offline real-subprocess tests:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s evaluation/engine07-realworld-runner-v1 -p 'test_*.py' -v
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s evaluation/engine07-realworld-operations-v1 -p 'test_*.py' -v
```

`fixture_child.py` is an offline test process: it never imports a provider or opens a socket. Tests exercise shared quota, capacity failure, known Groq spawn0, unknown usage, reserve exhaustion/overshoot, changed artifacts, credential exceptions, graceful cancellation, bounded hard kill, killed-parent durable inflight and restart refusal.

A prepared/frozen operational plan can run with:

```sh
python3 evaluation/engine07-realworld-runner-v1/runner.py --plan /absolute/frozen-plan.json --output /absolute/new-first-output
```

Outputs are private local evidence. Do not put credentials in command arguments/plans or publish private traces. This scheduler is operational preparation, not a new canonical real-source acceptance identity. The actual paired CLI host/build plan, normal Engine path trace, Mistral rw03 fixed-response diagnosis, numeric live budget model, fresh independent24-task corpus, provider completeness, source release and CLI distribution gates remain open. #540 cannot be closed solely by these offline tests.

## Independent review repair

Original43 controls passed, but independent review of `ed2c8de` reproduced three P1 failures. Preserve that first FAIL and50-file archive in `../engine073-safe-runner-independent-review-v1`. The repair detaches the entire caller plan into immutable serialized storage; exposed plan reads return new copies. Frozen artifacts are checked again after credential callbacks before execution. Cancellation waits for the entire isolated process group, including descendants after leader exit, before bounded hard kill; a normal leader exit with a live descendant is incomplete and also contained. A development RED caught redundant cleanup of an already cancelled group on macOS and is retained; final repair avoids that second cleanup.

The21 current subprocess tests plus26 inherited controls pass. `repair-verification-v2.json` pins code and logs; original17/26 verification records are unchanged. New controlled CLI/runner evidence is saved separately in `../engine07-cli-path-preparation-v2`. Independent re-review remains pending. These POSIX controls assume an operator-controlled cooperative artifact freeze, not atomic check/exec security against a hostile filesystem writer. Abrupt parent SIGKILL still requires external containment/inspection before a new experiment. Full numeric live budgets and callback/dependency closure, real providers and real-source acceptance remain open; Groq is not restarted.
