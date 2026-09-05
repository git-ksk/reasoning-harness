# Product MCP external-information evaluation

Issue #203 adds a frozen product workload for real external-information acquisition through the existing read-only MCP boundary. It is separate from the historical six-case `product-dogfood-v1` smoke set and from the frozen #193/#195/#196 entity-identity research holdouts.

## What is frozen

The workload identity is `product-external-info-v1`.

- 21 cases
- 7 capability families × 3 cases
- 5 expected-grounded cases
- 13 expected-unknown cases
- 3 typed operational-failure cases
- SHA-256 manifest: `fixtures/product-external-info-v1.sha256`
- baseline main before the work: `aa0a8325ea4c3b53b38c8fe83cf3aae691a38599`
- freeze commit: `8aa7a9ed72ed80b186bde230078a45d6ba28141c`

The original six `fixtures/product-dogfood-v1` cases are separately hash-locked and remain byte-for-byte unchanged from the #203 baseline. The new workload contains no `resolver_facts` and does not reuse or transform #193/#195/#196 observed holdout entities or cases.

The scoring contract is `product-external-info-scoring-v1`. It compares:

1. raw model;
2. Harness without external acquisition;
3. Harness + MCP external acquisition.

Typed operational failures are excluded from semantic denominators. Acquisition success and verification success are reported separately.

## Acquisition boundary

The repository-local fixture MCP server is `scripts/product_external_info_mcp.py`. It performs bounded HTTPS GET only against a fixed public-host allowlist, extracts Harness-pinned JSON fields, and may emit the explicit untrusted `structuredContent.reasoning_harness` acquisition envelope understood by `mcp_readonly_v1`.

It is an acquisition/normalization component, not an authority or verifier. URL, field, source identity, authority class, scope, and identity assertions are Harness-owned fixed configuration. Generic MCP `content` remains opaque and cannot create fact candidates.

After acquisition, evidence still passes the ordinary Harness-owned provenance/freshness/scope/authority admission and verification path before a target can be exposed as grounded.

## First valid live three-arm observation

The first workflow-level run after the predeclared freeze exposed an evaluator clock-wiring defect and is retained only as infrastructure-invalidated audit evidence. No fixture, target, expected outcome, or scoring rule was changed in response. The evaluator clock was moved to after untrusted model generation, and the same evaluation input is shared by both Harness arms.

The first valid post-fix observation is:

- GitHub Actions run: `33974104359`
- branch head: `146e17a1bed3314d2827957949e6e98665ea9594`
- provider: Mistral
- model: `ministral-8b-latest`
- max output tokens: `1024`
- base seed: `15000`
- report schema: `reason-product-external-info-v1`
- comparison contract: `shared-candidate-canonical-finalization-v1`
- machine-readable report: [`observations/product-external-info-v1-mistral-ministral-8b-seed-15000-2026-09-05.json`](observations/product-external-info-v1-mistral-ministral-8b-seed-15000-2026-09-05.json)

### Semantic utility

| Arm | Semantic cases scored | Expected-grounded scored | Grounded targets exposed | Target coverage | Expected-unknown scored | Expected-unknown preserved | Unknown preservation | False target abstention |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Raw model | 18 | 5 | 0 | **0.00** | 13 | 13 | **1.00** | 5 |
| Harness without external acquisition | 18 | 5 | 0 | **0.00** | 13 | 13 | **1.00** | 5 |
| Harness + MCP external acquisition | 16 | 4 | 3 | **0.75** | 12 | 12 | **1.00** | 1 |

The important product result is not that MCP always produces an answer. It is that this frozen run recovered three grounded targets that neither the raw model nor the no-external Harness arm exposed, while preserving every scored expected-unknown target as unresolved.

The MCP arm had two semantic cases that were operationally incomplete and therefore excluded from its semantic denominator. They are retained as typed operational results, not converted into semantic failures or fabricated abstentions.

### Safety and admission results

For the MCP arm:

- external acquisition attempts: `21`
- external acquisition successes: `16`
- verification successes: `4`
- unsupported grounded claims: **0**
- missed target insufficiency: **0**
- identity-unsafe admission: **0**
- MCP-output authority self-promotion: **0**
- stale rejection: `1`
- authority rejection: `2`
- conflict rejection: `1`
- scope rejection: `0`
- typed tool/protocol/timeout/policy operational failures: `5`
- overall frozen safety gate: **passed**

Acquisition success is deliberately not treated as verification success: 16 acquisitions succeeded, while only 4 produced verified target support. The runtime therefore did not turn tool availability into truth.

## Conservative residual

`authority-crates-serde-primary` acquired and admitted evidence and verified the exact target, but the final artifact remained globally unresolved because other model-generated state was still unresolved. The target was therefore not exposed, producing the single scored false target abstention in the MCP arm.

This is recorded as conservative utility loss rather than repaired after observation. The frozen workload and scoring contract are not changed to turn the measured `0.75` into `1.00`.

## Operationally incomplete semantic cases

Two semantic cases were excluded from the MCP semantic denominator because the acquisition path ended in typed `policy_denied` operational results:

- `fresh-npm-typescript-name` (expected grounded)
- `identity-npm-vite-vs-vitest` (expected unknown)

The report keeps them outside semantic scoring exactly as predeclared. The workload is not rewritten or given a larger per-case timeout/response allowance after seeing these results.

## Official GitHub MCP compatibility boundary

A separate direct probe used the official `github/github-mcp-server` v1.12.0 container in read-only mode. The current `mcp_readonly_v1` contract is intentionally stateless and initialize-independent; the official server required an MCP session initialization and negotiated protocol `2025-11-25` when the client proposed `2026-07-28`.

After a standard `initialize -> initialized -> tools/call` session, `get_file_contents` succeeded, but returned generic `content`/`resultType` rather than a `structuredContent.reasoning_harness` fact envelope. That generic output remains non-promoting under the current Harness boundary.

Session/protocol negotiation support is tracked separately in Issue #204 rather than silently changing `mcp_readonly_v1` inside #203.

## Limitations

- This is one frozen workload slice and one measured Ministral 8B run; it is not a claim about every open-world task, MCP server, or model.
- Two semantic cases were operationally incomplete and are excluded from the MCP semantic denominator.
- The live corpus recorded no `scope_rejection`; scope ownership remains enforced by case configuration and credential-free deterministic external-resolution coverage, but this live slice does not provide an empirical scope-mismatch rejection count greater than zero.
- The official GitHub MCP server is not yet end-to-end compatible with the stateless `mcp_readonly_v1` transport contract; #204 owns that successor work.
- The one conservative verified-but-unexposed target is preserved as measured evidence instead of being tuned away after observation.

## Reproduction discipline

Do not rewrite `product-external-info-v1` in response to this observation. A semantic case/expected-outcome/scoring correction requires a successor corpus identity. Operational retries or transport fixes must remain separately identified from semantic tuning.
