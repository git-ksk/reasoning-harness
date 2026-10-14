# Engine 0.7.2 verification coverage audit

Read-only repository review. Inputs/labels were saved before API execution; this is root-authored targeted synthetic audit evidence, not independent acceptance or a representative error rate.

The current core/lock/manifests are byte-identical to published engine-v0.7.2 at 296690a86e049b75ee693ae5032a71504366137c; the probe reports ENGINE_VERSION=0.7.2. evidence_relevance.rs SHA-256 is 5343e722618721e58ad6a24b12c55c1272eb3c581bb3ada26b24d8596dfebbf5 before and after.

Minimal contrast, exact target Silver Lens and optimistic untrusted proposal/raw qualification:

- `Silver Lens was launched in 2026. The newsroom retracted this claim.` -> v17 unresolved / v30 ambiguous (correct withholding).
- `Silver Lens was launched in 2026. The newsroom retracts this claim.` -> v17 requested_relation / v30 relevant (incorrect acceptance).
- `Silver Lens was launched in 2026. We retract this claim.` -> v17 requested_relation / v30 relevant (incorrect acceptance).

The source adverse-word list at evidence_relevance.rs:1916 includes retracted/retraction, denied and withdrew/withdrawn but lacks retract/retracts, denies and withdraw. The governing check at line1955 fails to reject these ordinary present-tense retractions/denials. Other direct non-event denials (`That did not happen`, `This event never happened`) also remain accepted in this targeted matrix.

Fourteen purposely targeted inputs include one factual positive and three known blocked controls; ten additional negative variants returned Relevant. See cases.json, results.jsonl and summary.json. This demonstrates an unfixed source-local qualification boundary; it does not demonstrate a downstream wrong answer or Known/Supported promotion. No release/Issue/source was changed.

The first probe compile used an incorrect dependency search directory and made no API call. Correcting only that compile command produced the first and only API observation saved here.
