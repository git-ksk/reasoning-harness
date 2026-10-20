# Consumed real24: measured repair priorities

2026-10-20. [First development score](../evaluation/engine073-realworld-v2-consumed-score-v1/README.md) is unchanged: Google ordinary case F1 .583333 in both arms, helper .222222→.400000. Full Mistral/Groq F1 remains null. Engine release/CLI adoption remains NO_GO. [Detailed Japanese findings](engine073-consumed-real24-findings.ja.md).

The acquisition mode is `host_curated_facts_with_full_captured_context`: full official documents support independent annotation, but the runtime receives host-supplied typed facts. This is not automatic full-text extraction, open-Web discovery or an Engine-only score. Failures include information lost in those supplied facts as well as CLI investigation/finalization. Never solve this by trusting arbitrary generated prose.

| Priority | Saved failures | Minimal repair direction |
| --- | --- | --- |
| 1 | rw02/06 typed values lose source conditions/reasons; rw11/17 finalization loses applicability/unresolved facets; ordinary citations0/28 | Preserve source/admission-bound conditional information, unresolved facets and displayed citations through finalization. Do not feed gold into runtime. |
| 2 | rw10/13 stop after accepting only one comparison branch; rw16 stops with an unbound explanation | Distinguish surviving-claim acceptance from task/plan coverage; acquire required branches within the same bounds or name unresolved facets. |
| 3 | rw04/20 omit non-execution notices; rw19 omits separate prerequisites | Display actual host-controlled execution limitations and source-bound prerequisite scope. |
| 4 | helper rw05/08/15 omit known shared-origin/non-corroboration notices despite eligible qualified status | Render verified captured URL/version/span lineage, without inferring independence or relaxing correct duplicate-source refusal. |
| 5 | helper rw01/14/21 public equivalence review refuses acquired scoped rules | Investigate condition/platform/requested-property comparison; advice/reference Booleans cannot override authority. Internal root cause remains unconfirmed. |

Actual host8525330 callsites: ordinary early Accept main.rs3461/3768; action stop3579–3581; finalization4297; partial display4348. Current main53be613 counterparts were inspected statically, not rerun. Independent diagnosis anchor `63156810a73981c68532f63c32d5039f301231c461e9e61e4f3938a8da3b0713`; separate stop-location correction anchor `ed101902ea072f944518760ea9a196c67a503728c13d82cc83764776e1315f81`. Raw/annotations/full diagnosis stay private. No scoring recalculation, relabel, Engine/model execution or source edits during diagnosis.

Mistral is separate: merged PR549 fixes known-usage loss, not256-token planner JSON truncation or proven whole-cohort completion. Quota is unverified and new calls are held. Groq remains deferred. #491 now has measured task-coverage examples warranting a minimal coverage fix; verified-origin display does not authorize all of #488, and no new trusted-supersession implementation for #489 is established here.

Use these24 only for development improvement/regression measurement. Freeze functional integration, complete M0 and all-provider readiness before new independent authorship. Keep F1≥.90/P≥.95/R≥.85, zero hard safety/positive regressions and measured utility improvement. These are pending repair priorities, not completed fixes or acceptance.
