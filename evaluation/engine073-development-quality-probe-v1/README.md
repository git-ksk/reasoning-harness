# Engine 0.7.3 Core — Development-only source quality probe (2026-11-14)

**Not a formal model evaluation, independent holdout, release gate, or CLI adoption result.** These 40 fictional English source excerpts were created for this development exercise (20 explicitly supported launch statements and 20 negative/unsupported ones). Labels are root-authored, not reviewed by an independent evaluator. They partly exercise patterns similar to prior launch-scoping tests; no claim of unseen independent validation is made.

This compares two actual pinned native Rust Core sources using two existing exported APIs: v17 RequestedRelation and v30 Relevant, under the same exact-target proposal and source signal configuration. No language model, network, source fetching, provider authentication or billing is used. It does **not** measure a user's final answer, hallucinations, source quotation fidelity, human reviewer decisions, or appropriate model-level abstention.

## Identity and reproducibility

- Baseline Core 0.6.1: 4abee90f0501f0d8fc09b453a0b45c942fbed69e
- Candidate 0.7.3 development Core: ab3c6322921c8c738ec35bd018345917b5afa538. Candidate package manifest remains 0.7.2.
- Both source trees come from the previously pinned 988b69f086ac4a209fe93eb10cf1fa6786977567 common-CLI paired build with the same #552 resolution patch. This is not a built-in version of the published Reason CLI.
- Cases: cases.json. Frozen initial observations: baseline-output.jsonl, candidate-output.jsonl. Summary and file SHA256: report.json.
- The Rust runner is source_probe.rs. It has been compiled and run from **isolated private copies** of the two pinned source trees; original pinned binaries and old v1–v6 observations have not been edited.

To reproduce without modifying the pinned source trees, create separate isolated copies of the baseline and candidate source export. For each copy, place source_probe.rs at crates/reasoning-harness-core/examples/source_probe.rs and cases.json at crates/reasoning-harness-core/examples/source_probe_cases.json. Then run separately in each copy:

    CARGO_TARGET_DIR="$PWD/target" cargo run --offline --locked \
      --manifest-path "/private/tmp/YOUR_COPY/Cargo.toml" \
      -p reasoning-harness-core --example source_probe --quiet \
      > "/private/tmp/YOUR_COPY-output.jsonl"

Compare the preserved outputs entirely offline:

    python3 evaluation/engine073-development-quality-probe-v1/score.py
    PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover \
      -s evaluation/engine073-development-quality-probe-v1 -p 'test_*.py' -v

The scorer rejects missing/duplicated/forged results and label changes; tests make sure the score cannot silently grant adoption or formal F1.

## First diagnostic result

| Source relevance axis | Baseline 0.6.1 | Candidate Core |
|---|---:|---:|
| True positives (out of 20) | 0 | 16 |
| True negatives (out of 20) | 20 | 20 |
| False positives | 0 | 0 |
| False negatives | 20 | 4 |
| Precision, diagnostic only | undefined | 1.000 |
| Recall, diagnostic only | 0.000 | 0.800 |
| F1, diagnostic only | 0.000 | 0.8889 |

Both existing Core axes agreed on every source in this development set. The old 0.6.1 APIs classify all 40 sources as unresolved/nonrelevant under this exact-target policy; that does **not** establish how historical user-facing 0.6.1 CLI answers would behave.

The candidate incorrectly withholds four clearly positive target-source claims:

- p04 — nominal phrase "The launch of Quartz Lantern occurred..."
- p11 — a fictional announcement for an **unrelated** product, followed by a factual Quartz Lantern launch
- p12 — a denial of an **unrelated** product's launch claim, following the Quartz Lantern announcement
- p15 — a withdrawn screenshot, **not the launch event** itself

Follow-up candidate-only diagnostics (stored in temporary local QA output, not scored or submitted as a new first run) showed v17 relation_scope=unresolved, v30 disposition=ambiguous/path=conservative_fallback, with local_qualification_disagreement as a reported reason in all four. These are **false negatives/overwithholding**, not unsafe affirmative acceptance.

**Interpretation:** the candidate improves this Core-level source classification against the 0.6.1 version but remains over-conservative on four developer-authored positive statements. The synthetic numeric recall and F1 here are below the numerical threshold of #531, but they are **not the same metric**, cohort or source-qualified independent assessment. Never rescore older originals or promote these measurements to a 24-case independent final-answer GO. Alterations to the parser must not relax the 20/20 negative safety performance.

## Adoption impact

Development quality finding: **four reproduced source-only false negatives**. No evidence of false positive in these 20 negatives. No new real-model observations. Existing historical Mistral six-positive-run results remain preserved. Mandatory unknown/negative/withhold behavior of the full *model-driven CLI* and fresh independent 24-question labels are still missing. Official formal F1 remains null, quality_go=false, adoption_go=false. Mac Mistral API key is absent; current Free/no-extra-charge account entitlement is unverified.

Issues: #531 formal adoption; #574 development live evaluation. Historical results and future test data are tracked separately.
