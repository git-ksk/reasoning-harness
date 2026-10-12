# Bounded bilingual preparation relevance (#512 / PR #513)

This is an unreleased Engine source correction on top of #514/#515. Package
versions, release tags, Cargo.lock and original provider archives are unchanged.

## Problem and recovered behavior

The Japanese question `Provisioned Concurrencyは何を準備するか？` lacks literal
English relation-token overlap with the public AWS documentation's affirmative
setup/initialization statement. The historical v16/v29 contract safely abstains.
The successor v17/v30 recovers the explicitly named AWS Lambda preparation
relation from a complete local excerpt or fact, while preserving that historical
result.

The original PR candidate `0cc3e06` used sentence-wide token co-occurrence and
substring negation checks. The #514/#515 alignment review reproduced these false
positives, retained in `launch-source-frames-514-515.md`:

- `Provisioned Concurrency is a pricing option, while Other Feature initializes execution environments.`
- `Provisioned Concurrency does not actually initialize execution environments.`

The first was reconfirmed after merging current main into the PR at `d115034`.
Both now remain non-Relevant under the successor path.

Independent review of intermediate source `c4f6e16` then found two additional
false positives: a denial/simulation/retraction in another statement was lost
when an isolated affirmative segment was selected, and an alternate provider
could name its service Lambda before an abbreviated Lambda preparation frame.
The corrected source `91f9399` requires all factual statements/signals to fit
the supported affirmative frame. Public regressions retain both findings and
preceding/following, split-signal and statement-boundary variants.

## Source and authority boundaries

- The bridge is limited to General relation, the explicitly bounded Japanese
  preparation-question forms, and canonical Provisioned Concurrency identities.
  A guide, another provider's product, unrelated Japanese question or English
  question does not gain this new bridge.
- Only Excerpt/Fact content can prove the relation. A complete statement must
  bind AWS Lambda and Provisioned Concurrency to an affirmative initialization
  predicate, its execution-environment object and a complete bounded tail.
- The public setup condition (`If you set provisioned concurrency on a function,
  Lambda initializes ...`) is a supported feature-to-action frame. Arbitrary
  conditionals, other actors, negation, speculation and simulated statements are
  outside that frame.
- Every nonempty factual statement must independently fit a supported complete
  frame, and every Excerpt/Fact signal must contain such a statement. Periods,
  semicolons, line breaks and full-width statement boundaries cannot discard
  unsupported governing context. Mixed factual context conservatively abstains.
  Titles, URLs and metadata cannot provide a missing subject/predicate. Quotes,
  questions, colons and parenthetical ownership are not flattened into proof.
- Recovery requires exact independent target/relation bindings, raw requested
  relation, exact effective identity, a Harness anchor, no risk and no explicit
  local absence. The bridge never clears risk or alternate/absent relations.
- v30 uses the existing v22 identity/scope materialization gates only when this
  successor local proof exists. Other General cases retain v23 composition.

This remains bounded text recognition. Unlisted passive/idiomatic wording,
unsupported modifiers, unrelated neutral factual paragraphs and a bare feature
name without local Lambda ownership may abstain. This bridge does not resolve
arbitrary enclosing discourse. No generic translation or model-asserted relation
authority is added.

## Historical compatibility and verification

`requested_relation_locally_present`, v16 qualification and v29 materialization
are byte-identical to main `2bfd31f`. The original public AWS example remains
Ambiguous under v29 and becomes Relevant only under v30. Launch v17/v30 behavior
from #514/#515 is retained and is included in the full regression run.

`crates/reasoning-harness-core/tests/bilingual_preparation_512.rs` covers the
original example, owned affirmative variants, both prior review failures,
negative/uncertain/simulated/question cases, same-name other-provider and document
subjects, statement/signal isolation, nonfactual-only signals, missing or
conflicting bindings, identity/scope risks, and question/entity boundaries.

Final source `91f93990e6e248a00aa594ff31006889ebd7c15b` passed the full locked
offline workspace on Rust 1.88 / macOS: **1,466 passed, 0 failed, 6 existing
ignored**, including all 288 existing #514/#515 v1-v9 regression controls.
Format, diff checks and strict all-target workspace Clippy pass. The standard
20-case fixture benchmark retains accuracy 1.000 and zero unsupported accepted
claims. The original Engine archive retains all 13 exact SHA-256 checks and its
frozen score. OAuth loopback tests ran outside the local-listener sandbox with
Cargo offline.

An independent read-only subagent reviewed the exact source/test hashes, with
no repository corpus, result, documentation or evaluation access. Both original
P2 findings are resolved; no remaining P1/P2 was found. Its own 133 probes pass
(38 positives, 95 negatives), v16/v29 have zero differences versus main across
133 rows, and four fresh literal Launch controls have zero differences. These
are reviewer-authored synthetic probes, not a separate fresh holdout claim.
The unchanged-source public regression suite passes all six test groups.

The raw completed report is preserved verbatim in
`evaluation/bilingual-preparation-512-review/independent-review.json`; source
hashes, commands, counts and verification limits are in the adjacent
`local-validation.json`. Exact-head cross-platform GitHub checks are a separate
merge gate. This record does not claim live-provider, integration or release
acceptance.
