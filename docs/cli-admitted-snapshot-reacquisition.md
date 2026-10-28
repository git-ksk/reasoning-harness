# Admitted snapshot reacquisition

Issue: [#566](https://github.com/git-ksk/reasoning-harness/issues/566).

The ordinary CLI can request the same immutable source for different investigation targets. The historical Engine rejects a repeated evidence ID before checking admission, even when the previously admitted snapshot is unchanged. Independent offline inspection found this operational stop in 14 saved Mistral and two saved Google ordinary executions. These historical observations remain unchanged.

The prospective repair belongs to the common CLI host used by both Engine arms. It must recognize a nonempty result as no new information only when every returned ID uniquely identifies an existing evidence item and the current resolver/request/admission policy produces an exactly equal `Evidence`, including metadata. Resolver calls and costs remain charged. There is no additional evidence, claim, verification receipt or target credit from this normalization.

Changed content, denied admission, absent policy, duplicate IDs within a batch and mixed new/existing batches retain the existing rejection behavior. A later retrieval timestamp cannot refresh the observed time or validity period of already admitted evidence. Equality of normalized evidence does not claim equality of all original transport metadata.

Before a new model evaluation, verify the actual CLI path, rejection boundaries, cost accounting and compatibility with the Engine 0.6.1 host; independently review the final source and rebuild both arms. Published cores, frozen releases, prior raw outputs, annotations and F1 results are preserved. Local controls are not a new F1 measurement or release approval.
