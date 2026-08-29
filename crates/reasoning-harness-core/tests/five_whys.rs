use reasoning_harness_core::frameworks::five_whys::{FiveWhysTrace, WhyLink, validate_trace};

#[test]
fn flags_a_circular_restatement() {
    let trace = FiveWhysTrace {
        symptom: "request failed".into(),
        links: vec![WhyLink {
            effect: "request failed".into(),
            cause: "request failed".into(),
            evidence_ids: vec![],
        }],
        root_cause: "request failed".into(),
    };
    assert_eq!(validate_trace(&trace).len(), 1);
}
