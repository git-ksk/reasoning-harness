# Codex and OpenCode design notes for the current repair

Reviewed2026-10-18; applies to consumed v25/#520 development. No product/SDK integration was introduced.

| Primary-source observation | Local adaptation | Status |
| --- | --- | --- |
| [Codex App Server](https://learn.chatgpt.com/docs/app-server) exposes typed items with IDs, lifecycle states and completion events. | Link case/signal/segment and original text spans; separate traces recorded during the API call from later helper diagnostics. | Separate test-only copy; no new public trace API. |
| [Codex iterative repair](https://developers.openai.com/cookbook/examples/codex/build_iterative_repair_loops_with_codex) saves review/repair/validation for each attempt. | Preserve hypothesis, source/build hashes, metrics and failed controls per prototype. | The proposal admitting two negatives is retained as rejected; safety boundaries repaired before new measurement. |
| [OpenCode Server](https://docs.opencode.ai/docs/server/) and [SDK](https://docs.opencode.ai/docs/sdk/) separate message information from parts and use message/part IDs for references. | Keep declarations, correction referents and event owners distinct; bind the same role and prevent ownership borrowing. | Copular/colon declarations, same-role owner frames and explicitly foreign correction scopes repaired. |

These adaptations are our implementation hypotheses. The cited systems do not demonstrate this natural-language parser or its F1 improvement. Structurally valid records do not prove semantic support or truth.

On source9505300, all25 consumed misses first reached typed_reference_not_excluded in the governing-context check;24 had no registered role declarations and one had a registered Item. Effective tuples match the immutable first outputs. A source span is emitted only for a unique exact UTF-8 byte match; reconstructed/repeated segments retain null offsets.

The repair maps The Object is … and Object: … into the same role processing while preserving value/owner proof. Short Object owner forms and immediate/competing author replies participate in uniqueness. Explicit foreign denials retain their own event scope. Denying clauses after while/before cannot inherit property exclusion. Conflicting event-owner descriptions remain withheld.

Keep original independent first, consumed development, fresh acceptance and real-answer evaluation distinct. Gates and labels are unchanged. No freshv26 authoring before the known repairs/verification. See [current measurements and remaining work](engine073-v25-decision.md). Groq stopped;provider/model/Oracle additional0;real24=0/24;release/CLI adoption NO_GO.

Candidate9082365 applies the same distinction to inert printed literals versus quoted discourse. Colon/copular outside-type proofs are shared by governing checks and risk projection;actual correction clauses,unknown/competing owners and raw-risk authority stay intact. The1340 development controls pass;this does not establish new independent or real-world acceptance.
