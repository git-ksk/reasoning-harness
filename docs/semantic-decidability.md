# Semantic decidability and evidence-sufficiency research

Issue #73 follows the rejected R4 semantic successor study. The goal is not to make model
agreement more authoritative. The goal is to identify harness-owned conditions under which an
assertive soft semantic decision should not be allowed to proceed.

`soft-semantic-v3` remains the runtime baseline. This document describes calibration-only
research and does not change runtime behavior.

## Research phase naming

Phase labels are local to their research issue; they are not runtime or release versions.

- `R1`–`R4` are the Issue #59 semantic-successor **research** stages. `R4` specifically means the
  fourth stage: the frozen independent successor/holdout-v4 evaluation.
- `D1`–`D3` are the Issue #73 **decidability** stages. `D1` is the deterministic gate contract and
  calibration surface, `D2` is provider-backed decidability calibration, and `D3` is candidate
  freeze/adoption preparation if D2 passes.

The prefixes intentionally prevent a new #73 calibration phase from being confused with the
historical #59 successor sequence. `D2` does not mean "version 2" and is not newer than `R4` in a
runtime-version sense.

## Research question

Can a provider-neutral, harness-owned decidability/evidence-sufficiency gate reduce correlated
semantic over-assertion by forcing `abstain` when typed binding or evidence qualification is
insufficient, without weakening the existing deterministic authority boundary or collapsing useful
decision coverage?

The gate is a control-plane mechanism, not a truth estimator.

## R4 lesson and data boundary

R4 showed that cross-model disagreement is useful risk evidence but cross-model agreement is not
correctness evidence. The rejected successor also demonstrated the specific failure mode this phase
targets: multiple models can make the same stable assertive semantic error.

Holdout-v4 is observed immutable diagnostic history. It must not be relabelled, copied, transformed,
used to derive gate rules, or used for calibration. No holdout-v5 is created during D1/D2. A future
holdout-v5 is allowed only after a calibration candidate has been frozen with predeclared adoption
criteria.

## Design principle: permission, not correctness

A deterministic D1 gate reports whether a known harness-owned blocker requires abstention. It does
not report that an otherwise permitted decision is correct.

```text
DecidabilityDisposition
  permit
  force_abstain
```

`permit` means only that the gate found no deterministic blocker in the typed information it owns.
It is not evidence, a verification conclusion, a confidence score, or a verdict.

The initial deterministic reason vocabulary is deliberately small:

```text
missing_target_binding
missing_proposition_binding
no_evidence_for_explicit_requirement
no_qualified_evidence_for_explicit_requirement
conflicting_qualified_evidence
```

Detailed evidence-qualification reasons remain owned by the existing
`EvidenceQualificationInspection`; the decidability layer should reference or summarize those
results rather than create a competing qualification ontology.

## Monotone composition

The gate may only preserve a soft semantic decision or make it more conservative.

```text
(base finding,    permit)        -> finding
(base no_finding, permit)        -> no_finding
(base abstain,    permit)        -> abstain
(base finding,    force_abstain) -> abstain
(base no_finding, force_abstain) -> abstain
(base abstain,    force_abstain) -> abstain
```

Operational or protocol failure is not a semantic result and never enters this composition table.
Malformed semantic output is not repaired by the gate.

The gate cannot:

- create a soft or hard finding;
- turn `abstain` into an assertive decision;
- create trusted evidence or a `VerificationReceipt`;
- promote epistemic state;
- determine a final verdict;
- reinterpret model agreement as truth;
- convert operational failure into `no_finding`, `finding`, or `abstain`.

## Existing typed information to reuse

D1 should reuse existing harness-owned contracts rather than asking a model to repeat metadata the
harness already owns.

### Target and binding

`SoftJudgeRequest.target` already distinguishes proposition, causal-relation, claim, and inference
targets. For claim/inference targets, the harness can check that referenced artifact objects exist
and that propositions needed for qualification are actually bound.

A missing claim/inference or a required missing proposition binding is a structural blocker. The
model should not be asked to infer that binding from prose.

### Evidence requirements and qualification

The core already owns:

- `EvidenceRequirement { proposition, as_of_unix_seconds, scope, minimum_authority_class }`;
- `EvidenceMetadata { temporal, scope, provenance_class }`;
- `EvidenceAuthorityPolicy`;
- `EvidenceQualificationInspector`;
- `EvidenceQualificationAssessment { Qualified, Disqualified, Unknown }`;
- explicit temporal, scope, authority, metadata, and conflict reason codes.

These are the primary source of deterministic evidence-sufficiency signals only when the semantic
question is directly bound to that proposition requirement. Endpoint requirements must not be
promoted into relation-level or applicability-level requirements by inference.

Absence of an `EvidenceRequirement` is **not** itself evidence of insufficiency. A deterministic
D1 blocker is allowed only when the harness has made an explicit requirement/binding and can show
that the requirement is unsatisfied or unqualifiable.

This avoids turning the mechanism into “abstain unless every semantic fact is formalized.”

