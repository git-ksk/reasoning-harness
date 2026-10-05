# Engine 0.6 evidence relevance holdout v9

Status: corpus-freeze candidate. Successor-v9 semantics and the corrected V9 runner are frozen; a fresh 26-case holdout-v9 corpus is authored and validated offline. No provider observation has occurred.

## Frozen semantic input

- successor semantics tag: `engine-0.6-evidence-relevance-successor-v9-semantics-freeze`
- tag target: `3e49a9fd2f7827b7c707516d2150e2f0f16e9e17`
- effective qualification: v11
- materialization: v23
- historical v9/v22 predecessor functions remain frozen and unchanged

## Runner wiring

The dedicated binary is `reason-evidence-relevance-holdout-v9-study` from `crates/reasoning-harness-cli/src/bin/evidence_relevance_holdout_v9.rs`.

The V9 profile is wired before corpus authoring with:

- configuration: `evidence-relevance-live-holdout-v9`
- suite: `evidence-relevance-holdout-v9`
- annotation protocol: `evidence-relevance-effective-qualification-v11`
- fixed core: `evidence-relevance-fixed-core-v9`
- expected relative directory: `fixtures/evidence-relevance-holdout-v9`
- expected case count: 26
- checkpoint profile: holdout / scorable only when complete
- effective qualification: v11
- materialization: v23

The runner completeness test covers every declared profile including V9. No `fixtures/evidence-relevance-holdout-v9` directory exists at runner-freeze preparation time, so the fresh corpus cannot influence runner wiring.

## Runner freeze

The first runner freeze, `engine-0.6-evidence-relevance-holdout-v9-runner-freeze` / `17ea49bc6b332bb985971b661830c80665ad5dd4`, was found during pre-observation validate-only audit to have inherited the v8 `issue == 462` binding. The tag remains immutable evidence and is not moved or deleted. That v1 runner cannot accept the #468 corpus and is not an acceptance runner.

The corrected runner makes the issue binding profile-specific: V9 requires `issue == 468`, while historical V1-V8 and development profiles remain `issue == 462`. Runner-freeze-v2 is fixed at commit 4674ea9a923788835d0e9bf53bdd7747d6648af8 / tag `engine-0.6-evidence-relevance-holdout-v9-runner-freeze-v2` and was pushed after exact-head CI 8/8 PASS.

The fresh holdout-v9 corpus was authored only after that v2 tag existed. The corpus must be independently authored with zero overlap in case ID, canonical entity, task, exact signal, and exact 8-token candidate-signal n-grams against holdout v1-v8 and all successor-v5/v6/v7/v8/v9 development surfaces. Provider observation remains prohibited until the corpus itself is frozen.


## Fresh corpus

The V9 corpus was authored only after runner-freeze-v2 existed. The abandoned pre-v2 draft was deleted before the v2 freeze and was never committed, frozen, or observed.

The candidate has 26 cases: 8 Relevant / 10 Irrelevant / 8 Ambiguous. It covers all six coarse relation kinds and v11/v23-specific controls for semantic-frame positives, distinct-target same-relation negatives, surface lookalikes, explicit target/relation absence, prompt-injection inertness, identity mapping, URL ownership gaps, shared ownership, truncation, single-near-sibling ambiguity, cross-frame general-availability language, and exact-target numeric observations that do not establish a hard limit.

Independence is CI-enforced against holdout v1-v8 and successor-v5/v6/v7/v8/v9 development manifests. Required overlap is zero for case ID, canonical entity, task, exact signal, and exact 8-token candidate-signal n-grams.

Offline validation under frozen v11/v23 is exact for all 26 expected observations. Validate-only confirms issue 468, the V9 configuration/suite, annotation protocol v11, fixed core v9, 26 planned cases, 0 completed cases, and non-scorable validate-only status.

## Canonical provider gate

The one-shot acceptance surface restores three required providers: Mistral ministral-8b-latest, Google gemini-3.5-flash-lite, and Groq openai/gpt-oss-120b.

All arms must complete 26/26 with zero provider failures. Required correctness keeps wrong-target Relevant at 0. Required utility keeps false relevance rejection, Relevant-left-Ambiguous, and utility miss at 0. Required materialization is 26/26 exact. Required effective authority qualification is 26/26 exact with zero identity, relation, or scope-risk misses. Workflow reruns are rejected.

## Groq admission

The v9 admission anchor is recalculated from the actual Groq arm of immutable canonical holdout-v8 run 36946457925:

- observed tokens: 66,212
- Groq completion: 2026-10-02T02:44:50Z
- modeled starting headroom: 55,000 tokens
- inter-case delay: 300,000 ms
- minimum request interval: 10,000 ms
- modeled refill during the 26-case arm: approximately 17,962.96 tokens
- conservative modeled post-v8 headroom: approximately 6,750.96 tokens
- required v9 starting headroom: 55,000 tokens
- self-budget: 70,000 tokens
- conservative 55K-headroom floor: 2026-10-02T08:32:14Z

The floor has elapsed. Any known material intervening organization-level Groq usage invalidates this modeled anchor and requires re-anchoring before the canonical freeze tag is pushed.

## Corpus freeze

Intended corpus freeze coordinate: engine-0.6-evidence-relevance-holdout-v9-freeze.

Before that tag is pushed, the candidate must pass exact-head CI plus surface checksum verification, frozen-successor semantics verification, runner-freeze-v2 invariance, offline holdout tests, validate-only, Clippy, rustfmt, YAML parse, and diff checks.

Provider observation starts only from the corpus freeze tag.
