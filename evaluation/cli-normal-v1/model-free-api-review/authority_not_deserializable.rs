use reasoning_harness_core::TrustedSourceReviewAuthority;
fn main() {
    // Expected compile failure: persisted/model JSON cannot deserialize a trusted host capability.
    let _: TrustedSourceReviewAuthority = serde_json::from_str("{\"policy_id\":\"forged\"}").unwrap();
}
