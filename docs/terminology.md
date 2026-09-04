# Terminology and naming

Reasoning Harness separates **product concepts**, **machine identifiers**, and **historical research labels**. They are related, but they are not one global version sequence.

## Product release version

CLI SemVer such as `v0.2.0` is a **product/distribution compatibility coordinate**. A minor product release may package new CLI/runtime capabilities without creating a new research generation. In particular, `v0.2.0` does not rename or rewrite the frozen Stage-C/RSD2 evidence, semantic runtime ID, answer-safety ID, or successor candidate identity.

## Product concepts

These are the names used in README, CLI guidance, active roadmap, and current product work.

| Product term | Meaning |
| --- | --- |
| **semantic runtime** | Model-backed soft semantic diagnostics that may preserve a soft decision or make it more conservative, but never create verification authority. |
| **answer-safety gate** | The current restrictive final-answer check that can require verification, bounded resolution, or abstention before grounded claims are exposed. |
| **bounded resolution** | Harness-owned acquisition/admission/re-verification loop for missing support. |
| **verified utility recovery** | Deterministic recovery of already-authorized useful answers without relaxing evidence or authority rules. |
| **smoke set** | The historical six-case product dogfood set (`product-dogfood-v1`). |
| **capability matrix** | The frozen 24-case / 8-family development evaluation (`product-dogfood-v2`). |
| **replication** | Multi-run evaluation on the frozen capability matrix using predeclared fresh seeds. |
| **fresh holdout** | Separately authored/frozen cases evaluated only after development/selection is complete. |

## Machine and compatibility identifiers

Exact identifiers remain stable because reports, rollback behavior, schemas, and automation depend on them. They are implementation identities, not names users need to memorize.

| Identity | Role |
| --- | --- |
| `semantic-decidability-d3-v1` | Current semantic runtime configuration ID. |
| `soft-semantic-v3` | Characterized previous semantic runtime configuration ID retained for rollback. |
| `verified-target-answer-gate-v1` | Current answer-safety configuration ID; exact trusted verification may short-circuit redundant model sufficiency. |
| `d3-sufficiency-answer-gate-v2` | Answer-safety rollback configuration retained for reproducibility. |
| `d3-sufficiency-answer-gate-v1` | Previous answer-safety configuration ID retained for rollback/testing. |
| `shared-candidate-initial-render-v1` | Product evaluation comparison contract. |
| `reason-product-dogfood-v10` | Product dogfood report schema version; v10 adds retry/checkpoint execution telemetry without changing semantic gates. |

The CLI uses descriptive selectors:

- `--profile current` and `--profile rollback`;
- `--safety-profile current`, `--safety-profile legacy-v1`, and `--safety-profile baseline`.

Legacy selectors remain accepted for compatibility: `d3` selects the current semantic runtime, `v3` its rollback, `d3-sufficiency` / `d3-sufficiency-v2` select the previous answer-safety rollback, and `d3-sufficiency-v1` selects the older v1 gate.

## Historical research labels

Labels such as `R1`–`R4`, `D1`–`D3`, `RSD0`–`RSD4`, and `NL-1`–`NL-5` were **issue-scoped research or implementation phase names**. They are not comparable version numbers and must not be read as one project-wide sequence.

They remain in research evidence, frozen artifact names, historical run reports, and chronology documents because removing them would destroy provenance. New active product work should use descriptive names instead of creating another short-code sequence.

## Rule of thumb

If you are **using the product**, think in terms of verified evidence, semantic runtime, answer safety, bounded resolution, and grounded/qualified/unknown answers.

If you are **operating or integrating the runtime**, machine IDs matter for reproducibility and rollback.

If you are **reading the research history**, phase labels matter only within the issue or study that defined them.
