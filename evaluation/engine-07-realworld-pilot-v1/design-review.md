# Independent pilot design review — 2026-10-15 JST

Reviewer: separately invoked `engine07_realworld_independent`. Read-only, allowed new pilot runner/probe/corpus and output metadata only, no Engine implementation, prior holdouts, additional root synthetic audit, provider calls, edits or new acceptance authoring. This is a **design review**, not fresh independent corpus acceptance or authenticated human approval.

The reviewer verified current runner/probe/corpus hashes against preflight, quote self-hashes, paired row order and summary totals. Actual tags are compared through separate archived locked/offline builds and the same linked probe. No actual input/source drift was found. The following limitations prevent acceptance:

1. The four Ambiguous positive rows are author-label mismatches, not proven contract violations. `require_harness_anchor` and excerpt-only inputs omit source context, version and as-of. Separate contract disposition, human source relevance and satisfied preconditions before defect triage.
2. All negative rows are Ambiguous, and both mutation parents were already Ambiguous. Wrong-target cases also change relation/question. Use a confirmed Relevant parent and change one factor at a time; require the tested boundary to be active before counting a negative control as effective.
3. Relevance, affirmative claim support, conflict retention and authority refusal differ. Retraction may remain relevant conflict information; inert injected instructions need not invalidate an otherwise relevant quote. Use the actual API contract and score these axes separately.
4. Quote hashes only verify the saved normalized string. No raw page/section/normalization hash or historical page version is present; latest AWS source_version is null. Mutations reuse original source IDs while the expanded-input hash covers appended text. Require raw/locator/normalization and derived-input lineage in the next runner.
5. Metadata is outside Engine inputs, so time, origin independence and source-version binding are untested. Three excerpts share AWS origin; eleven rows reuse five sources. Report source/origin/family denominators separately.
6. `capture_output` saves stdout only after process success, losing partial rows on panic/failure. Probe/input are reread after preflight. Use frozen copies, arm-local hash checks and durable streaming before failure propagation in the new runner.

The parent accepted these findings. Original first observations and v1 runner remain unchanged. The four author-label mismatches are not filed as four product bugs; v1 negatives are not used as safety acceptance. The complete evaluator/host is still awaiting freeze, and no new independent acceptance set has been consumed.
