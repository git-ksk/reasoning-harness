# v2 first observation archives

**Frozen FAIL; incomplete measurement / adoption NO-GO.** [English results](../../docs/engine07-realworld-development-results.md) · [日本語の結果](../../docs/engine07-realworld-development-results.ja.md). This is development evidence, not independent acceptance.

`index.json` records exact first-run/tag/commit/artifact coordinates, the unchanged frozen summaries, original and published archive hashes, raw terminal-status counts, typed failures and separate normal/helper operational reasons. `linux-build-identity.json` proves the exact actual source builds in the same CLI0.5.5 host. `original-verification.json` checks the privately preserved original archives; `publication-verification.json` checks these public archive derivatives.

Mistral and Google ZIPs are byte-identical to the GitHub first artifacts. Groq ZIP is a clearly labeled **publication derivative**: only provider organization identifiers are replaced in stdout errors; `publication-redactions.json` maps original and published file hashes/rules/counts. Original ZIPs remain private under the primary Git common directory `private-evaluation-artifacts/engine07-realworld-v2/`, outside tracked files, and in the original GitHub artifacts while retained. Score/summary/journal files are unchanged. Public hash verification of a derivative does not assert that its redacted stdout is byte-identical to the original; the original private verification remains separate.

To verify originals without a model call:

```sh
python3 evaluation/engine07-realworld-v2-results/verify.py \
  --raw-root /path/to/originals-with-mistral-google-groq-directories \
  --frozen-suite evaluation/engine07-realworld-v2
```

To verify extracted publication ZIPs under provider directories, add `--publication`. Check ZIP SHA-256 against `index.json` before extraction. The verifier checks every START/FINISH stdout/stderr hash (or the explicit publication hash mapping), individual saved rows, exact frozen source/protocol identities and frozen summary aggregation; it does not rescore final-answer meaning or turn a failed result into PASS. Each provider has111 actual invocations,222 stdout/stderr hash checks,48 normal rows and24 helper rows. Complete journals are not the same as complete successful measurements.
