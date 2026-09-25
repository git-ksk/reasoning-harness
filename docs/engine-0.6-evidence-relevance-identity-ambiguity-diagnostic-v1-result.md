# Engine 0.6 evidence relevance: identity ambiguity diagnostic v1 result

Status: frozen PASS as a diagnostic candidate study. It does not change runtime semantics or accept #462.

## Frozen identity

- issue: #462
- tag: `engine-0.6-evidence-relevance-identity-ambiguity-diagnostic-v1`
- freeze commit: `18a226e5ab0b7d022c587a6c4ee7e57b1c154bb1`
- first/only Actions run: `36029430165`
- run attempt: 1
- corpus: 12 fresh synthetic cases, excluding v7 case-21 names and wording
- trials: 3 matched seeds per provider
- required diagnostic providers:
  - Mistral `ministral-8b-latest`
  - Groq `openai/gpt-oss-120b`

## Result

Both provider arms completed 36/36 observations with zero provider failures.

### Mistral

- primary target-binding exact: 25/36
- primary false `different` on expected-unresolved cases: 11
- baseline disposition exact: 25/36
- one-sided distinctness exact: 36/36
- false `confirmed_different` on unresolved/exact cases: 0
- missed `confirmed_different` on explicit-different controls: 0
- gated disposition exact: 36/36

The primary assessor repeatedly converted open-world identity uncertainty into `different`, especially possible-successor, unregistered-alias, cross-language-name, and truncated-mapping cases. The one-sided verifier consistently returned `not_confirmed`, causing the diagnostic candidate policy to abstain to `ambiguous` rather than reject.

### Groq

- primary target-binding exact: 35/36
- primary false `different` on expected-unresolved cases: 1
- baseline disposition exact: 35/36
- one-sided distinctness exact: 36/36
- false `confirmed_different` on unresolved/exact cases: 0
- missed `confirmed_different` on explicit-different controls: 0
- gated disposition exact: 36/36

The only primary miss was a possible-successor case. The one-sided verifier returned `not_confirmed` and the diagnostic candidate policy recovered the expected `ambiguous` disposition.

## Interpretation

The v7 `21_unknown_rename` miss is not adequately explained as a one-off seed accident. A fresh corpus reproduced the same structural risk: an advisory binding model may over-commit from unknown identity to `different`, and materialization v3 currently gives that advisory negative binding enough authority to force `irrelevant`.

The fresh one-sided confirmation candidate was exact on all 72 provider observations while preserving all explicit-different controls. This is evidence that a negative identity decision should require affirmative distinctness evidence, while unconfirmed rename/alias/successor/lineage mappings should conservatively remain unresolved.

This result does not justify patching case 21 or directly modifying v3 in place. It justifies designing a fresh materialization successor with explicit negative-identity confirmation semantics, then calibrating it under a new identity.

## Next steps

1. Keep v7 immutable FAIL/incomplete.
2. Keep materialization v3 unchanged as historical evidence.
3. Design a fresh successor policy that requires confirmed target distinctness before `target=different` may force `irrelevant`.
4. Preserve operational failures separately from semantic abstention.
5. Before choosing the next required-provider set, complete the separately frozen Google 26-case operational requalification study.
6. Independent holdout authoring remains blocked until a fresh successor calibration passes.
