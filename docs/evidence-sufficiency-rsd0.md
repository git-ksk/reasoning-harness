# RSD0 residual evidence-sufficiency discovery

Tracking: #91, #116.

RSD0 asks a deliberately narrower question than the adopted D3 runtime:

> After every blocker that D3 can currently express has passed, are there still cases where the
> supplied evidence is relevant but not sufficient to justify the target conclusion?

The answer on the fresh calibration-only RSD0 corpus is **yes**. This is a research result about a
control-plane blind spot, not a claim that a model-backed successor is already safe to adopt.

## Data boundary

RSD0 is fresh and pre-observation:

- fixtures live only under `fixtures/evidence-sufficiency-rsd0/`;
- no semantic holdout-v4/v5 fixture or observed result is read, transformed, relabelled, or used to
  derive the corpus;
- labels and rationales are committed before any provider-backed sufficiency run;
- no provider/model output, score, confidence, verification receipt, or authority-bearing field is
  part of the RSD0 annotation contract;
- operational failure is not a sufficiency label.

The fixture validator also rejects any explicit `EvidenceRequirement`. That is intentional: if a case
can already be expressed as a typed D3 requirement, it belongs to the existing decidability surface,
not the residual corpus.

## Label contract

RSD0 predeclares exactly three diagnostic labels:

```text
sufficient
insufficient
mixed
```

Interpretation:

- `sufficient`: the selected evidence covers the decision-critical information declared by the
  harness-owned request well enough to permit an answerability decision. It is **not** correctness
  evidence and cannot create a `VerificationReceipt` or epistemic promotion.
- `insufficient`: relevant evidence exists, but one or more decision-critical information needs are
  absent, so an assertive answer should not proceed without resolution/additional evidence.
- `mixed`: the evidence is materially split or only partially complete in a way that makes a simple
  globally-sufficient judgment unsafe. For product control it is conservatively compatible with the
  same resolution/abstention direction as `insufficient`, but it remains a separate research label.

## Fresh corpus

The initial corpus contains 12 synthetic calibration cases, four workload families with one case per
label:

| Family | Sufficient control | Insufficient residual | Mixed residual |
| --- | --- | --- | --- |
| incident root cause | incident connection + alternative separation + targeted recovery | correlated DB latency only | DB failures plus simultaneous network-path loss |
| backup / RPO | complete backup coverage + successful restore evidence | backup schedule only | one required state restored, another unresolved |
| rollout safety | representative full-window canary + error/latency guardrails | early partial observation only | error guardrail passes while latency guardrail fails |
| capacity planning | peak demand + every declared bottleneck/headroom | average demand + compute ceiling only | compute headroom good while DB bottleneck violates threshold |

The cases intentionally target residual information patterns that are not represented by D3's current
typed blocker vocabulary: completeness across required components, observation-horizon adequacy,
alternative elimination, aggregation/globality, and materially mixed indicators.

## Deterministic RSD0 result

`semantic_sufficiency_rsd0` validates every artifact and then evaluates the same target through the
unchanged D3 decidability function.

Result:

```text
fixtures:                     12
D3 permit:                    12 / 12
predeclared sufficient:        4 / 12
predeclared insufficient:      4 / 12
predeclared mixed:             4 / 12
non-sufficient surviving D3:   8 / 12
```

Every family contains a sufficient control, so a future RSD1 gate cannot obtain a superficially safe
score merely by always abstaining. Conversely, all eight predeclared `insufficient | mixed` cases
survive D3 as `permit`, demonstrating a measurable residual gap beyond the current typed gate.

This does **not** mean D3 failed its contract. D3's `permit` has always meant only "no deterministic
blocker owned by this gate was found." RSD0 demonstrates that the natural-language product needs an
additional evidence-sufficiency coordinate if it wants to make `permit` useful as an answerability
control on broader tasks.

## Literature rationale

The design follows two useful distinctions from recent work:

- Joren et al., *Sufficient Context: A New Lens on Retrieval Augmented Generation Systems* (ICLR
  2025), treats whether the supplied context contains enough information to answer as a separate
  property from answer generation itself: <https://openreview.net/forum?id=Jjr2Odj8DJ>.
- Gu et al., *Bridging the Detection-to-Abstention Gap in Reasoning Models under Insufficient
  Information* (2026) highlights that detecting missing information is not enough if generation still
  proceeds to an unsupported final answer: <https://arxiv.org/abs/2605.28070>.
- SConU (ACL 2025) remains a later RSD3 anchor for calibrated selective uncertainty rather than an
  RSD0 authority mechanism: <https://aclanthology.org/2025.acl-long.934/>.

These papers motivate the research question and metrics. They do not define Harness authority or
labels.

## RSD0 decision

RSD0 passes its predeclared acceptance criteria:

- four workload families and all three labels are present;
- every fixture is valid and D3-permitted;
- both `insufficient` and `mixed` residual cases survive D3;
- every family has a sufficient control;
- frozen holdout paths are outside the loader;
- the fixture contract contains no model-owned authority.

Therefore **RSD1 is justified**. Its frozen calibration contract is documented in [evidence-sufficiency-rsd1.md](evidence-sufficiency-rsd1.md). The next phase may test a narrow model-backed
`sufficient | insufficient | mixed` coordinate, with the monotone product rule unchanged:
`sufficient` cannot create authority; `insufficient | mixed` may only preserve/force conservative
resolution or abstention.
