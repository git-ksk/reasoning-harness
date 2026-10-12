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
  bounded dotted organization tokens such as `Z.ai` and `Ruzani.Labs`.
  Repeated exact makers before `has made`, and single-dot alphabetic organization
  tokens with a suffix of at least two letters, retain their internal periods
  regardless of suffix casing.
  Target-edge periods, trailing abbreviation periods, unknown multi-dot continuations,
  semicolons, newlines, and ordinary/full-width sentence boundaries stay separate.
- All v17 launch proofs require a directly owned subject or bounded direct
  object. Recognized prefixes include `the` and `the latest version of`;
  documentation or a competitor before the target cannot own its predicate.
  `has made … generally available` objects allow bounded product qualifiers
  and vendor modifiers. A finite clause inside a maker modifier cannot supply
  a separate owner's availability. Explicit falsification is a negative control.
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

The v17 proof no longer falls back to the historical permissive subject matcher;
v16/v29 still retain that historical behavior. Current unsupported subject
introductions may abstain rather than borrowing authority from another noun.

For locally proven positive launch frames, v30 uses v17's authority floor with
the existing v22 identity/scope/materialization gates. Reapplying v23's historical
v11 relation vocabulary would discard a recovered dotted-name frame. Other
relations and unresolved/negative launch cases retain the v23 composition.

## Validation

`launch_source_frames_514_515.rs` includes the public reproducers, additional
synthetic development controls, cross-sentence/signal and sibling negatives,
aliases, Unicode text, metadata-only rejection, missing bindings, identity/scope
guards, and historical compatibility checks.

Local validation commands:

```sh
cargo fmt --all --check
cargo clippy --offline --locked --workspace --all-targets -- -D warnings
cargo test --offline --locked --workspace --no-fail-fast -- --test-threads=1
cargo run --offline --locked -q -p reasoning-harness-cli --bin reason -- eval fixtures --format human
python3 scripts/verify_engine_0_7_independent_v1_archive.py
```

Final source `506e7b1` local results on Rust 1.88 / macOS: format, strict
locked offline all-target workspace Clippy, and diff checks PASS. Full workspace
excluding the unseen v9 test passed 1,458 tests / 0 failures / 6 existing ignored;
the v9 integration test then separately passed on identical source with all
32 controls correct. Combined test results are 1,459 PASS / 0 FAIL / 6 existing
ignored. All 256 previously observed v1-v8 regression controls pass unchanged.
The standard 20-case fixture evaluation has accuracy 1.000 and zero unsupported
accepted claims. The original observation archive retains all 13 exact SHA-256
checks and its frozen score. OAuth loopback tests required execution outside
the sandbox's local-listener restriction, with Cargo kept offline.
Exact commands, source hash, counts and corpus hashes are retained in
`evaluation/launch-514-515-final-validation/local.json`.

## Independent review and fresh holdout protocol

An independently instructed subagent with no inherited conversation authored
`fixtures/launch-514-515-independent-v1/manifest.json` without implementation,
test, documentation, history, or engine-output access. The source candidate
`a6b6f94` and evaluator `e3959f5` were frozen before authoring; the exact 32-case
corpus (12 positives, 20 negatives) was committed at `32b67df` before evaluation.
Its SHA-256 and authorship constraints are retained in the fixture provenance.

The **first observation failed: 30/32**, with all 12 positives recovered and two
false-positive documentation subjects. That original result is preserved in
`evaluation/launch-514-515-independent-v1-first-observation/`. The labels were
not changed. A retrospective baseline replay on `a48a33d` has seven false
negatives and one pre-existing documentation false positive (24/32 correct);
source candidate `a6b6f94` recovers those seven positives but adds the other
documentation false positive. The corrective subject check addresses both.
This corpus became a regression set after source correction and is no longer
counted as a fresh acceptance holdout.

A separate read-only reviewer found two introduced P2 classes in `a6b6f94`:
indirect subjects such as documentation/competitors, and explicitly falsified
launch propositions. Both have direct regressions in the implementation test
suite. The first correction was frozen at `d10cbad`. A second reviewer pass verified
those four direct examples and found three additional issues: an enclosing
rejection around `has made`, harmless date adjuncts being rejected, and May
being treated as a modal instead of a month.

The independent v2 corpus was authored by a new subagent with no inherited
history, after evaluator `1699c56` was frozen, and committed at `3fd9727` before
execution. Its **first observation failed: 25/32**, with all 20 negative controls
passing but seven affirmative recall misses (active launch, trailing conjunctions,
and completed-preview context). The original outcome and labels remain in
`evaluation/launch-514-515-independent-v2-first-observation/`; v2 is now also a
regression set rather than a fresh acceptance gate.

