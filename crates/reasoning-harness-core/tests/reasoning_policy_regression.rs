use std::{collections::BTreeSet, fs, path::Path};

use reasoning_harness_core::{
    PolicyInvalidationTarget, ReasoningArtifact, ReasoningPolicy, Verdict, apply_reasoning_policy,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct PolicyRegressionFixture {
    id: String,
    artifact: ReasoningArtifact,
    #[serde(default)]
    previous_policy: Option<ReasoningPolicy>,
    policy: ReasoningPolicy,
    expected_verdict: Verdict,
    #[serde(default)]
    expected_invalidated_claim_ids: BTreeSet<String>,
    #[serde(default)]
    expected_invalidated_inference_ids: BTreeSet<String>,
    #[serde(default)]
    expected_invalidated_receipt_ids: BTreeSet<String>,
    expected_finalization_invalidated: bool,
}

fn load_fixtures() -> Vec<PolicyRegressionFixture> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/policy");
    let mut paths = fs::read_dir(root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect::<Vec<_>>();
    paths.sort();
    paths
        .into_iter()
        .map(|path| serde_json::from_slice(&fs::read(path).unwrap()).unwrap())
        .collect()
}

#[test]
fn policy_regression_fixtures_invalidate_only_new_snapshot_and_propagate_dependencies() {
    let fixtures = load_fixtures();
    assert_eq!(fixtures.len(), 4);

    for fixture in fixtures {
        let original = fixture.artifact.clone();
        let transition = apply_reasoning_policy(
            &fixture.artifact,
            fixture.previous_policy.as_ref(),
            &fixture.policy,
        )
        .unwrap_or_else(|error| panic!("{}: {error}", fixture.id));

        assert_eq!(
            transition.verdict_after_re_evaluation, fixture.expected_verdict,
            "{} verdict",
            fixture.id
        );
        assert_eq!(
            transition.finalization_invalidated, fixture.expected_finalization_invalidated,
            "{} finalization invalidation",
            fixture.id
        );

        let invalidated_claim_ids = transition
            .invalidations
            .iter()
            .filter_map(|invalidation| match &invalidation.target {
                PolicyInvalidationTarget::Claim { claim_id } => Some(claim_id.clone()),
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        let invalidated_inference_ids = transition
            .invalidations
            .iter()
            .filter_map(|invalidation| match &invalidation.target {
                PolicyInvalidationTarget::Inference { inference_id } => Some(inference_id.clone()),
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        let invalidated_receipt_ids = transition
            .invalidations
            .iter()
            .filter_map(|invalidation| match &invalidation.target {
                PolicyInvalidationTarget::VerificationReceipt { receipt_id } => {
                    Some(receipt_id.clone())
                }
                _ => None,
            })
            .collect::<BTreeSet<_>>();

        assert_eq!(
            invalidated_claim_ids, fixture.expected_invalidated_claim_ids,
            "{} claim invalidations",
            fixture.id
        );
        assert_eq!(
            invalidated_inference_ids, fixture.expected_invalidated_inference_ids,
            "{} inference invalidations",
            fixture.id
        );
        assert_eq!(
            invalidated_receipt_ids, fixture.expected_invalidated_receipt_ids,
            "{} receipt invalidations",
            fixture.id
        );
        assert!(transition.artifact.inferences.iter().all(|inference| {
            !fixture
                .expected_invalidated_inference_ids
                .contains(&inference.id)
        }));
        assert!(
            transition
                .artifact
                .verification_receipts
                .iter()
                .all(|receipt| !fixture
                    .expected_invalidated_receipt_ids
                    .contains(&receipt.id))
        );

        // Policy application returns a new accepted-state snapshot; historical input is untouched.
        assert_eq!(fixture.artifact, original, "{} mutated input", fixture.id);
    }
}
