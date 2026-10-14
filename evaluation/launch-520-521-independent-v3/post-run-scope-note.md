# Post-run scope clarification — P323 (no rescoring)

After receiving the complete v3 RED result, root clarified the existing v17/v30 strict local launch floor: the policy-owned canonical name or a Harness-declared alias must own the affirmative launch predicate in the same actual local statement. Cross-sentence pronoun/coreference bridging is outside this corrective patch, even when a reader can resolve the pronoun. Root identified previous #515 as preserving local statement/owner boundaries. Source-created aliases and owner links are not sufficient.

P323's frozen input is “Ilveta Gallery is the product covered by this record. It was launched on 8 May 2053.” Its expected positive relies on crossing the sentence boundary to resolve “It”. P323 therefore is an unsupported positive expectation under the clarified strict local contract. Withholding is the intended behavior; no owner/coreference expansion is requested for this patch.

The initial corpus, expectations, observed outputs, 47/56 result, eight positive misses, one false Relevant, exit status and first-summary gates remain unchanged. This is a separate post-run scope note, not a rescored result. The evaluator has not rerun v3 or edited earlier evidence.

The historical v3 test now has an explicit ignore attribute. Its original formatted implementation and expectations are recoverable at freeze commit `0bbd47f91a1e4daa07d4ac9b569375b8f21c9212`; complete first evidence is in `50b5401cb081c98882f30a55501a1c58c0ca87ec`. `artifact-sha256.json` records the original first-run files and original runner hash before ignore-only maintenance. The original runner is side-effect-free in normal mode and uses exclusive creation in first mode; neither mode was rerun by the evaluator.

No v4 corpus has been authored and no v4 API execution has occurred. The evaluator awaits a newly fixed candidate identity. Any future independent corpus must keep the canonical name or Harness-declared alias in the actual affirmative statement and treat unbound pronoun-only predicates as withholding.
