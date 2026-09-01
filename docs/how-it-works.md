# How Reasoning Harness works

[日本語](how-it-works.ja.md) | English

This document explains the product execution model behind `reason run`, especially the mode where an external AI/agent produces a `ReasoningCandidate` and Reasoning Harness itself does not call an AI endpoint.

> **Product direction:** the structured objects described here remain the runtime's inspectable internal/advanced contracts. The primary end-user direction is an AI-backed natural-language CLI ([Issue #107](https://github.com/git-ksk/reasoning-harness/issues/107)) that constructs and moves through these same boundaries rather than bypassing them.

## The central trust boundary

Reasoning Harness separates **proposal** from **authority**.

- A model/agent may propose claims, epistemic labels, and inference edges.
- `HarnessInput` owns the task, evidence, evidence requirements, assumptions, and authority policy.
- Trusted verifier code may create `VerificationReceipt` records.
- The acceptance policy, not the model, decides `accept | reject | unknown`.

```text
              UNTRUSTED                       TRUSTED / HARNESS-OWNED

 external model / agent
          |
          v
 ReasoningCandidate  -------------------+
  claims / edges                         |
                                        v
 HarnessInput --------------------> materialization
 task / evidence                         |
 requirements / policy                   v
                                  validation + passes
                                        |
             +--------------------------+-------------------------+
             |                          |                         |
             v                          v                         v
      verification receipts       diagnostics              artifact state
             |                          |                         |
             +--------------------------+-------------------------+
                                        |
                                        v
                               acceptance policy
                                        |
                              accept / reject / unknown
```

## Two `run` modes, one checking pipeline

### Existing candidate: no AI call required

```bash
reason run \
  --input evidence.json \
  --candidate model-candidate.json \
  --no-config \
  --format json
```

The candidate could have come from any system capable of producing the `reasoning-candidate-v1` shape: an application-owned LLM call, a RAG pipeline, another agent, recorded output, or deterministic code.

After the JSON is provided, this path does not need an AI provider.

### Live provider: AI generates the candidate

```bash
reason run \
  --input evidence.json \
  --provider mistral \
  --model ministral-8b-latest \
  --format json
```

Here `reason` calls the provider only to obtain the candidate. The provider output then enters the same untrusted materialization and verification path as a supplied `--candidate`.

This distinction matters: using a better model may improve candidate quality, but it does not grant that provider verification authority.

## Step 1: candidate materialization removes self-certification

A `ReasoningCandidate` is syntax proposed by an untrusted producer. It is not accepted reasoning state.

The current materializer applies the following conservative state mapping:

| Candidate `proposed_state` | Initial artifact state |
| --- | --- |
| `unknown` | `unknown` |
| `assumed` | `assumed` |
| `known` | `assumed` |
| `supported` | `assumed` |
| `inferred` | `assumed` |
| `contradicted` | `assumed` |

Therefore the model cannot write `"proposed_state": "supported"` and thereby obtain a supported claim. Strong state must be re-established inside the harness boundary.

Materialization also normalizes references. For example, duplicate claim IDs and invalid inference references are dropped/isolated and recorded as `candidate_diagnostics` rather than being silently trusted.

## Step 2: every pass must preserve a valid artifact

`run_harness` validates the materialized artifact before passes execute. After **every pass**, the resulting `ReasoningArtifact` is validated again. A pass that produces invalid state causes a fail-closed harness error rather than continuing with a corrupted artifact.

The current `reason run` product pass chain includes:

1. structured adversarial discovery;
2. evidence qualification;
3. structured-fact verification;
4. explicitly supplied trusted verification receipts;
5. Five Whys restatement checks;
6. assumption discovery.

The exact diagnostics are useful for inspection, but discovery/diagnostic code does not automatically acquire final-verdict authority.

## Step 3: deterministic structured verification can establish hard state

A claim may contain a typed proposition such as:

```json
{
  "key": "service.region",
  "value": "us-east-1"
}
```

Harness-owned evidence may contain structured facts:

```json
{
  "id": "e1",
  "facts": {
    "service.region": "us-east-1"
  }
}
```

The built-in structured verifier performs deterministic comparison of the proposition key/value with the evidence facts.

### Matching observed values

