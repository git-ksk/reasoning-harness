# Mac/native local authority acceptance — 2026-11-03 JST

The [local execution policy](../../../docs/local-execution-policy.ja.md) makes
the single-host Mac profile the current development and acceptance target.
This is implementing-agent review and local verification, not independent
human acceptance or a real model-quality result.

## Verified

- Python 3.13 and 3.14: **134/134 tests pass on each**. The
  [machine-readable record](local-acceptance-20261103.json) freezes the tested
  source and log hashes; the earlier 118-test record is preserved separately.
- A fixed operator-owned local profile persists receipt/run/cohort,
  reserve/inflight/observed and first-score records without GCS or network.
  Actual OS-process races yield one run/score winner. SIGKILL and ambiguous
  storage errors retain claims and prevent implicit restart/refund.
- Root/identity replacement, unsafe permissions, symlink/hardlink/FIFO,
  alternate evaluation ID and output-path replay controls pass. API roles
  reject executor-created receipts and first scores.
- All 48 inert native subprocesses complete through the local authority;
  unknown usage on the first child suppresses the other 47. The synthetic
  48-observation collector → signed review → first-score path also uses the
  real local store. Test signatures are not an actual review decision.
- The Rust CLI builds with `cargo run --offline --locked -p
  reasoning-harness-cli --bin reason -- --help`. A no-config local run with
  `examples/input.json` and `examples/candidate.json` produces the expected
  sample `unknown` verdict. Verification of its artifact payload returns
  `valid: true` with no diagnostics. This smoke is separate from the v3
  provider-evaluation scheduler.
- No model API calls or cloud resource operations were performed for this
  local acceptance. Existing v1/v2 evidence and first scores are unchanged.

## Limits and remaining work

The OS owner and fixed operator profile are trusted. These controls do not
provide hostile same-UID/root isolation, cross-host locking, physical
power-loss acceptance, or proof that detached descendants have terminated.

Actual native HTTP/usage capture (#581), full display/source/native-bundle
binding and pre-advance safety (#582), real enrolled semantic review (#583),
and cancellation/unknown descendant accounting (#540) remain open. A new
real F1, independent acceptance, CLI adoption and release are not claimed;
`quality_go` remains false. GCS/VM/KMS/IAM/Docker/two-host deployment are not
requirements for this local profile.
