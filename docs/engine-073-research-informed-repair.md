# Engine 0.7.3 — research-informed repair

[日本語](engine-073-research-informed-repair.ja.md) | English

Reviewed2026-10-17. Applies to [#520](https://github.com/git-ksk/reasoning-harness/issues/520), [#531](https://github.com/git-ksk/reasoning-harness/issues/531) and [the0.7.3 roadmap](engine-0.7.3-roadmap.md). This records public research and engineering guidance, our adaptation, and its implementation status.

## Sources and adaptations

| Source | Relevant finding or technique | Application here |
| --- | --- | --- |
| [Anthropic: Demystifying evals for AI agents](https://www.anthropic.com/engineering/demystifying-evals-for-ai-agents) (2026) | Capability and regression evaluation answer different questions; inspect transcripts and outcomes. | Track consumed regressions, new-source capability and real CLI outcomes separately. Original v19 FAIL remains blocking. |
| [OpenAI: Build an Agent Improvement Loop with Traces, Evals, and Codex](https://developers.openai.com/cookbook/examples/agents_sdk/agent_improvement_loop) (2026) | Recorded traces and feedback inform reusable evaluations and targeted harness changes. | Diagnose the actual rule/scope path before the next repair. Record deterministic decisions and evidence references. |
| [OpenAI: Build iterative repair loops with Codex](https://developers.openai.com/cookbook/examples/codex/build_iterative_repair_loops_with_codex) (2026) | Separate review, focused repair and validation with explicit feedback. | One stated scope/reference hypothesis per repair; use observed failures as development inputs, then freeze before fresh acceptance. |
| [CheckList](https://aclanthology.org/2020.acl-main.442/) (ACL2020) | Minimum-functionality, invariance and directional tests expose linguistic behaviors. | Development matrix: negation, quotation, ownership, references, conjunction and signal boundaries; test meaning-preserving changes and explicit adverse additions. |
| [HANS](https://aclanthology.org/P19-1334/) (ACL2019) | Lexical overlap/subsequence/constituent heuristics can hide entailment failures. | Shared product words and an embedded launch sentence do not establish a factual target event. Test enclosing scope and owner changes. |
| [FActScore](https://aclanthology.org/2023.emnlp-main.741/) (EMNLP2023) | Evaluate supported atomic facts rather than one whole-text verdict. | Future real24 rubric records each answer claim and its evidence. A correct claim does not cancel an unsupported one. |
| [ALCE](https://aclanthology.org/2023.emnlp-main.398/) (EMNLP2023) | Evaluate answer correctness and citation quality in end-to-end retrieval/generation. | Measure final-answer support and citation coverage separately from helper relevance displays. |
| [Claude citations](https://platform.claude.com/docs/en/build-with-claude/citations) | Citations identify source document locations, with explicit index conventions. | Proposed internal evidence references retain source identity, original span and offset unit; pointer validity and semantic support are separate checks. |
| [Anthropic: Building effective agents](https://www.anthropic.com/engineering/building-effective-agents) (2024) | Simple workflows, environmental feedback and bounded iteration can be sufficient. | Keep the current provider-neutral workflow; add complexity only with measured benefit. |

The following representation is **our implementation hypothesis**, not a demonstrated method for solving the v19 cases in those publications. FActScore is an evaluation method; it does not establish a correct natural-language parser.

## Next repair: inspect scopes and references

1. Add a development-only deterministic trace at the actual qualification path. Record signal identity, original text span, context (assertion/report/quoted literal), owner, relation, polarity, resolved reference and blocking rule. This is an audit record of code decisions, not private model reasoning.
2. Represent local claims and their reference edges explicitly. Keep the original text, quotation boundaries and enclosing denial. Every coordinated object needs its own owner/type proof. A fully identified property cannot supply ownership to a launch claim.
3. Use source-relative UTF-8 byte ranges for a Rust prototype, with bounds/character-boundary and exact-text checks. Claude's character indices require an explicit conversion; they are not Rust byte offsets. Valid references do not prove entailment or truth.
4. Preserve unknown owners, clipped context, target/global denials, raw risk and historical v16/v29 behavior. A model proposal remains untrusted input. Evaluate a small internal prototype before expanding public contracts.
5. Cover each changed behavior with development MFT/INV/DIR checks, preserving all first results. After a semantic repair, pin a new source before an independently authored fresh gate. Existing source/real24 acceptance conditions remain unchanged.

## Implemented now

[Archive diagnostics v1](../evaluation/engine073-launch-research-diagnostics-v1/README.md) verifies all444 frozen artifacts, reads preserved grades and groups the30 contrast pairs. It executes no Engine, scorer or provider. The saved result remains FAIL49/60:19 pairs pass both sides,11 fail only the positive side, negative violations0. All11 failed positives share `local_qualification_disagreement`; the archived output lacks a rule path and source-span/reference edges. This establishes a diagnostic gap, not the cause of those11 failures.

Five checks cover the saved report, changed first output, an extra unrecorded file, changed predecessor evidence and a replaced manifest anchor. Those archive diagnostics do not repair semantics. A permanent trace/public generic claim graph remains unimplemented.

[Temporary actual-rule tracing](../evaluation/engine073-launch-trace-development-v1/README.md) subsequently identified the11 blocking branches, then its hooks/module were removed and the exact base restored. [Sourcefd091b1 repair](../evaluation/engine073-launch-development-v19-repair/README.md) now binds property references to complete adjacent propositions/exact roles, keeps coordinated roles until actual predicates and honors explicitly foreign event owners. Consumed960 and related83/workspace1547 checks pass. Initial repair reporting-conjunction regressions and their RED remain saved. Fresh independently authoredv20 first FAIL51/60 (unsafe3, positive misses6). This is a bounded internal scope/reference implementation, not a generic graph or a new public contract.

[Actual-rule diagnosticv2](../evaluation/engine073-launch-trace-development-v2/README.md) and [sourced647ca4](../evaluation/engine073-launch-development-v20-repair/README.md) repair distinct unresolved-event ownership, role-prefixed target nonexistence and complete property/apposition/reference/comma predicates. Balanced printed literals are retained; the successor-only view replaces only proven quoted bodies, preserving outside omissions and every raw risk.1020 seen cases/91 related/workspace1555 pass,597 historical outputs identical. Intermediate RED and a parent build binding mistake detected before independent execution are preserved. Freshv21 firstFAIL48/60 (unsafe0, positive misses12) followed corrected compiler artifacts/new freeze; synthetic source acceptance remains open.

Real24 remains0/24 and CLI adoption1/6. Actual0.6.1/successor comparison, critical unsupported claims, useful-answer rate, citations, contradiction/time handling and operational cost need the complete frozen host/evaluator/model/budget protocol. No population accuracy, improvement rate or completion percentage follows from different synthetic corpora. Groq remains stopped; live callback/dependency/numeric-budget gates remain open. See [the preserved v19 observation](../evaluation/engine073-launch-independent-v19/README.md).
