# Engine 0.6 evidence relevance: identity ambiguity diagnostic v1

Status: fresh calibration-only diagnostic. It does not modify the runtime relevance materializer and cannot rescore v7.

## Motivation

Frozen v7 produced one Mistral utility miss on `21_unknown_rename`: the advisory model proposed `target=different`, and materialization v3 therefore emitted `irrelevant` even though the supplied material did not establish whether the new name was a rename/replacement of the Harness target.

This is treated as an open-world entity identity / abstention problem, not a one-case prompt bug.

Relevant prior art:

- Sun, Chen, and Hu, *Knowing the No-match: Entity Alignment with Dangling Cases* (ACL-IJCNLP 2021): entity alignment must detect entities for which no alignment is established and may abstain rather than force an alignment.
- Zhu et al., *Learn to Not Link: Exploring NIL Prediction in Entity Linking* (ACL Findings 2023): NIL / missing-entity prediction is a distinct entity-linking problem; models need explicit treatment of unknown mappings.
- Gangrade, Kag, and Saligrama, *Selective Classification via One-Sided Prediction* (AISTATS 2021): abstention can reduce false positives by requiring one-sided evidence before committing to a class.
- Tan et al., *Too Consistent to Detect* (EMNLP 2025): repeated same-model samples can remain consistently wrong, motivating an orthogonal cross-model risk signal rather than treating seed agreement as correctness.

References:
- https://aclanthology.org/2021.acl-long.278/
- https://aclanthology.org/2023.findings-acl.690/
- https://proceedings.mlr.press/v130/gangrade21a.html
- https://aclanthology.org/2025.emnlp-main.238/

## Fresh corpus

The diagnostic uses 12 newly authored synthetic cases. It excludes the names and wording of v7 case 21.

- six open-world identity cases expected to remain `target=unresolved`;
- four explicitly distinct-product controls expected to be `target=different`;
- two exact canonical/registered-alias controls expected to be `target=exact`.

Families cover possible rename, possible successor, unregistered acronym, unregistered cross-language name, truncated identity mapping, unknown version lineage, explicit separate product, explicit comparison, explicit not-a-rename statement, explicit structured distinctness, canonical exact identity, and registered alias identity.

## Candidate measured without runtime adoption

Every observation records two bounded model outputs under the same case deadline:

1. the existing binding proposal v2;
2. a fresh one-sided distinctness verifier that may return only:
   - `confirmed_different`: the local material affirmatively establishes a distinct entity/product;
   - `not_confirmed`: distinctness is absent, uncertain, inferred only from a different name, or the mapping is missing/truncated.

The verifier is not allowed to create aliases or identity facts. Candidate text remains untrusted.

For diagnostic scoring only, if the primary proposal says `target=different` but the one-sided verifier does not confirm distinctness, the effective target binding is conservatively changed to `unresolved` before calling the unchanged materialization v3. This computes a candidate abstention policy; it does not change runtime semantics.

## Execution design

Required diagnostic sources:

- Mistral `ministral-8b-latest`
- Groq `openai/gpt-oss-120b`

Each source evaluates all 12 cases across three matched seed trials. Thus each source contributes 36 observations and 72 bounded model requests before adapter-internal retries/fallbacks.

The providers receive the same semantic prompts and labels. Provider differences are limited to adapter mechanics and operational pacing.

## Precommitted diagnostic gates

A provider arm is operationally complete only with 36/36 successful observations.

The one-sided candidate is considered promising enough for a separately designed runtime successor only if each required provider independently has:

- false `confirmed_different` on expected unresolved/exact cases: 0;
- missed `confirmed_different` on explicit-different controls: 0;
- gated disposition exact: 36/36;
- provider failures: 0.

Primary binding accuracy and baseline materialized accuracy are diagnostic and are not silently substituted into these gates. A passing diagnostic does not accept #462; it only justifies designing a fresh materialization successor. A failed diagnostic is frozen and not tuned against by editing these 12 cases.
