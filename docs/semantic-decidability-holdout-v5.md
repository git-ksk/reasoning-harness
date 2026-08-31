# Semantic decidability holdout-v5

Holdout-v5 is the first independent adoption surface for frozen D3 candidate
`semantic-decidability-d3-v1`. It was authored only after the D3 candidate freeze merged on main at
`ca8b0e48bd3e06b16f56b0be670c0eb45ba21962`.

No provider observation is part of corpus construction. Source fixtures contain no recorded
observations, and the holdout runner rejects any holdout-v5 source that acquires one before provider
initialization. Holdout-v4 remains immutable diagnostic history; holdout-v5 was authored from fresh
scenarios rather than transforming or copying v4 failures. The uniqueness regression compares the
new requests only with the calibration and pre-v4 holdouts so v4 is not used as a construction/tuning
input.

## Frozen candidate

```text
candidate_id              semantic-decidability-d3-v1
semantic_baseline         soft-semantic-v3
materialization_contract  materialization-r2-v1
decidability_contract     deterministic-explicit-typed-preconditions-v1
composition               permit_preserves__force_abstain_only
```

The candidate cannot create a finding, evidence, receipt, epistemic promotion, verdict, or semantic
result from an operational failure. `permit` preserves the R2 semantic decision. `force_abstain` can
only move an assertive decision to `abstain`.

## Corpus shape

The semantic source corpus is `fixtures/semantic-judges-holdout-v5/`; typed eligibility manifests
are `fixtures/semantic-decidability-holdout-v5/`.

- 24 fresh semantic cases;
- 6 cases for each of contradiction, unsupported-premise, causal-gap, and counterexample;
- 8 positive, 8 negative, and 8 ambiguous labels;
- 24 permit controls;
- 10 clear cases with one additional typed-insufficiency `force_abstain` variant;
- no ambiguous case has a force variant;
- no causal-gap case has a force variant, because D3 does not own a relation-level evidence
  requirement contract;
- one unsupported-premise case targets a typed inference so holdout-v5 exercises structural
  inference binding that D2 did not measure.

## Semantic label/spec review

| ID | Kind | Label | Pre-observation rationale |
| --- | --- | --- | --- |
| v5h01 | contradiction | positive | AES-256 candidate directly conflicts with same-policy AES-128 snapshot. |
| v5h02 | contradiction | positive | mTLS requirement directly conflicts with client-certificate authentication being disabled. |
| v5h03 | contradiction | negative | Candidate and schedule both state 02:00 UTC. |
| v5h04 | contradiction | negative | Candidate and retry configuration both define four total attempts including the initial attempt. |
| v5h05 | contradiction | ambiguous | 300-second default conflicts with a 120-second group override only if the service belongs to that unresolved group. |
| v5h06 | contradiction | ambiguous | v2 snapshot conflicts with v3 only if the unresolved snapshot time applies to the deployment. |
| v5h07 | unsupported premise | positive | Japan residency appears only in the candidate; supplied requirements do not establish location. |
| v5h08 | unsupported premise | positive | At-least-once semantics do not establish the candidate's exactly-once premise. |
| v5h09 | unsupported premise | negative | Active lifecycle policy explicitly supplies the 30-day premise. |
| v5h10 | unsupported premise | negative | The inference's versioning premise is explicitly supplied; the target is premise support, not conclusion validity. |
| v5h11 | unsupported premise | ambiguous | A global report covers every listed region, but the omitted list leaves target-region applicability unresolved. |
| v5h12 | unsupported premise | ambiguous | Role action permits decrypt in one policy layer, while key-policy/condition applicability is unavailable. |
| v5h13 | causal gap | positive | CPU/latency correlation lacks direction, intervention, or mechanism evidence. |
| v5h14 | causal gap | positive | Release/error correlation is confounded by a simultaneous traffic-mix shift. |
| v5h15 | causal gap | negative | Repeated disable/re-enable intervention under fixed workload supports the proposed relation. |
| v5h16 | causal gap | negative | Mechanism traces plus replacement intervention under the same load support the relation. |
| v5h17 | causal gap | ambiguous | Autoscaler intervention coincides with cache warm-up, leaving causal attribution mixed. |
| v5h18 | causal gap | ambiguous | Temporal ordering exists, but a shared-path observer leaves measurement bias unresolved. |
| v5h19 | counterexample | positive | Successfully imported in-scope row has an empty identifier. |
| v5h20 | counterexample | positive | Signed in-scope production build fails provenance verification. |
| v5h21 | counterexample | negative | Kernel 6.6 observation is explicitly staging, outside the production generalization. |
| v5h22 | counterexample | negative | Missing-key observation is explicitly an initial attempt, outside the retry generalization. |
| v5h23 | counterexample | ambiguous | Unencrypted device is a counterexample only if pending enrollment already makes it managed. |
| v5h24 | counterexample | ambiguous | Unhealthy replica is a counterexample only if it was active during the unresolved transition interval. |

