from pathlib import Path
import gzip,subprocess,hashlib
p=Path.cwd()/'crates/reasoning-harness-core/src/evidence_relevance.rs';s=p.read_text();Path('/private/tmp/engine073-source27-stage03-source.rs.gz').write_bytes(gzip.compress(p.read_bytes(),mtime=0))
a='''            if !passive && launch_single_retracted_role_v17(predicate).as_deref() != selected_label
            {'''
b='''            let active = launch_single_retracted_role_v17(predicate).is_some_and(|role| Some(role.as_str()) == selected_label);
            if !passive && !active {'''
assert s.count(a)==1;s=s.replace(a,b,1)
a='''    if launch_explicit_property_context_v17(policy, &format!("We withdraw {value}")) {
        return Scope::Excluded;
    }

    if original.split_whitespace().any(|word| {'''
b='''    if original.split_whitespace().any(|word| {'''
assert s.count(a)==1;s=s.replace(a,b,1)
a='''        return Scope::Unresolved;
    }
    let targets = target_identity_phrases(policy);'''
b='''        return Scope::Unresolved;
    }
    if launch_explicit_property_context_v17(policy, &format!("We withdraw {value}")) {
        return Scope::Excluded;
    }
    let targets = target_identity_phrases(policy);'''
assert s.count(a)==1;s=s.replace(a,b,1)
p.write_text(s);subprocess.run(['cargo','fmt','--all'],check=True);print(hashlib.sha256(p.read_bytes()).hexdigest())
