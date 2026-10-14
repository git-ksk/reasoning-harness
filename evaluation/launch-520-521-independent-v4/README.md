# Independent v4 first evidence

Fresh corpus/protocol: `fixtures/launch-520-521-independent-v4/`.
Formatted recorder/replay runner: `crates/reasoning-harness-core/tests/launch_context_markers_independent_v4.rs`.

The corpus, protocol and runner are frozen before any v4 compilation/API execution. Explicit `LAUNCH_INDEPENDENT_V4_RECORD_FIRST=1` records immutable first files with exclusive creation. Default replay writes no files and cannot overwrite the first result. Earlier independent RED records remain separate and unchanged.
