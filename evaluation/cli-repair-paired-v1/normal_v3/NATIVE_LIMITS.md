# Native v3 pre-send limits and pacing — #581

New native provider runs require an operator-frozen `http_limits` record in
every job. `authority.freeze_jobs` copies validated limits into the genuine
receipt; paired arms must use identical limits for each case. The canonical
plan/receipt hashes bind the full policy. CLI/output data cannot choose it.

The defaults preserve the consumed development protocol:

| Condition | Limit |
| --- | ---: |
| Original unredacted ModelRequest UTF-8 JSON | 12,000 bytes |
| Input reservation estimate | original request bytes + 512 |
| Input per generation | 12,512 tokens |
| Requested output per generation | 1,024 tokens |
| Observed input per job | 24,000 tokens |
| Observed output per job | 4,096 tokens |
| Adapter generations / physical HTTP attempts per job | 24 |
| CLI logical operations | 12 |
| Minimum generation-start interval | 10,500 ms |
| Job wall deadline / cancellation grace | 360 seconds / 1 second |

Limits can be tighter, never larger. The input-estimate overhead remains 512.
Only explicit numeric-loopback/fixture-model controls can shorten pacing.
They cannot authorize external model runs. No retry, model/tier change,
additional fallback, weaker output DTO or larger budget is introduced.

## Before the physical request

The diagnostic adapter measures the original request before redaction. The
Mistral adapter independently reserializes the actual ModelRequest before
building its HTTP body, then checks the original hash, byte count, output cap
and actual wire `max_tokens`. A mismatch is a durable preflight refusal.
The record never contains Authorization or the full request body.

`reason-cli-evaluation-http-journal-v2` adds a create-only, fsynced `preflight`
record before the asynchronous start-spacing wait. Each record contains trace
identity, actual request byte count, input/output reservation and prior known
input/output totals. The next physical send requires byte/per-generation/count
limits, separate input/output total reservations, room in the frozen job
reserve, no pending/failed/cancelled/duplicated/concurrent generation, and the
minimum client generation-start interval.

After waiting, `request` records the start time relative to a monotonic clock
before transport sends. Dropping the future while waiting appends an aborted
record and latches stop, even when no physical HTTP exists yet. Transport
disables redirects/retries and performs one physical send. A structured
fallback is another measured generation under its existing diagnostic
correlation and consumes the same frozen generation budget.

Responses retain actual byte hash and provider usage. Missing, contradictory,
non-2xx, over-reservation or over-limit usage latches stop before another send.
Known usage stays in the private journal. No reservation is refunded on an
unknown or interrupted operation.

The bytes+512 rule is the frozen conservative client estimate, not a provider
tokenizer or billing guarantee. If actual input exceeds that reservation, the
response is retained and the run stops. Ending the local process does not
cancel remote work already accepted by the provider.

## Controller, capture and review

`NativeLane` requires these limits for provider admission and checks CLI caps
against them. Before every job it preserves the historical initial delay,
which also spaces starts across separate CLI processes. Waiting and the child
share the same job wall deadline; cancellation during initial delay spawns
nothing and retains the durable reservation.

The controller hashes canonical plan bytes, then decodes and uses that same
snapshot for all arguments, limits, timeout and grace. It rechecks the public
plan immediately before spawn and before accepting review. A plan changed
during pacing cannot supply new spawn conditions. The original independent
race reproduction is retained privately.

The `pacing.json` hash is in the signed pre-advance packet and is rechecked
before advancement, alongside full streams, HTTP, source, execution and
supervision evidence. Python normalization checks exact receipt limits,
preflight/trace metrics and order, actual usage, monotonic start intervals and
the full chained journal. Missing or modified preflight/pacing evidence cannot
become an admissible observation.

Journal-v1 remains for historical/control compatibility. New provider execution
refuses a missing policy; old preparations are not relabeled or resumed as this
profile. Generate a new build/bundle/plan. Use the immutable paired build's
Python modules for preparation, so active checkout edits do not replace the
enrolled runtime files.

## Acceptance boundary

Loopback controls exercise actual native CLI children, HTTP byte capture,
zero-send refusals, stopping the second send with ample combined reserve,
production 10.5-second spacing, future cancellation, concurrency, unknown and
overshoot usage, plan mutation during initial delay, all 48 signed pre-advance
records and the once-only score claim. These are engineering controls, not
real model quality, provider billing or semantic first scoring.

New operator/source/reviewer runtime admission, current provider readiness,
fee conditions, a new genuine receipt/one-shot plan/tag, 48 real observations
and independent semantic review remain before a real F1 or quality GO.
GCS, VMs, cloud signers and a new cloud project are not required.
