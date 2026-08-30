# Prior art and design influences

This project intentionally studies existing LLM harnesses and structured-generation systems without taking them as correctness authorities or runtime dependencies.

## Patterns worth borrowing

### Constrained structured generation

Projects such as Outlines show that structural invalidity can sometimes be prevented during generation rather than repaired afterward. The harness should support provider capabilities such as JSON Schema output when available, but schema conformance remains only a structural property. It never upgrades a claim to `known` or `supported`.

### Validator / retry separation

Guardrails-style validation and re-ask loops are useful operational patterns. This project keeps the separation explicit:

1. generation produces a candidate;
2. deterministic validators produce diagnostics;
3. retry policy may request another candidate within a budget;
4. acceptance policy alone decides `accept | reject | unknown`.

A retry cannot suppress a hard diagnostic, and a provider cannot mark its own output verified.

### Evaluation before optimization

DSPy and established evaluation harnesses reinforce a useful research discipline: define metrics and a baseline before optimizing prompts, passes, or model choice. This repository therefore treats fixtures, deterministic metrics, and regression thresholds as first-class artifacts.

### Record / replay for CI

Agent evaluation harnesses commonly separate live model-quality experiments from deterministic contract regression. This project follows that principle: live provider calls are research runs, while committed fixtures and recorded normalized candidates are the CI gate.

### Edge-local process and causal inspection

Process-supervision and causal-evaluation work informs diagnostic granularity without becoming correctness authority. PRM800K and ProcessBench motivate localizing failures to a reasoning step/edge; CLadder motivates typed cause/effect bindings and explicit causal direction; FActScore motivates atomic evidence inspection; UK AISI Inspect and lm-evaluation-harness reinforce solver/scorer and deterministic/live separation. NoisyCausal is used only as a corpus-design influence for mismatch, partial-observability, and distractor cases.

The project adopts those structural patterns only. Learned process rewards, LLM graders, extracted causal graphs, retrieval entailment, or semantic similarity remain soft unless an independent trusted oracle establishes the relevant proposition or relation.

### Robustness, repeated trials, and evaluation integrity

lm-evaluation-harness robustness tasks reinforce measuring consistency under related variants rather than treating point accuracy as sufficient. UK AISI Inspect's epochs, confidence-interval metrics, multi-judge reliability, and transcript scanners reinforce keeping repeated observations, statistical uncertainty, and evaluation-integrity findings separate from task success. METR's evolving time-horizon suites reinforce that benchmark composition, difficulty distribution, saturation, and version changes can materially change the quantity being measured.

Reasoning Harness borrows these measurement disciplines without copying their orchestration surfaces: metamorphic invariance (#10), diagnostic stability (#11), soft-judge calibration (#13), and corpus versioning/stratification (#14) remain subordinate to the existing harness-owned authority boundary.


### Durable control-plane patterns

Mature agent runtimes reinforce separating permission/policy from execution authority and separating ephemeral model turns from durable runtime state. Codex App Server exposes durable create/resume/fork/archive thread lifecycle plus an event stream, while Codex operational guidance separates sandbox limits from approval policy. LangGraph uses thread-scoped checkpoints for interrupt/resume and non-destructive forks. Claude Code exposes resumable/forkable sessions, lifecycle hooks, permissions, and optional subagents.

ADR-0003 borrows only the provider-neutral control-plane ideas: composable reasoning policy, explicit invalidation, durable typed thread events, and checkpoint/resume/fork. Skills/subagents and generic agent orchestration remain deferred.

## Deliberate differences

- No framework is trusted merely because it emits schema-valid structured output.
- No LLM judge is a hard correctness oracle.
- Prompt optimization is not the primary correctness mechanism.
- Provider retry/re-ask behavior cannot bypass harness-owned verification passes.
- CI must remain useful without credentials or live model access.
- First-party components remain Rust-only.

## References

- Outlines: https://github.com/dottxt-ai/outlines
- Guardrails AI: https://github.com/guardrails-ai/guardrails
- DSPy: https://github.com/stanfordnlp/dspy
- lm-evaluation-harness: https://github.com/EleutherAI/lm-evaluation-harness
- harness-evals: https://github.com/harness/harness-evals
- agent-harness: https://github.com/nderman/agent-harness
- PRM800K: https://github.com/openai/prm800k
- ProcessBench: https://github.com/QwenLM/ProcessBench
- CLadder: https://arxiv.org/abs/2312.04350
- FActScore: https://github.com/shmsw25/FActScore
- UK AISI Inspect scorers: https://inspect.aisi.org.uk/scorers.html
- UK AISI Inspect scanners: https://inspect.aisi.org.uk/scanners.html
- UK AISI Inspect metrics: https://inspect.aisi.org.uk/metrics.html
- lm-evaluation-harness robustness/SCORE tasks: https://github.com/EleutherAI/lm-evaluation-harness/tree/main/lm_eval/tasks/score
- lm-evaluation-harness decontamination: https://github.com/EleutherAI/lm-evaluation-harness/blob/main/docs/decontamination.md
- METR Time Horizon 1.1: https://metr.org/blog/2026-1-29-time-horizon-1-1/
- NoisyCausal: https://arxiv.org/abs/2605.04313
- OpenAI Codex App Server: https://openai.com/index/unlocking-the-codex-harness/
- OpenAI Codex safety controls: https://openai.com/index/running-codex-safely/
- LangGraph persistence: https://docs.langchain.com/oss/python/langgraph/persistence
- LangGraph time travel/fork: https://docs.langchain.com/oss/python/langgraph/use-time-travel
- Claude Code Agent SDK: https://docs.anthropic.com/en/docs/claude-code/sdk
