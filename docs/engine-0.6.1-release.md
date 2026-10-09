# Harness Engine 0.6.1 source release

## Release coordinates

- Source package: reasoning-harness-core 0.6.1
- Independent source tag: engine-v0.6.1 (only after acceptance)
- Release issue: #475, milestone #9
- Fixes/mitigations: #474, #476, #477, #479
- Published Reason CLI 0.5.3 continues to bundle Engine 0.5.0. This is not a CLI update.

## Safety changes

Positive ChangeOrLaunch evidence requires a target-owned affirmative predicate. The latest v17/v30 qualification rejects unrelated subject launches, conditional statements, denials, questions, fiction, and unverified claims. Advisory model output cannot override this Harness-owned authority floor.

Historical versioned relation semantics (including v11/v23 and their frozen v10/v12 replays) remain unchanged.

## Provider mitigation and residual risk

The historical mcp_readonly_v1 adapter still has a potentially unbounded stdin write and cannot be patched without breaking its frozen source invariant. Its supported v2/v3 successors apply whole-invocation subprocess deadlines. The v2 blocked-stdin regression passes; v1 users must migrate. This source release does not claim that v1 itself is repaired.

## Independent acceptance

- PR #480 exact-head CI: all PASS.
- First precommitted independent v1 corpus: FAIL on 3 cases, preserved as a historical failure; no rewrite or rescore.
- New independently precommitted v2 corpus: 30/30 deterministic PASS (12 positive, 18 negative); SHA-256 dea6536caf02ba015b1beeb1b7a92989fb61143639bb10e6e8192cecb57de3a3.
- New Mistral / Google / Groq live matrix (Actions run 37935245178, attempt 1): **PASS**. All three model arms completed 12/12; aggregate 36/36, false positive Relevant 0, positive utility misses 0, provider failures 0. Models: ministral-8b-latest, gemini-3.5-flash-lite, openai/gpt-oss-120b.
- Exact release-candidate workspace tests, Clippy, fmt, platform/installation CI and CLI/Engine identity: required before tagging.
- Record the merge SHA, immutable tag and GitHub Release URL in #475 after publication.

This release does not rewrite Engine 0.6.0, frozen evaluation results, or published Reason CLI binaries.
