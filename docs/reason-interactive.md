# Reason interactive terminal UX

Reason CLI 0.5.0 development builds an everyday terminal surface without changing Harness Engine 0.4.2 authority or correctness semantics.

## Dispatch boundary

- Bare `reason` starts the interactive REPL only when stdin is a TTY and effective output is human-readable.
- `reason "TASK"` remains the one-shot path.
- Piped/non-TTY stdin never auto-enters the REPL. Piped stdin remains untrusted context for an explicit task.
- `--format json`, including an effective JSON default from config, never auto-enters the REPL. Automation therefore cannot accidentally block on a prompt.
- Structured subcommands are unchanged.

## Commands

- `/add <path>` adds a UTF-8 regular file as untrusted context for later prompts in the current REPL. Quoted paths with spaces are accepted literally; no shell expansion or evaluation occurs.
- `/files` lists context files active in the REPL.
- `/clear` clears context files and in-memory conversation context.
- `/help` shows the interactive commands.
- `/exit` or `/quit` exits.
- A trailing `\` continues a multiline prompt.

## Authority and privacy

Each prompt still goes through the existing Harness-owned natural execution path. Prior exposed user/Reason exchanges are made available to later prompts only as `untrusted_context`; they do not become verified evidence and must be re-verified before they can support a factual claim. Hidden chain-of-thought is neither stored nor displayed.

Phase #364 does not write a shell-style prompt-history file or a managed session file. EOF exits cleanly. A process-level Ctrl-C cannot leave an interactive checkpoint half-written because there is no persistent interactive checkpoint in this phase. Managed continuation/checkpoint persistence and crash/concurrency durability are handled by #365 and #381; retention and purge policy is handled by #379.
