# Proxy, custom CA, and headless network environments

**Status:** implemented on the Reason CLI 0.5.0 development line. Harness Engine 0.4.2 reasoning/authority semantics are unchanged.

Reason uses the standard reqwest system-proxy behavior for HTTP clients. `HTTP_PROXY` / `http_proxy`, `HTTPS_PROXY` / `https_proxy`, `ALL_PROXY` / `all_proxy`, and `NO_PROXY` / `no_proxy` are honored by provider adapters, remote MCP, MCP OAuth discovery/token exchange, and CLI release/update traffic. Diagnostic output reports only whether those variables are present; proxy URLs, usernames, passwords, and bypass lists are never echoed.

## Custom CA bundle

Set `REASON_CA_BUNDLE` to a PEM certificate bundle when a corporate TLS interception proxy or private CA must be trusted in addition to Reason's normal built-in trust roots. The bundle is additive: built-in certificate verification remains enabled. Reason rejects an empty path, a non-regular file, an empty/oversized bundle, or invalid PEM. The current limit is 1 MiB.

Reason intentionally has no product option that disables TLS verification, accepts invalid certificates, or globally weakens certificate checks. Recovery guidance should fix the proxy, DNS, certificate chain, system trust, or `REASON_CA_BUNDLE` instead.

## Doctor diagnostics

`reason doctor --format json` works noninteractively/headlessly and reports a `network` object with proxy presence booleans, custom-CA status, and live-probe status. The default doctor run is local-only. `reason doctor --live-check` performs a bounded HTTPS connectivity probe to the configured provider host before the provider/authentication readiness request. Any HTTP response is sufficient to establish the transport path; provider authentication/service status is diagnosed separately.

Network failures remain operational failures and are typed separately from provider outage/credential failures:

- `network_dns`
- `network_proxy`
- `network_tls_certificate`
- `network_connectivity`
- `network_timeout`
- `network_transport`
- `network_custom_ca`

These diagnostics do not become semantic `unknown` and do not alter evidence admission, verification, or finalization.
