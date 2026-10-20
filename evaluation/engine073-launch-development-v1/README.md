# Engine0.7.3 #520 successor — development verification

Pinned implementation: `b6c1598655a826b3eb5d3527d613dd50ed8b9bbc`; `evidence_relevance.rs` SHA-256 `8acd799ae90a1407320b8899a437e0d0db3c2104a60a90db138fd76dd601a6f8`. Package metadata deliberately remains0.7.2 until a dedicated version-promotion/release PR. This directory records root-authored/seen offline development checks, not fresh real-source/provider acceptance.

- `original-audit-before.log`: before the repair, the unchanged14-input published0.7.2 audit reproduces10 false-Relevant negatives. The test-only revision was based on main5c033a73ad3ea7acfaee0ca8c5d6f95b2fcb25ee with the published source hash5343e722618721e58ad6a24b12c55c1272eb3c581bb3ada26b24d8596dfebbf5. The original audit and its first observation remain untouched.
- `boundary-before.log`: the first root-authored boundary matrix's failed observation. Later root scope/coordination controls are additional development coverage; this log is not a first observation of all subsequently extended tests.
- `focused-after.log`:33 regression tests covering #520/#521 context/markers, #514/#515 launch ownership/dotted names and #512 bilingual preparation pass on the candidate.
- `original-audit-after.jsonl`: all14 original API inputs now match their labels; the factual positive remains Relevant and all13 negatives withhold. This is a seen replay, not a new independent corpus or downstream final-answer result.
- `historical-current.jsonl`: all597 complete v16/v29 outputs are byte-identical to `../launch-520-521-final-validation/historical-base.jsonl`. Inputs and historical probe are reused unchanged from that bundle.

The initial focused command named a nonexistent `lambda_preparation_512` target and performed no tests; the corrected target is `bilingual_preparation_512`. The saved final log uses the real target. No provider/model API calls occurred.

Reproduction: build the pinned core with `cargo test --locked --offline -p reasoning-harness-core --test launch_context_markers_520_521 --test launch_source_frames_514_515 --test bilingual_preparation_512 --jobs 2`. Compile the unchanged `evaluation/engine-0.7.1-rereview-20261013/probe.rs` and `evaluation/launch-520-521-final-validation/historical-compatibility-probe.rs` against that build's normal rlibs, then use the original14 cases and597 historical inputs. Save any replay to new files. Never overwrite earlier observations.

Fresh independently authored narrow synthetic verification is separate from this directory. Full #531 real-source24-task/provider acceptance and safe live-runner integration remain incomplete; no Engine release or CLI adoption is established here.

Final local checks on this unchanged implementation:1498 tests PASS /0 FAIL /8 existing ignored; strict all-target Clippy, fmt,20 fixtures and13 original archive files pass. `summary.json` and the full logs retain the evidence. Provider tests run serially with local-only fixtures.
