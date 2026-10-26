# Ordinary CLI evaluation / model-free API verification

[Plan (Japanese)](../../docs/engine-073-cli-evaluation-plan.ja.md) | [Plan (English)](../../docs/engine-073-cli-evaluation-plan.md)

This is a preparation and diagnostic checkpoint, not a live execution receipt or a new F1 result. The ordinary CLI uses real models for quality evaluation. The auxiliary APIs are checked with zero model calls; the earlier model-assisted helper experiment is retained separately.

- [Independent model-free API verdict](model-free-api-review/review.json): PASS for pinned candidate API contracts,44 direct groups,9 legacy compatibility cases,960 existing core tests passed with2 historical ignored controls preserved. No provider crate in the direct probe, no model API calls. Caller authentication, semantic equivalence determination and freshness policy remain host responsibilities.
- [Independent Mistral raw diagnosis](mistral-independent-diagnosis/diagnosis-summary-v1.md):48 preserved normal runs; plan limit/identity rejection and malformed candidate wire responses are distinguished from duplicate acquisition, early Reject, selector and explanation issues.
- [Preparation state](preparation-status.json): CLI boundary repairs for Issues562/563 are awaiting bounded review. No source runtime approval or new execution tag is implied.

The copied evidence files retain their independently authored bytes/hashes. Complete local logs, binaries and remaining manifest entries are preserved in ignored target/private-evaluation-pending/cli-normal-evaluation-v1. Historical v1/v2 reports and verdicts are unchanged. API contract PASS is not CLI adoption or distribution acceptance.
