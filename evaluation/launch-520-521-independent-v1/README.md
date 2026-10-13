# Independent evaluation evidence

The frozen corpus and protocol are in `fixtures/launch-520-521-independent-v1/`.
The direct API runner is `crates/reasoning-harness-core/tests/launch_context_markers_independent_v1.rs`.

No API calls or compilations were performed before the freeze commit. `freeze.json` records the freeze commit, candidate commit, source identity and corpus hash. Initial logs, complete per-case observations and aggregate gates will be retained here even on failure. This directory is evidence, not implementation or prior reproduction material.

The historical v1 test is explicitly ignored in workspace CI because its initial result is RED and its first observations must not be rerun, rescored, or overwritten. The original runner and expectations remain recoverable at freeze commit `1feccdf4f500d5585219d9f1b0e1187c407de536`; original observations and RED gates were committed in `af98cf95d7043b2ec475f4a36acb94637fc44228`. `artifact-sha256.json` records the original first-run artifacts and original runner hash, before this ignore-only CI maintenance. No initial evidence or score was changed.
