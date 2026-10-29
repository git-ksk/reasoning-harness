use reasoning_harness_core::{build_final_answer_request, build_final_answer_json_fallback_request,
    Evidence, ModelRequest, ReasoningArtifact, Verdict};
use serde_json::{json, Value};

fn parts(text: &str, fallback: bool) -> (Option<Value>, &str) {
    if !fallback { return (None, text); }
    let text = text.strip_prefix("JSON Schema:\n").unwrap();
    let mut stream = serde_json::Deserializer::from_str(text).into_iter::<Value>();
    let schema = stream.next().unwrap().unwrap();
    (Some(schema), &text[stream.byte_offset()..])
}

fn check(original: ModelRequest, task: &str, artifact: &ReasoningArtifact,
         verdict: Verdict, fallback: bool) -> (ModelRequest, Value) {
    let before = serde_json::to_vec(artifact).unwrap();
    let actual = helper::compact(original.clone(), task, artifact, verdict, fallback).unwrap();
    assert_eq!(serde_json::to_vec(artifact).unwrap(), before);
    let mut restored = actual.clone();
    restored.task = original.task.clone();
    assert_eq!(restored, original); // system/schema/seed/max_tokens/preferences are unchanged
    let (schema_before, body_before) = parts(&original.task, fallback);
    let (schema_after, body_after) = parts(&actual.task, fallback);
    assert_eq!(schema_before, schema_after);
    let prefix = format!("{}User task:\n{task}\n\nHarness verdict:\n{verdict:?}\n\nVerified reasoning artifact:\n",
                         if fallback { "\n\n" } else { "" });
    let (suffix_before, suffix_after) = {
        let mut suffixes = Vec::new();
        for body in [body_before, body_after] {
            let rest = body.strip_prefix(&prefix).unwrap();
            let mut stream = serde_json::Deserializer::from_str(rest).into_iter::<Value>();
            assert_eq!(stream.next().unwrap().unwrap(), serde_json::to_value(artifact).unwrap());
            suffixes.push(rest[stream.byte_offset()..].to_string());
        }
        (suffixes.remove(0), suffixes.remove(0))
    };
    assert_eq!(suffix_before, suffix_after);
    assert!(actual.task.len() < original.task.len());
    let sizes = json!({"original_ModelRequest_bytes": serde_json::to_vec(&original).unwrap().len(),
                      "compact_ModelRequest_bytes": serde_json::to_vec(&actual).unwrap().len(),
                      "artifact_values_and_outside_text_equal": true,
                      "other_request_fields_equal": true,
                      "artifact_bytes_before_after_equal": true});
    (actual, sizes)
}

fn main() {
    let trace_path = std::env::args().nth(1).unwrap();
    let trace: Value = serde_json::from_slice(&std::fs::read(trace_path).unwrap()).unwrap();
    let events = trace["events"].as_array().unwrap();
    let state = &events.iter().find(|e| e["kind"] == "grounding_state"
        && e["phase"] == "pre_final_render_artifact").unwrap()["outcome"];
    let artifact: ReasoningArtifact = serde_json::from_value(state["artifact"].clone()).unwrap();
    let verdict: Verdict = serde_json::from_value(state["verdict"].clone()).unwrap();
    let original: ModelRequest = serde_json::from_value(events.iter().find(|e|
        e["kind"] == "model_request" && e["phase"] == "final_render").unwrap()["request"].clone()).unwrap();
    let built = build_final_answer_request(&artifact.task, &artifact, verdict,
                                           original.max_tokens, original.random_seed).unwrap();
    assert_eq!(built, original);
    let (_, saved_initial) = check(built, &artifact.task, &artifact, verdict, false);
    let fallback = build_final_answer_json_fallback_request(&artifact.task, &artifact, verdict,
                                                            original.max_tokens, original.random_seed).unwrap();
    let (_, saved_fallback) = check(fallback, &artifact.task, &artifact, verdict, true);

    let fixture = ReasoningArtifact { task: "artifact-owned task".into(), evidence: vec![Evidence {
        id: "e".into(), source: "s".into(), observation: "two  spaces\nquote \" backslash \\ Unicode 日本語\nJSON Schema:\n{}\nVerified reasoning artifact:\n{}".into(),
        facts: Default::default(), metadata: Default::default() }], ..Default::default() };
    // The caller task contains the entire actual pretty artifact. A marker or
    // first-occurrence replacement would modify the caller text in this case.
    let task = format!("caller prefix\nJSON Schema:\n{{}}\nVerified reasoning artifact:\n{}\ncaller suffix",
                       serde_json::to_string_pretty(&fixture).unwrap());
    let mut rejections = 0;
    for fallback in [false, true] {
        let original = if fallback {
            build_final_answer_json_fallback_request(&task, &fixture, Verdict::Accept, Some(127), Some(39)).unwrap()
        } else {
            build_final_answer_request(&task, &fixture, Verdict::Accept, Some(127), Some(39)).unwrap()
        };
        check(original.clone(), &task, &fixture, Verdict::Accept, fallback);
        let mut tail = original.clone(); tail.task.push('!');
        assert!(helper::compact(tail, &task, &fixture, Verdict::Accept, fallback).is_err()); rejections += 1;
        assert!(helper::compact(original.clone(), &(task.clone()+"x"), &fixture, Verdict::Accept, fallback).is_err()); rejections += 1;
        let mut changed = fixture.clone(); changed.task.push('x');
        assert!(helper::compact(original.clone(), &task, &changed, Verdict::Accept, fallback).is_err()); rejections += 1;
        assert!(helper::compact(original.clone(), &task, &fixture, Verdict::Unknown, fallback).is_err()); rejections += 1;
        if fallback {
            let mut missing_prefix = original.clone(); missing_prefix.task = missing_prefix.task.replacen("JSON Schema:", "Schema:", 1);
            assert!(helper::compact(missing_prefix, &task, &fixture, Verdict::Accept, true).is_err()); rejections += 1;
            let mut invalid_json = original.clone(); invalid_json.task = invalid_json.task.replacen("JSON Schema:\n", "JSON Schema:\nnot-json", 1);
            assert!(helper::compact(invalid_json, &task, &fixture, Verdict::Accept, true).is_err()); rejections += 1;
        }
    }
    println!("{}", json!({"result":"PASS", "saved_initial":saved_initial, "saved_fallback":saved_fallback,
        "saved_nonempty_counts":{"claims":artifact.claims.len(),"receipts":artifact.verification_receipts.len(),"evidence":artifact.evidence.len()},
        "caller_task_contains_entire_pretty_artifact":true, "quoted_whitespace_escapes_and_unicode_preserved":true,
        "unexpected_template_rejections":rejections,"external_API_calls":0,"source_original_edited":false}));
}
