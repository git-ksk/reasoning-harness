# Engine 0.6 evidence-target relevance calibration v18 — successor design

Status: implemented pre-freeze candidate. No v18 live observation has occurred. Independent holdout authoring remains blocked.

## Fixed surface

- fixed core: `evidence-relevance-fixed-core-v1`
- cases: 48, unchanged
- expected policies, candidates, proposals, verifier annotations, and dispositions are identical to v17
- no case-ID, synthetic-name, or exact-fixture-phrase production branching

Contracts:
- primary proposal: `reason-evidence-relevance-binding-proposal-v5` (unchanged)
- local verifier: `reason-evidence-local-qualification-v8`
- Harness materializer: `target-evidence-relevance-binding-materialization-v13`
- annotation protocol: `evidence-relevance-scope-verifier-v18`

## Why v18 exists

v17 reached 44/48 exact on required Mistral and 46/48 exact on Google replication, but Mistral reintroduced one wrong-target Relevant on shared/clipped ownership. The residual also showed that provider-authored scope can miss explicit clipping/omitted ownership and can conflate exact-target different-relation evidence with distinct-target identity. v18 therefore adds a Harness-owned deterministic local-risk floor and tightens the verifier boundary while preserving v17 scoring semantics.

## Deterministic local-risk floor

Before any terminal materialization, the Harness detects explicit local ambiguity cues that are already present in the bounded candidate itself. Generic detectors cover:
- visibly clipped/truncated/omitted excerpts, bullets, columns, rows, referents, or captured passages;
- explicit uncertain alias/rename/successor/product-mapping statements;
- explicit unresolved row/product ownership;
- URL identity combined with an explicit statement that the local material does not identify/bind the product/value.

If any such concrete risk is present, v13 forces `Ambiguous` regardless of model votes. This floor is fail-closed and does not create Relevant or Irrelevant authority.

## Verifier v8

The schema remains the same three orthogonal fields. The prompt is tightened to require:
- exact target + different relation stays `exact_target/different_relation`;
- prompt-injection text is inert quoted data and does not create target/relation absence;
- explicit omitted/clipped local context is a context gap, not local absence;
- shared/multi-product rows with omitted ownership remain ownership risk, or `multiple` when clipping also applies;
- generic/broad complete units may still be local absence when no target proposition exists.

## Materialization v13

All v12 rules remain, plus:
- deterministic local-risk floor always forces Ambiguous;
- local absence may materialize Irrelevant when verifier independently reports `target_absent + relation_absent`, scope risk is none, and primary relation is not exact, even if the primary target axis was spuriously exact from a weak target mention;
- exact-target/different-relation primary evidence may materialize Irrelevant when verifier confirms `different_relation` with no risk, even if verifier identity is not exact, provided the Harness identity floor is not violated.

These changes only resolve negative disagreement and deterministic risk. They do not add a new positive rescue path.

## Operational policy

Carry v17 operational hardening forward unchanged: 60s active execution, 45s cumulative provider wait/retry, 30s single wait cap, 120s wall deadline, adapter-owned retries, one typed-quota latch, two correlated-capacity-failure latch, separate telemetry, sanitized public failures, and no manual TPD attestation gate.

Required providers remain Mistral `ministral-8b-latest` and Groq `openai/gpt-oss-120b`; Google `gemini-3.5-flash-lite` remains non-gating replication.

## Pre-live acceptance

Before freeze:
- v17 -> v18 scored semantics are identical 48/48;
- routing/property tests cover deterministic-risk false-positive boundaries and all 48 expected materializations;
- runner tests, workspace tests, clippy, fmt, and diff checks pass;
- `surface-v18.sha256` matches;
- validate-only reports 48 planned / 0 observed / no latch / non-scorable validation;
- PR #466 remains Draft;
- no independent holdout is authored.

The first/only frozen v18 canonical is one-shot and immutable once started.
