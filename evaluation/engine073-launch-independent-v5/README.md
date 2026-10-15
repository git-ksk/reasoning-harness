# Independent synthetic launch-context check v5

**First semantic result: FAIL, 43/60 precommitted outcomes matched.** All 60 outputs were observed, exit status was 0, stderr was empty and operational errors were zero. Positives passed 15/30 and negatives passed 28/30. Two negatives retained both v17 `requested_relation` and v30 `relevant`; 15 positives became v17 `unresolved` / v30 `ambiguous`.

| Frozen requirement | v17 | v30 |
| --- | --- | --- |
| Positive factual launch retained | requested_relation 15/30 | Relevant 15/30 |
| Negative launch claim withheld | no requested_relation 28/30 | non-Relevant 28/30 |

Candidate source: `7e107f1c2c71cf55cd395d2b3aecb889e4f166a1`. Source-file SHA256: `939f9b788666a61da59b558cbc03f7741dd5e96e698951823f736bbedf6d4d1c`. Actual linked core rlib SHA256: `69a325c0ff64264161c09f2dfe67e410fa4bfd3c9a8dd3681a32dbafd35ff2a1`. Package metadata remains `0.7.2`; immutable source SHA identifies the unreleased candidate. Actual HEAD, hash-only source bytes, core/serde rlibs, dedicated probe executable, complete labels/inputs, scorer, protocol and all frozen authored files were verified immediately before the one invocation. The completed build was parent-supplied and independently hash-checked/linked; no rebuild or production implementation inspection was performed.

V5 contains new complete discourse/control constructions, 30 positive/negative matched pairs using 30 fictional target names. Contexts include interviews, current author replies, direct current quoted denials, parenthesized annotations, intervening attribution/content clauses, illustration alt text, service-guide discussion, possessive scope with a neighboring record, contact/navigation/download artifacts, correspondence, enclosure lists, lost transcript referents, detached signature pages, joint responses, attachments and orphan footnotes. Prior v2/v3/v4 cases and first results were known to this evaluator and are seen development controls, not v5. Their files and v1's operational FAIL remain unchanged. Parent-reported historical/full/development replays were not independently rerun or inspected here.

Every case supplies one separate unadorned Excerpt exactly `TARGET was launched.` The target is the canonical name and direct subject. No new affirmative launch predicate, date interpretation, numbered fact prefix, alias mapping, trusted reference AST or preparation feature is required. The operative positive context expressly identifies a non-launch property/activity, a named different product or a neutral document/contact/navigation artifact. Preserving launch does not approve the unrelated property as a product fact. Dialogue labels and attribution are context: positive operative clauses independently identify their object, so arbitrary dialogue-role inference is not required. Eight explicitly owner/reference-missing launch corrections were negative before observation. Scope and ambiguity decisions are fixed in `pre-freeze-scope-review.json` and `protocol.json`.

The two unsafe results were anticipated as explicit negations of the existing launch proposition before observation: the contraction `wasn't launched` and the existential non-event `There has been no launch`. Neither introduces an alternate affirmative launch predicate. The first denies the same subject/predicate as the direct positive signal. The second explicitly says that no launch of the named target exists. Both must withhold under the fixed same-target denial/non-event rubric; no post-result label change or implementation-derived rationale is used.

Unsafe negative observations, each v17 requested_relation / v30 Relevant:

- `li5-024` (contracted_was_not_same_or_other_predicate): [heading] Author current correction / [excerpt] Lucentford Wedge was launched. / [excerpt] Lucentford Wedge wasn't launched. I issue this as a correction of the launch claim.
- `li5-028` (existential_no_event_statement): [excerpt] There has been no launch of Nimbuscove Prism. The publisher issues this statement about launch. / [excerpt] Nimbuscove Prism was launched.

Fifteen positive disagreements against frozen controlled labels:

| Case | Explicit non-launch/other-owner context | v17 | v30 |
| --- | --- | --- | --- |
| li5-001 | interview_explicit_retraction_answer | unresolved | ambiguous |
| li5-005 | author_letter_intervening_relative_phrase | unresolved | ambiguous |
| li5-007 | attributed_present_quoted_denial | unresolved | ambiguous |
| li5-009 | parenthesized_current_author_annotation | unresolved | ambiguous |
| li5-011 | illustration_alt_text_withdrawn_object | unresolved | ambiguous |
| li5-013 | service_discussion_current_reply | unresolved | ambiguous |
| li5-021 | possessive_hereby_retracted_subject | unresolved | ambiguous |
| li5-029 | contact_route_vs_propositional_content | unresolved | ambiguous |
| li5-031 | diagram_link_vs_launch_statement | unresolved | ambiguous |
| li5-035 | navigation_artifact_vs_history_assertion | unresolved | ambiguous |
| li5-039 | possessive_owner_with_neighbor_reference | unresolved | ambiguous |
| li5-041 | topic_and_final_owner_in_author_reply | unresolved | ambiguous |
| li5-043 | multi_product_reply_object_or_owner_missing | unresolved | ambiguous |
| li5-047 | correspondence_denial_owner_absent | unresolved | ambiguous |
| li5-051 | transcript_explicit_property_or_lost_referent | unresolved | ambiguous |

Examples include a torque-ceiling retraction in an interview answer, a maintenance-interval withdrawal with an intervening attribution, a current quoted enamel-coating denial, a resin-content retraction, an acoustic-noise specification retraction, an old contact address or diagram/navigation link withdrawn, and a launch retraction expressly owned by a different named product beside the target's record. These are launch-preservation controls, not tests to approve those property/artifact facts. Exact inputs, expectations and observations are preserved in `cases.json`, `inputs.json`, `first.stdout.jsonl` and `first-results.json`. Serialized core reason names `model_relevant` and `model_ambiguous` do not mean that a model ran.

Stages are separate. `static-preflight.json` is STATIC_ONLY PASS with 0 Engine invocations and 0 semantic observations: it checks 60 complete inputs, 30/30 labels, positive budget 2/192/90000, the direct supported target-subject affirmative, label exclusion, code syntax, byte-identical inherited scorer/probe and predecessor hashes. There is no v5 semantic sentinel. After freeze, one deterministic public v17/v29/v30 process produced the first 60 semantic observations, and the unchanged scorer was applied once to the first raw output. V29 is diagnostic only. No permissive scoring change, retry, rescoring, input/label edit or first-result overwrite occurred. The gate requires every negative to lose v17 requested relation and avoid v30 Relevant, and every positive to retain both; it was not lowered based on results.

This is a narrow synthetic #520 source-fix confirmation, separate from the unperformed #531 real-primary-source 24-question evaluation. It is not acquired-document/final-answer E2E, provider recovery/cost/quota/headroom, release or CLI adoption evidence. Only the candidate was run; no baseline delta is measured, and percentages across v2/v3/v4/v5 are not comparable improvement measurements because cases and candidates differ. Sixty scenarios are thirty paired controlled constructions, not independent real documents or exhaustive natural-language coverage. Controlled expectations are not external human-approved universal truth. Other-property truth, arbitrary reference ASTs/affirmative predicates, aliases, temporal reasoning and preparation/future-release behavior are outside this check. V5 is consumed after this observation; later runs are regression/development.

Production implementation, parent/root tests, historical fixtures/corpora and other independent sets were not read. Oracle, additional agents, provider API calls, cargo/shared-target changes, Git mutations and commits were zero. Repository writes were confined to v5. Rustc wrote a dedicated probe under `/private/tmp/engine073-launch-independent-v5-probe`. First stdout/stderr and create-exclusive started/finished identity records remain intact. `manifest.json` is the pre-run freeze, while `receipt.json` and `final-artifact-manifest.json` record the first result and post-run hashes of v5 and unchanged v1-v4.