### Causal information

The existing causal inspector is also harness-owned, but ordinary `CausalSupportStatus::Unknown`
or `MissingCausalEvidence` must not automatically force abstention. Those states can be the reason
a soft semantic judge is useful in the first place.

D1 may use structural causal binding failures, but should not reinterpret a deterministic causal
`Unknown` as proof that semantic inspection is impossible. In D1 v1, a generic `EvidenceRequirement`
on a cause or effect proposition is **not** treated as a requirement for the directional causal
relation. A future causal sufficiency gate needs an explicit typed relation-level binding first.

## D1 deterministic algorithm

The first research implementation should be a pure deterministic function over a
`SoftJudgeRequest` plus the relevant `ReasoningArtifact`.

High-level behavior:

1. validate target identity against the artifact when the target is a claim or inference;
2. derive explicit proposition requirements only for diagnostic kinds whose semantic question is
   directly about proposition conflict/support (`contradiction` and `unsupported_premise`);
3. if required proposition binding is missing, return `force_abstain`;
4. run/reuse `EvidenceQualificationInspector` for explicit relevant requirements;
5. if an explicit requirement has no candidate evidence, return `force_abstain`;
6. if an explicit requirement has candidates but none are `Qualified`, return `force_abstain`;
7. if qualified evidence for an explicit requirement is conflicting, return `force_abstain`;
8. otherwise return `permit`.

The exact target-to-proposition derivation must remain conservative. For `contradiction` and
`unsupported_premise`, a proposition target maps to itself and a claim may map through its explicit
proposition binding. Claim/inference targets still require structural target/proposition bindings
where applicable. `counterexample` does not inherit a generic proposition evidence requirement as
an applicability rule, and `causal_gap` does not inherit cause/effect proposition requirements as a
relation-level sufficiency rule. No free-text semantic binding is invented.

If a target has no explicit evidence requirement, or the requirement is not explicitly typed as a
precondition for that semantic question, D1 does not infer one merely because it would be useful.

## D1/D2 calibration surface

Create a new calibration-only fixture family separate from all historical holdouts. The fixture
format should contain both the ordinary semantic request and a concrete harness-owned artifact so
that the decidability result is reproducible without a provider.

Use paired/metamorphic cases. Each pair preserves the semantic target/concern and mutates only a
typed sufficiency precondition.

Required mutation families:

1. complete proposition binding -> remove the binding;
2. qualifying evidence -> remove all evidence satisfying an explicit requirement key;
3. sufficient authority -> lower provenance below the minimum authority class;
4. applicable scope -> narrow or make the scope disjoint;
5. temporally valid evidence -> stale or not-yet-valid at the required `as_of`;
6. complete required metadata -> remove temporal/scope/provenance metadata;
7. one qualified value -> conflicting qualified values.

Every insufficiency mutation must have a paired control that remains `permit`.

The deterministic D1 mutation corpus covers the three kinds for which v1 has a defensible typed
blocker: proposition support/conflict (`contradiction`, `unsupported_premise`) and structural claim
binding (`counterexample`). `causal_gap` is intentionally a permit-only control until the harness
owns an explicit typed relation-level evidence requirement. D1 labels are gate eligibility, not
semantic truth. Provider-backed D2 semantic labels remain separately reported across all four kinds.

## Predeclared deterministic gates

Before any provider-backed D2 run:

- 100% insufficiency-mutation monotonicity: declared mutations move `permit -> force_abstain`;
- 100% paired-control preservation: sufficient controls remain `permit`;
- 100% composition invariants: the gate can only preserve a decision or move it to `abstain`;
- malformed/missing target references fail closed without creating semantic output;
- zero authority-boundary regressions in existing deterministic tests;
- no holdout path is accepted by the calibration runner.

These are contract gates, not model-quality claims.

## D2 label axes and provider-backed metrics

D2 must not collapse semantic polarity and permission to assert into one label. A fixture therefore
needs two independent pre-observation labels:

```text
semantic_label     = positive | negative | ambiguous
assertive_eligibility = permit | force_abstain
```

`semantic_label` describes the diagnostic concern in the supplied semantic content. The eligibility
label describes whether the harness-owned typed preconditions allow an assertive soft decision.
Neither label is derived from provider output.

This separation avoids a metric bug: a positive or negative case whose explicit evidence requirement
is unsatisfied should be conservatively forced to `abstain`. Counting that expected abstention as a
false negative would make correct gating lower semantic recall by construction.

For a matched D2 case, run the unchanged R2 materialized semantic request once per provider/seed.
Apply the deterministic gate to the paired harness-owned artifact variants afterward. If two variants
have identical semantic request content and differ only in harness-owned qualification metadata,
they must reuse the same provider observation rather than sampling the model twice. That isolates the
gate intervention from model-sampling noise and reduces provider calls.

