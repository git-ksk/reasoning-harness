# ADR 0004: Bounded agentic evidence search and evaluation isolation

Status: experimental candidate

## Context

The external-evidence experiment adds a planner-driven search loop on top of typed MCP acquisition. The model may reformulate a query after observing search results, but the Harness remains the authority for evidence admission, sufficiency, truth, and termination.

This creates two distinct engineering risks:

1. an agent loop can continue indefinitely or waste budget without making epistemic progress;
2. live public APIs can fail independently of planner or semantic quality, contaminating evaluation results.

The current experiment already records rounds, tool calls, planner calls, model tokens, latency, stop reasons, operational failures, and multiple trials. It also separates dev and holdout suites.

## Prior art considered

- OpenAI Codex uses a harness-owned agent loop: model inference may request a tool, the harness executes it, appends the observation, and re-runs inference until termination. Long loops make context and orchestration a harness responsibility.
  - https://openai.com/index/unrolling-the-codex-agent-loop/
- OpenAI Harness Engineering emphasizes enforceable invariants, legible feedback loops, and evaluation infrastructure rather than asking an agent to simply “try harder.”
  - https://openai.com/index/harness-engineering/
- LangGraph requires a termination condition for loops and separately exposes a recursion/super-step limit, including proactive remaining-step handling for graceful degradation.
  - https://docs.langchain.com/oss/python/langgraph/use-graph-api
  - https://docs.langchain.com/oss/python/langgraph/graph-api
- Microsoft AutoGen models termination as stateful runtime conditions that can be composed, including message, token, timeout, external, and functional termination conditions.
  - https://microsoft.github.io/autogen/stable/user-guide/agentchat-user-guide/tutorial/termination.html
- Anthropic recommends multi-trial agent evaluations that preserve trajectories, outcomes, tool-call counts, tokens, and latency, while preferring deterministic graders when possible.
  - https://www.anthropic.com/engineering/demystifying-evals-for-ai-agents
- Anthropic also reports that infrastructure configuration/noise alone can move agentic benchmark scores by several percentage points, so infrastructure effects must not be silently attributed to model capability.
  - https://www.anthropic.com/engineering/infrastructure-noise

## Decision

### 1. Planner proposes actions; Harness controls the loop

The planner is untrusted and may only propose a typed next search action or stop proposal. It does not decide whether evidence is true, sufficient, admissible, or final.

The Harness/controller owns deterministic limits and stop semantics. The experimental defaults remain:

- max rounds: 6
- max external tool calls: 10
- wall-clock budget: 30 seconds
- model-token budget: 8,000
- no-progress cutoff: 2 rounds
- normalized duplicate-query rejection
- immediate stop when the target is supported or refuted
- budget exhaustion or unresolved ambiguity produces a safe unknown rather than a guessed answer

A model-requested stop is only a search-planning signal; final epistemic status remains Harness-owned.

### 2. Progress is measured from typed state, not model narration

No-progress detection is based on changes in typed search state such as candidate identifiers, resolved entities, corroborated titles, and property values. The controller does not accept “I made progress” text from the planner as evidence of progress.

### 3. Evaluation has separate signals

Do not collapse these into one score:

- **semantic safety**: especially false acceptance / unsupported acceptance;
- **agent capability**: whether the bounded planner reaches the expected outcome without planner/budget failure;
- **infrastructure health**: transport/protocol/provider failures from live dependencies;
- **efficiency**: rounds, tool calls, planner calls, tokens, latency.

Operational failure can make a live trial fail, but it must not be reclassified as a semantic regression.

### 4. CI isolates live dependency classes

The MCP knowledge probe runs these on separate fresh GitHub Actions runners:

- deterministic control contracts;
- live Layer A/B adapter and fixed-policy checks;
- live Layer C agentic planner dev suite;
- natural-language MCP smoke;
- frozen holdout, only by explicit workflow dispatch.

Each live lane uploads its own report and has explicit gates. A combined attribution report is produced only after the per-lane reports exist.

### 5. Deterministic controls precede live interpretation

Transport retry/accounting and progress-control behavior must have deterministic fixture-based tests. Live Wikipedia/Wikidata behavior remains valuable dogfood but is not the only proof of loop correctness.

### 6. Holdout remains frozen after first observation

The holdout is dispatch-only. Once observed, its cases, budgets, planner prompt, stop rules, and expected outcomes must not be retuned in response to its results. Follow-up tuning requires a new independent holdout.

## Consequences

Benefits:

- infinite loops are bounded by runtime policy rather than model obedience;
- external outages are visible without being mislabeled as semantic failures;
- live failures can be rerun in an isolated lane instead of re-running unrelated high-load API traffic;
- metrics align with established agent-evaluation practice;
- the design stays source-independent and can extend beyond Wikipedia/Wikidata MCP adapters.

Costs:

- CI has more jobs and repeated checkout/build setup;
- aggregate reporting is slightly more complex;
- a full clean live run can still fail when public dependencies are unhealthy, but the failure provenance is explicit.

## Non-decisions

- This ADR does not create a new semantic-runtime generation such as “D4.”
- This ADR does not promote arbitrary MCP text into trusted facts.
- This ADR does not make live public APIs the authoritative frozen benchmark environment.
- The separate acquisition-time versus semantic/as-of-time bug remains a distinct product issue; the experimental future evaluation-time workaround is not the intended fix.
