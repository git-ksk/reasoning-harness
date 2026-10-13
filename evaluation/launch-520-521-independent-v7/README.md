# Independent v7 first evidence

Fresh bounded 50-case corpus/protocol: `fixtures/launch-520-521-independent-v7/`.
Formatted API recorder/replay runner: `crates/reasoning-harness-core/tests/launch_context_markers_independent_v7.rs`.

Inputs, expectations, protocol and runner are committed before v7 compilation/API execution. Explicit `LAUNCH_INDEPENDENT_V7_RECORD_FIRST=1` exclusively creates immutable first records; default replay writes no files and never overwrites them. Earlier initial results and scope notes remain unchanged.
