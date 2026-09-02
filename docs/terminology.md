# Terminology and naming

Reasoning Harness separates **product concepts**, **machine identifiers**, and **historical research labels**. They are related, but they are not one global version sequence.

## Product concepts

These are the names used in README, CLI guidance, active roadmap, and current product work.

| Product term | Meaning |
| --- | --- |
| **semantic runtime** | Model-backed soft semantic diagnostics that may preserve a soft decision or make it more conservative, but never create verification authority. |
| **answer-safety gate** | The current restrictive final-answer check that can require verification, bounded resolution, or abstention before grounded claims are exposed. |
| **bounded resolution** | Harness-owned acquisition/admission/re-verification loop for missing support. |
| **verified utility recovery** | Current work to recover already-authorized useful answers without relaxing evidence or authority rules. |
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
| `d3-sufficiency-answer-gate-v2` | Current answer-safety configuration ID. |
| `d3-sufficiency-answer-gate-v1` | Previous answer-safety configuration ID retained for rollback/testing. |
| `shared-candidate-initial-render-v1` | Product evaluation comparison contract. |
| `reason-product-dogfood-v8` | Product dogfood report schema version. |

The CLI uses descriptive selectors:

- `--profile current` and `--profile rollback`;
- `--safety-profile current`, `--safety-profile legacy-v1`, and `--safety-profile baseline`.

Legacy selectors `d3`, `v3`, `d3-sufficiency`, and `d3-sufficiency-v1` remain accepted aliases for compatibility.

## Historical research labels

Labels such as `R1`–`R4`, `D1`–`D3`, `RSD0`–`RSD4`, and `NL-1`–`NL-5` were **issue-scoped research or implementation phase names**. They are not comparable version numbers and must not be read as one project-wide sequence.

They remain in research evidence, frozen artifact names, historical run reports, and chronology documents because removing them would destroy provenance. New active product work should use descriptive names instead of creating another short-code sequence.

## Rule of thumb

If you are **using the product**, think in terms of verified evidence, semantic runtime, answer safety, bounded resolution, and grounded/qualified/unknown answers.

If you are **operating or integrating the runtime**, machine IDs matter for reproducibility and rollback.

If you are **reading the research history**, phase labels matter only within the issue or study that defined them.
