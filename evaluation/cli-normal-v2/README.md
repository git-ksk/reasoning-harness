# Ordinary CLI v2 preparation

This is an unadmitted, Mistral-only rerun preparation on the previously consumed 24 real-document cases: 48 paired normal CLI jobs, not an independent accuracy acceptance set. Historical v1 run 40873570944, its raw results, receipt and score remain unchanged. v1 recorded 2 completed, 1 failed and 45 suppressed jobs. No v2 model run or score exists here.

The generator derives its normal runtime from the exact v1 source commit `7073471d7490c64841200ae9ddaf252317c34721`, with explicit per-file replacement counts and before/after SHA-256 recorded in `preparation.json`. The inherited pinned Git objects come from the passed `--repo`; all preparation overlays come from the frozen snapshot. v2 has its own profile, receipt identity, tag, workflow paths, `normal_v2` namespace, host verifier and operational-review scope. The runtime inventory includes the v2 workflows and checkout-side verifier. The common CLI host is specified by `config.json` and must be independently reviewed.

The offline workflow job is available without provider credentials. The metadata and real-model jobs retain the v1 conditional routing and require a fresh v2 formal review, runtime reference or receipt and other credential-before-admission gates; none is present. The config records that execution is not admitted; it is not a workflow switch. The new profile and generated artifacts do not grant permission to call a model. A prior v1 verdict is not a v2 verdict. Before any live execution, independently verify the exact final source, binary builds, runtime image and workflow; issue a separately bound final v2 operational verdict; verify current provider readiness and pricing; freeze a new v2 receipt; and ensure the execution tag has never previously been used. CLI change, CI, release and independent accuracy evaluation remain separate gates.

The v2 runtime image build explicitly requests `linux/amd64` and disables BuildKit provenance and SBOM attestations. The inherited archive verifier did not rely on those Docker-generated attestations; the sealed runtime source inventory, ten ELF binary hashes, archive/config digests and independent proof boundaries remain in force. These build flags do not guarantee an archive format: the next offline build must pass the existing archive parser and independently verify the actual loaded Docker image ID before any runtime admission.

To prepare an unused output directory without credentials, network, provider calls, tags or scores:

```sh
python3 evaluation/cli-normal-v2/prepare_runtime.py --repo . --output /tmp/UNUSED-v2-preparation
python3 -B -m unittest discover -s evaluation/cli-normal-v2 -p 'test_*.py' -v
python3 -B -m unittest discover -s /tmp/UNUSED-v2-preparation/repo/evaluation/cli-repair-paired-v1/normal_v2 -p 'test_*.py' -v
```

The `--write-derived` mode is only for creating missing v2 checkout-side workflows, checkout-side tag admission script and verifier from the pinned tree. It refuses to overwrite those files. Normal preparation compares those files byte-for-byte with the derivation and refuses modified or missing files. Any future host change requires updating the config and regenerating the derived files, followed by a new review. No old files or receipts are altered by this preparation.
