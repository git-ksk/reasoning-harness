# Engine 0.6 evidence relevance independent holdout v3

Status: pre-freeze / unobserved.

Successor v3 semantics were frozen first at `engine-0.6-evidence-relevance-successor-v3-semantics-freeze` / `8c1f02c181e2a7c4e267d10ce610e0517162004d`. This holdout was authored only after that freeze and evaluates effective qualification v5 + materialization v18 without modifying the frozen successor.

## Independence boundary

Holdout v3 is a new 26-case surface with new case IDs, entity names, candidate text, and labels. It does not rewrite or relabel either immutable predecessor:

- holdout v1: canonical run `36495389012`, immutable FAIL;
- holdout v2: canonical run `36533340582`, immutable FAIL.

Before freeze, holdout v3 must have zero case-ID overlap, zero canonical-entity overlap, and zero exact 8-token candidate-signal overlap with both v1 and v2. It must not contain the failed v1/v2 entity names or failed case IDs.

The v3 surface includes both a fresh repeated-sibling negative and explicit abstention controls such as single-signal near-sibling evidence, URL-only identity, mapping uncertainty, shared ownership, and truncated context. Those cases were independently labeled from the annotation contract, not from observed provider output.

## Frozen candidate contract

- suite: `evidence-relevance-holdout-v3`
- cases: 26
- distribution: Relevant 8 / Irrelevant 10 / Ambiguous 8
- required providers:
  - Mistral `ministral-8b-latest`
  - Groq `openai/gpt-oss-120b`
  - Google `gemini-3.5-flash-lite`
- annotation protocol: `evidence-relevance-effective-qualification-v5`
- materialization: v18
- fixed core: `evidence-relevance-fixed-core-v3`
- seed: `4629303`
- intended freeze tag: `engine-0.6-evidence-relevance-holdout-v3-freeze`
- canonical policy: first tag-triggered run, attempt 1 only; no rerun or replacement canonical observation

The runner remains backward-compatible: v1 selects v3/v16, v2 selects v4/v17, and v3 selects v5/v18.

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

Operational failure remains operational and cannot be converted into semantic PASS/FAIL evidence.

## Groq TPD admission

The v3 workflow re-anchors its conservative TPD model from immutable holdout-v2 actuals:

- v2 observed tokens: 65,555;
- v2 completion anchor: `2026-09-29T09:05:28Z`;
- v2 configured start headroom: 55,000;
- v2 frozen pacing: 300,000 ms between cases + 10,000 ms minimum request interval;
- modeled post-v2 headroom: 7,407.96 tokens;
- v3 required start headroom: 55,000;
- v3 self-budget: 70,000 tokens;
- pre-case reserve: 4,000 tokens;
- v3 pacing: 300,000 ms between cases + 10,000 ms minimum request interval;
- earliest modeled floor: `2026-09-29T14:48:08Z` / `2026-09-29 23:48:08 JST`.

Tiny readiness probes remain transport/credential/TPM/RPD evidence only and do not prove TPD headroom. Any known or suspected material organization-level Groq usage after the v2 anchor invalidates this model and requires re-anchoring before the freeze tag is pushed.

## Freeze discipline

Before pushing the holdout-v3 freeze tag:

1. validate v1/v2/v3 study profiles;
2. rerun only deterministic replay/regression tests over immutable recorded observations, never the canonical live v1/v2 observations;
3. run CLI/core focused tests, workspace Clippy, fmt, diff, workflow YAML, checksum, independence and public-safety scans;
4. require exact-head PR CI to be green while PR #466 remains Draft and Issue #462 remains open;
5. verify the Groq floor has been reached and the TPD anchor is still valid;
6. report the pre-freeze position before starting the one-shot live observation.

If v3 fails, record it immutably and begin a separately versioned successor. Do not tune v5/v18 against the same frozen v3 surface.
