# Security Policy

## Reporting a vulnerability

Please do not open a public issue for vulnerabilities that could expose credentials, bypass the trusted verification boundary, or cause untrusted model output to gain authority.

Use GitHub's private vulnerability reporting for this repository when available. If that channel is unavailable, contact the repository owner privately through their GitHub profile rather than posting exploit details publicly.

## Security boundaries

The following are security-sensitive invariants:

- provider credentials must never enter artifacts, logs, fixtures, or model-visible prompts;
- `ReasoningCandidate` cannot create evidence or verification receipts;
- only harness-owned passes may promote claims to trusted states or establish contradiction authority;
- verification receipts must bind to exactly one claim and valid harness-owned evidence;
- live-provider CI must remain manual/isolated and must not expose repository secrets to untrusted pull-request code.
