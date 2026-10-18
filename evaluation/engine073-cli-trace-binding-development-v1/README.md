# Ordinary CLI trace binding — development preparation

This joins a **saved, controlled offline ordinary-CLI capture** to the actual
captured document bytes. It does not execute an Engine, provider, model, old
scorer, or independent case. It is not an evaluation M0, a generic prose scorer,
or evidence that the current candidate improves ordinary answers.

`trace_binding.py` verifies:

- pinned file hashes, CLI argv/task/provider/model, process exit and strict JSON;
- ordered model request/response identities and final output vs terminal trace;
- acquisition question/target → admitted evidence content → host-owned scope,
  authority/effective time → exact structured-fact receipt → actual display;
- full retained HTML hash → reproduced fixed normalization → exact quote span;
- actual canonical display bytes, rather than unexposed model-renderer prose.

Host configuration supplies admission authority. A model/source claim of
authority cannot substitute for it. Internal claim duplicates remain traceable
without becoming extra displayed claims. Internal evidence links are never
counted as user-visible citations. Missing/bad records raise a failure, not a
successful semantic abstention or true negative.

Supported subset: the legacy `reason-natural-output-v4` canonical grounded
surface, captured HTML normalization v1, `host_curated_facts`, an unambiguous
acquisition identity, and structured-fact verification receipts. Other terminal
states, free prose, transformed claims, other adapters and full semantic/citation
judgments need a separate contract. This component does not authenticate a
publisher, independently attest the recorder, or defeat a hostile filesystem
writer. Hashes bind retained bytes, not external truth.

The inspection reuses `engine07-cli-path-preparation-v3`'s **development** Node
question: common CLI0.5.5 host `ec6e55f`, baseline core0.6.1 `4abee90`, old candidate
`01dc78b`. This is not the preserved canonical v2/v3 provider experiment, a replay
of the unavailable original provider response, or a new current-candidate run.
The current source `ab3c632` is only hash-identified; it is not executed here.

Both saved arms join structurally to one displayed proposition, two internal
claim/receipt chains, and one source. The full retained Node HTML, normalized
text and quote span match their original hashes. The actual display is the
canonical `node.buffer.uint8array = true`; the natural renderer sentence was
not exposed. The saved instrumentation report records zero relevance-API calls
on that ordinary path, with separate positive counter controls.

Consequently the report preserves these blockers: no displayed citation in this
surface, no semantic adjudication, an inactive target feature in the saved path,
and no current-candidate active-path measurement. `actual_answer_f1` stays null
and `acceptance` stays `NOT_EVALUATED`. The source of a successful internal
receipt is not a successful citation or useful-answer judgment.

The private HTML/normalized files are read and hash-checked; they are not copied
or republished. The report keeps hashes, quote spans, the existing short quote
and recorded source metadata. The capture inventory and old scorer/gold/first
observations remain unchanged. Groq and live calls stay stopped.

Run unit checks:

`python3 -B -m unittest discover -s evaluation/engine073-cli-trace-binding-development-v1 -p 'test_*.py' -v`

Run the inspector with explicit retained-input paths and a pre-recorded capture
index SHA256; output must be a new directory:

```sh
python3 -B evaluation/engine073-cli-trace-binding-development-v1/inspect_saved_pair.py \
  --repo "$PWD" \
  --capture evaluation/engine07-cli-path-preparation-v3 \
  --corpus evaluation/engine07-realworld-v3/corpus.json \
  --snapshots /private/tmp/engine07-realworld-snapshots-v2 \
  --expected-index-sha256 b876e06c7c36b31a8e98c657301c0a6f50df5178a7c87eafa07e1c9249a13e16 \
  --output /private/tmp/engine073-new-trace-inspection
```

Next: a declared experimental host/profile that actually invokes the evaluated
source APIs, separately from the unchanged ordinary-CLI control; complete
segmentation/support/citation adjudication and runner budget/dependency gates.
Freeze those before authoring the unused independent real24. Existing claim
arithmetic/display preparation is reused unchanged. No new synthetic60 cases.
