# Update, rollback, and uninstall

[日本語](update-rollback-uninstall.ja.md) | English

**Status:** Reason CLI 0.5.0 development line. Split releases (`reason-v*`) only.

Reason lifecycle changes are explicit, provenance-verified, and reversible. Historical unified releases through `v0.4.2` predate the split-release attestation contract and are not self-update/rollback targets.

## Check before changing anything

```bash
reason update --check
reason update --check --format json
```

`--check` discovers published `reason-v*` releases, downloads only `release-manifest.json`, verifies its GitHub OIDC/Sigstore attestation, and reports Reason CLI and Harness Engine versions separately. It never downloads or replaces the native archive.

For an explicit target:

```bash
reason update --check --version 0.5.1
```

## Apply an update

```bash
reason update
```

Interactive terminals show the selected CLI/Engine identities and ask for confirmation. Automation must be explicit:

```bash
reason update --yes --format json
```

The apply path verifies, in order:

1. attested `release-manifest.json` for the exact repository, signer workflow, and `reason-vX.Y.Z` source ref;
2. target CLI/Engine identities from that manifest;
3. the platform archive's GitHub/Sigstore attestation;
4. archive and `SHA256SUMS` digests against the attested manifest;
5. the archive entry against `SHA256SUMS`;
6. archive path safety on Unix and the extracted `reason --version` identity;
7. an atomic same-directory replacement on Unix, or a staged replacement immediately after process exit on Windows.

If the Harness Engine SemVer changes, apply fails until the user explicitly acknowledges it:

```bash
reason update --allow-engine-change
```

A presentation-only CLI update therefore cannot silently look identical to a reasoning/correctness engine change.

## Explicit rollback

Rollback is a separate operation and only accepts an older split release:

```bash
reason update --rollback 0.5.0
```

The same provenance, manifest, checksum, Engine-change, and confirmation rules apply. `reason update` refuses an older version and points to `reason update --rollback VERSION`; rollback mode refuses a same/newer version. This prevents a confused update channel from silently downgrading the executable.

## Uninstall

Preview first:

```bash
reason uninstall --dry-run
```

Then remove the current executable:

```bash
reason uninstall
```

Defaults are deliberately conservative:

- native OS provider credentials are retained;
- `config.json` and `project-trust.json` are retained;
- explicit-path session files are never searched for or deleted;
- provider environment variables are never modified.

Explicit purge options are available:

```bash
reason uninstall --purge-data --purge-credentials
```

`--purge-data` is narrowly scoped to Reason-managed `config.json` and `project-trust.json`; it does not recursively hunt for arbitrary session files. `--purge-credentials` removes supported provider entries from the native OS credential store only. Non-interactive mutation requires `--yes`.

## Trust root

Lifecycle mutation never falls back to SHA-256 alone for `reason-v*`. GitHub CLI 2.93.0+ is required for attestation verification. See [Release provenance](release-provenance.md).
