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

## Plain and accessible terminal presentation

Use `reason --plain` for static, accessibility-friendly human terminal output. Plain mode suppresses progress/decorative presentation while keeping prompts and Ctrl+C semantics intact. The same plain policy is selected automatically when `NO_COLOR` is present, when `TERM=dumb`, or when the human output streams are redirected/non-TTY. Reason does not introduce ANSI color, spinner animation, or cursor-control dependence in plain mode.

Human text is emitted without terminal-width byte truncation, so Unicode content is preserved rather than sliced at an unsafe byte boundary. JSON output is unaffected by this presentation policy.

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

Manage one active local or remote read-only MCP acquisition source without hand-authoring low-level JSON:

```text
reason mcp add inventory --program /path/to/mcp-server --arg=--stdio --tool lookup_item
reason mcp add-remote docs --endpoint https://mcp.example.com/mcp --tool search --issuer https://auth.example.com --authorization-endpoint https://auth.example.com/authorize --token-endpoint https://auth.example.com/token --client-id https://client.example.com/reason.json --scope mcp:read --replace
reason mcp login docs
reason mcp login docs --no-browser
reason mcp status docs
reason mcp test docs
reason mcp logout docs
reason mcp remove docs
```

`reason mcp test` verifies read-only discovery without invoking the selected tool. Remote OAuth uses authorization-code + PKCE; access/refresh tokens live only in the native OS credential store, while config contains non-secret endpoint/issuer/client metadata. `--no-browser` supports headless authorization. Remote project config remains subject to `reason trust`.

The optional `reason-mcp` binary is a different surface: it exposes selected Reason operations to external MCP clients. MCP acquisition output remains data rather than authority. See `docs/mcp-resolver.md` and `docs/mcp-product-surface.md`.

## Update

Use `reason update --check` to inspect availability, `reason update` to apply a provenance-verified update, and `reason update --rollback VERSION` for an explicit rollback. See `docs/update-rollback-uninstall.md` for the full lifecycle contract.
