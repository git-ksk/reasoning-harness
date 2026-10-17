from pathlib import Path
import subprocess,hashlib
p=Path('crates/reasoning-harness-core/src/evidence_relevance.rs');s=p.read_text();assert 'fn launch_owned_appearance_subject_v17(' in s and 'fn launch_complete_printed_attribute_v17(' not in s
anchor='fn launch_complete_property_proposition_v17(';assert s.count(anchor)==1;s=s.replace(anchor,Path('/private/tmp/engine073-printed23.rs').read_text()+'\n'+anchor,1)
needle='    if launch_complete_appearance_proposition_v17(policy, value) {';assert s.count(needle)==1;s=s.replace(needle,'    if launch_complete_printed_attribute_v17(policy, value)\n        || launch_complete_appearance_proposition_v17(policy, value) {',1)
anchor='fn launch_printed_quoted_property_v17(policy: &EvidenceRelevanceTargetPolicy, value: &str) -> bool {';assert s.count(anchor)==1;s=s.replace(anchor,anchor+'\n    if launch_complete_printed_attribute_v17(policy, value) { return true; }',1)
anchor='mod tests {\n    use super::*;';assert s.count(anchor)==1;s=s.replace(anchor,anchor+'\n'+Path('/private/tmp/engine073-printed23-tests.rs').read_text(),1);p.write_text(s);subprocess.run(['cargo','fmt','--all'],check=True);print('printed outside type/owner prototype applied',hashlib.sha256(p.read_bytes()).hexdigest())
