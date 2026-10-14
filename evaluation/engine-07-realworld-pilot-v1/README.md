# Real-source development pilot v1 — inconclusive for adoption

Issue #531. **Not canonical provider/E2E acceptance.** This first pilot is preserved together with its design limitations; do not tune inputs, relabel, replace the first files or describe its negative rows as passed safety acceptance.

Five short official excerpt snapshots were inspected on 2026-10-14 UTC: [AWS Lambda launch](https://aws.amazon.com/blogs/aws/new-provisioned-concurrency-for-lambda-functions/), [AWS Lambda preparation](https://docs.aws.amazon.com/lambda/latest/dg/provisioned-concurrency.html), [S3 consistency announcement](https://aws.amazon.com/blogs/aws/amazon-s3-update-strong-read-after-write-consistency/), [CPython3.13](https://docs.python.org/3.13/whatsnew/3.13.html), [Kubernetes Ingress](https://kubernetes.io/docs/reference/using-api/deprecation-guide/#ingress-v122). Each source is stored once with URL, publisher, bounded date/context annotations and a SHA-256 of the short whitespace-normalized UTF-8 excerpt. No raw page snapshot, verified historical page version or publisher signature is supplied. Three of five excerpts share AWS origin.

`corpus.json` fixes five author-labeled positive inputs, four wrong-target policy variants reusing real excerpts, and two explicitly synthetic appended-text variants. They are eleven development inputs, not eleven independent real documents. Inputs/labels preceded observation. The Engine process receives policies, text signals and fixed optimistic **untrusted** proposal/raw qualification; it never receives expected labels. Metadata/as-of annotations are not passed as Engine signals. Consequently this pilot does not test time, origin independence, source-version binding, live models, source reconciliation, an authenticated reviewer or final-answer quality.

## Actual first observations

Both original released source tags were archived without changing the tags/locks, built separately with `cargo build --locked --offline -p reasoning-harness-core`, and linked to the same `probe.rs`. `ENGINE_VERSION`, exact tag commit, source tree/hash, lock, compiled rlib/binary, corpus/input/probe/runner and host identity are retained in `first-observation/`. There is one completed execution per arm.

- Engine0.6.1: `engine-v0.6.1` / `4abee90f0501f0d8fc09b453a0b45c942fbed69e`.
- Engine0.7.2: `engine-v0.7.2` / `296690a86e049b75ee693ae5032a71504366137c`.
- Both arms returned Relevant for CPython and Ambiguous for four other author-labeled positives. Observed gain0; positive regression0. All six negative-labeled inputs returned Ambiguous.
- The negative counts **do not establish safety**: the AWS launch parent was already Ambiguous in both arms, so the two mutations do not prove retraction/instruction detection. Wrong-target cases change relation/question as well as entity and do not isolate identity rejection.
- The four author-label mismatches **are not four established API/product bugs**: `require_harness_anchor` preconditions, truncated source context and the distinction between source relevance and affirmative support require review. Do not use this sample as a representative accuracy/utility rate.
- No model/API call occurred. Tokens/cost are unavailable; local process timing is not task/API latency. All observed v29 objects are equal across these eleven inputs only, not a fresh full historical acceptance claim.

The raw summary remains the original output, including its `candidate_false_relevant`/`mismatches` fields. Interpret these only against author labels and the limitations above. Known #520 remains a separate directly reproduced adoption blocker; this pilot's natural launch wording did not trigger that path.

## Independent design review and next runner

[Review and required corrections](design-review.md) records the independent review. The next evaluation needs:

- separate API-contract expectation, human source relevance, affirmative support, conflict retention and authority refusal;
- raw/full bounded source context, locator, normalization version and derived-input hash/lineage, with no fabricated dates;
- confirmed positive parents and one-factor wrong-target/control/retraction ablations, with active-boundary preconditions scored separately;
- complete ordinary CLI and opt-in reconciliation evaluators, not just this public-API probe;
- frozen copies of input/probe verified immediately before each arm, and streamed durable stdout/stderr preserving partial observations before any process failure.

V1 buffers probe stdout until success and rereads live probe/input paths. No observed identity drift occurred (reviewed hashes match preflight), but it does not meet the required canonical partial-failure protocol. Preserve this limitation rather than repair/re-run consumed v1 in place. A corrected operational/semantic runner uses a new version and a separately frozen manifest; fresh independent authoring starts after the complete evaluator is fixed.

## Local reproduction (seen-data replay only)

Use absent, private work/output paths; the runner refuses existing locations:

```sh
python3 evaluation/engine-07-realworld-pilot-v1/run.py \
  --repo /path/to/reasoning-harness \
  --work-dir /private/tmp/engine07-pilot-v1-replay-build \
  --output /private/tmp/engine07-pilot-v1-replay-result \
  --expected-corpus-sha256 700eb92263df742b050c2fd4cf5bcfd8802a906d163044139225b94dd34c787f
```

A replay is not a new independent observation. Do not publish binaries, copied source archives, build targets or credentials. The saved first observation is immutable; [adoption plan](../../docs/engine-07-realworld-evaluation.md) remains0/6 and NO-GO.
