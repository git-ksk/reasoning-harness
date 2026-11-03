# Native Mac process supervision — #540

The prospective native bundle **v2** freezes a process policy as well as source,
binary and verifier identity. `macos-no-fork-lifeline-v1` is the admitted Mac
profile. The controller passes a private socket pair to the measured CLI.
Before loading configuration, credentials or provider work, the CLI applies the
macOS process-creation restriction, acknowledges its lifeline and waits for GO.
The controller durably records the actual argv/PID/deadline/grace before GO.
Failure to establish the restriction, handshake or launch record stops before
provider work. There is no automatic fallback to an uncontained launch.

macOS denies both `fork` and `posix_spawn`, preventing the detached-descendant
counterexample by disallowing descendants. This is a deliberately restricted
single-process curated-source evaluation profile. It supports measured local
`--file` context and in-process acquisition; an external `--resolver-command`
or a configured `program` is rejected before planning. Arbitrary external
resolvers/MCP stdio programs are **not admitted by this profile**. Their future
admission requires a separately reviewed containment policy. Ordinary CLI
invocation without these opt-in evaluation controls retains its usual paths.

The lifeline FD is duplicated and made close-on-exec. Its EOF, unexpected input
or I/O error terminates the CLI with exit 125. SIGKILL of the controller closes
its peer even though Python cleanup cannot run. No credentials, model responses
or usage are created by the request planner. The owner, selected native source,
OS kernel/runtime and sandbox implementation are trusted; this is not hostile
same-UID/root isolation or protection from a compromised native binary. The
restriction is established before GO, not before the OS loads the executable.

The Linux POSIX lifeline profile is explicitly **control-only**. It checks
protocol/accounting/stream behavior in CI but does not prove no descendants.
`NativeLane` rejects provider admission outside the Mac no-fork policy. Windows
and arbitrary non-native commands have no production supervision claim here.
macOS `sandbox_init` is deprecated; a system where it is unavailable is refused,
with no silent downgrade. It is used only for this opt-in evaluation process.

## Cancellation and durable state

The fixed plan supplies the wall timeout and a positive cancellation grace of
at most five seconds (one second when omitted). Timeout, stream overflow,
SIGINT/SIGTERM and explicit `NativeLane.cancel()` never produce an admitted
observation. The controller first sends SIGINT, drains bounded partial streams
during that grace, then uses SIGKILL if needed and waits for process termination.
Even a child closing its streams must exit before the same wall deadline.
Signal handlers are restored after cleanup and persistence. Cleanup failures
revoke the lifeline and remain operational failures, with uncertain termination
never promoted to completion.

Cancellation sets the shared event before attempting ledger I/O. It persists a
create-only `halted` record with the exact retained reserve/inflight identity.
Authority state transitions are serialized against cancellation, so a stop
racing an in-progress reserve cannot underreport the charged reservation or
release work. Failed/ambiguous stop writes latch the controller. A completed
run is terminal; a later cancellation is rejected, and any durable stop marker
blocks first-score admission. New controllers still cannot restart the burned
evaluation or relabel it through another output directory.

Full streams, process completion and supervision record hashes are included in
the signed pre-advance packet and rechecked before advancement. Losing the
controller while waiting for a review leaves the reservation/inflight state;
there is no running native child at that point. SIGKILL/power loss cannot promise
new Python output records, but previously fsynced authority state is retained.

Terminating a local process does not revoke work already accepted by a remote
provider or prove invoiced charges. Partial/missing usage remains unknown and
the full reservation is retained. No implicit refund, retry, model switch or
automatic recovery is introduced. Historical runner/v1/v2 data and the original
uncontained detached-child regression remain unchanged.
