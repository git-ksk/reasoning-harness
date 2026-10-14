# Independent v9 first evidence

Fresh 50-case corpus/protocol: `fixtures/launch-520-521-independent-v9/`.
Formatted recorder/replay runner: `crates/reasoning-harness-core/tests/launch_context_markers_independent_v9.rs`.

Inputs, expectations, protocol and runner are committed before v9 compilation/API execution. Explicit `LAUNCH_INDEPENDENT_V9_RECORD_FIRST=1` exclusively creates immutable first files; normal replay has no filesystem writes and never overwrites them. Earlier independent first records remain unchanged.
