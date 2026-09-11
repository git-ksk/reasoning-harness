# Google canonical request pacing

Future natural-language E2E canonical successors use the repository policy in
`config/cross-model-concurrency-policy.json` rather than copying an ad-hoc Google
request rate into a workflow.

The prospective policy after immutable v34 is:

- minimum Google request-start interval: **6000 ms**;
- sustained start rate: **at most 10 requests/minute**;
- headroom reference: the **15 requests/minute** free-tier ceiling observed in v34, leaving **5 requests/minute (~33%)** request-count headroom;
- inter-case delay remains **3000 ms** and is not repurposed as the provider rate limiter;
- Gemini and Gemma model jobs remain serialized;
- concurrent Gemma investigation workers must share one request pacer;
- adaptive follow-up, MCP non-promotion, and session/stateful cases remain serial;
- paired control runs before candidate under the same pacing environment.

`validate_google_canonical_pacing.py` validates the repository policy by default.
A fresh successor must additionally pass its workflow and manifest with
`--workflow` and `--manifest` before provider credentials are exposed.

This policy is prospective only. It does not rewrite, rerun, rescore, or reclassify
v34 or any earlier frozen evidence, and it does not change Google quota
classification, provider retry semantics, scoring, or correctness boundaries.
