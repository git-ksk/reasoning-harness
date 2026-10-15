# Practical-evaluation provider controls

Status: **offline guard implemented and tested; not wired into the already frozen v2/v3 runners, no new live run launched**. The next runner must incorporate this module, state and tests into its own manifest before any API observation. These controls do not turn the original failed results into passes.

Past implementation and results to preserve:

- [calibration v17 operational policy](../../docs/engine-0.6-evidence-relevance-calibration-v17.md): one typed quota latches an arm; two consecutive capacity failures latch; adapter owns retries; suppressed cases stay operational failures; a known exhausted daily window blocks the next freeze push.
- [v17 first result](../../docs/engine-0.6-evidence-relevance-calibration-v17-result.md): first quota stopped further requests; 41 cases were suppressed, not scored as successful negatives.
- [holdout v5 admission model](../../docs/engine-0.6-evidence-relevance-holdout-v5.md): prior actual usage anchors headroom, conservatively reserve the next request, impose an arm self-budget, and invalidate the model when material intervening organization usage is known or suspected. Its historical 55K/70K/300s values were for that smaller experiment; they are not automatically sufficient for 48 practical CLI runs plus helper calls.

The new guard shares one instance across normal baseline, candidate and helper invocations for a provider/model arm. `invoke` calls the supplied operation once at most; on quota, unknown token/attempt accounting, a host interruption, two consecutive capacity failures or insufficient reserve it stops subsequent external requests. Persist `snapshot()` and record every suppressed planned row as incomplete. Use the actual typed provider class, not keyword matching in prose. Keep independent provider arms independent.

`groq-state.json` preserves the observed daily-quota latch. TPM headers, a 10-second request interval, a tiny successful readiness call or a retry-after value do not prove enough **daily** headroom for a whole experiment. The recorded 8K TPM figure is a minute limit. Do not launch another Groq identity in this known exhausted window, silently switch model/tier, increase retries, or rerun the frozen failed identities. Recovery needs a new conservative model anchored to actual usage and refill time; unknown shared organization usage invalidates a claimed headroom estimate. This is an operational evidence requirement, not a request for a manual capacity attestation.

Before the next live freeze, wire and test the guard in the real runner (including normal/helper transitions), pin the repaired CLI diagnostic host, calculate a practical arm reserve/self-budget from first observations, and use graceful cancellation before a bounded hard kill. #535 preserves early returns and cooperative cancellation; no destructor can flush a process killed by SIGKILL. Google v3's 16 wall timeouts therefore remain incomplete.

Run offline: `python3 -m unittest discover -s evaluation/engine07-realworld-operations-v1 -p 'test_*.py'`. This module makes **zero** model/API calls. It is preparation evidence, not provider recovery or Engine acceptance.

## Post-observation independent operational review

The independent reviewer reproduced a connection hazard: the ordinary CLI emits `quota` while the helper emits `provider_Quota`. A raw string must not reach the guard as a trusted failure class, especially after #536 makes some failed-run usage available. `bridge.py` now maps each exact envelope to `FailureClass`; the guard stops on unknown/non-enum classes even when usage is known. Eleven additional offline controls include actual first Groq archives (47 normal quota/1 unmeasurable and13 helper quota/2 success), known partial usage, malformed frames and cancelled lower-bound accounting. These are representation/accounting replays, not a semantic rescore or new provider observation.

There are23 offline tests in total. Real-runner integration, loading the persisted start prohibition/recovery model, durable reserve/inflight/latch persistence and first-run-only restart protection remain mandatory before any new live identity. The standalone bridge is not a claim that these remaining pieces are implemented. Historical frozen v2/v3 and their12-test review evidence remain unchanged.

Repair review also reproduced cancellation with known usage and a successful contract combined with an explicit cancelled status. Canonical cancellation now always latches; its claimed totals are not accepted as complete. A success envelope cannot carry a failure/interruption/status frame. Three further controls cover these normal/helper contradictions without provider calls.
