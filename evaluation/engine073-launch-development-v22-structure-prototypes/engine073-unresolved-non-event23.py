from pathlib import Path
import subprocess
p=Path('crates/reasoning-harness-core/src/evidence_relevance.rs');s=p.read_text()
needle='''            if launch_target_nonexistence_v17(policy, &segment.raw) {
                return true;
            }''';assert s.count(needle)==1
s=s.replace(needle,needle+'''
            // An expressly unknown event owner cannot inherit a preceding
            // foreign role or property scope merely through conjunction.
            if [" was not launched", " was not released"].iter().any(|suffix| {
                text.strip_suffix(suffix).is_some_and(|owner| {
                    matches!(owner, "an unknown product" | "the unknown product" | "an unidentified product" | "the unidentified product" | "an unnamed product" | "the unnamed product")
                })
            }) { return true; }
''',1)
p.write_text(s);subprocess.run(['cargo','fmt','--all'],check=True)
