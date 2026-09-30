# Engine 0.6 evidence relevance authority redesign v1

Status: architecture redesign candidate after repeated independent-holdout utility failures.

## Why the v3-v6 / v16-v19 line stops here

The historical line successfully tightened safety: recent independent holdouts retained zero wrong-target Relevant outcomes. However, the effective-qualification and materialization chain now encodes correction rules as successive exceptions around advisory model votes.

The symptom is stable across independent surfaces: the Harness can usually detect that a candidate is unsafe to accept, but it does not have one coherent representation for who owns the proposition. That produces repeated conservative Irrelevant -> Ambiguous misses and occasional overconfident negative identity from weak local evidence.

Adding another fixture-shaped branch to v19 would increase semantic debt without fixing the abstraction.

The redesign therefore freezes v6/v19 as historical semantics and starts a new authority layer rather than a v7 patch chain.

## Core invariant: mention is not ownership

A target name may occur as:

- the subject that owns the proposition;
- a comparison/context object;
- navigation/footer text;
- URL routing metadata;
- an explicitly absent target;
- part of an uncertain alias/rename/successor mapping;
- one member of mixed/shared ownership.

Only the first case can create positive target ownership.

Likewise, a different-looking name does not automatically create negative ownership. A distinct subject requires structural corroboration.

## Pipeline

The new pipeline is:

1. Local feature extraction
   - classify target mention roles;
   - classify structural distinct-subject evidence;
   - classify relation evidence;
   - classify blocking risk.
2. Harness-owned authority resolution
   - resolve identity ownership;
   - resolve relation coverage;
   - attach authority source.
3. Materialization table
   - map resolved authority to Relevant / Irrelevant / Ambiguous.
4. Model advice
   - proposal/verifier outputs can corroborate unresolved axes;
   - model votes never directly materialize a final disposition.

This replaces recursive “effective qualification vN delegates to vN-1 then overrides one case family” composition.

## Authority types

Identity authority:

- exact_target
- distinct_target
- target_absent
- unresolved

Relation authority:

- requested_relation
- different_relation
- relation_absent
- unresolved

Authority source:

- harness_deterministic
- harness_corroborated
- model_consensus
- unresolved

Blocking risk remains orthogonal:

- none
- identity_mapping
- ownership_scope
- context_gap
- multiple

The resolved object keeps identity, relation, risk, and the source of each axis separately. This makes precedence auditable and prevents one axis from laundering another.

## Identity precedence

The resolver is fail-closed.

1. A blocking mapping/ownership/context risk prevents terminal authority and therefore final materialization is Ambiguous.
2. Explicit target-absence evidence corroborated by the local verifier yields target_absent.
3. A direct target-owned subject occurrence yields exact_target only when it is not context-only, navigation-only, URL-only, or absence-only.
4. An authorized canonical/alias subject repeated across independent local signals may recover exact_target even when advisory identity votes disagree.
5. Explicit separation or context-only target mention plus a stable repeated non-target subject yields distinct_target.
6. A stable repeated sibling/non-target subject may yield distinct_target; a single-signal near sibling cannot.
7. Under allow_semantic_equivalent, model consensus may establish exact_target when no Harness-owned contradiction or blocking risk exists.
8. Otherwise identity is unresolved.

If positive and negative ownership proofs of comparable authority coexist, resolve to unresolved, not by arbitrary precedence.

## Relation precedence

Relation is resolved independently from identity.

1. Blocking risk prevents terminal materialization.
2. Explicit local exclusion of the requested relation plus corroborating model evidence yields different_relation.
3. Requested relation present in substantive local content plus model support yields requested_relation.
4. Explicit local relation absence plus corroboration yields relation_absent.
5. Otherwise relation is unresolved.

Distinct/absent identity can make a candidate Irrelevant without requiring a terminal relation axis. Exact-target Irrelevant requires a terminal different/absent relation.

## Materialization table

With risk = none:

| Identity | Relation | Final |
|---|---|---|
| exact_target | requested_relation | Relevant |
| exact_target | different_relation | Irrelevant |
| exact_target | relation_absent | Irrelevant |
| distinct_target | any | Irrelevant |
| target_absent | any | Irrelevant |
| unresolved | any | Ambiguous |
| exact_target | unresolved | Ambiguous |

