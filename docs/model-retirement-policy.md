# Provider/model retirement and fallback policy

[日本語](model-retirement-policy.ja.md) | English

**Status:** implemented on the Reason CLI 0.5.x product line. Harness Engine 0.4.2 reasoning and authority semantics are unchanged.

Reason does **not** silently change provider/model execution identity to recover from model retirement, incompatibility, quota, outage, or provider errors.

## Catalog lifecycle state

`reason models` reports an `availability` value separately from compatibility evidence:

- `current` — eligible for ordinary configured use when also marked general-use.
- `deprecated` — trustworthy lifecycle evidence says the identity is being retired; Reason refuses it as a product default.
- `unavailable` — trustworthy evidence says the identity is no longer usable; Reason refuses it as a product default.
- `known_incompatible` — Reason has evidence that the identity is incompatible with the current product protocol/role.
- `unlisted` — the configured identity is not in the curated catalog; absence from the catalog is not itself a provider-retirement claim.

`deprecated` and `unavailable` are assigned only when the repository has trustworthy lifecycle evidence. Reason does not infer retirement from a transient rate limit, quota error, outage, DNS failure, or one failed request.

## No silent fallback

`reason setup`, `reason model set`, and local doctor validation fail closed for non-current catalog identities and point to `reason models <provider>` for an explicit replacement choice. Ordinary provider failures remain typed operational failures; they do not trigger a hidden switch to another model or provider.

Reason CLI 0.5.0 does not implement automatic provider/model fallback chains. If an explicit fallback feature is added later, it must validate compatibility before execution and record the provider/model that actually ran in human output, JSON, and persisted session provenance.

## Persisted sessions

Managed sessions pin the recorded provider/model runtime identity. `--continue` / `--resume` reuses that identity, and an explicit conflicting provider/model is rejected as `session_incompatible`. A user who wants a different execution identity starts a new session (or an explicit future migration/fork flow); Reason never rewrites a prior session identity in place.

## Diagnostics

`reason doctor` reports the configured model's catalog `model_availability` and gives an explicit `reason models <provider>` recovery path when local compatibility/lifecycle validation fails. A live provider failure is reported as an operational readiness failure, not as proof that the model is retired.
