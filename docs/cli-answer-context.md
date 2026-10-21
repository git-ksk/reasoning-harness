# Bound source context in ordinary CLI answers

The optional `answer_context` section of `reason-config-v1` preserves complete source excerpts and host-declared unresolved input keys after ordinary finalization. It addresses the measured loss of conditions/reasons in rw02/rw06 and applicability limitations in rw11/rw17. The Engine's original hard finalization and safety decision remain separately recorded. This is a CLI component change, not Engine0.7.3 acceptance or a measured F1 improvement.

The configuration is a host-owned registry, supplied independently of model output and scoring gold. It is not inferred from arbitrary resolver prose. Its schema is returned by `reason schema config`:

```json
{
  "schema_version": "reason-config-v1",
  "answer_context": {
    "schema_version": "reason-answer-context-config-v1",
    "sources": [],
    "unresolved_inputs": ["project.policy_settings"]
  }
}
```

Each source entry requires an exact `proposition`, `evidence_id`, `source_id`, `evidence_sha256`, `quote_pointer`, `url_pointer`, `version_pointer`, expected `url`, and expected `source_version`. Pointers select whole strings in the unchanged JSON observation; normalized document offsets are not observation offsets. The URL is the declared source URL, not a newly fetched/verified website. No new source independence, freshness or semantic support is inferred from a URL.

`evidence_sha256` is lowercase SHA256 of compact Rust `serde_json::to_vec(&Evidence)`, using the current `reasoning-artifact-v1` Evidence fields and BTreeMap ordering. It binds the admitted observation, facts, identity, scope, time and provenance metadata. The host must construct this from its actual admitted snapshot. Evidence is never rewritten to decoded text. Source URL/version must also match the same hash-bound observation. Duplicate JSON keys, malformed RFC6901 pointers and non-string selections fail closed.

Only selected Grounded propositions with exact current authorized claims and direct structured receipts can receive a citation. The receipt must match proposition, claim ID, optional statement and existing evidence IDs. Ordinary investigation may perform scope/time admission before a `structured_fact_equality` receipt; the qualified verifier variant is also supported. Current qualification requirements are rechecked deterministically. Conflicting values, contradictions, qualification findings, hard adversarial findings, missing evidence, changed hashes and wrong versions refuse the binding. Unselected source entries are not exposed.

The composition runs once over the final artifact after retries and the answer-safety gate. It adds source-local context only to an already exposable hard answer, producing `qualified_partial_answer`. It cannot turn `requires_verification`, `abstain` or `unresolved` into an answer. Model prose is never copied into the display. The artifact, truth claims, verification receipts, hard claim counts/coverage and original hard finalization are unchanged. Supplementary quote support is not measured by hard claim coverage.

The entire excerpt is displayed as a JSON string, preserving conditions while escaping newline/tab display boundaries; unsafe terminal controls and directional controls are rejected. The citation records both the fact attachment span and the encoded quote span in UTF8 bytes of `finalization.text`, plus the decoded quote and its SHA256. Decode a `json_string` quote span before comparing it with the source. Per-quote limit32KiB, total quotes64KiB, and at most32 source bindings/32 unresolved keys are enforced by rejection, never silent truncation.

Unresolved keys are supplied by the host from task inputs. They say only what this run has not established, without asserting a value, falsity or real-world absence. Provided but unverified claims and qualification reasons remain visible as typed limitations. An already established input cannot be relabeled as unknown.

Configured runs use `reason-natural-output-v5` and `harness-bound-source-context-v1`; unconfigured runs retain v4/v1. `answer_context` holds source-local citations, selected claims, typed limitations and the original hard-finalization snapshot. `finalization.text` is the shared composed display for JSON, text, trace and session output. Human fact/source sections read the final artifact and respect the gate/selected context claims; blocked answers do not reexpose a separate verified-facts list.

Explicit and managed sessions persist the whole registry and view. On load, each exposing hard answer must be reconstructible using its checkpoint's actual verdict and existing core finalization/partial-recovery rules. Stored hard status, text and selection are not authority by themselves. Current-profile restore rechecks semantic decidability and requires exact Supported claims/direct receipts meeting the trusted safety short circuit. It refuses model-dependent approvals that cannot be revalidated offline; this limitation also applies when saving context-enabled sessions. The explicit baseline profile retains its existing bypass semantics. Blocked results cannot contain exposed text. Context is suppressed during invalidation.

Both resume paths pin the complete saved registry (including an absent policy). A missing configuration file does not remove the saved policy; an explicitly different registry is refused before provider startup, even if it is otherwise valid. Start a new session to change this contract. Prior conversational text remains untrusted memory. Old sessions without this optional policy retain their original behavior and cannot silently acquire a different policy during continuation.

Validation includes the actual natural execution path with the current safety gate, text/JSON/session persistence and tamper rejection, and offline presentation of four saved Google artifacts with all8 quote occurrences byte-matched. The first rejected replay is preserved separately; it revealed the ordinary investigation lane's admission-before-plain-structured-receipt path, which is now supported with the same evidence/receipt checks. This rendering check uses no provider calls, new accuracy questions or F1 calculation. Original M0/M1, raw, gold, annotations and first scores remain immutable.

Host registry preparation and a separately frozen paired live evaluation remain necessary to measure answer quality. This patch does not fix incomplete acquisition, invent execution/origin notices, implement all of #488/#489/#491, release Engine0.7.3 or adopt it into the CLI. Mistral/Groq execution holds remain in force.
