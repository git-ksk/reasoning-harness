# CI v3 independent reviewer: offline signature operator

`ci_review_sign.py` makes the three existing signed review stages usable from
a separately appointed reviewer environment. **It never generates semantic
judgments, guesses decisions, calls a model, publishes to GitHub, starts a CI
run or changes the original cohort.** The person or independently appointed
review agent must inspect the **actual** full frozen sources, stdout/stderr and
display before creating a decision file. The signing key only proves key
custody; it does not certify semantic truth or human participation.

The signing seed is **not committed**, uploaded with a GitHub artifact or
printed. The registered reviewer should retain it under a private
`0700` directory as a `0600` file. Use the hash-pinned
`review-requirements.txt` in a dedicated reviewer Python environment.

## Stages and required inputs

### Start

Use the already documented `ci_operator.py archive` command to decrypt the
**live** `native-v3-bootstrap-RUN_ID` artifact into an empty private directory,
after verifying its GitHub run/source identity. Independent verification must
cover all source versions, both pinned cores, all 48 first requests, the
24-case gold, the exact registered key and the free-only/zero-extra-charges
account condition. Ensure the real provider's free quota is usable; a metadata
HTTP 200 alone proves neither quota nor billing status.

The reviewer authors a private decision JSON (no generated defaults):

```json
{
  "contract": "reason-cli-v3-independent-reviewed-start-v1",
  "decision": "approve",
  "start_request_sha256": "<exact SHA-256 of start-request.json>",
  "full_sources_reviewed": true,
  "notes": "<actual independent review findings, at least 30 characters>"
}
```

Then sign without publication:

```sh
python ci_review_sign.py start \
  --operator PRIVATE_BOOTSTRAP/operator.json \
  --signing-key PRIVATE_REVIEWER/reviewer-seed.bin \
  --decision PRIVATE_REVIEWER/start-decision.json \
  --bootstrap PRIVATE_BOOTSTRAP \
  --output PRIVATE_REVIEWER/start-signed.json
```

The script **refuses** prepare-only sessions, missing 48/24 inventory,
unknown readiness, revoked/unmatched keys and missing review evidence. When
ready, the independent reviewer uses the existing `ci_operator.py send`
with the exact live run ID, session, and stage `start` (no automatic retry).

### Each of 48 responses

Use `ci_operator.py receive` to decrypt exactly one signed-session packet
(stage `job-00` ... `job-47`), then independently inspect its complete
source and stream bytes, evidence identities, and source correspondence.
The reviewer authors:

```json
{
  "contract": "reason-cli-v3-independent-reviewed-job-v1",
  "packet_sha256": "<SHA-256 of canonical packet, excluding envelope>",
  "decision": "safe",
  "full_sources_reviewed": true,
  "notes": "<actual independent findings, at least 30 characters>",
  "segments": {
    "stdout": [{"start_byte": 0, "end_byte": 10, "label": "fact", "source_ids": ["<actual source id>"]}],
    "stderr": []
  }
}
```

The segments must partition every real stdout/stderr UTF-8 byte, without
gaps, overlaps or unbound factual source IDs. The example byte offsets above
are placeholders; **never** use them without verifying the actual packet.
A negative review may explicitly use `unsafe` or `unknown`, which is
signed and causes the CI to halt; it is never silently promoted to `safe`.

```sh
python ci_review_sign.py job \
  --bootstrap PRIVATE_BOOTSTRAP \
  --operator PRIVATE_BOOTSTRAP/operator.json \
  --signing-key PRIVATE_REVIEWER/reviewer-seed.bin \
  --decision PRIVATE_REVIEWER/job-00-decision.json \
  --packet PRIVATE_REVIEWER/job-00.json \
  --output PRIVATE_REVIEWER/job-00-signed.json
```

The tool invokes the exact production `verify_pre_advance` for safe
decisions. Send the signed envelope with `ci_operator.py send` stage
`job-00-review`; do not move on to `job-01` until the controller does.

### Final semantic adjudication and one-time first score

After all 48 observations, decrypt the `final` stage and independently
produce the complete, case-specific 48-entry `reviews` object required by
`annotate.validate_review`. The top-level reviewer decision must contain:

```json
{
  "contract": "reason-cli-v3-independent-reviewed-final-v1",
  "final_request_sha256": "<SHA-256 of canonical final packet>",
  "full_sources_reviewed": true,
  "complete_display_reviewed": true,
  "reviews": {
    "<exact job id>": {"<complete reason-cli-normal-v3-independent-review-v1 object>": "..."}
  }
}
```

```sh
python ci_review_sign.py final \
  --bootstrap PRIVATE_BOOTSTRAP \
  --operator PRIVATE_BOOTSTRAP/operator.json \
  --signing-key PRIVATE_REVIEWER/reviewer-seed.bin \
  --decision PRIVATE_REVIEWER/final-decision.json \
  --packet PRIVATE_REVIEWER/final.json \
  --output PRIVATE_REVIEWER/final-signed.json
```

The tool enforces 48/48 observations and reviews, frozen indexes, complete
review-packet integrity, each independently authored verdict against the
correct gold/observation, and a valid reviewer Ed25519 final attestation.
Only then may that signed final response be published at `final-review`.

No unsigned JSON, retrospective self-grade, automatically generated `safe`,
unreviewed sources or synthetic fixture is accepted as the real evaluation.
All intermediate outputs remain private. The irreversible live tag must not
be pushed unless an actual independent reviewer can sustain **every** stage
inside the existing six-hour CI deadline.
