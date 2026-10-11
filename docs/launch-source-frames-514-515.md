# Successor launch-source qualification fixes (#514 / #515)

This is an unreleased source candidate after Engine 0.7.0. Published Engine and
CLI tags, package versions, frozen corpora, and provider observation archives are
unchanged. An independently reviewed successor release remains a separate gate.

## Reproduction and behavior

The committed deterministic reproducer first observed nine false negatives:
`was launched` / `has been launched` (two), and seven affirmative launch frames
with decimal versions or dotted names. All initial negative controls remained
non-Relevant. The same controls now pass, with additional adversarial controls.

- v17 recognizes direct, affirmative `TARGET was launched` and
  `TARGET has been launched` predicates. A product's documentation, a rival, or a
  nearby sibling name cannot supply the new predicate's subject.
- v17 preserves periods within an exact policy-owned name, numeric versions, and
  single dotted alphabetic tokens with lowercase suffixes such as `Z.ai`.
  Target-edge periods, trailing abbreviation periods, uppercase continuations,
  semicolons, newlines, and ordinary/full-width sentence boundaries stay separate.
- Newly recovered `has made … generally available` objects allow bounded product
  qualifiers and vendor modifiers. Documentation, pricing, or a sibling's launch
  cannot be substituted for the target product.
- Questions (including full-width question marks), denial, speculation, fictional
  scenarios, title/navigation-only content, missing qualification, unresolved
  identity, explicit absence, and scope risks do not gain positive authority.

This is bounded deterministic text recognition, not general sentence parsing or
language understanding. Unsupported or ambiguous name/abbreviation forms may
still abstain. Dotted organization tokens do not authenticate a source or create
an alias; the independent identity and provenance boundaries remain necessary.

## Compatibility

`normalized_factual_segments` and the historical launch vocabulary are unchanged.
The new tokenizer and positive-recall rules are confined to v17/v30. Explicit
v16/v29 checks retain their pre-fix results for both public reproducers. Existing
historical replay and frozen launch tests continue to run without relabeling.

For locally proven positive launch frames, v30 uses v17's authority floor with
the existing v22 identity/scope/materialization gates. Reapplying v23's historical
v11 relation vocabulary would discard a recovered dotted-name frame. Other
relations and unresolved/negative launch cases retain the v23 composition.

## Validation and remaining gates

`launch_source_frames_514_515.rs` includes the public reproducers, additional
synthetic development controls, cross-sentence/signal and sibling negatives,
aliases, Unicode text, metadata-only rejection, missing bindings, identity/scope
guards, and historical compatibility checks.

Local validation commands:

```sh
cargo fmt --all --check
cargo clippy --offline --locked --workspace --all-targets -- -D warnings
cargo test --offline --locked --workspace
cargo run --offline --locked -q -p reasoning-harness-cli --bin reason -- eval fixtures --format human
python3 scripts/verify_engine_0_7_independent_v1_archive.py
```

Final local results on Rust 1.88 / macOS: format, strict locked workspace Clippy,
and diff checks PASS; workspace tests 1,449 PASS / 0 FAIL / 6 existing ignored;
the standard 20-case fixture evaluation has accuracy 1.000 and zero unsupported
accepted claims. The original independent observation archive retains all 13
exact SHA-256 checks and its frozen score. OAuth loopback tests required running
outside the sandbox's local-listener restriction, with Cargo kept offline.

The additional controls were authored by the implementer. They are not an
independently authored, frozen holdout, a new live-provider acceptance run, or
evidence of a published patch. Independent regression review, a fresh holdout,
and exact-head cross-platform CI remain prerequisites for promotion/merge.
The separate #512 / PR #513 General-relation candidate is not included here.

## #512 alignment review

A temporary copy combined this candidate with PR #513 (`0cc3e06`), leaving the
tracked branch unchanged. For the General target `Provisioned Concurrency`, the
Japanese question `Provisioned Concurrencyは何を準備するか？`, exact advisory
bindings, and exact-target/requested-relation/no-risk local qualification, both
of these adversarial excerpts incorrectly materialized as `Relevant` under v30:

| Excerpt | Expected | Observed |
| --- | --- | --- |
| `Provisioned Concurrency is a pricing option, while Other Feature initializes execution environments.` | non-Relevant | Relevant |
| `Provisioned Concurrency does not actually initialize execution environments.` | non-Relevant | Relevant |

The bilingual helper checks sentence-level token co-occurrence and a limited
negation substring list. Those checks do not establish predicate ownership or
exclude an intervening adverb in the negation. PR #513 needs separate corrective
work and regression evidence before integration; its current safety claims are
not accepted by this review. These findings do not affect the ChangeOrLaunch
fixes in this branch.
