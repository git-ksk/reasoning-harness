# Engine 0.6 evidence relevance holdout successor v4

Status: design/implementation candidate after immutable holdout v3a FAIL.

## Versioning

- effective qualification: v6
- materialization: v19
- historical v3/v16, v4/v17 and v5/v18 semantics remain unchanged
- v3a replay is a new regression surface only; historical artifacts are not rewritten

## Successor rules

1. Repeated authorized positive identity:
   - strict Harness anchor policy
   - deterministic risk none
   - the same canonical name or authorized alias appears in a title/heading-like signal and a separate excerpt/fact-like signal
   - target occurrence is not comparison/context-only
   - requested relation is locally present
   - both model stages completed and independently agree on requested relation
   - v6 effective identity is exact
   - v19 may materialize Relevant even if model identity votes incorrectly say different/distinct

2. Single-signal near sibling:
   - no target anchor or URL-only target anchor
   - deterministic risk none
   - exactly one identity-capable signal
   - partial target-token overlap plus a distinct non-generic token
   - requested relation locally present
   - no explicit deterministic distinct-target cue
   - v6 forces identity unresolved and v19 returns Ambiguous

3. Context-only target plus repeated sibling:
   - every target occurrence is comparison/context-only
   - a stable sibling subject repeats across separate local signals
   - deterministic risk none
   - v6 may establish distinct-target ownership
   - relation changes to different only if the text explicitly excludes the requested relation and both model stages independently say different relation
   - v19 may return Irrelevant, never Relevant, from this rule

4. URL-only unnamed ownership:
   - only URL carries target identity
   - local text leaves owning product unnamed/unidentified
   - v6 sets unresolved + context_gap
   - v19 remains Ambiguous

## Regression contract

Before semantic freeze:
- focused v19 controls PASS
- immutable v23 replay 48/48 x3
- immutable holdout v1 replay 26/26 x3
- immutable holdout v2 replay 26/26 x3
- immutable holdout v3a replay 26/26 x3 under successor semantics
- wrong-target Relevant stays 0
- workspace tests, Clippy, rustfmt, workflow/YAML and special-case scans PASS
- production branching contains no provider name, fixture ID, synthetic entity name, or exact holdout text

Only after the successor-v4 semantics surface is frozen may a fresh independent holdout v4 be authored.

## Pre-freeze validation

- focused v19 controls: 6/6 PASS
- successor-v4 replay: v23 48/48 x3, holdout v1 26/26 x3, holdout v2 26/26 x3, holdout v3a 26/26 x3
- core: 585 passed / 0 failed
- providers: 153 passed / 1 ignored / 0 failed
- CLI: 368 passed / 4 ignored / 0 failed
- workspace all-target Clippy with -D warnings: PASS
- rustfmt check: PASS
- git diff check: PASS
- workflow YAML parse: 92/92
- production special-case scan: clean
- v3a replay provenance shape: 3 providers x 26 PASS
