# v0.3.0 external-resolution acceptance

Issue #179 adds a deliberately **non-frozen product acceptance lane**. It does not read, mutate, relabel, or tune against historical Stage-C, RSD2, semantic holdouts, or other observed research corpora.

`reason-external-resolution-acceptance` runs eight deterministic workloads through the real subprocess adapter boundaries: the binary re-invokes itself as `external_command_v1` and, where required, as `trusted_command_verifier_v1`. This keeps CI credential-free while exercising process I/O, admission, qualification/re-verification, trusted verification, budgets, typed operational failure, and finalization.

The declared CI set covers:

- fresh exact external fact -> safe recovery;
- opaque acquired data -> independent trusted verifier -> safe recovery;
- stale evidence -> `unknown`;
- wrong-scope evidence -> `unknown`;
- fresh but irrelevant data -> `unknown`;
- admitted conflicting evidence -> `reject`;
- transport failure -> typed operational terminal with semantic `unknown`;
- budget exhaustion -> `exhausted` with semantic `unknown`.

The gate requires unsupported grounded claims = **0**, missed target insufficiency = **0**, false abstentions = **0**, all expected-unknown targets to remain ungrounded, and at least one initially-unsupported -> verified recovery. The first accepted run produced 8/8 expected outcomes, two recoveries, six acquisition successes, one separate trusted-verifier success, and mean final-claim coverage 1.0.

## Live public-information smoke

Network-dependent evidence is intentionally not a required CI dependency. On 2026-09-04 the same binary was run with `--live-aws` against the public AWS What's New RSS feed. The resolver subprocess performed the HTTP request, emitted source `aws:whats-new:feed` with fresh acquisition timestamps, and the Harness admitted/re-verified the exact typed fact `aws.whats_new_feed_available=true`.

Recorded result: HTTP 200, RSS `lastBuildDate=Fri, 04 Sep 2026 03:03:01 GMT`, initial verdict `unknown`, final verdict `accept`, one external call, unsupported grounded claims `0`. The machine-readable observation is committed at `docs/observations/v0.3.0-external-resolution-live-2026-09-04.json`.

This smoke demonstrates current/public provenance and freshness handling. It is an observation, not a frozen benchmark and not a permanent claim that the AWS endpoint will remain available.