Source `48c97d9` corrects these findings. Active `launched` requires an intransitive
temporal/use tail rather than an object that the target launched. A leading
bounded temporal adjunct is allowed before the directly owned product subject.
A numeric May date is distinguished from a modal. Completed preview clauses
are removed only from the proof copy; current preview status stays blocked.
Trailing conjunctions cannot supply a subject or direct object, and embedded
`that … has made` propositions cannot independently establish launch authority.
The v3 first observation evaluated 29/32 cases before an author-supplied `title`
kind failed the API enum; one evaluated positive missed a completed private
preview. The raw corpus remains byte-identical. A separately hashed schema
projection maps only that signal to `source_title`, without changing any text or
label. Original log/results are retained under the v3 first-observation folder.
V3 is a regression set, not a successful fresh acceptance run.

The third review pass verified prior fixes and identified a delayed direct object
after an active-launch time adjunct, a numeric speculative `may` misread as May,
and enclosing doubt without optional `that`. Source `6c5f914` fixes these with a
complete bounded adjunct tail, date-position checks, and enclosing doubt guards.
All 96 v1-v3 controls then passed as regression.

V4 was frozen at `0fdea0b` after evaluator `94083cd`; its first result was 31/32,
with all 20 negatives passing and one uppercase dotted maker false negative.
The final review pass on `6c5f914` additionally found three explicit simulation
phrases gaining factual authority. Both classes were corrected at `b735598`.
All 128 v1-v4 cases pass as regression; no labels were changed. V4's first
result is retained separately. A review of `b735598` then reproduced an unspaced
boundary such as `from copper.Labs are generally available`, which lent the
laboratories' availability to the product. `2cf7add` excludes common finite copulas, with six direct regressions; all 128
cases still pass. Re-review found that unlisted verbs (`offer`, `stay`) still
entered the modifier. `7658e6b` instead requires the complete maker suffix to
match its original lexical span: one opaque raw token, or a bounded multiword
proper name also used as the raw subject before `has made`. Three exact
regressions and a multiword maker control were added. All 128 regression cases
and eight public test groups pass. Re-review then exposed dash-separated
clauses inside one whitespace token. `5c15e00` confines raw maker lexemes to
alphanumeric characters and internal periods, with four direct regressions.
`5c15e00` re-review exposed possessive/parenthetical ownership. `32486c4`
validates the complete original made-object immediately before a bare
`generally available` predicate, ties it to the nearest raw `has made`, and
requires a complete bounded availability tail. Possessives, punctuation asides,
attributive equipment, and other frames cannot supply that proof. Unsupported
punctuated or unconfirmed multiword modifiers and unrecognized tails may abstain.
Re-review of `32486c4` identified equal raw objects being reused across sentences,
including a denied second sentence authorizing an invalid first one. `9077f84`
keeps original and normalized text paired at each actual statement boundary;
raw made-object authority is derived only within that same segment. Four order
and punctuation regressions cover this isolation. Compound audience/region
wording and `and registration opened` may still abstain under the bounded tail.

V5's evaluator was frozen at `8cc8909` and its exact raw 32-case corpus at
`6dc5d2e`, before any evaluation or case-content inspection by the implementer.
The raw corpus metadata names the original `b735598` source. The review-only
corrections through `9077f84` were made while v5 remained unseen and unevaluated;
results record the actual evaluated source separately, without rewriting the
corpus metadata, text, or labels. Any correction based on v5 outcomes would
make v5 a regression set and require a new unseen holdout.

