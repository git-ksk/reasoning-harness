# Reason interactive terminal UX

Reason CLI 0.5.0 development builds an everyday terminal surface without changing Harness Engine 0.4.2 authority or correctness semantics.

## Dispatch boundary

- Bare `reason` starts a managed interactive session only when stdin is a TTY and effective output is human-readable.
- `reason "TASK"` remains the one-shot path.
- `reason -c` continues the most recently updated compatible managed session for the canonical current project directory.
- `reason -r <id>` resumes a selected managed session by full or short stable id. Bare `reason -r` opens a numeric picker on a human TTY.
- `reason session list` lists managed sessions without exposing backing file paths; `--format json` is available for inspection/automation.
- Piped/non-TTY stdin never auto-enters the REPL. `-c/-r` also fail closed outside a human TTY instead of blocking for input.
- `--format json`, including an effective JSON default from config, never auto-enters the REPL.
- Structured subcommands and the existing low-level explicit `reason session ... --store` contract remain compatible.

## Commands

- `/add <path>` snapshots a UTF-8 regular file as persisted untrusted context for later prompts. Quoted paths with spaces are literal; no shell expansion or evaluation occurs.
- `/files` lists persisted untrusted context snapshots active in the managed session.
- `/status` shows verified facts, unresolved/qualified items, typed acquisition notes, and status from the last completed turn.
- `/evidence` shows supporting evidence provenance and separately labeled untrusted-context sources from the last completed turn.
- `/usage` shows cumulative tracked provider/resolver usage for the managed session; configured budgets apply across turns and resumes.
- `/clear` stops carrying prior conversation/context into later prompts. Existing typed turn history is not deleted.
- `/help` shows the interactive commands.
- `/exit` or `/quit` exits.
- A trailing `\` continues a multiline prompt.

## Managed-session model

A managed conversation is a product-layer `reason-managed-session-v1` wrapper. Each successful user prompt is persisted as its own existing typed `SessionFile` / `ReasoningThread` with a safe checkpoint. This deliberately preserves Core's immutable per-thread task identity instead of changing Engine semantics to imitate a chat transcript.

Only a successfully completed turn advances the managed checkpoint. A failed provider/model run leaves the previous persisted state intact. On resume, the persisted provider/model/max-token runtime is pinned; explicit incompatible overrides fail instead of silently switching models.

Prior exposed user/Reason exchanges are made available to later turns only as `untrusted_context`. They never become verified evidence merely because they appeared in a prior answer. Hidden chain-of-thought is neither stored nor displayed.

## Durability and privacy boundaries

#365 establishes managed session selection, safe completed-turn checkpoints, and stable ids. #381 owns store locking, optimistic concurrency/generation, interrupted-write recovery, corruption handling, and update/rollback migration guarantees. #379 owns private local permissions, retention/purge, ephemeral/no-persist mode, and outbound-data disclosure.
