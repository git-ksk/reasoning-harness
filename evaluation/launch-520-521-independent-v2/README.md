# Independent v2 evidence

Fresh corpus/protocol: `fixtures/launch-520-521-independent-v2/`.
Public API runner: `crates/reasoning-harness-core/tests/launch_context_markers_independent_v2.rs`.

No v2 compilation or API execution precedes the freeze commit. Explicit `LAUNCH_INDEPENDENT_V2_RECORD_FIRST=1` records immutable first evidence with exclusive creation. Default integration-test execution replays the corpus with no filesystem writes and never overwrites first evidence. Historical v1 RED results remain separate and unchanged.
