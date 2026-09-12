# Managed session durability

Reason CLI 0.5.0 keeps interactive conversation ergonomics in a product-layer managed store while every successful turn remains an ordinary typed `SessionFile` / `ReasoningThread` checkpoint. Harness Engine 0.4.2 authority semantics are unchanged.

## Store and concurrency

Managed sessions live under Reason's user data/config root in `sessions/`. The store uses an OS advisory lock for mutations and recovery. A loaded session also carries an in-memory SHA-256 digest of the exact bytes it came from. The digest is deliberately not serialized. Before a save, Reason locks the store and compares the current file bytes with that digest. If another process changed or removed the session, the write fails with `session_conflict` instead of silently overwriting newer state.

Writes use a new temporary file, `sync_all`, and an atomic replacement primitive. Unix uses same-filesystem `rename`; Windows uses `MoveFileExW` with replace-existing and write-through flags. The session directory is synced where the platform supports directory syncing. Abandoned `.tmp-*` files are removed only while holding the store lock.

## Corruption and compatibility

`reason session list` never invokes providers, MCP, resolvers, or replay. It reports valid sessions separately from corrupt or incompatible files. The interactive picker only offers compatible sessions.

The persisted contract remains `reason-managed-session-v1`. Concurrency metadata is intentionally in-memory so the #365 format stays readable by supported earlier 0.5.x builds. There is no destructive migration in the current 0.5.x policy. A future format that cannot be read by an older supported CLI must use a new contract/version and require an explicit backup/export step before destructive migration; update/rollback must otherwise refuse rather than silently rewrite incompatible state.

This durability layer does not store hidden chain-of-thought or credentials. Privacy, permissions, retention, purge, and ephemeral behavior are specified separately by #379.
