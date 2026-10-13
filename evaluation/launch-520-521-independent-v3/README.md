# Independent v3 first evidence

Fresh 56-case corpus/protocol: `fixtures/launch-520-521-independent-v3/`.
Recorder/replay runner: `crates/reasoning-harness-core/tests/launch_context_markers_independent_v3.rs`.

Corpus, protocol and formatted runner are frozen before v3 compilation/API execution. Explicit `LAUNCH_INDEPENDENT_V3_RECORD_FIRST=1` records immutable first evidence using exclusive creation. Normal CI replays the frozen cases without filesystem writes or overwriting first observations. Prior v1/v2 RED records remain unchanged.
