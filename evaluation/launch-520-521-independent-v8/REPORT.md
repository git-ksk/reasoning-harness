# Engine #520/#521 independent v8 — first observation

Author: `/root/launch_072_independent`. Fresh 50-case bounded local/global authority corpus, authored after fixed identity without implementation/root regression reads. Each positive has same-statement canonical or policy alias direct subject/object and declared predicate only. No coreference adds positive ownership. No retry, default replay, expectation edit, rescore or runtime change was performed.

**RED: 48/50 passed.** Safety FAIL: 24/25 negatives withheld; N803 became both RequestedRelation and Relevant. Positive recall FAIL: 24/25 positives passed; P802 missed. API errors: 0. Cargo exited101 only after all50 complete observations and aggregate gates were saved.

Candidate: `bcf65d9b2f0edb88c05780e4322a633c720d8a8a`.
Source: `crates/reasoning-harness-core/src/evidence_relevance.rs`.
Source SHA-256 before/after: `69f3e3116ae0a887cf5b83401d2355485edc5dd877a983c94587c3f1cbb00322` (unchanged).
Freeze commit: `fd313614009061eb46ab24e817c00f6aa8cf8070`, before v8 compilation/API execution.
Corpus SHA-256: `0283ed080ff0ca4cda1562359175bca7fd0859ab4980b776f1a9ed821d9c3e0b` (preserved across authoring interruption and reverified after first execution).

All cases use exact/exact proposal and optimistic exact-target/requested-relation/no-risk raw qualification. Positive requires RequestedRelation AND Relevant; negative requires neither. Factual proof comes only from excerpt/fact/structured_metadata; heading/title govern/deny only; URL/navigation have no authority. These are source-local relevance outcomes, not externally verified hard truth.

## Failures

### P802 — date_first_actor_launches_named_object

Policy target: Evranto Shelf.

fact: “On 2 May 2391, Calbora Labs launched Evranto Shelf.”

Frozen expected: {"claim_scope": "source_local_only", "v17_relation_scope": "requested_relation", "v30_disposition": "relevant"}.

Observed v17: `{"identity_scope": "exact_target", "relation_scope": "unresolved", "scope_risk": "none"}`.

Observed v30: `{"contract_id": "reason-evidence-relevance-binding-proposal-v5", "disposition": "ambiguous", "evidence_id": "v8-evidence-P802", "materialization_policy_id": "target-evidence-relevance-binding-materialization-v30", "path": "conservative_fallback", "policy_id": "independent-v8-local-global-launch-policy", "reasons": ["harness_canonical_name_anchor", "local_qualification_disagreement", "model_ambiguous"], "source_id": "v8-source-P802", "target_id": "v8-target-P802"}`.

### N803 — plural_statements_retracted_after_two_products

Policy target: Rilvanto Panel.

excerpt: “Rilvanto Panel was launched on 4 May 2422. Sorbora Cove was introduced on 5 May 2422. These statements were retracted.”

Frozen expected: {"claim_scope": "source_local_only", "v17_relation_scope": "not_requested_relation", "v30_disposition": "not_relevant"}.

Observed v17: `{"identity_scope": "exact_target", "relation_scope": "requested_relation", "scope_risk": "none"}`.

Observed v30: `{"contract_id": "reason-evidence-relevance-binding-proposal-v5", "disposition": "relevant", "evidence_id": "v8-evidence-N803", "materialization_policy_id": "target-evidence-relevance-binding-materialization-v30", "path": "model_assisted", "policy_id": "independent-v8-local-global-launch-policy", "reasons": ["harness_canonical_name_anchor", "positive_target_local_binding_confirmed", "model_relevant"], "source_id": "v8-source-N803", "target_id": "v8-target-N803"}`.

P802 is a direct-object named-target launch in the same actual statement, with an on/day-May-year adjunct at the front. N803's plural “These statements” refers to both immediately preceding release statements, including the policy target's launch. The retraction must not become local only to the most recent different product. Neither expectation adds ownership by coreference or a new release predicate. Both expectations were frozen before first execution and remain unchanged.

## Passing controls

Positive IDs: P801, P803, P804, P805, P806, P807, P808, P809, P810, P811, P812, P813, P814, P815, P816, P817, P818, P819, P820, P821, P822, P823, P824, P825.

Negative IDs: N801, N802, N804, N805, N806, N807, N808, N809, N810, N811, N812, N813, N814, N815, N816, N817, N818, N819, N820, N821, N822, N823, N824, N825.

Passing positives include all declared launch/GA families outside date-first actor/object P802; policy alias/dotted/decimal ownership; date/as-of/record adjuncts; feature question and adjacent false property statement; direct feature negation; same-target property strike; marked different-target launch; and foreign fictional passive introduction, denied active launch and hypothetical active rollout headlines. URL/navigation without authority and feature-question heading also preserved factual proof.

All-following fiction and entire-report hypothesis remained governing despite a foreign named launch (N801/N802). False/denied/fictional/withdrawn report/claim contexts, all original symbol/strike controls, conditional/question/rumor/negative actor claims, actor-as-target, wrong owner, pronoun-only proof and heading/title/URL/navigation proof floors all withheld correctly. The only safety failure was plural retraction after two named release statements in N803.

## Evidence and interrupted authoring

Authoring was interrupted before runner/protocol/freeze creation because automatic approval review hit a usage limit. On user-authorized resumption, only author_corpus.py/corpus.json/manifest.json existed. No first observation/runner existed, the fixed candidate was unchanged, and the corpus hash/expectations were preserved. This was the first v8 API execution, not a retry.

- `freeze.json` / `first-run-start.json`: frozen and first-execution provenance.
- `first-observations.jsonl`: complete input and both unmodified outputs/errors for all50 cases.
- `first-failures.json`: exact P802/N803 first failure subset.
- `first-summary.json`: immutable initial gates and source identity before/after.
- `first-run.log` / `first-run-exit-status.txt`: complete compiler/test output and exit101.
- `artifact-sha256.json`: frozen corpus/protocol/runner and initial evidence hashes.

Explicit first mode exclusively creates immutable first files. Default replay has no filesystem writes or dependency on first-file existence. The evaluator did not perform a default replay after initial execution. All earlier first results/scope notes remain unchanged. Root owns diagnosis/repairs; this observation alone does not establish whether either gap was introduced by the candidate.

Offline synthetic source-local public-API relevance only; no provider, human, user, device, production or external factual-truth acceptance.
