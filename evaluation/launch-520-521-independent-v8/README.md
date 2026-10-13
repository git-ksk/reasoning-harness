# Independent v8 first evidence

Fresh 50-case corpus/protocol: `fixtures/launch-520-521-independent-v8/`.
Formatted API recorder/replay runner: `crates/reasoning-harness-core/tests/launch_context_markers_independent_v8.rs`.

Inputs, expectations, protocol and runner are committed before v8 compilation/API execution. Explicit `LAUNCH_INDEPENDENT_V8_RECORD_FIRST=1` exclusively creates immutable first evidence; normal replay performs no filesystem writes and never overwrites it. Earlier independent first results/scope notes remain unchanged.
