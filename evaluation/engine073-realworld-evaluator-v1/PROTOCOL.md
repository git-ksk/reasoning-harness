# Prospective independent real-source evaluation v1

This is a successor evaluator, not a published Engine change. The production
candidate is ab3c632 (package 0.7.2); no 0.7.3 release is implied. Original runs,
F1 scores, source corpora and component implementations remain immutable.

## Ordering and scope

M0 is complete only when the protocol, rubric, scorer, execution bounds, actual
paired hosts, complete runtime image and independent implementation review are
frozen together in an external COMPLETE_FROZEN receipt. Image bytes include
Python/stdlib, native libraries, exact provider patches and both binaries. Before
M0, engineering fixtures are allowed; fresh accuracy questions/gold are not.

After M0, an independent agent authors exactly 24 real-document tasks, disjoint
from AWS/Lambda/S3, Python, Node and Kubernetes development material. At least
12 documents across six other products are required. Each case has two exact,
unmutated source records and a question with explicit scope/as-of. Official
immutable raw GitHub commit URLs and complete captured context are used. The
24 cases cover compatibility, conflict, scope, revisions/history, duplication or
unknown lineage, missing/unrelated evidence, wrong targets and untrusted control.
Each track requires at least eight clear positives and one clear negative.
Questions are fixed independently of candidate output. M1 freezes all source
bytes, visible inputs, private gold, host policy and their M0 provenance.

The same 24 cases run on 0.6.1 and ab3c632 for each of Mistral, Google and Groq,
in two separately reported tracks: 3 x 2 x 2 x 24 = 288 answer observations.
Each provider schedules 48 normal CLI jobs and 24 helper jobs. The helper uses
one untrusted adviser response replayed into both actual Engine hosts, isolating
Engine behavior. It does not measure two separate end-to-end helper model runs.
Neither cross-provider pooling nor helper gains can satisfy normal-track gates.
Source facts/admission and helper authorization are explicitly host curated.
They are not candidate model truth, general autonomous research, or human review.
The normal CLI executable uses the same released common CLI main and provider
host on both arms, with identical audit-only retry/pacing/budget modifications.
The candidate core is a normal non-test build. Version alone does not activate
Engine features. A missing normal integration can validly yield NO_GO.

## Annotation rubric

A separate agent reviews complete normalized document bytes and displayed output,
with provider/arm hidden where feasible. Each row binds its full observation hash
and the pre-frozen gold-record hash. The agent segments every UTF-8 display byte;
only factual spans get claims. Each displayed fact occurrence has its own claim
ID, including repetitions on another line/card. Proposition IDs may repeat and
unique-proposition arithmetic remains the preserved metric's definition.
Nonfactual spans include formatting, source labels/IDs, explicit withholding,
questions, and disclosures without new factual assertions. Factual explanations
inside a withholding still count and require support/citations. Never omit an
inconvenient factual span to improve precision.

Each grounded claim lists the acquired and admitted source/version plus a
full-context support rationale, and only satisfied frozen gold groups. External
review handles negation, version, scope, time, correction and source lineage;
the scorer does not infer support from matching words. Unknown or disputed
support remains unreviewed/incomplete rather than auto-grounded. Wrong scope,
untrusted promotion, hidden material conflicts and wrong as-of are hard safety
violations even when a sentence sounds plausible. Every actual citation attachment
edge overlapping a fact is reviewed. Internal evidence IDs are not citations.
Literal fidelity compares exact captured quote bytes; it is separate from whether
the quote supports the claim in full context. A faithful irrelevant quote fails
support. Claim citation coverage, edge precision and quote fidelity are separate.
Zero applicable facts/edges/quotes means undefined, never an invented score of 1.

