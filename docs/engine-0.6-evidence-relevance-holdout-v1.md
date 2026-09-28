# Engine 0.6 evidence relevance independent holdout v1

Status: pre-freeze, unobserved. No holdout freeze tag or live observation exists.

## Independence boundary

Calibration v23 passed its first/only canonical run `36400085595` on commit `0871169303d20c464b0454dbe0a150d2fef0ec43` before this holdout corpus was authored.

Holdout identity:

- suite: `evidence-relevance-holdout-v1`
- issue: #462
- cases: 26
- corpus: `fixtures/evidence-relevance-holdout-v1/manifest.json`
- planned freeze tag: `engine-0.6-evidence-relevance-holdout-v1-freeze`
- workflow: `.github/workflows/engine-0.6-evidence-relevance-holdout-v1-live.yml`
- seed: `4629101`
- required providers: Mistral `ministral-8b-latest`, Groq `openai/gpt-oss-120b`, Google `gemini-3.5-flash-lite`
- annotation protocol: `evidence-relevance-effective-qualification-v23`
- materialization: v16

The holdout uses new IDs, tasks, target questions, canonical entity names, and signal texts. Exact reuse against the 48-case v23 calibration corpus is rejected by the holdout test surface before any live observation.

## Coverage

The 26 independently authored cases contain 8 Relevant, 10 Irrelevant, and 8 Ambiguous expectations. Coverage includes:

- exact target availability;
- Harness-authorized alias binding;
- semantic-equivalent positive relevance;
- distributed title/body change evidence;
- Japanese task with English evidence;
- structured metadata identity;
- multi-section benefit evidence;
- relevant evidence whose URL omits the target name;
- same target / wrong relation;
- sibling target / same relation;
- navigation-only target mention;
- generic landing page with no target-local support;
- unrelated announcement;
- comparison-only target mention;
- prompt injection attempting to self-declare relevance;
- explicitly distinct structured sibling;
- sibling target plus different relation, exercising the v23 composition boundary;
- possible rename, alias, and successor ambiguity;
- partial/multi-product identity ambiguity;
- truncated relation context;
- URL-only identity;
- shared-row ownership ambiguity.

The production motivating content is not included.

## Frozen semantics and gates

The holdout does not introduce a new production semantic rule. It executes effective qualification v3 plus materialization v16 exactly as accepted by calibration v23.

All three provider arms are required. A PASS requires every arm to be operationally complete over 26/26 cases and to satisfy:

- wrong-target relevance retention = 0;
- false relevance rejection = 0;
- Relevant left Ambiguous = 0;
- utility misses = 0;
- materialized exact = 26/26;
- authority-qualified effective qualification = 26/26;
- effective qualification risk misses/spurious risks = 0;
- authority identity/relation misses = 0.

Provider failures remain separate from semantic failures but make a required arm incomplete. The holdout is first/only: reruns are rejected. Any failure is immutable evidence and requires a new successor identity rather than tuning/rescoring v1.

## Groq admission plan

Calibration v23 used 120,075 Groq tokens. To avoid immediately reusing the same TPD headroom, the holdout uses a conservative lower-bound model that assumes only the configured 105K v23 start headroom and the frozen minimum v23 pacing refill, ignoring the extra refill from v23 starting later than its floor and ignoring refill during model execution.

That yields a conservative modeled post-v23 headroom of about 8.9K tokens at `2026-09-28T11:50:54Z`.

Holdout v1 therefore uses:

- modeled Groq start headroom: 55K;
- earliest modeled floor: `2026-09-28T17:22:57Z` (`2026-09-29 02:22:57 JST`);
- 26 cases;
- inter-case delay: 300 seconds;
- provider request spacing: 10 seconds;
- minimum deliberate pacing: 7,760 seconds;
- modeled pacing refill: about 17.96K tokens;
- modeled supply: about 72.96K tokens;
- observed-token hard cap: 70K;
- pre-case reserve: 4K;
- projected demand from v23 per-case usage: about 65.0K tokens.

Known or suspected material organization-level Groq usage invalidates the model and requires delaying or re-anchoring before freeze. A fresh synthetic readiness check is required immediately before freeze, but remains transport/credential/TPM/RPD evidence only and is not TPD-headroom proof.

## Freeze rule

Before the first/only holdout observation:

1. exact candidate is clean and standard CI is green;
2. holdout checksum passes;
3. validate-only reports 26 planned / 0 observed;
4. exact calibration-content reuse checks pass;
5. v23 regression tests and holdout-runner tests pass;
6. the modeled Groq floor has elapsed;
7. fresh Groq readiness passes on the exact candidate;
8. create the annotated holdout freeze tag once.

After freeze, corpus text, expected labels, scorer, thresholds, provider roles, prompt contract, and production semantics are immutable for this holdout identity.
