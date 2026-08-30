# Assumption and unsupported-premise diagnostics

The assumption diagnostic layer asks a narrow process question: **which propositions are used as inference premises without trusted support or explicit permission from the harness input?** It is not a general fact checker and does not infer arbitrary world knowledge.

## Input authority

`HarnessInput.assumptions` contains propositions the harness explicitly permits as premises. It is harness-owned and absent from `ReasoningCandidate`, so a model cannot authorize its own assumption. Assumptions are distinct from `hypotheses`: a hypothesis is a proposition the task asks to evaluate, while an explicit assumption is a premise the task permits without claiming independent verification.

## Statuses

- `supported`: the premise claim is `known`/`supported`, or is an `inferred` claim whose derivation bottoms out in trusted supported/known claims or explicit input assumptions.
- `explicit_input_assumption`: the premise proposition exactly matches a harness-owned explicit assumption.
- `unsupported`: the premise has a typed proposition but neither trusted support nor explicit assumption status. This is a hard **process** finding relative to the supplied context.
- `unbound`: the premise has no typed proposition. Its support cannot be checked deterministically, so the finding is soft.

Candidate-authored epistemic labels do not grant support. In particular, proposing `inferred` does not make a premise trusted unless its derivation is independently grounded as described above.

## Distinction from `unknown`

`unknown` is an epistemic state for a claim: the harness does not have enough authority to accept or reject it. `unsupported_premise` is a diagnostic about **use**: a typed proposition is actively used as an inference premise even though the supplied context does not support or explicitly permit it. An artifact may therefore contain an unsupported-premise finding while still preserving an overall `unknown` verdict.

## Distinction from unsupported causal edges

An unsupported premise asks whether an input to an inference is grounded. An unsupported causal edge asks whether a specific typed cause→effect relation has causal evidence. A premise can be supported as a fact while the causal relation that uses it remains unsupported, and a causal relation cannot repair an otherwise ungrounded premise. The two diagnostic families remain separate.

## Reporting and authority

The five-case deterministic corpus under `fixtures/assumptions/` is reported separately from final verdict accuracy and the causal corpus. Repeated-trial diagnostic aggregation can include assumption findings, but findings never create verification receipts, mutate claim state, or directly force `accept | reject | unknown`. Semantic extraction of assumptions from free-form prose remains a future soft diagnostic problem.