For binary case quality, positive means a full or qualified partial answer is
possible from the fixed sources; negative means an affirmative answer is not
justified. Ambiguous labels stay outside the binary denominator. A positive
answer counts as a useful accept only when all required groups, unresolved facets,
disclosures, source support are satisfied, with an eligible
Engine terminal status. Citation quality is measured separately; missing displayed citations blocks the citation gate and complete-useful-answer gains, while supported-answer case F1 remains descriptive. The unchanged final GO requires both. Otherwise it is not an accepted supported answer. An unsafe affirmative
on a negative or ambiguous case remains an accept (FP/hard failure), never TN
merely because the scorer cannot ground it. Unsafe factual assertions trigger the
hard gate independently of binary class. A correct, explicit withholding can be
TN on negative cases; abstention on an answerable positive is FN. Every partial,
failed, cancelled, suppressed, unreviewed or missing row remains incomplete and
prevents completion. It is never relabeled as a semantic TN. Partial visible bytes
and raw artifacts remain available for safety audit; unknown binding is not proof
of a safety violation. Review inconsistency blocks scoring rather than guessing.

Claim precision = supported unique propositions / displayed unique propositions.
Claim recall = satisfied required groups / required groups. F1 is their harmonic
mean. Display occurrence and citation audit counts are also reported. Per-case,
per-family, per-track/provider/arm and pooled descriptive values must retain their
denominators; no population accuracy claim is supported by this small correlated
24-task sample. Independent-agent judgment is not human attestation.

## Gate crosswalk (no finish-line movement)

The case gate stays F1 >= .90, precision >= .95, recall >= .85 for every provider
and track, all 24 complete. F0.5 >= .95 remains a stretch goal, not a gate. Before
case authorship, the claim-level practical gate is fixed to the same .95/.85/.90
thresholds. All hard safety categories must have zero violations and no known
unrepaired critical defects. Citation edge precision, coverage and applicable
literal fidelity must each be 1; null applicability does not create credit.

Original paired utility gates are preserved: at least two net additional useful
normal answers per provider, at least two net authorized helper improvements per
provider, and zero positive-control regressions. Helper utility concerns declared
review-qualified answers, not canonical truth promotion. Original efficiency
gates are preserved exactly: mean logical model calls added <= 1, aggregate token
ratio <= 1.25, p95 elapsed ratio <= 1.5. A proposed median-ratio replacement was
rejected before M0. Ratios need all 24 pairs; zero/unknown baseline is unavailable.

The current ordinary canonical CLI exposes no citation objects. This known integration limitation is predeclared: do not manufacture citations from internal IDs, suppress correct semantic F1 solely for absence of citation objects, or claim final GO while the citation/usefulness gate fails. The evaluation may conclude NO_GO and identify CLI integration work.

Normal bounds remain six logical operations, 12 total provider attempts,
24,000 input/4,096 output tokens, four acquisitions/two reacquisition rounds and
180s including pacing. The evaluator tightens adapter HTTP attempts to one per
generation (12 generations maximum), no automatic retry/model/tier switch.
Before every request, observed usage plus serialized UTF-8 request bytes +512
input allowance and requested output cap must fit the aggregate reservation.
Unknown/mismatched usage stops the lane. This is a conservative local budget,
not a guarantee about a provider's invoice or undocumented tokenization. Full
wire body/settings and returned usage are retained, secrets excluded. Helper is
one generation/attempt, 4,096 input/384 output, 100s. Generation starts are at
least 10.5s apart, provider concurrency is one. Durable pre-auth reservations,
inflight persistence, no refunds, exclusive first-run output and process-group
cancellation are inherited from the frozen reviewed scheduler. The container's
external lifetime/cleanup handles parent interruption separately.

Groq's historical depletion record is never reset in place. A separate pinned
recovery decision must combine new owner testimony (no other usage, full balance),
actual tier/org headroom and a verified full-cohort capacity budget. Headers that
say token limit are TPM, not daily remaining tokens. A low-cost successful probe
cannot establish full daily headroom. Until recovery passes, its required lane
is suppressed/incomplete; other lanes cannot substitute for it. Free daily quota
may require a separately planned evaluation identity/schedule, never silent paid
upgrade. All three provider lanes are required for a GO decision.

Only the first canonical live run and first external annotation/scoring count.
Do not retry a failed cohort, alter gold, relabel cases or add new cases after
seeing results. Repairs require a separately declared successor identity; the
original first remains intact. Passing completes only this declared quality
scope. Merge, release and CLI distribution/platform acceptance are separate.
