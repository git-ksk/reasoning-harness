from pathlib import Path
import hashlib,json
r=Path.cwd();p=r/'crates/reasoning-harness-core/src/evidence_relevance.rs'
base=Path('/private/tmp/engine073-trace-current26-base.rs').read_text();assert hashlib.sha256(base.encode()).hexdigest()=='9d8d3fa48a67f197d32282314813d992a374b9f54a6b079c39498b9a67f9207f';s=base
old='''                if launch_owner_frame_role_v17(&label).is_some() {
                    declarations.push((label, raw[index + 4..].trim().to_owned()));
                }'''
new='''                let role = label.strip_prefix("the ").unwrap_or(&label);
                if ["object", "subject", "item"].contains(&role) {
                    // Copular and colon declarations carry the same local role.
                    // The value still needs its own complete scope/owner proof.
                    declarations.push((role.to_owned(), raw[index + 4..].trim().to_owned()));
                } else if launch_owner_frame_role_v17(&label).is_some() {
                    declarations.push((label, raw[index + 4..].trim().to_owned()));
                }'''
assert s.count(old)==1;s=s.replace(old,new)
needle='''    let plain_tail = tail.iter().all(|word| {'''
insert='''    if label.is_some() {
        // A finite claim wrapper points to the already selected role; it
        // supplies no event ownership or scope of its own.
        for prefix in [
            &["the", "event", "claim", "represented", "by"][..],
            &["the", "claim", "represented", "by"][..],
            &["the", "statement", "represented", "by"][..],
        ] {
            if let Some(rest) = tail.strip_prefix(prefix) {
                tail = rest;
                break;
            }
        }
    }
'''
assert s.count(needle)==1;s=s.replace(needle,insert+needle)
p.write_text(s)
print(hashlib.sha256(p.read_bytes()).hexdigest())
