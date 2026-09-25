# Engine 0.6 candidate: evidence-target relevance calibration v14

Status: pre-freeze successor to immutable v13 FAIL. No v14 live model observation has occurred. Independent holdout authoring remains prohibited.

## Why v14 exists

v13 changed local qualification from open-world three-state risks to binary observable cues, but the canonical run still failed.

Required Mistral completed 81/81 with no provider failures but materialized only 61/81 exactly. It recorded 7 blocking-cue misses, 5 spurious cue blocks, 1 wrong-target/false Relevant, 2 false relevance rejections, 6 Relevant -> Ambiguous outcomes, and 19 utility misses.

Google replication completed 81/81 and materialized 65/81, with 3 blocking-cue misses, 4 spurious cue blocks, 3 Relevant -> Ambiguous outcomes, and 16 utility misses.

The required Groq canonical arm stopped after 3/81 because strict-JSON Text local qualification was truncated at the 192-token completion budget. A separate noncanonical full-case postmortem is running with a 512-token Groq transport floor and no consecutive-failure circuit; it cannot change the v13 result and must complete before v14 is frozen.

## Successor hypothesis

The v13 guard still did too much. It independently reclassified target support, relation support, three blocking cues, and explicit local absence.

Those support fields duplicated the primary target/relation proposal and frequently disagreed with it for reasons unrelated to relevance, including freshness, relation mismatch, and identity wording.

A counterfactual replay over the fixed 48-case successor core used the already-observed v13 proposals/cues but removed `target_support` and `relation_support` from materialization. No new model calls were made.

Results:
- Mistral: 32/48 -> 36/48 exact, with no previously-correct case becoming incorrect.
- Google: 37/48 -> 40/48 exact, with no previously-correct case becoming incorrect.

This does not prove v14 will pass. It is evidence that redundant support votes were a real source of disagreement and motivates reducing the guard contract rather than adding more fields or more cases.

## Fixed calibration core

v14 uses `evidence-relevance-fixed-core-v1`.

The corpus is fixed at 48 cases:
- Relevant: 14
- Irrelevant: 18
- Ambiguous: 16

Historical v13 cases were compacted by semantic coverage before any v14 live observation. The core is not expanded when a model misses a case.

The compact v14 blocker labels have isolated coverage:
- `none`: 32
- `identity_mapping`: 5
- `ownership_scope`: 1
- `context_gap`: 5
- `multiple`: 5

Four cases contain explicit local absence.

The older v13 cue representation over the same fixed core covers identity-mapping cue 6, ownership-scope cue 5, context-gap cue 10, and explicit local absence 4.

Production motivating content remains excluded.

## Binding proposal v4

The primary model still returns:
- `target_binding`: `exact | different | unresolved`
- `relation_binding`: `exact | different | unresolved`

The axes remain independent.

v4 clarifies two previously unstable boundaries.

First, `identity_requirement=allow_semantic_equivalent` means a locally specific semantic description can be `target_binding=exact` even if the canonical product name is absent. This is required for semantic paraphrase controls.

Second, explicit local identity uncertainty must remain `unresolved`. Text such as “may be an alias,” “does not establish whether X succeeds Y,” or “does not state whether X replaces Y” must not be hardened into `different`.

Relation binding remains a coarse relation-kind classification independent of target owner. Sibling pricing is still relation `exact` for a pricing question. Generic landing copy, navigation-only target mentions, explicit local absence, and visibly missing relation content are `unresolved`, not `different`.

Freshness, truth disagreement, authority, sufficiency, and untrusted page instructions do not change binding labels.

## Compact local guard v4

The independent guard is reduced to two fields:

- `blocking_reason`: `none | identity_mapping | ownership_scope | context_gap | multiple`
- `explicit_local_absence`: `present | absent | unresolved`

It no longer votes on target support or relation support.

`blocking_reason` is only for concrete local ambiguity that must preserve Ambiguous:
- `identity_mapping`: uncertain/conflicting alias, rename, successor, version, or cross-language identity mapping;
- `ownership_scope`: shared/unassigned substantive row/value/section ownership;
- `context_gap`: supplied material is visibly clipped, truncated, has an omitted referent, or explicitly says required local context is outside the supplied material;
- `multiple`: at least two of the above;
- `none`: no such concrete ambiguity.

Ordinary wrong-target evidence, wrong-relation evidence, generic landing pages, navigation/footer text, archived/stale values, factual disagreement, explicit `distinct_from` evidence, and explicit target absence are not blockers by themselves.

Explicit local absence is negative evidence, not a context-gap blocker.

## Materialization v9

Harness final authority remains fail-closed.

Ambiguous:
- any non-`none` blocking reason;
- CanonicalUrl-only identity floor;
- missing primary proposal;
- missing compact guard;
- strict required identity anchor missing on an otherwise positive exact/exact proposal;
- remaining unresolved binding without explicit local absence.

Relevant:
- primary exact/exact;
- compact blocker `none`;
- strict Harness-owned anchor when required, or policy explicitly permits semantic equivalent.

Irrelevant:
- primary target `different` and blocker `none`; or
- primary target `exact` + relation `different` and blocker `none`; or
- primary target non-exact + explicit local absence `present` + blocker `none`.

No model field can override the Harness-owned strict identity floor.

## Provider transport and diagnostic completeness

Mistral / Google:
- JsonSchema primary;
- one bounded strict raw-JSON Text transport fallback on structured capability/parse failure.

Groq:
- strict raw-JSON Text primary;
- no preceding JsonSchema request;
- transport completion floor 512 tokens for both semantic stages;
- malformed Text remains a typed protocol failure and is not semantically repaired.

All providers:
- no JSON extraction;
- no Markdown repair;
- no field synthesis;
- no fuzzy repair;
- no semantic retry;
- no third model call;
- shared 60-second per-case deadline.

v14 canonical runs use `--continue-after-operational-failures` so every fixed-core case is attempted for diagnostic completeness. This does not relax acceptance: any provider failure makes required operational completeness false.

## Canonical roles and acceptance

Required:
- Mistral `ministral-8b-latest`
- Groq `openai/gpt-oss-120b`

Full non-gating replication:
- Google `gemini-3.5-flash-lite`

Each required arm must independently satisfy:
- 48/48 completed cases;
- 48 successful provider cases;
- provider failures 0;
- provider-attempt telemetry complete;
- compact guard invoked 48/48;
- blocking-reason misses 0;
- wrong-target / false Relevant retention 0;
- false relevance rejections 0;
- expected Relevant left Ambiguous 0;
- utility misses 0;
- materialized exact 48/48.

Primary proposal exactness, full compact-guard exactness, and spurious blocker counts remain diagnostic. User-visible final disposition exactness and the hard safety counters remain authoritative.

The first/only frozen v14 canonical run is never rerun or rescored. Only canonical PASS permits authoring a fresh independent holdout. PR #466 remains Draft through canonical, holdout, and runtime acceptance.
