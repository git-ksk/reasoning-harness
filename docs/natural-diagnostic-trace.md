# Natural diagnostic trace

`reason` can emit an opt-in diagnostic trace for postmortem analysis of the natural-language generation path without changing the normal product output contract:

```sh
reason "TASK" \
  --provider mistral \
  --model ministral-8b-latest \
  --format json \
  --diagnostic-trace /tmp/reason-natural-trace.json
```

The product stdout/result remains `reason-natural-output-v4`. The diagnostic file uses the separate `reason-natural-diagnostic-trace-v1` contract and is intended to be uploaded alongside a future frozen acceptance artifact. Existing frozen acceptance workflows and historical artifacts are not modified by this feature.

The trace records ordered typed events for model request/response correlation, initial generation, investigation planning/action selection, evidence admission, post-investigation regeneration, grounding, final rendering, answer-safety observations, and finalization transitions. Provider/model, seed, round/attempt, token usage, finish reason, response byte count, admitted evidence IDs, and fact keys are retained where available.

## Security boundary

The trace is limited to explicit product I/O and typed product state. It never intentionally records API keys, authorization credentials, password/private-key fields, or hidden reasoning. Secret-like structured fields are redacted recursively. Common hidden-reasoning fields (`analysis`, `reasoning`, `thinking`, chain-of-thought, scratchpad, and equivalents) are omitted. Provider response text is retained only when it parses as structured JSON; unstructured response prose is not persisted, and trailing non-JSON text is omitted.

Diagnostic recording is implemented as a transparent `ModelAdapter` wrapper: it observes the exact `ModelRequest` before forwarding that same request value to the provider adapter. Tests assert serialized request identity and response identity with diagnostics enabled. Diagnostic state is not part of `NaturalOutput`, evaluator input, selection state, evidence admission, grounding, or finalization authority.

## Acceptance usage

A future successor evaluator may pass a unique path per live invocation, for example:

```sh
--diagnostic-trace "/tmp/acceptance-traces/${CASE_ID}.json"
```

and include that directory in its existing artifact upload. This is artifact plumbing only: evaluator/scoring/metric semantics and the frozen measurement ruler must remain unchanged. Do not rerun or rescore a frozen acceptance solely to obtain a trace.
