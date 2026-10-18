from pathlib import Path
import gzip,subprocess,hashlib
p=Path.cwd()/'crates/reasoning-harness-core/src/evidence_relevance.rs';s=p.read_text();Path('/private/tmp/engine073-source27-stage06-source.rs.gz').write_bytes(gzip.compress(p.read_bytes(),mtime=0))
a='''| ["is" | "was" | "are" | "were", verb, "by", "us" | "me" | "author" | "editor" | "publisher"] if launch_retraction_word_v17(verb));'''
b='''| ["is" | "was" | "are" | "were", verb, "by", "us" | "me" | "author" | "editor" | "publisher"]
                | ["is" | "was" | "are" | "were", verb, "by", "its" | "the" | "our", "author" | "editor" | "publisher"] if launch_retraction_word_v17(verb));'''
assert s.count(a)==1;s=s.replace(a,b,1)
a='''    fn v30_local_role_appositions_and_denial_targets_need_complete_scope() {
''';b=a+'''        check_current_role_context("Item: the launch-button width claim for Silver Lens. This item—and only this physical width claim—is withdrawn by its author.", true);
''';assert s.count(a)==1;s=s.replace(a,b,1)
# Exercise the generic quoted event with all balanced quote styles.
a='''            check_current_role_context(
                &format!(
                    "The Object is the quoted launch claim: 'The product was launched.' Immediate author reply: {owners}. We retract this Object."
                ),
                expected,
            );'''
b='''            for (open, close) in [("'", "'"), ("\\\"", "\\\""), ("‘", "’"), ("“", "”")] {
                check_current_role_context(&format!("The Object is the quoted launch claim: {open}The product was launched.{close} Immediate author reply: {owners}. We retract this Object."), expected);
            }'''
assert s.count(a)==1;s=s.replace(a,b,1);p.write_text(s);subprocess.run(['cargo','fmt','--all'],check=True);print(hashlib.sha256(p.read_bytes()).hexdigest())
