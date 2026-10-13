# Bounded bilingual preparation relevance (#512 / PR #513)

These corrections are included in the [Engine 0.7.1 source patch](engine-0.7.1-release.md). The validation below distinguishes earlier review from the final deterministic follow-up; published CLI adoption remains separate.

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

Previously reviewed source `91f93990e6e248a00aa594ff31006889ebd7c15b` passed the full locked
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

## Follow-up: raw punctuation and documentary context

The continuation review reproduced 21 newly introduced false positives on that
previous source: six unsupported raw forms, twelve denial/simulation/retraction
contexts in headings, titles or structured metadata, and three alternate-service
Lambda naming contexts in those same signal kinds. All 21 were `Ambiguous`
under historical v29 and incorrectly `Relevant` under the previous v30. Exact
inputs and original outputs are preserved in
`evaluation/bilingual-preparation-512-followup/`.

The successor proof now validates raw source punctuation before normalization.
Only alphabetic source words, whitespace, supported statement boundaries,
hyphenated pre-initialization spellings and commas at the supported setup/leading
adjunct boundary can enter the bounded grammar. Strikethrough, negation symbols,
quotes, arbitrary comma joins and other unsupported syntax cannot be erased into
an affirmative predicate. The public AWS setup condition and supported owned
pre-initialization variants still recover.

Titles, headings and structured metadata remain unable to supply positive proof.
Explicit bounded English denial, uncertainty, simulation and retraction cues,
common negation/strikethrough symbols, instruction markers and explicit local
service/product naming frames can now withhold the successor recovery. These
checks never establish an absent or different relation. Ordinary FAQ/configuration
titles remain allowed; URL and navigation wording do not govern the excerpt.
This is still bounded recognition, not arbitrary discourse or multilingual
context understanding. Neutral documentary text containing a blocking word may
conservatively abstain.

The prior independent reviewer corpus and report remain unchanged. Its row 118
contains `This claim is false.` as structured metadata but originally expected
`Relevant`. This continuation deliberately abstains on that explicit disclaimer;
it does not rewrite the old expectation or describe a replay as fresh independent
review. Updated source/test hashes and final validation are recorded separately
in `evaluation/bilingual-preparation-512-followup/validation.json`. The earlier
independent review applies to the earlier source, not to these follow-up edits.

Follow-up source `a058277` passes the full locked offline workspace: **1,468
passed, 0 failed, 6 existing ignored**. The eight public #512 test groups, all
288 existing Launch controls, strict workspace Clippy, formatting, the 20 standard
fixtures and the 13-file historical archive verification pass. All 21 new
regressions now remain `Ambiguous`. Prior reviewer expectations match on 132/133
rows, with only the intentional row 118 difference above; historical v16/v29
outputs match on all 133 rows, and four Launch control outputs are unchanged.
These follow-up checks are local deterministic validation, not new independent
review, live-provider acceptance or a release.
