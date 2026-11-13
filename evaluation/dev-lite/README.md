# Development Lite — repeatable paired CLI smoke evaluation (#574)

**Development diagnostic only, not a release / accuracy acceptance gate.** This
is a separate path from the historical V3–V6 one-shot, signed, independently
reviewed 24-case evaluation. Do not edit, rerun, regrade, or substitute their
receipts, scoring records, tags, or source evidence.

The evaluator uses 4 cases by default (`rw-01,rw-06,rw-07,rw-23`; conventional
answers and two private/unknown-answer controls), or an explicit selection of
4–6 from the **previously consumed development corpus**. Both arms run the same
frozen case question/config and model. The execution order alternates by case,
and the binary and config hashes are checked before running. There are **no
reviewer secrets, approval signatures, mandatory CI service or one-shot tags**.

## Provider-free plan on the Mac

Use a matching previously prepared native two-arm build and the matching
48-job request inventory. Existing development-only working directories may
be reused; source/build versions are printed rather than mislabeled as current.

```sh
python3 evaluation/dev-lite/run.py \
  --requests .local-v3-preparation-20261104/preparation-581-02/requests.json \
  --build .local-v3-preparation-20261104/build-581-02/build.json
```

This is a **0-provider-call** static check. To invoke the real baseline and
candidate binaries' first-request planners and verify that all **8 live CLI
request shapes match pairwise**, add `--mode preflight`. Preflight needs no API
key, performs no inference, and prints a per-case SHA-256 inventory.

No reviewer key, API token, source download or user configuration is loaded. To compare the latest CLI revision,
rebuild and reprepare the paired CLI using
`normal_v3/build_native_pair.py` and `normal_v3/prepare_local.py` with a
specific committed source SHA, and pass those matching outputs.

## Optional live development diagnostic

The `live` mode also runs the native provider-free preflight above before
launching any inference. It does not rebuild the binaries: the paired build
commit printed by `plan` must match the revision you intend to compare.
By default the live runner refuses a pinned build older than checkout HEAD;
`--allow-older-build` is an explicit opt-in for historical comparisons.

First independently verify the real provider organization is **Free/no extra
charges**, usable quota remains, and the keys are allowed for the fixed
`ministral-8b-2512` model. A completed prior API test or an old free-tier
declaration is **not current billing/quota evidence**. Never bypass provider
rate limits, switch tiers, or add retries.

```sh
# Set MISTRAL_API_KEY from a safe local secret source; never place it in Git.
export REASON_DEV_LITE_FREE_CONFIRMED=1
python3 evaluation/dev-lite/run.py \
  --requests .local-v3-preparation-20261104/preparation-581-02/requests.json \
  --build .local-v3-preparation-20261104/build-581-02/build.json \
  --mode live --confirm-no-extra-charges \
  --output "$HOME/.local/share/reasoning-harness/dev-lite/run-$(date -u +%Y%m%dT%H%M%SZ)"
```

Default: 4 cases / 8 physical CLI runs, maximum **6 logical model calls per
arm**, 512 generation tokens per call, 1,536 cumulative output tokens and
12,000 cumulative total tokens **per arm**. Maximum theoretical eight-arm
reservation = **96,000 tokens**. CLI budget controls and a conservative
cross-arm reserve prevent initiating extra work above the declared ceiling.
They **do not** prove externally shared org quota or guarantee no charges;
the external account configuration must enforce no extra charges. No implicit
retry on timeout, missing usage, nonzero CLI exit, or ambiguous result.

Outputs (private `0700` folder, `0600` files) include exact stdout/stderr,
SHA-256, observed provider usage and per-job checkpoints. Each run creates a
new directory; it does not restart/overwrite a partial prior run. The corpus
and outputs can contain source/private-context material, so never commit,
publish or upload them to public Actions artifacts.

Review each actual answer, quotation, source support and abstention manually,
record TP/FP/FN/TN for development feedback if useful, and report missing or
invalid observations separately. **No automatically claimed F1, independent
approval, semantic acceptance, safety certification, formal quality GO,
release decision or CLI adoption** is produced.

Before formal acceptance, freeze an accepted candidate, author **new
independent** test questions, and apply the release quality/safety gates from
#531. This opt-in development profile cannot replace those tests.

The development run is intentionally **not** launched automatically by tests
or by the default command. On the present Mac, the read-only native credential
status reports Mistral as missing; the existing Mistral GitHub Actions Secret
is not retrievable to the Mac, and must not be copied into logs or the repo.

## Offline tests

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s evaluation/dev-lite -p 'test_*.py' -v
```