Any blocking risk => Ambiguous.

No proposal target/relation vote appears in this table.

## Structural evidence requirements

### Positive ownership

Harness-owned exact identity requires a canonical or authorized alias occurrence in an identity-capable local signal. Context-only, navigation/footer, and URL-only occurrences do not qualify.

A repeated authorized identity across separate title/heading/metadata and excerpt/fact signals is stronger corroboration and may override advisory identity disagreement.

### Negative ownership

Negative ownership may be established from:

- explicit “separate/distinct service/product” structure;
- target mention only in comparison/context plus a stable repeated substantive subject;
- a stable repeated sibling/non-target subject across independent local signals;
- explicit target absence.

Mere absence of a target anchor is never negative authority.

Single-signal near-name evidence is never enough for distinct_target.

### Conflict handling

Mixed multi-product material, omitted owner columns, uncertain mappings, and truncated referents remain Ambiguous even if one advisory model is confident.

## Model role

The current binding proposal and local qualifier remain usable as advisory inputs during the first redesign iteration so immutable observations can be replayed.

Their role changes:

- they may corroborate a Harness-visible structural claim;
- they may fill semantic-equivalent identity where policy explicitly permits it;
- they may help resolve relation coverage;
- they cannot directly override a stronger Harness proof;
- two model votes are not equivalent to Harness ownership proof.

A future span-backed model contract may replace coarse advisory fields, but it is not required for authority-v1.

## Required regression evidence before any new semantic freeze

- generic property tests for each authority source and every conflict boundary;
- immutable v23 replay;
- immutable holdout v1 replay;
- immutable holdout v2 replay;
- immutable holdout v3a replay;
- immutable holdout v4 provider observations once the one-shot v4 run completes;
- zero wrong-target Relevant across every replay;
- no increase in false relevance rejection;
- full core/providers/CLI, all-target Clippy, fmt, YAML and special-case scans;
- no provider name, fixture ID, synthetic entity, or exact holdout string in production branching.

The frozen holdout v4 remains immutable FAIL even if authority-v1 replays it perfectly.

Only after authority-v1 semantics freeze may a new independently authored holdout be created.

## Current prototype evidence

The first authority-v1 implementation is isolated in a new core module rather than appended to evidence_relevance.rs.

Prototype API:

- EvidenceLocalAuthorityV1
- EvidenceAuthoritySource
- resolve_evidence_local_authority_v1
- materialize_evidence_relevance_v20

Validation on the current redesign branch:

- generic authority/property controls: 6/6 PASS
- immutable v23 provider-observation replay: 48/48 x 3
- immutable holdout v1 provider-observation replay: 26/26 x 3
- immutable holdout v2 provider-observation replay: 26/26 x 3
- immutable holdout v3a provider-observation replay: 26/26 x 3
- completed v4 Mistral artifact replay under authority-v1: 26/26
- completed v4 Google artifact replay under authority-v1: 26/26
- core full suite: 597 passed / 0 failed
- providers: 153 passed / 1 ignored / 0 failed
- CLI: 368 passed / 4 ignored / 0 failed
- workspace all-target Clippy with -D warnings: PASS
- rustfmt / diff check: PASS
- production special-case scan: clean

The v4 Mistral/Google replay is postmortem evidence only. The frozen v4 result is not rescored or converted to PASS. Groq v4 must complete before the v4 result record and any authority-v1 semantic freeze are finalized.

## Design lessons from the prototype

The replay work exposed four boundaries that are now explicit authority primitives instead of case-specific patches:

1. model-only blocking risk cannot automatically override stronger Harness-visible structure;
2. generic/local target absence is distinct from target-specific requested-relation absence;
3. semantic-equivalent policy cannot treat a repeated paraphrased subject as negative ownership merely because it lacks the canonical name;
4. a longer distinct entity containing the target tokens does not become exact-target ownership when explicit structural distinctness is present.

These are properties of authority composition, not fixture exceptions.
