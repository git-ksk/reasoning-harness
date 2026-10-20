# Independent synthetic launch-context check v4

**First semantic result: FAIL, 46/60 precommitted outcomes matched.** All 60 outputs were observed, exit status was 0, stderr was empty and there were no operational errors. Positives passed 20/30 and negatives passed 26/30 under the unchanged joint v17/v30 rubric.

| Frozen requirement | v17 | v30 |
| --- | --- | --- |
| Positive factual launch retained | requested_relation 20/30 | Relevant 20/30 |
| Negative launch claim withheld | no requested_relation 26/30 | non-Relevant 30/30 |

**No negative became v30 Relevant.** The four negative failures instead retained v17 `requested_relation` while v30 used its blocking cue to return `ambiguous`. They violate the precommitted requirement that a negative must both lose v17 requested relation and remain non-Relevant in v30. This is distinct from a negative v30 false-Relevant failure. Ten positive controls became v17 `unresolved` / v30 `ambiguous`.

Candidate source: `4e36fa5184e76601643b6d12575f5d26c8fc1eec`. Source-file SHA256: `2757449cd606bb62c875c8078a3596f6faeba159049a88daeb1ced9cb75a64d3`. Actual linked core rlib SHA256: `58e2806e142655f0a7e9c6b9dda2dd95ea74e37e63bc0b11884e232dc75e0821`. Package version remains `0.7.2`; immutable source SHA identifies the unreleased candidate. Immediately before the one invocation the recorder verified actual HEAD, hash-only source bytes, core/serde rlibs, dedicated probe executable, complete inputs, labels, scorer, protocol and all frozen authored files. The parent supplied the completed build. The evaluator independently hash-checked and linked it with rustc; it did not rebuild or inspect production implementation.

This is a newly composed 60-scenario synthetic set, 30 positive/negative matched pairs using 30 fictional targets. Publication contexts include signed errata, journal corrigenda, press/customer responses, composition/electrical/dimension/physical-specification notices, manual/gallery/translation errata, syndicated/editor notices, article metadata/reuse permission, archival/proofreading notes and owner-missing withdrawal notices. These are new complete context/control constructions, not prior inputs with substituted names. V2 and v3 were already seen by this evaluator and are explicitly known development data. Their previous FAILs and v1's operational FAIL remain unchanged. The parent-reported v2/v3 regression replay and historical test PASS were not independently rerun or inspected here.

Every candidate contains a separate unadorned Excerpt with the canonical target as the direct subject of `was launched in May 2022`. There is no new affirmative launch predicate, required numeric fact prefix, alias mapping, trusted reference AST or preparation semantics. Positive controls scope denial/withdrawal to a directly named other product, a specific non-launch property/activity or neutral publication administration. They test preservation of the launch fact, not approval of that other property as a product fact. Seven owner/reference-missing launch withdrawals were precommitted as negative. An unknown owner coupled to an explicitly non-launch topic, such as strap length alone, was precommitted positive because launch is outside that specified topic; this scope choice is recorded before observation rather than inferred from results. All labels are controlled evaluator expectations, not external human-approved universal truth.

Stages are explicit: `static-preflight.json` is STATIC_ONLY PASS with 0 Engine calls and 0 semantic observations. It checked 60 complete inputs, 30/30 labels, budget 2/192/90000, one direct target-subject affirmative per case, labels excluded from Engine input, syntax, inherited byte-identical scorer/probe and all predecessor hashes. After freeze, one process produced the first 60 deterministic public v17/v29/v30 observations. There was no v4 semantic sentinel. The same precommitted scorer was applied once, with no permissive scoring change, retry, rescoring, label edit or output overwrite. V29 is diagnostic only.

Four negative v17 disagreements, all v30 Ambiguous:

- `li4-048` (detached_notice_owner_missing): [source_title] Publisher withdrawal: product identity omitted / [excerpt] Zelbrook Projector was launched in May 2022. / [excerpt] A launch claim has been withdrawn. The withdrawal notice does not name the product.
- `li4-052` (report_amendment_unidentified_statement): [excerpt] Bramshore Terminal was launched in May 2022. / [excerpt] Report amendment: We retract an assertion in our release-history report. The withdrawn launch statement is not identified, and its product name is omitted.
- `li4-054` (joint_notice_unknown_owner_known_topic): [source_title] Joint publisher notice: Corvenhill Clock and Marshglass Basin / [excerpt] Corvenhill Clock was launched in May 2022. / [excerpt] We retract one product's launch claim. The product names were omitted from the withdrawal notice.
- `li4-056` (publication_layout_vs_unknown_owner): [heading] Publisher update / [excerpt] Damarfen Badge was launched in May 2022. / [excerpt] We withdraw a launch statement. The owner of the withdrawn launch statement is unknown in this notice.

The four notices explicitly leave the owner or identity of a withdrawn launch claim unspecified. The frozen rubric requires withholding the v17 requested relation as well as blocking final relevance. The observation does not establish which implementation mechanism retained that relation because implementation source was not inspected.

Ten positive controlled-label disagreements:

| Case | Explicit non-launch/other-owner context | v17 | v30 |
| --- | --- | --- | --- |
| li4-007 | composition_vs_release_table | unresolved | ambiguous |
| li4-011 | described_dimension_text | unresolved | ambiguous |
| li4-015 | manual_errata_quoted_sentence | unresolved | ambiguous |
| li4-019 | catalogue_export_line_scope | unresolved | ambiguous |
| li4-021 | abstract_physical_spec_limit | unresolved | ambiguous |
| li4-027 | syndicated_notice_explicit_owner | unresolved | ambiguous |
| li4-035 | article_permission_vs_content | unresolved | ambiguous |
| li4-045 | addressed_paragraph_named_assertion | unresolved | ambiguous |
| li4-051 | report_amendment_unidentified_statement | unresolved | ambiguous |
| li4-053 | joint_notice_unknown_owner_known_topic | unresolved | ambiguous |

Examples include a stainless-steel composition claim withdrawn with composition-only scope, a thickness-only retraction, a dust-sealing sentence withdrawn at the author's request, a correction limited to a named other product, withdrawal of article reuse permission with the launch statement explicitly unchanged, and an anode-specification retraction. Exact inputs, frozen expectations and raw results remain in `cases.json`, `inputs.json`, `first.stdout.jsonl` and `first-results.json`. Serialized reason names `model_relevant`/`model_ambiguous` describe core enums here; no model was called.

This is a narrow synthetic #520 source-fix confirmation, separate from #531's unperformed real primary-source 24-question evaluation. It is not release/CLI acceptance, acquired-document or final-answer E2E, cost/quota/headroom evidence or provider recovery. Only this candidate was run; there is no measured baseline delta, and percentages from v2/v3/v4 are not comparable improvement measurements because sets and candidates differ. The 60 scenarios are 30 paired constructions, not 60 independent documents or an exhaustive language sample. No general property-fact correctness, arbitrary grammar/reference AST, alias, temporal reasoning, preparation/future-release or new launch-predicate capability is claimed. After this first semantic observation v4 is consumed; later runs are development/regression.

Production implementation, root tests, historical corpora/fixtures and other independent sets were not read. Oracle, additional agents, provider API calls, cargo/shared-target changes, Git mutations and commits were all zero. Repository writes were limited to v4; rustc wrote a dedicated probe under `/private/tmp/engine073-launch-independent-v4-probe`. Initial stdout/stderr and started/finished invocation identity records are intact. `manifest.json` is the pre-run identity/hash freeze. `receipt.json` and `final-artifact-manifest.json` record post-run hashes for v4 and preserved v1-v3.
