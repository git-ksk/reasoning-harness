# Project status

## Current phase

The repository is an early research prototype. The core authority boundary, native CLI, deterministic fixture benchmark, one live provider adapter, trusted verification receipts, and a narrow Five Whys restatement pass are implemented.

This is not a claim that open-world reasoning is solved. Current correctness gains depend on deterministic structure and on trusted oracles where a hard answer exists.

## Implemented

- Rust-only core, CLI, eval, and provider adapter crates. Mistral and Gemma 4 candidate-generation adapters are implemented.
- Harness-owned evidence and untrusted `ReasoningCandidate` boundary.
- Deterministic structural/provenance validation.
- `accept | reject | unknown` policy.
- Trusted verification receipts that are never model-owned or model-visible.
- Receipt-backed support promotion and contradiction rejection.
- Narrow deterministic Five Whys lexical-restatement removal.
- Twenty committed regression fixtures (5 accept / 6 reject / 9 unknown).
- Mistral live benchmark workflow plus optional Gemma 4 matrix, manually triggered and secret-isolated.
- GitHub CI, Dependabot configuration, contribution/security guidance, issue and PR templates.

## Known gaps

- Exact natural-language receipt binding was confirmed too brittle for live paraphrases. The current implementation now uses typed propositions and harness-owned structured facts for the built-in hard verifier; exact-string binding remains compatibility-only.
- Hard contradiction/counterexample discovery exists for structured harness-owned facts; semantic/model-backed discovery remains soft-only and is not yet implemented.
- Counterexample discovery coverage is still narrow outside explicit structured propositions.
- Five Whys pass is intentionally lexical and narrow, not a semantic causal judge.
- No assumption extraction, first-principles pass, semantic-loss verifier, or verification-budget policy yet.
- Live model results are too small and stochastic for broad model-quality claims.

## Release posture

No stable API guarantee is made yet. Breaking schema/runtime changes are acceptable while the research contracts are still being validated by fixtures and live experiments.

- Live Mistral testing exposed malformed inference suggestions as a separate provider-quality issue. The runtime now isolates structurally invalid inference edges and records them in `candidate_diagnostics` instead of failing unrelated claims.

## Latest live verification result

After migrating the built-in hard verifier to typed propositions, canonical verified rendering, and explicit normalization of malformed untrusted inference edges, the 2026-08-30 Mistral live benchmark completed 7/7 runs with zero deterministic verifier failures. The harness arm reached 6/7 verdict accuracy (85.7%), kept unsupported accepted claims at 0, achieved 100% accept recall and 100% unknown recall, and reached 50% reject recall. The remaining miss is now tracked as generic contradiction/counterexample discovery rather than verifier binding.

### Adversarial discovery

The core now has a provider-neutral `AdversarialDetector` contract and typed `AdversarialFinding` records. Structured harness-owned fact conflicts are classified deterministically as hard contradictions or counterexamples. Findings themselves remain observational; only the verifier boundary can change epistemic state or force rejection. The 20-case recorded corpus reaches contradiction detection 1.0 and counterexample detection 1.0 under deterministic structured-fact coverage.

### Benchmark hardening

The 20-case benchmark now uses typed proposition labels instead of provider-generated claim IDs. Harness-owned hypotheses formalize task propositions independently of model output, and `unsafe_accept_cases` distinguishes true final unsafe acceptance from strong intermediate claims inside an overall `Unknown` result. The manual Mistral workflow now compares Ministral 3B/8B/14B and Mistral Small on the same corpus.

### Cross-model observation

The first hardened 20-case Mistral matrix completed successfully for Ministral 3B/8B/14B and Mistral Small. Harness accuracy was 0.80 for 3B and 1.00 for 8B, 14B, and Small. Every harness arm recorded zero unsafe final accepts, 1.00 contradiction detection, 1.00 counterexample detection, and zero deterministic verifier failures. Mistral Small achieved the 20/20 result with substantially fewer tokens and lower latency than the 8B and 14B runs in this single trial; repeated trials are required before drawing a model-ranking conclusion.

- Gemma 4 support uses the current Google Gemini Interactions API and remains outside the correctness authority boundary. Live Gemma acceptance is pending a configured `GEMINI_API_KEY`.

### Gemma 4 provider validation

The Rust provider boundary now includes Google-hosted Gemma and Gemini text models through the Gemini Interactions API. The live diagnostic matrix includes Gemma 4 26B/31B plus Gemini 3.1 Flash-Lite and Gemini 3.5 Flash-Lite; managed agents such as Antigravity are intentionally excluded. A live `gemma-4-31b-it` run completed all 20 benchmark cases: baseline accuracy 0.85, harness accuracy 0.95, unsafe final accepts 0, reject/unknown recall 1.00, contradiction and counterexample detection 1.00, and deterministic verifier failures 0. This is the first cross-family live validation beyond Mistral. `gemma-4-26b-a4b-it` currently returns provider-side HTTP 403 for the GitHub project and remains an experimental matrix entry.