The two applicability-oriented ambiguous cases `v5h05` and `v5h11` were tightened during this static
review before any provider observation so their ambiguity depends on an explicit unresolved binding,
not merely missing detail. This is the final label/spec edit point for v5.

## Typed insufficiency mutations

The 10 force variants are independent scenarios that reuse the frozen D3 contract rather than adding
new gate behavior:

| Source | Mutation | Expected gate |
| --- | --- | --- |
| v5h01 | explicit requirement loses all candidate evidence | force_abstain |
| v5h02 | evidence authority falls below required class | force_abstain |
| v5h03 | evidence scope is disjoint from required service scope | force_abstain |
| v5h04 | evidence is stale at explicit `as_of` | force_abstain |
| v5h07 | required provenance metadata is absent | force_abstain |
| v5h08 | two qualified values conflict for the required proposition key | force_abstain |
| v5h10 | inference premise claim loses its explicit proposition binding | force_abstain |
| v5h19 | counterexample target claim loses proposition binding | force_abstain |
| v5h20 | counterexample target claim loses proposition binding | force_abstain |
| v5h21 | counterexample target claim loses proposition binding | force_abstain |

Every mutation has a matched permit control. Causal endpoint evidence requirements are intentionally
not introduced as relation-level requirements.

## Frozen independent observation plan

The checked-in `semantic-decidability-holdout-v5` workflow has no study-shaping inputs and fixes:

- Google `gemini-3.5-flash-lite` and Mistral `ministral-8b-latest`, reported separately;
- the complete 24-case semantic corpus and all 24 typed manifests;
- five trials with seeds `7000` through `7004`;
- 512 maximum output tokens;
- 120 provider calls per provider (24 cases x 5 trials), 240 total if both arms complete;
- one unchanged R2 semantic observation per source case/seed reused across its matched typed variants;
- 50 typed-insufficiency observations per provider (10 force variants x 5 trials);
- no cross-model voting, truth pooling, prompt branch, fixture subset, or post-observation threshold input.

The SHA-256 list in `fixtures/semantic-decidability-holdout-v5.sha256` covers every source and typed
manifest JSON file. The workflow verifies it before provider initialization.

## Frozen adoption gates

Each provider must independently satisfy:

- 120/120 provider/protocol-complete calls and 5/5 complete trials;
- aggregate eligible clear decision coverage >= 0.90;
- aggregate eligible precision and recall >= 0.95;
- each complete trial eligible clear coverage, precision, and recall >= 0.90;
- typed-insufficiency abstention exactly 1.0 in aggregate and every complete trial;
- composed unsafe assertions exactly 0;
- zero deterministic escalation on permit controls;
- zero cross-seed `decision_disagreement` on eligible clear semantic fixtures.

Across the two separately scored provider arms, at least one must have a non-zero base unsafe
assertion count on the force subset, or D3 has not demonstrated independent empirical utility.
Eligible ambiguous abstention is diagnostic and remains outside the D3 adoption threshold because
D3 does not alter permit-only semantic ambiguity.

An operationally incomplete arm may be repeated only with this exact frozen corpus/configuration.
A semantic gate failure rejects D3; it cannot be rescued by relabelling, editing the corpus,
changing thresholds, selecting seeds, or adding another model after observation.

## Observed pilot status

The frozen holdout-v5 surface has now been observed without changing its payloads or adoption gates.

| provider/model | operational result | clear coverage / precision / recall | typed insufficiency | base unsafe -> composed unsafe | ambiguous abstention | interpretation |
| --- | --- | --- | --- | --- | --- | --- |
| Mistral `ministral-8b-latest` | 120/120, 5/5 complete | 1.000 / 1.000 / 1.000 | 50/50 abstain | 50 -> 0 | 0.500 | pilot pass |
| Google `gemini-3.5-flash-lite` | operationally incomplete | not scored | not scored | not scored | not scored | AI Studio requests-per-day quota exhausted; exact frozen rerun only |
| Google-hosted `gemma-4-31b-it` replication | 120/120, 5/5 complete | 1.000 / 1.000 / 1.000 | 50/50 abstain | 50 -> 0 | 0.500 | cross-family replication pass; not retroactively added to the original provider matrix |
| NVIDIA `nvidia/nemotron-3.5-lightning-30b-a3b` bounded probe | D2 7/15 success; v5 timed out after fixture 18/24 | not scored | not scored | not scored | not scored | protocol-capability negative control; repeated forbidden `finding` fields under R2 materialization |

Ministral 8B and Gemma 4 31B produced identical base decisions for all 120 matched holdout-v5 case/seed observations. This supports stabilization of `semantic-decidability-d3-v1` for models satisfying the R2 materialized-decision protocol boundary, but it is not evidence of universal model compatibility.

The next repository phase is D3 stabilization and reversible runtime adoption, not further model-matrix expansion by default. Additional model runs should target a specific compatibility or capability hypothesis. Residual soft decidability is the first successor research hypothesis after stabilization if a fresh calibration corpus demonstrates insufficiency not representable by the deterministic typed gate.
