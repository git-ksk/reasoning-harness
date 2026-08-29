# Contributing

Thanks for helping improve reasoning-harness.

## Scope

The project is a provider-neutral Rust research harness for making stochastic candidate output pass through explicit deterministic correctness machinery. Contributions should preserve these boundaries:

- model output is untrusted candidate data;
- evidence and verification authority are harness-owned;
- deterministic validators and external oracles outrank model self-assessment;
- `unknown` is a valid successful outcome;
- soft semantic judges must never masquerade as hard correctness gates;
- first-party runtime components remain Rust-only.

## Development

Use Rust 1.88 or newer and keep `Cargo.lock` committed.

```bash
cargo fmt --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo run -q --locked -p reasoning-harness-cli -- eval fixtures --format human
```

Changes to reasoning semantics should add or update fixtures and explain why the benchmark expectation changed. Live provider results are research evidence, not deterministic CI gates.

## Pull requests

Keep changes focused. Describe the correctness boundary affected, new failure modes covered, and benchmark impact. Never include provider API keys, private datasets, credentials, or generated logs containing sensitive input.
