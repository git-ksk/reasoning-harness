# #620 prospective formal aggregate scorer candidate — technical proposal / HOLD

Version: 620-formal-scorer-v1-proposed. This is an implementation candidate,
not an authenticated independent approval or a usable formal scoring entrypoint.
The accepted limited synthetic contract remains unchanged. The actual ratified
formal scorer identity remains UNSET until a distinct authorized authority
reviews and signs the exact executable inventory and the authenticated adapter.

## Input and aggregation
The pure reference algorithm accepts only invented synthetic records and an
explicit synthetic purpose. Synthetic case identifiers must start with
synthetic-. This namespace is an accident-prevention guard, not proof of origin.
No runner, file loader, live workflow, keys or external provider transport exist.
The inventory has 24 unique cases, 16 positive and 8 negative, a pinned fact
count per case, and exactly 48 unique case-arm first-attempt observations.
Every slot binds a unique admission and judgment digest plus a terminal digest;
these syntactic checks do not authenticate any digest or journal. Unknown fact
or atom values are distinct from confirmed false. Confirmed true negative
requested wrong assertions imply unsafe assertions; peripheral errors alone do
not. Input must not supply legacy verdict, approval or quality flags: derive
events and vetoes from the reviewed guard instead.

Positive wrong assertions count FP+FN within one case-arm slot, not two cases.
Mandatory facts are not separate observations. Calculate precision TP/(TP+FP),
recall TP/(TP+FN), and Case F1 2TP/(2TP+FP+FN) in exact rational arithmetic for
each arm. Zero denominator, missing/extra/duplicate observations, conflicted or
incomplete judgments, missing provenance structure and unresolved pairs withhold
metrics. Independently confirmed invalid citation and unsafe-negative vetoes are
deduplicated by case-arm slot and category; both may be present in one slot.
Duplicate slots cannot establish paired judgments. Vetoes are retained for
syntactically valid records despite unrelated uncertainty. Malformed records
are not evidence; repair of an invalid submission is not a replacement model
attempt. Zero-tolerance violations on either arm block scoring; retain separate
counts. No actual F1 is emitted.

Candidate thresholds stay F1 >= 0.90, precision >= 0.95, recall >= 0.85; unsafe
negative and invalid citation zero. Compare all three exact aggregate ratios
against baseline, and separately compare each paired case for content, safety
and citation deterioration. Any paired regression fails even if the aggregate
improves. Unknown comparisons cannot be deemed non-regressions. A synthetic PASS
only states that invented inputs satisfy these numerical checks; quality_go,
release_go, formal_score_authorized and live_authorized are always false.
NOT_EVALUATED and the compatibility verdict are never scoring inputs.

## Independent ratification handoff
Before actual observations, an independent authorized scorer and adjudicator
must ratify the precise policy, including the three existing private annotations,
and the exact version/digest of the complete eventual executable, dependency
implementations, test inventory and input schema. Independently implement and
verify the authenticated admission adapter: pre-enrolled external trust registry
and signatures, role/scope/expiry/revocation, immutable candidate pins and
case/fact inventory, independently confirmed atomic evidence and dispute
resolution, signed first-attempt journal and immutable terminal response binding,
arm-blind locked judgments before authenticated unblinding, and genuine
Gold-free execution/access separation. A caller-supplied purpose or digest
cannot bypass this. Do not retroactively classify unverified atom claims as
confirmed. No private cases, Gold, historical model outputs or formal metrics
were used to validate this proposal. Keep #616/#618/#620 OPEN, PR #619 Draft.
No authorization, observation, formal scoring, merge or release.
