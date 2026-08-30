# Versioned benchmark corpus

Reasoning Harness treats benchmark composition as part of the measurement contract. A score is meaningful only when the evaluated case identities, scoring semantics, and corpus compatibility boundary are explicit.

## Corpus v1

`fixtures/corpus/v1.json` is the canonical manifest for corpus version `1.0.0` with score-compatibility ID `corpus-v1`.

The manifest covers **41 active deterministic cases**:

- 20 claim/verdict cases;
- 8 causal-diagnostic cases;
- 5 assumption-diagnostic cases;
- 8 evidence-qualification cases.

Metamorphic fixtures under `fixtures/metamorphic/` are transformation controls, not scored corpus cases, and therefore are not members of corpus v1.

Every case has a stable suite-prefixed `case_id`, its underlying fixture ID and path, a capability/diagnostic category, a difficulty stratum with rationale, scoring mode, provenance, redistribution status, contamination note, and lifecycle status.

Suite-prefixed IDs are required because fixture-local IDs may legitimately repeat across suites. For example, claim, causal, and evidence-qualification cases may use similar local terms without becoming the same benchmark case.

## Score compatibility

`corpus_version` identifies a published manifest revision. `score_compatibility_id` identifies whether aggregate scores may be compared directly.

Two runs are directly score-compatible only when their manifests have the same `score_compatibility_id` and the evaluated active case set/scoring contract is the same for the reported metric. The runtime exposes this ID rather than inferring compatibility from version-string ordering.

A metadata-only correction may increment the corpus patch version while retaining the compatibility ID if it does not alter case membership, fixture semantics, expected labels, scoring logic, or strata used by the reported metric. Any change to active membership, expected outcomes, semantic fixture content, scoring mode, or an aggregation-relevant category/difficulty assignment requires a new compatibility ID for full-corpus comparison.

When incompatible corpus versions must be compared, reports must either:

- compare only an explicitly identified stable-case intersection using unchanged scoring semantics; or
- present the versions as separate measurements.

A new version must never silently overwrite historical scores.

## Change discipline

Stable `case_id` values are never reused for a semantically different case.

When adding a case, create a new manifest version and assign a new stable ID. When changing a case in a way that changes what is measured, create a new case ID or mark the old entry `superseded` and point `superseded_by` at the replacement. Deprecated or superseded entries remain part of the historical manifest record; old manifests are immutable after publication except for an explicitly documented repository-integrity repair that does not alter benchmark meaning.

Removing an active case, changing its expected label, changing deterministic oracle semantics, or moving it between score-reported strata changes the full-corpus measurement and therefore requires a new compatibility ID.

Fixture-path moves that preserve exact semantic content may retain the case ID, but the manifest revision must record the new path and tests must prove that every active manifest entry resolves to a fixture with the matching fixture ID.

## Category and difficulty reporting

Recorded claim evaluation reports the existing overall `BenchmarkComparison` unchanged and adds category and difficulty slices derived from the manifest. Each slice reuses the same benchmark aggregation logic; it does not define a second correctness implementation.

Live repeated-trial output records `corpus_version` and `score_compatibility_id` for reproducibility, but does not synthesize pooled category/difficulty accuracy from partial or repeated observations. Repeated correctness continues to use the existing complete-trial semantics under `stability.correctness`.

The current difficulty names are `basic`, `standard`, and `stress`. They are benchmark strata, not universal claims about task difficulty for every model. Their rationale is stored per case so future changes can be reviewed explicitly.

## Contamination and redistribution posture

Corpus v1 is synthetic repository-authored material and is marked redistributable. The project cannot prove that a public model has never encountered these fixtures after publication, so the manifest records that limitation rather than claiming perfect decontamination.

The project does not scrape proprietary training corpora or infer secret training membership. A future imported benchmark case must record its provenance and redistribution status before inclusion. Restricted material must not be exported as though it were redistributable.

Provider credentials, private user data, model API secrets, and private application data are not corpus metadata. Public manifest validation checks the committed manifest for obvious provider/credential coupling and required CI remains credential-free.

Repeated live results are observations about models under a versioned corpus; they are not evidence that the corpus is uncontaminated.

## Saturation warning policy

A deterministic recorded fixture suite reaching 100% is a regression result, not evidence that a model benchmark is saturated.

A claim stratum becomes a **saturation candidate** only after at least three independent model families each reach at least 95% harness accuracy on that unchanged stratum across at least five operationally complete live trials. The report must still inspect unsafe accepts and class-specific failure modes before calling the stratum saturated.

When a stratum is a saturation candidate, do not mutate the old cases merely to lower scores. Preserve the old corpus and add harder or more discriminative cases in a new corpus version. If active membership or scoring meaning changes, assign a new score-compatibility ID. Historical results remain comparable within their original contract.

## Resolution-loop baseline identity

Future bounded-resolution research must reuse the same stable base `case_id` across direct-generation, diagnose-only, and resolution variants. Resolution outcomes and costs are additional observations over that base identity; recovering a previously unknown case must not remove or replace the original denominator.

This rule is a prerequisite for measuring whether resolution increases grounded answerability without increasing unsafe final answers.
