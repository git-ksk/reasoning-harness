# Active source host — unscored development controls

This is an **experimental offline host**, separate from the ordinary CLI and
from all accuracy/independent evaluations. It reuses the already observed Node
Buffer question and retained full source. It does not author a new case, execute
a model/provider, change Engine production code, or award answer F1.

| Control | Engine 0.6.1 baseline | Current repaired source |
| --- | --- | --- |
| `host.rs`: actual v17 → v30 → conditional source-local finalization | v17 relation unresolved, v30 ambiguous, final unresolved, citations0 | Same |
| `renderer_host.rs`: isolated manual exact-quote source binding, relevance APIs not called | qualified source-local text, structured citation1 | Same |
| Changed quote with unchanged retained byte span | Rejected before Engine calls, stdout empty | Same |

The relevance result is retained as observed. The direct Buffer subclass quote
does not satisfy this policy's bounded Definition relation cues. Controlled raw
qualification and an exact proposal do not override the Harness disagreement.
No Engine fix, input substitution or successful relevance result is invented.
The renderer isolation **does not rescue** that result or constitute a
successful end-to-end relevance/citation answer.

Both renderer outputs preserve the original exact quote and source/evidence/
binding identifiers, plus structured URL, section, retrieval time and raw-source
version. They are source-local qualification with zero truth claims and zero
verification receipts. This is saved JSON output, not a new ordinary-CLI UI.

## Identity and evidence

- Baseline core commit `4abee90f0501f0d8fc09b453a0b45c942fbed69e`, package0.6.1:
  clean Git archive, normal offline library build; every exported tracked file
  checked against its Git blob after the run.
- Candidate production source commit `ab3c6322921c8c738ec35bd018345917b5afa538`,
  evidence_relevance.rs SHA256
  `2e9207177d9f1e91b8c031322dbbf83c2b374d8847e123e135dcca727b0214de`.
  Package version remains0.7.2; no0.7.3 release is asserted. Reuses the pinned
  normal library from the earlier bounded development validation, not a cfg-test
  or instrumented library. The enclosing preparation checkout was `5d69935`.
- Each paired control uses identical Rust host source and input. Build/library/
  executable hashes, rustc identity, compile commands and process exits are
  retained. Build profiles differ; no latency/cost comparison is made.
- Parent ordinary-CLI development capture index SHA256
  `b876e06c7c36b31a8e98c657301c0a6f50df5178a7c87eafa07e1c9249a13e16`;
  the earlier trace-binding component reproduces its full-source normalization.
- Raw HTML SHA256
  `23f92fc924708065b4998712ee080dda049e9fd2b4e16558b396d68694d6fa47`;
  normalized text SHA256
  `8252ff58d1701eed6b80a20a183eef855dfde7b1b3b11bd9ffb2ec2c5125a4dc`;
  original normalized UTF-8 quote span10163–10228. Those private files are read
  and checked before/after execution, not copied into this bundle.

`validation.json` checks recorded outputs and all928 original independent/prep
references. Old arithmetic/display bytes and production source stay unchanged.
Six host processes comprise two relevance pipelines, two isolated renderers and
two quote guards that reject before Engine APIs. Each relevance pipeline makes
one direct v17 and one direct v30 call; nested internal calls are not counted.
These are engineering controls, not six evaluated questions.

`run_controls.py` and `verify_controls.py` record the original local preparation
commands and paths. They require retained private source/library inputs and
create new capture files; they are not a portable full runner or semantic scorer.
Baseline build output, host sources, input manifest, raw stdout/stderr,
invocations and identity checks are included. Binaries, libraries, private full
documents and the temporary baseline checkout are excluded. The artifact index
covers all included files except itself.

## Limits and next gate

The manually prebound Definition control is unrelated to fresh launch-repair
accuracy. No model generated the qualification. Source authenticity and semantic
scope/time correctness are not independently judged here. Same outputs in both
arms provide **no demonstrated quality gain**, answer F1 or performance gain.
`actual_answer_f1=null`, `acceptance=NOT_EVALUATED`, real24=0/24.

Keep the ordinary CLI control, active relevance pipeline and isolated renderer
as separate tracks. Next integrate external claim/support/citation annotation,
both case-quality and atomic-claim F1, operational completeness and bounded
runner dependencies/budgets; complete and independently review M0 before fresh
real24 authorship. No next synthetic60 repair loop, live calls, Groq, Oracle, release, merge or CLI
adoption is performed or justified by these controls.
