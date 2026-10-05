# Engine 0.6 evidence relevance calibration v22 result

Status: immutable canonical PASS.

- Freeze tag: `engine-0.6-evidence-relevance-calibration-v22-freeze`
- Candidate commit: `2e0268dc0fb565dab35d591e831aa5170ad75601`
- Canonical run: `36338187291`, attempt 1 only
- Cases: 48
- Seed: `4626210`
- Required providers: Mistral + Groq
- Replication provider: Google
- Annotation protocol: `evidence-relevance-effective-qualification-v21`
- Materialization: v16

## Required Mistral

`ministral-8b-latest` completed 48/48 with no operational abort or provider-arm latch.

- authority-qualified effective qualification: 48/48
- materialized disposition: 48/48
- wrong-target relevance retention: 0
- false relevance rejection: 0
- Relevant left Ambiguous: 0
- utility misses: 0
- provider attempts: 96/96 complete

## Required Groq

`openai/gpt-oss-120b` completed 48/48 with no operational abort, quota latch, or retry wait.

- authority-qualified effective qualification: 48/48
- materialized disposition: 48/48
- wrong-target relevance retention: 0
- false relevance rejection: 0
- Relevant left Ambiguous: 0
- utility misses: 0
- provider attempts: 96/96 complete
- observed total tokens: 120,426

The deliberately paced TPD plan therefore succeeded operationally. v22 remains immutable and must not be rerun.

## Google replication

`gemini-3.5-flash-lite` completed 48/48 and materialized 48/48 with zero correctness or utility misses. Authority-qualified effective qualification was 47/48.

The only authority miss was `76_v13_sibling_different_relation_no_cue`. The model's raw local qualification was already correct (`distinct_target` / `different_relation` / `none`), but its proposal said `target=different` / `relation=exact`. Effective qualification v2 preferred that proposal relation and changed the otherwise-correct raw relation to `requested_relation`. The target-negative terminal still materialized the correct final `irrelevant` disposition.

This is therefore a Harness-owned effective-qualification composition defect, not evidence that Google failed the final relevance decision.

## Final gate

The frozen v22 required-provider contract passed every top-level gate:

- required operational completeness: PASS
- required correctness: PASS
- required utility: PASS
- required materialization: PASS
- required qualification: PASS

v22 is accepted under its precommitted Mistral + Groq required-provider definition.

## Successor

The provider requirement is strengthened for v23: Mistral, Groq, and Google are all required. v23 keeps the same 48 frozen cases and materialization v16, and changes only the effective-qualification composition rule needed to preserve corroborated `distinct_target` + `different_relation` evidence when a conflicting proposal relation is not locally supported. The strict authority gate is retained; it is not weakened to make Google pass.
