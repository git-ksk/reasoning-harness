# Engine 0.6 evidence relevance independent holdout v2

Status: freeze candidate / unobserved. This surface evaluates the already-frozen Issue #462 successor semantics (effective qualification v4 + materialization v17). The semantic freeze is `engine-0.6-evidence-relevance-successor-v2-semantics-freeze` at `55c4163acb815bc31db22da531766a26020091bf`.

## Independence boundary

Holdout v2 was authored only after the successor semantic freeze. Its cases, entity names, candidate text, and labels are new. It does not rewrite, relabel, rerun, or convert independent holdout v1, whose canonical run `36495389012` remains immutable FAIL.

The failed v1 case `h14_negative_comparison_only_mention` is not copied or superficially paraphrased into this surface. Holdout v2 contains no `Umber DB` or `Violet DB` identity, and no holdout-v1 case ID. Before freeze, the candidate text also has zero exact shared 8-token n-grams with holdout v1 and zero canonical-entity overlap.

Observed holdout-v1 outputs may be used only for postmortem/replay of the frozen successor semantics. They are not a source of new holdout-v2 labels or production branches.

## Frozen candidate contract

- suite: `evidence-relevance-holdout-v2`
- cases: 26
- label distribution: Relevant 8 / Irrelevant 10 / Ambiguous 8
- required providers:
  - Mistral `ministral-8b-latest`
  - Groq `openai/gpt-oss-120b`
  - Google `gemini-3.5-flash-lite`
- annotation protocol: `evidence-relevance-effective-qualification-v4`
- materialization: v17
- fixed core: `evidence-relevance-fixed-core-v2`
- seed: `4629202`
- intended freeze tag: `engine-0.6-evidence-relevance-holdout-v2-freeze`
- canonical policy: first tag-triggered run, attempt 1 only; no canonical rerun or replacement observation

The runner remains backward-compatible with frozen holdout v1: v1 continues to derive effective qualification v3 and materialize with v16. Selecting v2 switches only the explicitly versioned study profile to v4/v17.

## Acceptance gates

All three required provider arms must complete 26/26 operationally. Each provider independently must satisfy:

- wrong-target Relevant retention = 0;
- false relevance rejection = 0;
- Relevant left Ambiguous = 0;
- utility misses = 0;
- materialized exact matches = 26/26;
- effective authority qualification exact matches = 26/26;
- effective identity/relation authority misses = 0;
- scope-risk miss/spurious-risk counters = 0;
- no provider-arm latch, operational abort, or incomplete provider attempt.

Operational provider failure stays operational and cannot be converted into a semantic pass or miss. Holdout v2 does not weaken the zero-wrong-target hard gate that holdout v1 failed.

## Groq TPD admission

Tiny readiness probes are transport/credential/TPM/RPD evidence only and are not TPD-headroom proof.

The v2 workflow re-anchors its TPD model from the immutable holdout-v1 Groq observation:

- actual observed tokens: 65,244;
- observation completion anchor: `2026-09-29T01:10:50Z` (canonical job log line containing the final total-token result);
- v1 configured start headroom: 55,000;
- v1 frozen pacing: 300,000 ms between cases plus 10,000 ms minimum request interval;
- conservative modeled v1 end headroom: about 7,719 tokens;
- v2 required modeled start headroom: 55,000;
- v2 26-case self-budget: 70,000 tokens;
- v2 reserve before another case: 4,000 tokens;
- v2 pacing: 300,000 ms between cases plus 10,000 ms minimum request interval;
- earliest precommitted modeled floor: `2026-09-29T06:51:16Z` (15:51:16 JST).

The workflow recomputes that floor and fails closed if the configured timestamp is earlier than the derived timestamp or if paced supply is insufficient for the 70K self-budget. Any known or suspected material organization-level Groq usage invalidates this model and requires re-anchoring before the freeze tag is pushed.

## Freeze discipline

Before pushing the holdout-v2 freeze tag:

1. validate v1 and v2 study profiles locally;
2. replay immutable calibration v23 and holdout-v1 observations under frozen v4/v17 semantics;
3. run full core/providers/CLI tests, workspace all-target Clippy, fmt, diff and workflow-YAML checks;
4. revalidate the exact surface checksum;
5. scan the public diff for secrets, local paths, provider/entity-specific production branching, and accidental holdout-v1 surface reuse;
6. require exact-head PR CI to be green while PR #466 remains Draft;
7. report the pre-freeze position before starting the long live observation.

Issue #462 remains open unless this fresh independent holdout passes its original acceptance gates.
