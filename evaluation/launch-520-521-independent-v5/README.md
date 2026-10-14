# Independent v5 first evidence

Fresh bounded 50-case corpus/protocol: `fixtures/launch-520-521-independent-v5/`.
Formatted recorder/replay runner: `crates/reasoning-harness-core/tests/launch_context_markers_independent_v5.rs`.

The corpus, protocol and runner are committed before v5 compilation/API execution. Explicit `LAUNCH_INDEPENDENT_V5_RECORD_FIRST=1` records immutable first files using exclusive creation. Default replay has no filesystem writes and never overwrites first evidence. Previous independent RED records and scope notes remain unchanged.