Final review of `9077f84` found no introduced P1/P2 findings: all 115 prior probes
and 59 additional isolation checks passed, along with the eight public groups.
The **v5 first observation passed 32/32** (12 positives, 20 negatives; zero false
positives or false negatives), without any source correction based on its cases
or output. Raw corpus SHA-256 is
`edd097b458777fac2f0fc33e1a076cfbb25c571aba67963f1041108c12d9586d`.
The original log and per-case result are retained under
`evaluation/launch-514-515-independent-v5-first-observation/`.
The subsequent full-workspace run found one pre-existing positive control
(`with no usage restrictions`) had become Unresolved under the bounded tail.
`21ae38e` restores that exact qualifier without changing the test. V5 therefore
becomes a 32-case regression set, preserving its successful first observation
on `9077f84`. V6 evaluator `f7cb9d8` was frozen for a new unseen acceptance set.
The next frozen legacy matrix exposed two more qualifier misses (`to all
workspaces`, `with no extra fees`). `4d4142c` restores those bounded forms,
without altering frozen controls. V6 raw corpus was committed at `e66b539`
before any case-content inspection or engine execution. Its declared source
remains `21ae38e`; the actually evaluated source is recorded separately for
legacy-test-only corrections while the corpus remains unseen. V6 SHA-256 is
`45953cc0417bcdd362f2abcc8c1a9fb9bfe8400858a29076a3431db5f3c8e54b`.
The full `4d4142c` legacy workspace passed (1,454 tests, zero failures, six
existing ignored, v6 skipped), and independent review found no introduced P1/P2.
The **v6 first observation was partial and failed: 29/30 evaluated cases correct**,
with one affirmative dotted-maker miss and two unexecuted cases after an invalid
`navigation` enum. The authoring specification supplied that unsupported enum;
this is not a successful acceptance run. Its exact first log/results are retained
under `evaluation/launch-514-515-independent-v6-first-observation/`.
A separately hashed schema-only projection maps `navigation` to
`navigation_or_footer`, `url` to `canonical_url`, and a nonfactual `scope_hint`
to `heading`. All original texts, targets, order and labels are unchanged.
`d1b76ba` fixes the positive miss by recognizing alphabetic single-dot maker
suffixes without depending on casing or a finite organization-designator list.
Complete original object, availability tail and same-segment checks still apply;
a direct negative covers `from copper.Works are generally available`.
V6 is now a regression set. A fresh v7 evaluator was frozen at `730a36a`
before a new independent author received only the bounded feature contract and
the seven actual signal enum values. Final independent review of exact `d1b76ba` found no introduced P1/P2 findings:
eight public groups, 202 replayed inputs/checks and 30 new casing, ownership and
boundary assertions passed using an isolated target. Source bytes were verified
against SHA-256 `3e8b9756553b8abb2ba6a07e844437af46a5d246db38bf07c68bfb1cfb66db70`.
The exact `d1b76ba` workspace passed (1,455 tests, zero failures, six existing
ignored, v7 skipped). The **v7 first observation failed: 25/32**, with zero false
positives and seven affirmative misses: plain `is generally available`, bounded
region names, a complete included-feature adjunct, `Release confirmed`, and
`as of` date/clock adjuncts. Its original result, corpus and labels remain
unchanged. V7 is now a regression set.
`23284e2` adds these bounded forms with complete tail checks for the new direct-GA
predicate and made-object frames. The exact confirmed prefix cannot authorize
a documentation/competitor subject; complete time/region and included-feature
adjuncts cannot hide a delayed object. Nine public groups pass, and the full
workspace passes 1,457 tests with zero failures and six existing ignored, with
the unseen v8 test excluded. All 224 v1-v7 regression cases pass.
The fresh v8 evaluator was frozen at `c8cd13f` before independent authoring;
the new reviewer and author remain independent of each other and prior
corpora. Before any v8 case-content inspection or evaluation, root code review
noticed that `as of May 6` was accepted by the temporal parser but rejected by
the outer modal-May guard. `836cd0c` adds only the exact preceding `as of` pair
to the existing numeric date exemption, retaining duration and modal checks.
Two positive and two negative public controls cover this consistency fix.
The raw v8 source metadata remains `23284e2`; actual evaluated source is recorded
separately. Independent review of exact `836cd0c` found no remaining introduced P1/P2:
nine public groups and 107 independently authored temporary probes passed.
The exact full workspace also passed 1,457 tests, zero failures, six existing
ignored, v8 excluded. The **v8 first observation failed: 30/32**, with zero false
positives and two affirmative misses (direct-GA `with no extra fees`, and a
leading `11:35 UTC today` without `as of`). Original first log, labels and raw
corpus remain unchanged. V8 is now a regression set.
`506e7b1` permits only the exact terminal affirmative no-fees/no-restrictions
phrases in successor direct-subject checks; historical permissive matching is
unchanged. A bounded numeric hour/minute plus UTC/GMT and today/yesterday can
introduce the directly owned subject. Negative public controls cover modal
negation, wrong subjects, delayed objects and statement boundaries.
V9 evaluator `51c40b0` was frozen before fresh independent authoring. Final
independent review of exact `506e7b1` found no introduced P1/P2: the public
suite passed 9/9, 165/166 reviewer probes matched their initial expectations,
and historical v16/v29 outputs were unchanged for all 166 probes. The one
low-severity observation is a malformed negative clock (`-1:00`) losing its
minus sign during normalization; the explicit target-owned affirmative GA
statement remains. This is not strict raw timestamp validation and did not
borrow another subject's predicate. The full regression then passed 1,458 tests, zero failures and six existing
ignored with v9 excluded. V9's exact raw corpus was frozen at `7dfb7f2` before
case-content inspection or Engine evaluation. Its **first observation passed
32/32** (12 positives, 20 negatives), with zero false positives/negatives and
no source correction based on v9. Source remained exactly `506e7b1` throughout
freeze and observation. Raw SHA-256 is
`96f4c99d9f02dc2d7cfcc90489d00060fc0886c265b72f80d833057c6fa18b68`.
Original log and per-case results are retained in
`evaluation/launch-514-515-independent-v9-first-observation/`.

These are independently authored synthetic deterministic controls and subagent
code review. They do not constitute live-provider observations, external human
review, a published patch, or CLI adoption. Exact-head cross-platform CI must
also pass before merge. The separate #512 / PR #513 General-relation candidate
is not included here.

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

The subsequent #512 correction and its validation are tracked in
`docs/bilingual-preparation-512.md`; the table above records the earlier
`0cc3e06` candidate rather than a result on that successor.
