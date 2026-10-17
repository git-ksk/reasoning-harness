# Engine0.7.3 #520 successor3 — seen development verification

Pinned implementation `4e36fa5184e76601643b6d12575f5d26c8fc1eec`; source SHA-256 `2757449cd606bb62c875c8078a3596f6faeba159049a88daeb1ced9cb75a64d3`. Package metadata stays0.7.2 until separate release promotion. No provider API calls occurred.

The earlier independently authored v2 first result50/60 and new v3 first result39/60 remain FAIL. Those corpora are now seen development coverage. Source gates/CI passed on their original candidates, but they did not establish independent acceptance. Full first records, labels, scorers and hashes remain unchanged.

This repair replaces feature-name enumeration with a bounded explicit object parser: a specific non-launch claim/specification/note can be independent of the launch. Launch synonyms, generic/global/metalinguistic references and missing or mixed objects cannot supply that exclusion. A visibly unrelated event can have its own nonoccurrence; `No launch for TARGET occurred` and an unspecified launch claim in a list still withhold. Longest canonical/alias owner binding prevents a name suffix or article from becoming a fake property modifier. Recognized passive negation preserves the same retraction scope.

- `seen-independent-v2-after.jsonl` and `seen-independent-v3-after.jsonl`: each previously observed60-input set matches the frozen expectations. This is regression/development replay, not fresh independent acceptance. Original FAIL and labels are untouched.
- `original-audit-after.jsonl`: original14-input audit passes, including factual positive1/1 and negatives13/13 withheld.
- `historical-current.jsonl`: all597 complete historical v16/v29 objects are byte-identical to `../launch-520-521-final-validation/historical-base.jsonl` using unchanged inputs/probe.
- `focused-after.log`:38 tests covering context/markers, seen v2/v3 scope controls, full alias/article ownership, historical v10 replay, dotted names and bilingual preparation pass.
- Full locked workspace1502 PASS /0 FAIL /8 existing ignored; strict all-target Clippy, format,20 fixtures and13 original archive files pass. Full logs and `summary.json` retain evidence.

Commands follow the local gates in `../launch-520-521-final-validation/README.md` with serial provider tests and local-only fixtures. Generic API probes are unchanged from `../engine-07-realworld-pilot-v1/probe.rs`, `../engine-0.7.1-rereview-20261013/probe.rs` and the historical bundle. Save future replays to new paths.

New independent v4 follows this semantic freeze and retains its own first observation. #531 real-source24-task/provider acceptance, #540 safe runner, Engine0.7.3 publication and CLI adoption remain pending. This bounded grammar/evidence does not establish general natural-language truth, model accuracy or final-answer utility.
