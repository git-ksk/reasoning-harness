# Engine 0.6 evidence relevance holdout successor v5

Status: successor-v5 semantics freeze candidate after immutable holdout v4 FAIL and completed two-provider development convergence.

## Versioning

- effective qualification: v7
- materialization: v20
- historical v1-v4 qualification/materialization semantics remain unchanged
- immutable v4 observations are replay-only regression evidence; historical artifacts are not rewritten

## Successor rules

1. Navigation-only target + stable repeated sibling owner:
   - strict Harness identity policy
   - deterministic scope risk none
   - target canonical name/alias appears only in navigation/footer, not in identity-capable local signals or canonical URL
   - a sibling-looking subject repeats across separate heading/title-like and body/fact-like signals
   - the subject shares identity-family vocabulary with the target and adds a distinguishing non-generic token
   - requested relation is locally present
   - v7 may establish distinct_target / requested_relation
   - v20 may materialize Irrelevant even when advisory stages incorrectly vote exact

2. Explicit local separation + stable repeated sibling owner:
   - deterministic scope risk none
   - local text explicitly states the repeated sibling is a separate/distinct service/product from the Harness target, or an equivalent explicit non-mapping relation
   - requested relation is locally present for that repeated sibling
   - v7 may establish distinct-target ownership across an exact-vs-distinct advisory disagreement
   - a single unscoped different name remains insufficient

3. Deterministic target/relation absence:
   - deterministic scope risk none
   - the bounded local unit explicitly establishes exact-target absence or target-specific requested-relation absence
   - v7 resolves to target_absent / relation_absent
   - with both advisory stages operationally present, v20 may materialize Irrelevant only when the local qualifier independently reports target or relation absence
   - an exact/exact positive conflict without qualifier-side absence does not become deterministic Irrelevant
   - this is local candidate rejection only; it does not assert global product absence

4. Context-gap relation floor:
   - visible clipping/truncation/omitted local context remains a typed context_gap
   - when advisory relation binding is unresolved, v7 forces the effective relation back to unresolved rather than treating missing context as relation_absent
   - terminal materialization remains Ambiguous

5. Existing positive and ambiguity floors remain:
   - repeated authorized positive identity from successor v4 is unchanged
   - a single near-sibling local signal remains Ambiguous when target identity is absent or appears only in non-owning URL/navigation context
   - URL-only unnamed ownership remains Ambiguous
   - URL/navigation context is never proposition ownership by itself
   - model output cannot create authority or bypass deterministic scope risk

No additional model stage is introduced.

## Provider-development protocol

To avoid Groq TPD becoming the iteration bottleneck, successor development is explicitly split from final cross-provider validation.

Development loop:
- required live providers: Mistral ministral-8b-latest + Google gemini-3.5-flash-lite
- Groq is excluded from candidate-shaping live iterations
- captured immutable Groq observations may be used only for offline replay/regression
- development fixtures/runs are never holdout evidence and cannot satisfy #462 acceptance
- repeated live development observations contaminate those fixtures for future holdout use

Candidate freeze:
- pass generic deterministic controls
- pass immutable v23 and holdout v1-v4 replay surfaces under successor semantics
- pass Mistral + Google development corpus with zero wrong-target Relevant and exact required materialization/authority gates
- pass workspace tests, Clippy, rustfmt, workflow/YAML checks and production special-case scan
- freeze successor semantics before authoring the next independent holdout

Final validation:
- author fresh independent holdout v5 only after semantics freeze
- required providers return to Mistral + Google + Groq
- Groq is used at this stage for independent cross-provider validation, not iterative tuning
- one canonical attempt only; any FAIL remains immutable
- Groq admission/readiness may be checked immediately before the canonical run, but no holdout observation is consumed early

## Current offline regression evidence

- generic successor-v5 controls: 8/8 PASS, including URL/navigation-only single-sibling ambiguity under conflicting advisory votes
- failed development run 36666145098 captured observations replay to 16/16 on Mistral + 16/16 on Google under the revised v7/v20 semantics; this remains development evidence, not holdout acceptance
- immutable v23 replay: 48/48 x 3
- immutable holdout v1 replay: 26/26 x 3
- immutable holdout v2 replay: 26/26 x 3
- immutable holdout v3a replay: 26/26 x 3
- immutable holdout v4 captured observations: 26/26 x 3 under v7/v20 successor semantics
- historical v1-v4 FAIL observations remain preserved rather than rewritten
- wrong-target Relevant under successor replay: 0

## Pre-freeze validation

- candidate commit before freeze-metadata commit: fbedfd6b8b7122273b3a4707c8165d758928127f
- two-provider development run: 36676852262
- Mistral ministral-8b-latest: 16/16 provider success, authority 16/16, materialization 16/16, utility miss 0, wrong-target Relevant 0, 28,298 tokens
- Google gemini-3.5-flash-lite: 16/16 provider success, authority 16/16, materialization 16/16, utility miss 0, wrong-target Relevant 0, 29,149 tokens
- development final-gate: PASS
- normal PR workflows on the exact candidate commit: 9/9 PASS
- core full suite: PASS
- providers: 153 passed / 1 ignored / 0 failed
- CLI: 205 passed / 3 ignored / 0 failed plus bin/integration suites PASS
- workspace all-target Clippy with -D warnings: PASS
- rustfmt / git diff check: PASS
- workflow YAML: 94/94 parse
- production special-case scan: clean
- immutable replay: v23 48/48 x3; holdout v1/v2/v3a/v4 26/26 x3
- failed development observation run 36666145098: recovered to Mistral 16/16 + Google 16/16 under revised v7/v20 replay
- successful development observation run 36676852262: captured as development-only regression evidence
- Groq was not invoked live anywhere in this development phase

Only after this evidence and the semantic-surface checksum are committed may a fresh independent holdout v5 be authored.
