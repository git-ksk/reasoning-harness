# Engine 0.7.0 — Target-local source-qualified answer presentation (#490)

[日本語](engine-0.7-target-presentation-v1.ja.md)

**Status: development candidate, 14/14 precommitted synthetic scenarios PASS. No independent provider acceptance or Engine 0.7.0 release.**

## Motivation and opt-in contract

Engine 0.6.1 can report a global `Conflict` for a multi-target turn even when one target's source wording has been explicitly approved as compatible. Legacy citations are preserved, but that global label does not expose which target is reviewed-compatible.

The additive `reconcile_source_attributed_targets` API returns `harness-source-reconciliation-target-answer-v1`, with per-target `target_id`, `status`, and exact unchanged v1 `original` text/citations. One target's reviewed-compatible wording cannot remove another target's conflict. The global legacy finalization is preserved exactly; concatenated target-local claim IDs/citations must match the original or the call fails closed. Models cannot self-authorize equivalence; no truth promotion, fabricated citations, provider calls or acquisition occur. Source-derived terminal and bidirectional control characters are escaped in text mode. Existing v1 contracts and sessions are unchanged.

## Developer-only CLI opt-in

Without approval files, the conservative target-specific view requires no reviewer key.

```bash
cargo run --locked -p reasoning-harness-cli --bin reason-source-review-local -- \
  show-targets --artifact /path/to/artifact.json \
  --target target-A --target target-B --json
```

For previously signed source-review approvals, provide the reviewer policy and approval files.

```bash
cargo run --locked -p reasoning-harness-cli --bin reason-source-review-local -- \
  show-targets --artifact /path/to/artifact.json \
  --target target-A --target target-B \
  --reviewer local-owner --approval /private/path/approval-A.json --json
```

Normal published `reason` CLI behavior remains unchanged.

## Frozen development results

Fourteen scenarios were committed **before implementation** under `engine-0.7-target-presentation-dev-v1-spec-freeze` (commit `f5c098b2f5df2c93f5385a96bde1719c5e313a37`; manifest SHA-256 `fe27ecc6704992109e8da100235bc4ba5ffbafec1073ebc4fa572993304b4214`). Eleven valid target-specific presentations, three fail-closed rejections. Six **synthetic development** cases expose reviewed-compatible target detail while the original global result is still Conflict; all original citations are preserved, replay matches, no hard truth promotion or external calls. Tests and report: `crates/reasoning-harness-core/tests/target_presentation_development_v1.rs` and `evaluation/engine-0.7-target-presentation-dev-v1-result.json`.

This is **not** measured real-source answer gain or independent acceptance. The fresh, separately frozen Mistral/Google/Groq baseline comparison, zero-hard-violation gate, source compatibility and Engine 0.7.0 release remain under [#492](https://github.com/git-ksk/reasoning-harness/issues/492).