The checked-in D2 v1 calibration manifest follows this design with 15 semantic cases sourced only
from `fixtures/semantic-judges/`: 11 eligible positive/negative cases and four eligible ambiguous
controls. Seven of the clear cases also have one paired `force_abstain` variant covering evidence
presence, scope, temporal validity, authority, required provenance metadata, claim binding, and
qualified-evidence conflict. The force subset spans three diagnostic kinds and intentionally
excludes `causal_gap`; causal cases remain permit controls until relation-level sufficiency is typed.
Existing semantic labels are copied into the D2 manifest and deterministically checked against their source fixture
before provider credentials are read.

`reason-decidability-study` validates the exact D2 path, resolves every typed gate expectation, and
then performs exactly one unchanged R2 materialization call per semantic case/seed. The same returned
decision is composed with every typed variant. Operational failure leaves all variant decisions
unset; it is never converted into abstention. No live D2 provider observation is recorded in the
repository at this stage.

Any optional model-backed residual decidability gate is a separate later arm and must not be mixed
into D1 results.

Report per provider/model and per trial:

- provider/protocol completion for the unchanged R2 semantic calls;
- semantic precision/recall on **eligible positive/negative** cases only;
- eligible clear-case decision coverage;
- eligible ambiguous abstention, reported separately from typed insufficiency;
- typed-insufficiency abstention rate on `force_abstain` variants;
- unsafe assertive rate on `force_abstain` variants before and after composition;
- gate escalation count/rate and deterministic reason distribution;
- overall decision coverage as descriptive only, because its maximum depends on the predeclared
  proportion of ineligible variants;
- cross-seed stability of the base semantic decision and composed result;
- token and latency cost, with deterministic gate overhead reported separately.

Never pool multiple models into a truth label. Never score a `force_abstain` case as an ordinary
positive/negative recall failure.

A deterministic-gate calibration candidate is worth freezing only if the same provider-neutral rule:

- has 1.0 typed-insufficiency abstention and 0 unsafe assertive decisions after composition;
- reduces a non-zero base unsafe-assertion rate on the typed-insufficiency subset for at least one
  evaluated provider, otherwise its empirical benefit remains unproven;
- retains eligible clear-case decision coverage >= 0.90 per provider;
- retains eligible assertive precision and recall >= 0.95 where defined;
- introduces zero gate escalations on predeclared `permit` controls;
- does not add provider/model-specific semantic branches;
- preserves the hard authority and operational-failure invariants.

An always-abstain mechanism fails through eligible clear-case coverage and permit-control preservation;
it cannot hide behind a high insufficiency-abstention score.

If the deterministic gate only catches tautological metadata failures and provides no meaningful
provider-backed reduction in over-assertion, D1 should be recorded as insufficient rather than
expanded post hoc against historical holdouts.

## Residual soft decidability is a separate hypothesis

D1 intentionally does not claim to detect every form of missing decisive distinction. Some
insufficiency is itself semantic and may not be represented by current typed metadata.

A later calibration arm may test a narrow model-backed decidability output such as
`sufficient | insufficient | mixed`, but only after the deterministic surface is characterized. If
such an arm is tested:

- it receives the same provider-neutral semantics across models;
- it cannot see or emit authority fields already owned by the harness;
- `insufficient`/`mixed` may only force `abstain`;
- `sufficient` still does not constitute correctness evidence;
- its operational failure remains separate from the semantic decision;
- it is evaluated as a distinct coordinate, not combined with model consensus or majority vote.

## Literature anchors

These sources motivate the separation of answerability/evidence sufficiency from answer generation;
they do not define harness authority semantics.

- Rajpurkar, Jia, Liang, [*Know What You Don’t Know: Unanswerable Questions for SQuAD*](https://aclanthology.org/P18-2124/) (ACL 2018): answerability must be detected rather than forcing a guess when the supplied context does not support an answer.
- Thorne et al., [*FEVER: a Large-scale Dataset for Fact Extraction and VERification*](https://aclanthology.org/N18-1074/) (NAACL 2018): `NotEnoughInfo` is distinct from support/refutation and evidence is part of the verification task.
- Xin et al., [*The Art of Abstention: Selective Prediction and Error Regularization for Natural Language Processing*](https://aclanthology.org/2021.acl-long.84/) (ACL 2021): abstention should be evaluated as a risk/coverage trade-off rather than accuracy alone.
- Gu et al., [*Bridging the Detection-to-Abstention Gap in Reasoning Models under Insufficient Information*](https://arxiv.org/abs/2605.28070) (2026 preprint): an explicit answerability control decision before solving targets cases where a model detects missing information but still answers assertively.

## Adoption sequence

1. D1 deterministic contract and metamorphic calibration fixtures;
2. D2 provider-backed calibration against unchanged R2 semantic decision semantics;
3. optional residual soft-decidability arm only if D1 leaves a measurable gap;
4. freeze one provider-neutral successor configuration and thresholds;
5. create a new observation-free holdout-v5;
6. run the independent adoption test once, without post-observation rescue.

Until step 6 passes, `soft-semantic-v3` remains the runtime baseline.
