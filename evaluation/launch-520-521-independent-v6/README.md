# Independent v6 first evidence

Fresh 50-case corpus/protocol: `fixtures/launch-520-521-independent-v6/`.
Formatted API recorder/replay runner: `crates/reasoning-harness-core/tests/launch_context_markers_independent_v6.rs`.

All inputs, expected outcomes, protocol and runner are committed before v6 compilation/API execution. Explicit `LAUNCH_INDEPENDENT_V6_RECORD_FIRST=1` exclusively creates immutable first records; normal replay writes no files and cannot overwrite them. Earlier independent first evidence and scope notes remain unchanged.
