# Entity identity gate

## Status

The MCP external-information / entity-identity research in Issues #193, #195, and #196 completed with the trusted-context v12 candidate eligible for adoption. The frozen candidate semantics coordinate is `d1db067e6efe6033656b8e7c3315a9fe322c015d`. The separately frozen Issue #196 one-shot holdout passed 16/16 on its first and only observation with zero semantic false decisions, zero context-unverified fact admissions, and zero planner/tool/operational/budget failures.

That result is historical evidence. The holdout must not be rerun, replayed, or used for tuning. The Issue #197 adoption branch starts from the frozen candidate semantics, not from the observed holdout evaluator.

## Authority boundary

The model / planner is untrusted. It may choose only a bounded next action exposed by the Harness; it cannot decide truth, entity-identity sufficiency, evidence admission, or terminal correctness.

The Harness owns these rules:

- candidate lists are plausibility evidence, never fact-admission authority;
- without trusted context, an entity fact is admitted only when the non-disambiguation Wikipedia top matches the resolved Wikibase item and cross-source corroboration rank is 1;
- with trusted identity context, bare-query rank-1 agreement is not sufficient. The Harness constructs the canonical `surface, context` query and the planner may only select `follow_suggested_query` or `stop`;
- trusted-context admission additionally requires deterministic context compatibility in the corroborating Wikidata label/description or Wikipedia top title;
- on the trusted-context coordinate only, insufficient search recall may use a bounded direct lookup of the exact Wikibase QID from the non-disambiguation Wikipedia top. The direct-QID bridge is not authority by itself and must still pass the same context-compatibility gate;
- no-context queries never use the direct-QID bridge and never relax the rank-1 rule;
- adapter-supplied suggested queries are observations only. Only a Harness-owned suggestion marked with origin `harness_trusted_identity_context` is executable;
- malformed, unavailable, or duplicate planner actions are rejected before external HTTP;
- when ambiguity cannot be resolved from trusted context, the result remains NIL / `unknown`.

## Stable adoption materialization

Issue #197 materializes v12 once into `crates/reasoning-harness-cli/src/bin/mcp_identity_gate_benchmark.rs`; the normal adoption test path does not reapply the research transformer chain. The frozen materialized identities are:

- candidate semantics: `d1db067e6efe6033656b8e7c3315a9fe322c015d`
- research materialization SHA-256 (pre-rustfmt): `d36bc7e0df7e423c96a8a7a3b7aa7846471d985e6dfffca9928b398e5557ff9f`
- canonical adoption benchmark SHA-256 (post-rustfmt, no semantic change): `bef5fe8722d3b29123a0099fbc6d6d3f896d3cd94ecb30bfc8f2f5a1fe1f5d6f`
- adapter SHA-256: `c8fb8f76a4e4abcd5b89161d98508e0146f9d329e8831073cd0f69530e8e7098`

`fixtures/mcp-identity-context-v12-adoption.sha256` and the adoption CI verify these identities directly. Research transformers remain only for provenance and reproducibility; the adopted source is tested directly.

## Change discipline

Any change to identity sufficiency, query policy, planner authority, evidence admission, budgets, stop rules, or terminal safety is new semantic research rather than adoption cleanup. It requires a new issue, a fresh development split, and a new independent holdout.

The frozen #193, #195, and #196 holdouts remain historical evidence and must not become tuning material.
