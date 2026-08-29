## Summary

## Correctness boundary affected

- [ ] Candidate/model authority
- [ ] Evidence/provenance
- [ ] Verification/oracle
- [ ] Deterministic pass
- [ ] Acceptance policy
- [ ] Provider adapter only
- [ ] Docs/project maintenance only

## Validation

- [ ] `cargo fmt --check`
- [ ] `cargo clippy --locked --workspace --all-targets -- -D warnings`
- [ ] `cargo test --locked --workspace`
- [ ] Recorded fixture benchmark reviewed if semantics changed

## Notes

Explain any intentional benchmark change and why it does not weaken the trusted boundary.
