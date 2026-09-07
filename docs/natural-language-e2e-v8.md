# Natural-language E2E v8 — corrected successor independent v0.4.0 measurement

Issue #243 defines `natural-language-e2e-v8` as a fresh successor after the observed v7 corpus wiring defect. v7 remains immutable historical evidence: Actions `34077700963` completed 11/11 cases with operational failures `0` and frozen-v7 correctness violations `0`, but seven external-command cases used fresh resolver sources while their admission source-map keys remained historical v6 identities. The released product correctly failed closed with `untrusted_source`; the intended acquisition/admission dimensions therefore were not measured. v8 does not repair, rescore, or rerun v7.

## Frozen identities and product coordinate

- corpus: `natural-language-e2e-v8`
- evaluator/report: `reason-natural-language-e2e-v8`
- scoring: `natural-language-e2e-scoring-v8`
- product tag: `v0.4.0`
- product commit: `50c750d976be63b4e489ba5d7d7f3225bdd839b8`
- natural output: `reason-natural-output-v4`
- session contract: `reason-session-v1`
- MCP adapter: `mcp_readonly_v3`
- provider/model: Mistral / `ministral-8b-latest`
- base seed: `47000`
- max tokens: `1024`
- inter-case pacing: `1500 ms`

The product source under measurement is unchanged from the release. All v8 changes are measurement-only.

## Successor corrections

v8 retains the v7 evaluator fixes:

- `RequiresVerification` uncovered propositions are blocked diagnostic/control state and are reported separately as `blocked_unverified_propositions`; they are not counted as unsupported final structured claims unless an answer-emitting finalization exposes unsupported claims.
- `reason-session-v1` fork scoring checks selected checkpoint reconstruction, independent lineage, source non-destruction, and external replay `0`; source turn finalization inheritance is not required.

v8 additionally closes the v7 corpus-construction gap before live observation.

### Bidirectional historical-marker exclusion

The fresh corpus must not only introduce new IDs/tasks/target keys; it must also contain none of the predecessor `fresh_markers` from observed v1-v7 surfaces. This catches accidental historical source identities embedded inside otherwise fresh configuration.

### Admission-wiring preflight

The no-model/no-network preflight executes every fixture resolver and checks its output against the configured admission contract:

- evidence source identities are inspected directly from resolver output;
- positive/tool/follow-up evidence must be allowlisted and mechanically admissible for freshness, scope, and authority;
- freshness-negative evidence must be allowlisted first, then mechanically stale;
- scope-negative evidence must be allowlisted first, then fail the required scope;
- authority-negative evidence must be allowlisted first, then carry the intended authority mismatch;
- identity-negative evidence must intentionally use a non-allowlisted source;
- no-result capabilities remain valid follow-up inputs without manufacturing evidence.

The frozen v8 preflight expects 10 resolver capabilities, 7 evidence-source checks, 6 allowlisted evidence sources, 1 intentional identity-negative source, 3 positive admissible fixture cases, and 4 intended rejection contracts.

## Fresh corpus

v8 contains eight fresh investigation cases and three fresh session cases. Investigation coverage includes unique-safe selection, ambiguous tool selection, stale rejection, scope rejection, adaptive follow-up, authority rejection, identity rejection, and pinned official GitHub MCP generic-output non-promotion.

The MCP lane uses `README.md` at `refs/tags/v0.4.0`, avoiding the observed v6 `Cargo.toml` and v7 `Cargo.lock` lanes. The official image remains pinned by digest and read-only proof remains mandatory.

## Scoring and live acceptance

The hard correctness gate remains zero for unsupported final structured claims, unsupported/contract-invalid exposed factual text, expected-unknown unsafe grounding, missed insufficiency, identity/authority/scope/freshness bypass, MCP self-promotion, session invalidation, and external replay.

The live v8 gate additionally requires all four intended admission-rejection cases to be observed (`4/4`, coverage `1.0`). This is a measurement-validity gate: it proves the frozen live run exercised the rejection dimensions it claims to measure. Utility such as target recall, grounded target coverage, tool selection, useful follow-up, and false abstention remains separately reported and is not tuned after observation.

Operational failures remain separate from semantic outcomes.

## Freeze and observation discipline

Before first live observation, corpus, SHA-256 manifest, evaluator, tests, scoring identity, provider/model, seed, token/pacing budgets, MCP coordinate, docs, and workflow are frozen and tagged `natural-language-e2e-v8-freeze`.

After the first operationally complete live observation, v8 becomes immutable historical evidence. No post-hoc rescoring or tuning is allowed. v1-v7 remain untouched.
