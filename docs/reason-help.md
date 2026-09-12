# Reason CLI help, examples, and completions

The everyday entry point is a natural-language task:

```text
reason "Explain the likely cause and separate verified facts from unresolved points"
```

Run bare `reason` on a human TTY for the interactive REPL. Continue the latest managed session with `reason --continue`, or choose/resume a session with `reason --resume`.

## Discoverability

`reason --help` keeps the common path visible and points to deeper surfaces. Use `reason help <command>` for command-specific contracts and `reason examples [topic]` for copy-paste workflows.

Useful examples include:

```text
reason setup
reason --file notes.txt "What does this evidence support?"
reason session list
reason --format json "Check this claim"
reason update --check
```

Files are untrusted context unless they are separately admitted and verified. JSON and piped modes remain non-interactive and decoration-free.

The structured `run`, `semantic-check`, `verify`, and `schema` commands remain available for advanced product/automation use. Research/evaluation commands remain available as `eval`, `eval-resolution`, and `eval-judges`; they are intentionally de-emphasized in the top-level help rather than removed.

`reason doctor` is not part of Phase 2 and is not advertised by the CLI.

## Shell completions

Completion scripts are generated to stdout and never mutate shell configuration:

```text
reason completions bash
reason completions zsh
reason completions fish
reason completions powershell
```

Installation is shell-specific; redirect/source the generated script according to your shell's normal completion mechanism.

## Authentication and setup

Use `reason setup` for first-run provider/credential/model readiness. `reason auth --help`, `reason models`, `reason model --help`, and `reason config --help` expose the lower-level product surfaces. Credentials remain in the native OS credential store; the config surface is non-secret.

## MCP

MCP product integration is a separate optional binary, not a `reason mcp` subcommand:

```text
reason-mcp --reason-command /path/to/reason
```

`reason-mcp` delegates selected operations to the native Reason runtime and preserves the native product contract. MCP results do not self-promote into trusted authority. See `docs/mcp-product-surface.md` for the protocol boundary.

## Update

Use `reason update --check` to inspect availability, `reason update` to apply a provenance-verified update, and `reason update --rollback VERSION` for an explicit rollback. See `docs/update-rollback-uninstall.md` for the full lifecycle contract.
