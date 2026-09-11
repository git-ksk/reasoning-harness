# Secure provider credentials

[日本語](secure-credentials.ja.md) | English

**Status:** implemented on the Reason CLI 0.5.0 development line. The tagged `v0.4.2` release still uses provider environment variables only.

Reason keeps provider API keys outside `reason-config-v1`, project files, sessions, evidence, and authority state. The 0.5.0 credential backend uses the native OS credential store:

- macOS: Keychain Services;
- Windows: Windows Credential Manager;
- Linux/*nix: Secret Service when available.

There is **no plaintext fallback** when the OS store is unavailable.

## Runtime precedence

For each provider, credential lookup is deterministic:

1. if the provider environment variable is present, use it;
2. otherwise read the Reason OS-store entry;
3. otherwise fail with a typed credential error.

Environment variables remain the preferred automation path for CI, containers, servers, and headless environments:

| Provider | Environment variable | OS-store account |
| --- | --- | --- |
| Mistral | `MISTRAL_API_KEY` | `provider:mistral:account:default` |
| Google / Gemma | `GEMINI_API_KEY` | `provider:google:account:default` |
| Groq | `GROQ_API_KEY` | `provider:groq:account:default` |
| NVIDIA Hosted NIM | `NVIDIA_API_KEY` | `provider:nvidia:account:default` |

The service identity is versioned as `io.github.git-ksk.reason-cli.credentials.v1`. Account identities follow `provider:<provider>:account:<name>`; 0.5.0 starts with `default`, so future `work`/`personal` accounts can be added under the same service without migrating raw secret bytes. Google and the compatibility `Gemma` selector intentionally share the same Google credential identity.

If an environment variable is present but empty or invalid, Reason fails rather than silently falling back to the OS store. This prevents a broken explicit override from unexpectedly selecting another credential source.

## Failure behavior

- missing credential: `credentials`;
- OS credential service unavailable/locked/not supported: `credential_store_unavailable`;
- malformed or otherwise unusable OS-store data: `credential_store_error`.

Error messages and JSON failures never include the credential value. A headless Linux environment without Secret Service receives actionable guidance to use the provider environment variable or configure a platform credential service; Reason does not create a plaintext credential file.

## Managing credentials with `reason auth`

On the Reason CLI 0.5.0 development line, the supported user-facing surface is:

```bash
# Hidden TTY entry; provider can be omitted in an interactive terminal to use the picker.
reason auth login mistral

# Automation-safe alternatives. The secret is never a normal argv value.
reason auth login mistral --from-env
printf '%s\n' "$MISTRAL_API_KEY" | reason auth login mistral --stdin

reason auth status mistral
reason auth list
reason auth logout mistral
```

`login` refuses to overwrite an existing OS-store credential unless `--replace` is explicit. `status` and `list` report only source/state (`environment`, `os_store`, `missing`, or typed invalid/unavailable states); they never print masked fragments, prefixes, suffixes, or the credential itself. `logout` deletes only the selected provider/default-account OS-store entry and never modifies an environment variable. If the provider environment variable remains set, it remains the active runtime source after logout.

Interactive entry uses hidden/no-echo TTY input. Non-interactive use must choose `--stdin` or `--from-env`; there is deliberately no `--api-key`, `--secret`, `--password`, or other secret-valued argv flag. Credential replacement is one logical OS-store update, and deletion is scoped to the selected provider account. The storage naming leaves room for future named work/personal accounts without moving raw secret bytes through config files.

## Trust boundary

Provider credentials are operational secrets only. Reading a key from Keychain/Credential Manager/Secret Service does not create evidence, authority, verification receipts, or a trusted claim. Harness Engine 0.4.2 correctness semantics are unchanged.
