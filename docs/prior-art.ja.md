# Prior art and design influences（先行研究と設計上の影響）

このプロジェクトは、既存の LLM harness と structured-generation system を研究対象とするが、それらを correctness authority や runtime dependency として扱うことは意図的に避ける。

## Patterns worth borrowing（取り入れる価値のあるパターン）

### Constrained structured generation（制約付き構造化生成）

Outlines などのプロジェクトは、構造的な不正を事後に修復するのではなく、生成中に防げる場合があることを示す。ハーネスは、利用可能な場合に JSON Schema output など provider の capability をサポートするべきだが、schema conformance は構造上の性質にとどまる。それによって claim が `known` や `supported` に昇格することはない。

### Validator / retry separation（validator と retry の分離）

Guardrails-style validation と re-ask loop は有用な運用パターンである。このプロジェクトでは分離を明示する。

1. generation が candidate を生成する
2. deterministic validator が diagnostic を生成する
3. retry policy が予算内で別の candidate を要求することがある
4. acceptance policy だけが `accept | reject | unknown` を決定する

retry は hard diagnostic を抑制できず、provider が自分の出力を verified と示すこともできない。

### Evaluation before optimization（最適化に先立つ評価）

DSPy と確立された evaluation harness は、prompt、pass、model choice を最適化する前に metric と baseline を定義する有用な研究規律を示す。そのためこの repository では、fixture、deterministic metric、regression threshold を first-class artifact として扱う。

### Record / replay for CI（CI のための record / replay）

agent evaluation harness では、live model-quality experiment と deterministic contract regression を分離するのが一般的である。このプロジェクトもその原則に従う。live provider call は research run であり、commit された fixture と recorded normalized candidate は CI gate である。

### Edge-local process and causal inspection（エッジ局所のプロセスと因果検査）

process-supervision と causal-evaluation の研究は、correctness authority になることなく diagnostic granularity に影響を与える。PRM800K と ProcessBench は failure を reasoning step/edge に局所化する動機となり、CLadder は typed cause/effect binding と explicit causal direction を促し、FActScore は atomic evidence inspection を促す。UK AISI Inspect と lm-evaluation-harness は solver/scorer と deterministic/live の分離を補強する。NoisyCausal は mismatch、partial-observability、distractor case の corpus-design influence としてのみ使う。

このプロジェクトが採用するのは、それらの structural pattern だけである。learned process reward、LLM grader、extracted causal graph、retrieval entailment、semantic similarity は、独立した trusted oracle が該当する proposition または relation を確立しない限り soft のままである。

### Robustness, repeated trials, and evaluation integrity（頑健性、反復試行、評価の完全性）

lm-evaluation-harness の robustness task は、point accuracy を十分なものとせず、関連する variant に対する consistency を測定することを補強する。UK AISI Inspect の epoch、confidence-interval metric、multi-judge reliability、transcript scanner は、repeated observation、statistical uncertainty、evaluation-integrity finding を task success と分離して保持することを補強する。METR の evolving time-horizon suite は、benchmark composition、difficulty distribution、saturation、version change が測定量を大きく変え得ることを補強する。

Reasoning Harness は、これらの measurement discipline を借用するが、orchestration surface はコピーしない。metamorphic invariance (#10)、diagnostic stability (#11)、soft-judge calibration (#13)、corpus versioning/stratification (#14) は、既存の harness-owned authority boundary に従属する。

### Durable control-plane patterns（永続的な control-plane パターン）

成熟した agent runtime は、permission/policy と execution authority を分離し、ephemeral な model turn と durable な runtime state を分離することを改めて示している。Codex App Server は durable な create/resume/fork/archive thread lifecycle と event stream を公開し、Codex operational guidance は sandbox limit と approval policy を分離する。LangGraph は interrupt/resume 用の thread-scoped checkpoint と non-destructive fork を使う。Claude Code は resumable/forkable session、lifecycle hook、permission、optional subagent を公開する。

ADR-0003 が借用するのは provider-neutral な control-plane の考え方だけである。すなわち composable reasoning policy、explicit invalidation、durable typed thread event、checkpoint/resume/fork である。skill/subagent と generic agent orchestration は延期する。

## Deliberate differences（意図的な相違点）

- schema-valid な structured output を生成するという理由だけで、どの framework も信頼しない。
- LLM judge を hard correctness oracle としない。
- prompt optimization を主要な correctness mechanism としない。
- provider の retry/re-ask behavior が harness-owned verification pass を迂回できないようにする。
- CI は credential や live model access なしでも有用でなければならない。
- first-party component は Rust-only のままにする。

## References（参考資料）

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
