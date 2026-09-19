# Action materialization v1 — failed measurement surface

Issue #283's first fresh adoption holdout was frozen and executed as `action-materialization-v1`. The run did **not** produce a valid product comparison. Both provider coordinates hit the same evaluation-fixture defect before a complete semantic trial could be established.

## Frozen coordinate

- freeze tag: `action-materialization-v1-freeze`
- freeze commit: `75d61b891ebd13f7ba43b3e4351baadf38bebb40`
- control product commit: `94ff1b79ac0e9c183d2fc30ceaf41f23f3927d3e`
- candidate product commit: `7a91d272af1bab0a97bf80ed7bba027ff253d50a`
- corpus: `action-materialization-v1`
- evaluator: `reason-action-materialization-v1`
- scoring: `action-materialization-scoring-v1`
- canonical Actions run: `35428076586`
- Mistral job: `105857443683`
- Google job: `105857443732`
- primary seeds: `97211`–`97215`
- observed all-k group: `k=5`

The surface, evaluator, scoring identity, provider/model coordinates, seeds, success predicate, coordinate ordering, and k group were frozen before the first provider call. No semantic rerun was performed.

## Observation

Both jobs completed the frozen live attempt and failed acceptance because every paired trial was operationally incomplete. The hard correctness boundary remained intact:

| provider/model | control complete | candidate complete | correctness violations, control | correctness violations, candidate |
| --- | ---: | ---: | ---: | ---: |
| Mistral `ministral-8b-latest` | 0/5 | 0/5 | 0 | 0 |
| Google `gemini-3.5-flash-lite` | 0/5 | 0/5 | 0 | 0 |

The failure was isolated to the grounded same-key-sibling case. The stale/unknown case completed normally. In every provider/coordinate trial, the grounded case recorded exactly one `malformed_output`.

The intended #283 path distinction was nevertheless observable before the fixture failed:

- control exposed the legacy executable-action model path for all same-key-sibling cases;
- candidate exposed `reason-investigation-intent-v1`;
- candidate materialized actions under `target-intent-materialization-v1`;
- candidate used zero legacy executable-action planner calls on the new path;
- intent rejection remained zero;
- same-key sibling target identities remained distinct;
- correctness-boundary violations remained zero.

These path observations are diagnostic only. They do not convert the incomplete run into an adoption result.

## Root cause

The v1 fixture resolver generated evidence IDs as:

```text
e2e:<source>:<fact_key>
```

Two distinct same-key sibling targets can legitimately acquire the same source/fact combination. The second sibling therefore received the same evidence ID as the first.

The ordinary resolution engine rejects acquired evidence whose ID is already present in the current input and records that attempt as `MalformedOutput`. This is the expected duplicate-ID safety behavior. The defect was in the evaluation fixture identity, not in the control or candidate product coordinate.

Both providers reproduced the same pattern, confirming that this was provider-neutral fixture behavior rather than a #283 candidate regression.

## Disposition

`action-materialization-v1` is preserved as failed measurement evidence and will not be repaired or rerun.

The successor is `action-materialization-v2`, which:

- keeps the exact same control and candidate product commits;
- uses new fresh cases and seeds rather than reusing v1 content;
- uses a dedicated target-aware fixture resolver whose evidence ID includes the exact canonical target ID;
- preflights that different sibling target IDs yield different evidence IDs;
- preflights that repeated acquisition for the same target ID remains idempotent;
- retains the same correctness/operational/path-separation acceptance model.

No product code was changed in response to the v1 failure.

## Preserved machine reports

- [Mistral raw machine report](observations/action-materialization-v1-mistral-run-35428076586-2026-09-19.json), SHA-256 `2b07140a7d5fcb675f14de269ee6ffc4c772842d51fca250f9afc6770c340081`
- [Google raw machine report](observations/action-materialization-v1-google-run-35428076586-2026-09-19.json), SHA-256 `00dfbda6dc447f1833119b5819c2ad26dbbe16e3dba76e56c91c04cf8af2590b`
