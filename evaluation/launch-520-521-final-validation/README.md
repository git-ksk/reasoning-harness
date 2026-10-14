# #520/#521 successor validation

Root/integration evidence; not a fresh independent holdout. Exact implementation candidate and final checks are recorded in `summary.json`. Source package metadata deliberately stays 0.7.1 until a dedicated release PR.

- `development-regressions.log`: direct v17/v30 tests for the original audit and seen v1–v9 cases. v1 P04 and v3 P323 are explicitly corrected only in the development replay, retaining their original independent RED scores.
- `original-audit-after.jsonl`: 56 complete audit inputs and current public-API outputs; original observations in the earlier audit directory remain unchanged.
- `root-additional-inputs.json` / `root-additional-after.jsonl`: four added root context controls. The additional governing, possession, global-colon and negated-actor inputs retain both their first failing development observations and their repaired outputs.
- `historical-compatibility-inputs.json`: complete policy/candidate/proposal/raw inputs from the audit and observed holdouts.
- `historical-compatibility-probe.rs`: prints complete v16 and v29 JSON results. `historical-base.jsonl` uses the unchanged engine-v0.7.1 source tag; `historical-current.jsonl` uses the fixed successor source. Comparison is byte-for-byte, not only the disposition column. The tag source was extracted with git archive and built using its locked normal dependencies.
- `reconciliation-controls.jsonl`: 22 additional source-reconciliation authority/replay controls, byte-identical to the original audit.
- Logs/standard fixtures/original archive verification record local gate outcomes; SHA-256 manifest identifies retained evidence. Earlier disk exhaustion and sandbox-denied loopback fixtures were operational failures; final runs use nonincremental/debug-free builds and local-loopback permission. They are not semantic retries or independent rescoring.

Independent first observations, corpus freeze/provenance and unchanged RED history are in adjacent `launch-520-521-independent-v*` directories. No external factual-truth, live-provider, real-user or physical-device acceptance is claimed. See [repair report](../../docs/launch-context-markers-520-521.md).

## Local gate commands

Use `CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0` and a disposable `CARGO_TARGET_DIR` when disk is constrained. Cargo dependencies must already be cached for `--offline`; local OAuth/subprocess fixtures require loopback access.

```sh
cargo test --locked --offline --workspace --exclude reasoning-harness-providers --jobs 2
cargo test --locked --offline -p reasoning-harness-providers --jobs 2 -- --test-threads=1
cargo clippy --locked --offline --workspace --all-targets --jobs 2 -- -D warnings
cargo fmt --all --check
cargo run -q --locked --offline -p reasoning-harness-cli -- eval fixtures --format json
python3 scripts/verify_engine_0_7_independent_v1_archive.py
```

The historical probe follows the [original audit's clean-tag compilation procedure](../engine-0.7.1-rereview-20261013/README.md), using this bundle's full-input JSON and v16/v29 runner. Compile the base from unchanged `engine-v0.7.1` with its locked normal dependencies and the candidate from the exact recorded successor source; compare both complete JSONL files. No compiled binaries or credentials are retained here.
