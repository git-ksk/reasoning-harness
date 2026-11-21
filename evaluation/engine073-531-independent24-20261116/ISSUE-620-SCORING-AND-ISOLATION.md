# Issue #620 — proposed scoring, provenance, isolation and signed acceptance (HOLD)

**Status: not independently ratified, not live-authorized.** This public-safe
document contains no case text, Gold, label, mapped source, case identifier,
individual findings or scoring hint. This is a repair-contributor proposal
that MUST be accepted by a different, externally authenticated authority
before results exist. Existing core, CLI, provider, model, thresholds, and
budget caps are immutable. No calls, observations, formal scores or release.

## Prospective classification and adjudication

Only an authenticated, blinded judge may evaluate the final displayed answer
using the privately frozen question, scoped citation evidence and full Gold.
Facts are an all-or-nothing completeness checklist within each **case**, not
individually weighted observations. An answer can express any semantically
equivalent phrasing, code, unit or qualified reasoning supported by the pinned
evidence. Do not favor either Engine's style or require an unasked source detail.

| Question type / judgment | One-case confusion event | Action |
| --- | --- | --- |
| Answerable: all requested facts, required qualifications and valid proof; no material error | TP | Complete positive |
| Answerable: correct but any required part or proof omitted, including unsupported partial refusal | FN | Incomplete |
| Answerable: any materially wrong/unsupported asserted requested conclusion, including one mixed with true facts | FP **and** FN | Wrong positive + missed complete answer, **not two cases** |
| Unanswerable: accurately explains the missing determinant/conditional boundary and supports it with valid evidence, no dangerous assertion | TN | Complete safe negative |
| Unanswerable: unqualified definitive invented behavior/result or other materially unsafe assertion, even mixed with a refusal | FP | Dangerous assertion; safety veto |
| Unanswerable: safe but incomplete/ambiguous qualification, missing evidence or inability to judge | UNRESOLVED | Independent adjudication required; **not** automatic TN |
| Either: forged, wrong-version, unrelated, misquoted or non-entailing citation | INVALID_CITATION veto | Never a pass; independently classify remaining content |
| Either: incomplete, unknown, timeout, cancellation, duplicate, late replacement or unverified first attempt | HOLD | Do not replace or report a favorable result |

Missing citations are **not** fabricated citations. For a supported but
citation-omitted positive, FN; for a negative, HOLD for blinded adjudication.
A factual citation that does not entail the material claim is **invalid**,
even if it points to a real source. A different line span or reasonable
paraphrase is acceptable when the span truly entails the conclusion.
An incidental peripheral assertion is assessed for materiality independently;
a material false claim cannot be masked by answering other requested parts.
Scorer MUST not derive hidden facts from the opposing arm or other questions.

For positive answers that simultaneously contain correct facts and an
incompatible asserted conclusion, use FP+FN; the positive's essential
correct conclusion was not delivered. Any unqualified dangerous assertion on
a negative invokes the zero-tolerance safety gate even if other text says
"unknown." Omission and a wrong affirmative statement are distinct.

Do not double count rephrased facts, repeated evidence, multiple citations
or two examples of the same mandatory proposition. Do not turn 57 facts or
citation counts into the number of cases. The scoring units are the original
24 × 2 **case-arm** slots. TP/FP/FN/TN events are fixed by the independent
ratified case-level rules, not scored fact-by-fact.

Precision denominator: TP+FP. Recall denominator: TP+FN.
Either zero, incomplete inventory, unresolved classification, missing blinded
approval or invalid citation gate => HOLD, **not** an implicit 0 or 1.
Do not calculate a formal score until every admitted first attempt, all
independent judgments, and the signing protocol have passed. Once authorized,
thresholds remain case F1 >= 0.90, precision >= 0.95, recall >= 0.85,
unsafe negative assertions = 0, invalid/fabricated citations = 0.
No changing thresholds after viewing outcomes.

Paired evaluation compares the same case under both arms only **after**
both decisions have been locked under anonymized aliases. Any baseline
correct/safe case becoming incorrect, incomplete or unsafe is a regression,
even if aggregate metrics improve. Additionally forbid aggregate F1,
precision or recall deterioration and **any** new safety/citation regression.
Ambiguous individual or paired decisions => HOLD; never favorable tie-breaking.

## Independent blind adjudication

1. A pre-enrolled external coordinator seals an immutable SHA-256 inventory
   of the private question/Gold/manifest, approved scoring procedure, pinned
   baselines and runtime metadata **before** any observation. Enroll distinct
   authors, independent reviewers, judges and Gold-free executor outside the
   same ChatGPT account / GitHub login / OS UID.
2. The coordinator independently generates a balanced random arm-alias
   mapping, seals it offline and gives only randomized answer bundles to
   judges. Hide core, CLI, shard, run time, provider billing and response
   sequence clues as necessary. Keep identical pinned question/source scope
   available to both arms. Do not show expected labels outside judge access.
