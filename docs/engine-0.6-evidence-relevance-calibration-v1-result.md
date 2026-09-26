# Engine 0.6 evidence-target relevance calibration v1 result

Status: frozen v1 observation completed operationally, but correctness and utility gates failed. This result is immutable historical calibration evidence; it must not be rerun or rescored under changed semantics.

## Frozen identity

- freeze tag: `engine-0.6-evidence-relevance-calibration-v1-freeze`
- candidate commit: `6c7551ecb9abffca5e42efc852cd4222995d9cc5`
- GitHub Actions run: `35991268202`
- suite: `evidence-relevance-calibration-v1`
- cases: 26
- seed: `4621601`
- Mistral: `ministral-8b-latest`
- Google: `gemini-3.5-flash-lite`

Preflight and both provider arms completed successfully. The final semantic gate failed because both arms retained one calibration case as `relevant` where v1 expected `ambiguous`, and additional utility mismatches remained.

## Raw v1 metrics

| metric | Mistral | Google |
| --- | ---: | ---: |
| successful provider cases | 26/26 | 26/26 |
| provider failures | 0 | 0 |
| proposal exact accuracy | 20/26 (76.92%) | 23/26 (88.46%) |
| materialized exact accuracy | 20/26 (76.92%) | 24/26 (92.31%) |
| reported wrong-target relevance retention | 1 | 1 |
| false relevance rejection on expected-relevant | 0 | 0 |
| expected-relevant left ambiguous | 0 | 0 |
| utility misses | 5 | 1 |
| deterministic safety overrides | 0 | 2 |
| lexical baseline exact accuracy | 12/26 (46.15%) | 12/26 (46.15%) |
| lexical baseline wrong-target relevance retention | 8 | 8 |
| lexical baseline expected-relevant misses | 2 | 2 |
| model calls | 26 | 26 |
| provider attempts | 26 | 26 |
| total tokens | 9,783 | 10,310 |
| model-call latency total | 15,332 ms | 20,440 ms |

The semantic path materially outperformed the deliberately simple lexical baseline, but v1 does not meet the release gate.

## Shared correctness finding: case 24

Both providers returned `relevant` for `24_conflicting_sections`, while v1 expected `ambiguous`.

The frozen v1 case contains two passages that both explicitly concern the exact target and requested availability relation, but disagree on the factual answer: one says West is supported and another says West is not yet supported.

On review, this fixture conflates **semantic relevance** with **truth/contradiction**. Material can be directly relevant to a target while containing mutually conflicting claims. The ordinary contradiction, qualification, verification, and finalization pipeline owns that conflict; the relevance gate must not suppress it merely because the content disagrees.

Therefore the v1 `wrong-target relevance retention` metric over-counts this case as a correctness violation. The frozen v1 result is not rewritten. A successor calibration must use a genuinely relevance-ambiguous conflicting/binding case rather than a truth-conflict case.

## Utility finding: irrelevant vs ambiguous

Mistral collapsed five v1 ambiguity cases to `irrelevant`: unknown rename, partial identity, mixed multi-product binding, insufficient local passage, and URL-only identity. Google collapsed only the unknown-rename case to `irrelevant`; its partial-identity `relevant` proposal was safely overridden by the strict Harness-owned identity floor.

This shows that the v1 model guidance under-specifies the boundary:

- `irrelevant` should require affirmative evidence that material is about a different target or relation;
- missing/partial binding, uncertain rename/alias, truncated local support, or unresolved multi-product applicability should be `ambiguous` rather than `irrelevant`;
- candidate insufficiency must not be treated as evidence of irrelevance.

This is a utility distinction: false rejection can trigger unnecessary additional acquisition or discard useful material.

## Confirmed v1 positives

v1 still establishes useful evidence:

- 26/26 provider calls completed on both arms with zero operational failures;
- the Harness blocked unsafe model `relevant` proposals when strict identity anchors were absent;
- URL-only and weak-signal identity did not self-authorize relevance;
- Japanese/English aliases and semantic-equivalent cases were handled without relying on lexical overlap alone;
- stale-but-relevant material remained a relevance question rather than being conflated with freshness;
- the semantic approach reduced lexical-baseline wrong-target retention from 8 cases to the single v1 conflict-label finding.

## v2 requirements

The successor calibration must not rewrite this frozen v1 identity. It must:

1. clarify the model decision rule so `irrelevant` requires affirmative wrong-target/wrong-relation evidence, while unresolved applicability/binding is `ambiguous`;
2. replace the truth-conflict calibration shape with a relevance/binding-conflict shape under a new suite identity;
3. preserve strict Harness-owned identity floors and no-authority-promotion invariants;
4. preserve separation of relevance from freshness, authority, verification, contradiction, truth, and answer sufficiency;
5. keep the production motivating incident excluded from tuning;
6. rerun canonical Mistral/Google calibration only under a new freeze identity.

No independent holdout may be authored until the successor calibration reaches the frozen acceptance criteria.
