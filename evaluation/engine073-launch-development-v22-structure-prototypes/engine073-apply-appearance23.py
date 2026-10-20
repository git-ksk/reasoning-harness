from pathlib import Path
import hashlib,subprocess
p=Path('crates/reasoning-harness-core/src/evidence_relevance.rs');s=p.read_text();assert hashlib.sha256(p.read_bytes()).hexdigest()=='6b5aa28035bf23e6ad5eded264b323109786963354f3769a74fffd41ebe54a31'
a=s.index('fn launch_complete_property_proposition_v17(');b=s.index('\nfn launch_printed_quoted_property_v17(',a);t=s[a:b]
start=t.index('    // A complete assertion wrapper');end=t.index('    let property = launch_property_words_v17',start)
t=t[:start]+'''    let value = launch_complete_assertion_body_v17(value);
    if launch_complete_appearance_proposition_v17(policy, value) {
        return true;
    }
'''+t[end:]
s=s[:a]+Path('/private/tmp/engine073-appearance23.rs').read_text()+'\n'+t+s[b:]
anchor='mod tests {\n    use super::*;';assert s.count(anchor)==1;s=s.replace(anchor,anchor+'\n'+Path('/private/tmp/engine073-appearance23-tests.rs').read_text(),1);p.write_text(s);subprocess.run(['cargo','fmt','--all'],check=True);print('complete owned appearance predicate prototype applied')
