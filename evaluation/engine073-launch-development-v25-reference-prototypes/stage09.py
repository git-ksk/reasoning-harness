from pathlib import Path
import gzip,subprocess,hashlib
p=Path.cwd()/'crates/reasoning-harness-core/src/evidence_relevance.rs';s=p.read_text();Path('/private/tmp/engine073-source27-stage08-source.rs.gz').write_bytes(gzip.compress(p.read_bytes(),mtime=0))
def rep(a,b):
 global s
 assert s.count(a)==1,(a[:90],s.count(a));s=s.replace(a,b,1)
anchor='fn launch_unowned_quoted_event_v17(value: &str) -> bool {'
rep(anchor,r'''// Colon and copular declarations establish the same outside printed type.
// Reply labels are wrappers only; they confer no source trust or event owner.
fn launch_printed_declaration_v17(policy: &EvidenceRelevanceTargetPolicy, statement: &str) -> bool {
    let statement = statement.split_once(':').map_or(statement, |(label, value)| {
        if launch_owner_reply_label_v17(label) { value.trim() } else { statement }
    });
    if statement.split_once(':').is_some_and(|(label, value)| {
        ["object", "subject", "item"].contains(&normalized(label).as_str())
            && launch_printed_quoted_property_v17(policy, value.trim())
    }) { return true; }
    statement.to_ascii_lowercase().find(" is ").is_some_and(|index| {
        let label = normalized(&statement[..index]);
        let role = label.strip_prefix("the ").unwrap_or(&label);
        ["object", "subject", "item"].contains(&role)
            && launch_printed_quoted_property_v17(policy, statement[index+4..].trim())
    })
}

fn launch_quoted_reply_body_v17(statement: &str) -> Option<&str> {
    let (label, value) = statement.split_once(':')?;
    let label = normalized(label);
    if !matches!(label.split_whitespace().collect::<Vec<_>>().as_slice(),
        ["quoted", "author" | "editor" | "publisher" | "newsroom", "reply" | "statement"]) { return None; }
    let (head, body, tail) = launch_literal_parts_v17(value.trim())?;
    (head.trim().is_empty() && tail.trim().is_empty()).then_some(body)
}

'''+anchor)
a='''        let mut pending = vec![segment.raw.clone()];'''
b='''        // A reported reply is discourse containing its own correction clauses,
        // rather than the inert literal value of a printed property.
        let raw = launch_quoted_reply_body_v17(&segment.raw).unwrap_or(&segment.raw);
        let mut pending = vec![raw.to_owned()];''';rep(a,b)
a='''            if segment.raw.split_once(':').is_some_and(|(label, value)| {
                ["object", "subject", "item"].contains(&normalized(label).as_str())
                    && launch_printed_quoted_property_v17(policy, value.trim())
            }) || launch_complete_printed_property_correction_v17(policy, &segment.raw)'''
b='''            if launch_printed_declaration_v17(policy, &segment.raw)
                || launch_complete_printed_property_correction_v17(policy, &segment.raw)''';rep(a,b)
a='''                let printed_property = segment.raw.split_once(':').is_some_and(|(label, value)| {
                    ["object", "subject", "item"].contains(&normalized(label).as_str())
                        && launch_printed_quoted_property_v17(policy, value.trim())
                });'''
b='''                let printed_property = launch_printed_declaration_v17(policy, &segment.raw);''';rep(a,b)
a='''    fn v30_printed_title_literals_keep_outside_property_scope() {
''';b=a+'''        for (open, close) in [("'", "'"), ("\\\"", "\\\""), ("‘", "’"), ("“", "”")] {
            for literal in ["We deny that Silver Lens was launched", "The product label is missing"] {
                check_current_role_context(&format!("The Object is the printed cover title {open}{literal}{close} on the Silver Lens booklet. We deny this Object."), true);
            }
        }
''';rep(a,b)
a='''    fn v30_local_role_appositions_and_denial_targets_need_complete_scope() {
''';b=a+'''        check_current_role_context("Item: the printed launch-selector font statement for Silver Lens. Quoted author reply: \\\"I withdraw that item because we deny the printed selector-colour claim for Silver Lens.\\\" The reply corrects two printed properties.", true);
''';rep(a,b)
p.write_text(s);subprocess.run(['cargo','fmt','--all'],check=True);print(hashlib.sha256(p.read_bytes()).hexdigest())
