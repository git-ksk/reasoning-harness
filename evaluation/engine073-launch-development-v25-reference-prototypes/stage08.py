from pathlib import Path
import gzip,subprocess,hashlib
p=Path.cwd()/'crates/reasoning-harness-core/src/evidence_relevance.rs';s=p.read_text();Path('/private/tmp/engine073-source27-stage07-source.rs.gz').write_bytes(gzip.compress(p.read_bytes(),mtime=0))
def rep(a,b):
 global s
 assert s.count(a)==1,(a[:90],s.count(a));s=s.replace(a,b,1)
a='''    let mut literals = Vec::new();
    let mut cursor = 0;
    while let Some((head, _, tail)) = launch_literal_parts_v17(&value[cursor..]) {
        let end = value.len() - tail.len();
        literals.push(cursor + head.len()..end);
        cursor = end;
    }
'''
rep(a,'''    let literals = launch_literal_spans_v17(value);
''')
anchor='fn launch_factual_segments_v17('
rep(anchor,'''fn launch_literal_spans_v17(value: &str) -> Vec<std::ops::Range<usize>> {
'''+a+'''    literals
}

'''+anchor)
rep('''    let mut declarations = Vec::new();
    for signal in candidate.signals.iter().filter(|signal| {''','''    let mut declarations = Vec::new();
    let mut appositions = Vec::new();
    for signal in candidate.signals.iter().filter(|signal| {''')
a='''                    declarations.push((
                        role,
                        value''';b='''                    appositions.push((
                        role,
                        value''';rep(a,b)
rep('''    let owners = declarations
        .iter()''','''    // Complete explicit declarations bind a later same-role apposition.
    // Only otherwise undeclared roles acquire a value from that apposition;
    // retain every competing fallback instead of silently choosing the first.
    let declared_roles = declarations.iter().map(|(label, _)| label.clone()).collect::<Vec<_>>();
    declarations.extend(appositions.into_iter().filter(|(label, _)| !declared_roles.contains(label)));
    let owners = declarations
        .iter()''')
rep('''        while let Some(raw) = pending.pop() {
            let folded = raw.to_ascii_lowercase();''','''        while let Some(raw) = pending.pop() {
            let literals = launch_literal_spans_v17(&raw);
            let folded = raw.to_ascii_lowercase();''')
rep('''                        " before ",
                        " as ",''','''                        " before ",
                        " but ",
                        " as ",''')
rep('''                        for (index, _) in folded.match_indices(marker) {
                            if marker == " as "''','''                        for (index, _) in folded.match_indices(marker) {
                            if literals.iter().any(|span| span.contains(&index)) { continue; }
                            if marker == " as "''')
a='''    fn v30_local_role_appositions_and_denial_targets_need_complete_scope() {
'''
b=a+'''        check_current_role_context("Object: a physical specification for Silver Lens. We deny this object, namely the complete release-lever width statement for Silver Lens, but we also deny the lever-height specification for Silver Lens. Both clauses concern physical dimensions.", true);
        check_current_role_context("Subject: the launch of Violet Bell. We withdraw this Subject, the event claim owned by Violet Bell. We deny the font claim about Silver Lens box label.", true);
        check_current_role_context("Subject: the launch of Violet Bell. We withdraw this Subject, the event claim owned by Silver Lens.", false);
''';rep(a,b)
p.write_text(s);subprocess.run(['cargo','fmt','--all'],check=True);print(hashlib.sha256(p.read_bytes()).hexdigest())
