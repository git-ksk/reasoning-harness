## Summary（概要）

## Correctness boundary affected（影響する correctness boundary）

- [ ] Candidate/model authority
- [ ] Evidence/provenance
- [ ] Verification/oracle
- [ ] Deterministic pass
- [ ] Acceptance policy
- [ ] Provider adapter only
- [ ] Docs/project maintenance only

## Validation（検証）

- [ ] `cargo fmt --check`
- [ ] `cargo clippy --locked --workspace --all-targets -- -D warnings`
- [ ] `cargo test --locked --workspace`
- [ ] Recorded fixture benchmark reviewed if semantics changed

## Notes（注記）

意図的な benchmark の変更があれば、その内容と trusted boundary を弱めない理由を説明してください。
