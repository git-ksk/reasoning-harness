# Engine 0.6 candidate: evidence-target relevance calibration v12

Status: pre-freeze successor to immutable v11 canonical evidence. Historical v1-v11 fixtures, tags, observations, and scores remain immutable. Independent holdout authoring is prohibited until the first/only frozen v12 canonical calibration passes.

## Why v12 exists

v11 established the intended ownership boundary: every case invokes an independent local qualification guard and the Harness alone materializes Relevant/Irrelevant/Ambiguous. The required Mistral arm exposed a contract mismatch: the guard was told to mark a risk unresolved whenever that risk could not be globally ruled out from the supplied material. Ordinary open-world uncertainty therefore became a local blocker and over-produced Ambiguous outcomes.

The mismatch was especially visible for Harness-owned aliases. The fixture correctly treated an alias already supplied by Harness policy as an authoritative local identity anchor, while the prompt simultaneously treated stated or plausibly-open alias equivalence as identity risk. v12 resolves that semantic contradiction without weakening the fail-closed materializer.

v11 also exposed an independent Groq transport failure mode: server-side structured generation repeatedly returned schema-generation failures and consumed the 60-second case budget. v12 keeps JsonSchema as the first call but changes the single bounded fallback to strict raw-JSON Text. The complete response must still parse directly into the typed contract. Extraction, fuzzy repair, semantic retry, and a third model call remain forbidden.

## Local qualification v2

The six fields remain unchanged: target_support, relation_support, identity_mapping_risk, ownership_scope_risk, context_completeness_risk, and explicit_local_absence.

The three risk fields now detect concrete local ambiguity signals rather than requiring proof that every hypothetical external risk is impossible.

- present: the supplied candidate contains a concrete local cue establishing the risk.
- unresolved: a specific risk-relevant cue exists, but the supplied local material cannot resolve it.
- absent: no concrete local trigger for that risk is present.

unresolved must not be emitted merely because unknown external facts might exist or because the candidate does not explicitly prove a risk impossible.

Harness-owned canonical names and aliases in the target policy are authoritative local identity anchors. Consistent use of such an alias does not create identity-mapping risk. Explicit possible rename/successor language, conflicting identity cues, shared unlabeled ownership, and visibly clipped/truncated/omitted context remain fail-closed.

Factual disagreement, staleness, source authority, verification, and answer sufficiency remain downstream concerns.

## Materialization boundary

Materialization policy v7 is intentionally unchanged. Hard Relevant still requires primary target/relation Exact, guard target/relation Supported, all three risks Absent, and the strict Harness-owned identity floor when required. Hard Irrelevant still requires independent risk-free negative agreement. Any Present or Unresolved qualification risk still materializes Ambiguous. Canonical-URL-only identity remains a hard Ambiguous floor.

v12 changes what constitutes a local risk signal, not what the Harness does once a risk exists.

## Transport

For both primary binding and local qualification:

1. first call: JsonSchema;
2. on structured transport failure or malformed response: one strict raw-JSON Text fallback embedding the same schema and preserving task/system/budget/seed semantics;
3. the entire fallback body must directly parse into the typed contract.

No JSON extraction, Markdown stripping, field synthesis, fuzzy repair, semantic retry, or third call is permitted. Primary and qualification continue to share the same 60,000 ms case deadline. The two-consecutive operational failure circuit remains.
For the Groq canonical arm, the adapter-internal structured-output retry limit is precommitted to zero so a server-side structured-generation failure surfaces as typed UnsupportedCapability after one provider attempt and the separate strict-JSON Text fallback can run within the case budget. The minimum Groq request interval is fixed at 10,000 ms against the 8k TPM constraint, covering schema-failure responses that may not provide usage telemetry. These are provider-specific operational pacing controls and do not change the semantic contract.

## Calibration corpus

v12 contains 73 synthetic calibration cases: all 65 v11 cases plus 8 fresh cases authored before any v12 live observation. The fresh set pairs no-risk and concrete-risk shapes: Harness-owned alias, exact canonical identity, explicit possible rename, single-owner structured value, shared unlabeled ownership, complete local context, clipped/omitted referent, and a clearly different target.

Expected final dispositions: 23 Relevant / 25 Irrelevant / 25 Ambiguous. Twenty-five cases carry at least one expected fail-closed qualification risk. Production motivating product content remains excluded.

## Canonical provider roles and acceptance gate

Required:
- Mistral ministral-8b-latest
- Groq openai/gpt-oss-120b

Full non-gating replication:
- Google gemini-3.5-flash-lite

Each required arm must independently satisfy 73/73 operational completion, provider failures 0, complete attempt telemetry, local qualification expected/invoked 73/73, qualification risk misses 0, wrong-target relevance retention 0, false relevance rejection 0, expected Relevant left ambiguous 0, utility misses 0, and materialized exact 73/73.

Primary proposal exactness, full qualification-field exactness, and spurious risk blocks remain diagnostic. The acceptance boundary remains final disposition plus missed safety risk.

The first/only frozen v12 canonical run is never rerun or rescored. Only canonical PASS permits authoring a fresh independent holdout. Runtime integration acceptance follows only after holdout PASS. PR #466 remains Draft through all stages.