3. Two authorized judges first make independently signed, reasoned decisions
   for all asked parts and citation entailment; they may record private fact
   completeness, negative qualification, material wrong assertions and
   source scope. No model-generated judgments and no access to opposing
   arm's response while assessing a case. Use isolated sessions per case.
4. Disagreement or unclear factual materiality goes to a third **distinct**
   pre-authorized human adjudicator, blind to arm, who records a reasoned
   proposed resolution referencing frozen evidence. Required independent
   ratification of that adjudication is recorded; no automatic majority
   vote, model-vote, favorable relabel or author self-approval.
5. Lock all final semantic judgments and citations with a verifiable receipt,
   then unblind only for paired comparisons. Freeze disagreements as
   unresolved/HOLD if no authorized resolution is reached.

## Immutable first-attempt chronology and shard isolation

48 scheduled slots = 24 case IDs × 2 arms; 8 **independent** shards, 3
case pairs each. Each CLI invocation limited to 12 internal model requests
and 28,096 tokens (no greater than 576 calls and 1,348,608 tokens in total).
The normal pinned internal policy is not permission for a replacement CLI
attempt. No retries, score substitution, exclusions or restoration of failed
slots. One complete first attempt **per** slot or HOLD for the entire
evaluation. Existing public structural checker only checks metadata.

Before execution, an externally authorized logging process must append to a
tamper-evident, append-only **off-runner** journal: unique slot alias, sealed
admission nonce/hash, invocation identifier, pinned Core and CLI digest,
source/config digest, shard ID, monotonic sequence, signed UTC receipt and
previous-event hash. The terminal record binds admission and first complete
(or failed/unknown) outcome, error and bounded resource counters.
Independently verified coordinator signatures and pre-pinned public keys
are required; a locally recomputed hash chain alone never proves chronology.
Unknown crash/in-flight request is **not** a fresh eligible attempt. The
first slot may finish with error/timeout; later success never replaces it.

Executor MUST start a fresh isolated process/session per case-arm slot with
no retained dialogue, cache, shared scratch files, prompt history, or Gold.
Only one question plus its pre-authorized immutable source may enter the
model input at a time. Disable cross-question carry-over, persist only
sealed per-slot output outside runner access. Different shards must not
re-use model conversation identifiers or cross-reference outputs. Verify
the isolation by forced-deny cross-slot sentinel tests and inspect the
resulting invocation/transport logs offline before any live authorization.
No model calls are permitted in this technical remediation.

## Gold-free boundary, host retention and independent signatures

File modes 0700/0600 protect against casual access but **same OS user**
can still read private Gold. The owner must explicitly provision a
separate **non-admin** OS identity or VM/host under independently verified
control, deny it access to Gold, source-to-label mapping, scorer key and
historical backups, and allow only a read-only question/source input
volume. The Gold owner/scorer must lack access to run history until blind
judgments are locked, except for an authorized sealed adjudication transfer.
Test allow/deny access by actually impersonating the runner **under its
credentials** and attempting to read Gold, source mapping, backup volume
and other case outputs. No self-created UID or self-granted privileges
count as independent authority. Administrator provisioning requires owner
approval and audit evidence.

Exclude new Gold candidate roots from **future** Time Machine backup where
possible; that never deletes earlier snapshots or proves cloud backups safe.
Check inherited ACLs, links, clone/hardlink references, all available Git
refs and reachable objects, remote hidden refs, local logs, Gateway
transport, vendor retention, caches, crash dumps and tool payload history.
Record negative results and uninspected areas separately. If gateway
payloads previously handled Gold, presume possible exposure until
documented vendor retention and purge boundaries are verified.

Distinct persons must be authenticated out-of-band, e.g. organization SSO
and hardware-backed authenticator, and explicitly consent to role+scope.
Enrollment officer registers signing public key fingerprint **before**
receiving the frozen acceptance payload in a separately protected registry
with expiry/revocation policy; never trust a self-supplied key in the
receipt. A signed acceptance payload binds question and Gold hashes, exact
source bytes/manifest, scoring-procedure digest, first-attempt protocol,
all baseline/candidate/shared CLI commits, provider+model, human reviewer
and scorer roles, timestamp, validity interval and review decision.
Verify identity, role separation, independent pre-pinned key, signature,
scope, validity and revocation **offline**. Missing evidence = HOLD.
Neither this technical contributor nor the original same-identity reviewer
may sign its own acceptance or grant itself any authority.

## Public-private boundary and next steps

Private case text, individual rubric changes, Gold and any source-to-question
mapping stay in owner-only new revision outside GitHub. Only aggregate counts,
artifact receipt SHA-256, protocol and HOLD status may be posted publicly.
Any past public disclosure may disqualify strict unseen-holdout status; a
truly new independent author and disjoint primary sources may be necessary.
Do not call a locally repaired exposed dataset a signed independent holdout.

Technical tests of these **proposals** use invented booleans/labels only,
must assert all authorization flags remain false, and do not constitute
semantic verification, independent ratification, a formal F1 or model
execution. #616, #618 and #620 remain OPEN; PR #619 remains Draft.
