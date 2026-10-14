# Independent v10 first evidence

Fresh 50-case corpus/protocol: `fixtures/launch-520-521-independent-v10/`.
Formatted recorder/replay runner: `crates/reasoning-harness-core/tests/launch_context_markers_independent_v10.rs`.

Inputs, expectations, protocol and runner are committed before v10 compilation/API execution. Explicit `LAUNCH_INDEPENDENT_V10_RECORD_FIRST=1` exclusively creates immutable first evidence; default replay has no filesystem writes and cannot overwrite it. Earlier independent first records remain unchanged.
