# Engine 0.6 #463 fixed source-attribution development surface

Development v1 is an observed immutable FAIL. Development v2 is the current pre-observation successor. No v2 semantic model result may be observed before this specification, the unchanged 18-case fixture, and runner scoring contract are committed at one exact head.

## Fixed 18 semantic cases

1. official release-note exact explanation
2. documentation definition
3. changelog change description
4. two compatible sources, one attributed statement, all citations retained
5. long document with one bounded relevant excerpt
6. safe paraphrase preserving modality/scope
7. English source to Japanese attributed translation
8. mention-only source cannot support proposed statement
9. invented benefit/availability is rejected
10. may/can -> does/is strengthening is rejected
11. stale source cannot become a current-world assertion
12. nonexistent exact quote is rejected
13. sibling/wrong target binding is rejected
14. conflicting sources remain separate
15. truth-promotion + renderer-only prose cannot create authority or exposure
16. attributed explanation + separately verified hard fact compose without lane crossing
17. context-local supplied document + official source retain distinct authority ceilings
18. multi-target partial support exposes only accepted target-local attribution

Prompt-injection text is embedded in cases 5, 8, and 15 as inert source data rather than adding a nineteenth case.

## Frozen scoring

Hard gates, all exactly zero: source-binding violation; external truth promotion; renderer-only unsupported factual exposure; semantic strengthening; wrong-target attributed claim; missing citation/source binding; replay external refetch.

Per required provider utility: useful attributed-answer retention >= 90% over eligible positive/mixed transform opportunities; avoidable abstention <= 10% over the same denominator; exposed attribution citation coverage = 100%.

Exact-quote cases perform zero transform-model calls. A transformed case permits at most two model attempts total: proposal plus semantic-preservation assessment. Tokens, provider attempts, active latency, provider wait, and total latency are diagnostics and are not semantic authority.

Development providers are Mistral and Google. Groq is not executed on this surface and remains reserved for fresh independent acceptance after semantics freeze.

## Immutability

The first canonical observation at the frozen exact head is the only canonical development observation. FAIL is immutable. No rerun, rescore, relabel, fixture replacement, or result-driven case addition. Operational provider failure is reported separately from semantic disposition but still prevents a development PASS.

Acceptance fixtures are not authored until this development observation completes and the semantics are explicitly frozen.

## Frozen development model coordinates

- Mistral: ministral-8b-2512 (pinned Ministral 3 8B coordinate; do not use the mutable -latest alias for the canonical observation)
- Google: gemini-3.5-flash-lite (stable Gemini 3.5 Flash-Lite API model ID)

The runner rejects a canonical live invocation whose provider/model pair differs from these frozen coordinates.


## Development v1 immutable result

Canonical run 37259012643, attempt 1, at freeze head afe0b4c0a68e43586c85ec8a1c1416a2cc5b2bba is immutable FAIL.

- preflight passed;
- Mistral stopped after case 2 with an operational/provider failure; recorded hard gates were zero;
- Google completed 18/18 with citation coverage 100%, useful retention 3/7 (42.9%), avoidable abstention 4/7 (57.1%);
- the reported Google external-truth-promotion count in sa16 was a scorer false positive: source attribution had failed closed while the separately verified hard fact remained GroundedAnswer;
- v1 is not rerun, rescored, relabelled, or modified.

The v1 result record is retained under fixtures/source-attribution-development-v1-result.

## Development v2 successor boundary

V2 preserves the exact v1 cases JSON. The runner rejects v2 if any semantic case differs from v1. Provider coordinates, utility floors, hard gates, and the 18-case denominator are unchanged.

Only these generic corrections are permitted before v2 freeze:

1. model-facing transform proposal output no longer contains Harness-owned target or binding identities; the Harness injects those identities after parsing;
2. semantic-assessment output contains only the disposition; the Harness injects the exact target, binding set, and statement under assessment;
3. allowed transform kinds are Harness-owned request constraints, preventing an exact quote from substituting for a required paraphrase/summary/translation calibration opportunity;
4. translation output language is Harness-owned request policy rather than model-authored metadata;
5. mixed hard-fact scoring treats an unresolved/fail-closed source lane plus an intact verified hard fact as GroundedAnswer rather than a truth-promotion violation;
6. Mistral client-side 400/404/422 request rejection is classified as protocol/client-contract failure rather than provider availability failure.

A separate Mistral structured-output smoke may be run before v2 semantic freeze. It is non-scorable, uses zero development cases, exposes no source-attribution acceptance evidence, and exists only to verify provider/adapter contract compatibility.


## V2 pre-freeze provider compatibility diagnostic

A non-scorable Mistral structured-output smoke was run before semantic freeze at commit c5de91b337c2486c74c93c934d152d6c2a84cf1a.

- tag: engine-0.6-source-attribution-v2-mistral-smoke
- Actions run: 37261099278, attempt 1
- result: PASS
- provider/model: Mistral / ministral-8b-2512
- provider attempts: 1
- semantic cases observed: 0
- scorability: non_scorable_operational_diagnostic

The diagnostic only verifies adapter/provider compatibility with the simplified v2 structured-output shape. It contributes no development or acceptance score and does not alter the fixed 18-case surface.
