# Progress, retries, and cancellation

Reason CLI 0.5.0 shows high-level operational progress only on a fully interactive human TTY. JSON output, piped stdin/stdout, and non-TTY automation stay quiet.

The visible lifecycle vocabulary is intentionally small: `Planning`, `Acquiring`, `Verifying`, and `Finalizing`. These are Harness-owned execution phases, not model reasoning and not hidden chain-of-thought. `--verbose` may add provider/model identity, attempt counts, latency, and typed failure class; it never prints prompts, model response bodies, credentials, or hidden reasoning.

Provider waits remain bounded by the existing adapter policies. After a provider call has been outstanding for 3 seconds on a human TTY, Reason emits a low-frequency waiting heartbeat stating that bounded provider-side retries may be active; it does not invent an exact intermediate retry count. When the adapter later reports more than one attempt, Reason prints the completed attempt count.

Ctrl+C is handled as the typed operational failure `cancelled`. It does not become epistemic `unknown` and does not grant partial authority. In-flight async provider work is dropped, while owned external-resolver, MCP, and trusted-verifier subprocesses observe the same cancellation flag and are terminated by their bounded deadline loops. Managed interactive turns advance only after successful completion, so an interrupted ordinary turn is not committed as a successful checkpoint. Existing typed session invalidation/revalidation rules remain authoritative for low-level session mutations. At an idle interactive prompt, Ctrl+C keeps ordinary terminal behavior and exits instead of being swallowed.

## Plain and accessibility mode

`--plain`, `NO_COLOR`, `TERM=dumb`, or redirected/non-TTY output selects the plain terminal presentation policy and suppresses phase/wait/retry progress decoration. Prompt and Ctrl+C cancellation semantics are unchanged, and JSON/piped output never receives progress text.