If observed values for the proposition key agree with the proposed value, the verifier creates a harness-owned receipt with conclusion `supported`.

Applying that receipt changes the claim state to `supported` and binds the receipt's evidence IDs.

### Conflicting observed value

Without qualification requirements, an observed different value produces a receipt with conclusion `contradicted`, which can change the claim state to `contradicted`.

### Missing evidence

If no matching structured fact exists, no hard receipt is produced. The claim is not guessed into `supported`; uncertainty remains.

### Evidence qualification

When evidence requirements are configured, the qualification-aware verifier filters evidence by harness-owned temporal/scope/provenance/authority requirements before it may create a hard receipt.

Insufficient qualification withholds hard authority. In particular, multiple conflicting qualified values are treated as an evidence conflict and the verifier withholds a hard receipt, preserving uncertainty instead of manufacturing certainty from ambiguous evidence.

## Step 4: trusted external verification can be added without giving the model authority

`reason run --receipts ...` can consume explicitly supplied `VerificationReceipt` values from a trusted external verifier compatibility path.

A stronger integration can implement a typed verifier adapter for a deterministic or domain-authoritative system such as:

- a test runner;
- a compiler/schema validator;
- a database/query result;
- a policy engine;
- a trusted human-review system;
- another domain-specific oracle.

The important boundary is that the same untrusted model that authored the claim should not also be treated as the hard verifier of that claim.

## Step 5: diagnostics expose problems without silently becoming authority

The product pipeline can record signals about contradictions/counterexamples, evidence qualification, assumptions, and reasoning structure.

Those signals are intentionally separated from trusted verification. A diagnostic may explain why more work is needed; it does not, merely by existing, create trusted evidence or a verification receipt.

The model-backed D3 semantic runtime is even more explicitly separated: it is exposed through `reason semantic-check`, can preserve a soft decision or force `abstain`, and cannot create trusted evidence, hard receipts, epistemic promotion, or final verdict authority.

## Step 6: the acceptance policy decides the aggregate verdict

The current `StrictAcceptancePolicy` runs only after the artifact has crossed materialization, validation, and the configured passes.

Its aggregate rule is conservative:

```text
no claims
   -> unknown

any claim == contradicted
   -> reject

else any claim == assumed or unknown
   -> unknown

else
   -> accept
```

This is why `unknown` is a normal successful epistemic result rather than a CLI error. A completed run that correctly discovers insufficient support exits process code `0`; consumers inspect `result.outcome.verdict` for the epistemic result.

Operational failures such as malformed input, invalid config, unavailable providers, timeouts, or invalid harness state are separate and return non-zero process status.

## What an AI-free run can and cannot determine

An AI-free `--candidate` run is powerful when truth conditions can be represented through typed evidence, deterministic verifiers, explicit assumptions, and conservative policy.

It does **not** magically understand arbitrary prose. If a candidate says "this architecture is probably resilient" and there is no typed proposition/verifier/evidence relationship capable of establishing that claim, the harness should remain uncertain rather than invent semantic understanding.

Use `reason semantic-check` when you intentionally want model-backed soft semantic diagnostics, but keep those results separate from hard authority.

## Practical integration pattern

A production application commonly owns the transformation into the contracts:

```text
raw documents / API data / test output
                 |
                 v
        application integration
          |                 |
          v                 v
     HarnessInput      model prompt/schema
   owned evidence            |
                             v
                    ReasoningCandidate
                             |
                 +-----------+
                 v
             reason run
                 |
       +---------+---------+
       |         |         |
     accept    reject    unknown
       |         |         |
   continue    block    retrieve/review
```

For RAG, retrieval does not automatically confer trust. The application must deliberately encode source/provenance and, where needed, evidence qualification or trusted verifier results.

## Related docs

- [CLI guide](cli.md)
- [Architecture](architecture.md)
- [Evidence qualification](evidence-qualification.md)
- [Grounded resolution](grounded-resolution.md)
- [ADR-0001: interface and packaging boundaries](adr/0001-interface-and-packaging-boundaries.md)
- [ADR-0002: grounded resolution and finalization](adr/0002-grounded-resolution-and-finalization.md)

- [product dogfood](product-dogfood.md)
