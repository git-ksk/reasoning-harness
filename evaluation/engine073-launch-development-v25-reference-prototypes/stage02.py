from pathlib import Path
import gzip,subprocess,hashlib
p=Path.cwd()/'crates/reasoning-harness-core/src/evidence_relevance.rs';s=p.read_text();Path('/private/tmp/engine073-source27-stage01-source.rs.gz').write_bytes(gzip.compress(p.read_bytes(),mtime=0))
a='''            if let Some((head, value)) = segment.raw.split_once(',') {
                if let Some(role) = launch_single_retracted_role_v17(head) {'''
b='''            if let Some((head, value)) = segment.raw.split_once(',') {
                if let Some(role) = launch_single_retracted_role_v17(head).filter(|_| {
                    let body = normalized(value);
                    matches!(body.split_whitespace().next(), Some("the" | "a" | "an"))
                }) {'''
assert s.count(a)==1;s=s.replace(a,b,1)
a='''            prepared.push(if keep_period { ' ' } else { character });
            raw_segment.push(character);
'''
b='''            prepared.push(if keep_period { ' ' } else { character });
            raw_segment.push(character);
            // A quoted sentence's terminal period is inside the closing quote.
            // An explicit adjacent owner/reply field starts after that quote.
            let end = index + character.len_utf8();
            if literals.iter().any(|span| span.end == end)
                && value[..index].chars().next_back().is_some_and(|c| ['.', '!', '?'].contains(&c))
                && value[end..].trim_start().split_once(':').is_some_and(|(label, _)| {
                    launch_owner_reply_label_v17(label)
                        || launch_owner_frame_role_v17(&normalized(label)).is_some()
                })
            {
                finish_segment(&mut prepared, &mut raw_segment, raw_start, end, false);
                raw_start = end;
            }
'''
assert s.count(a)==1;s=s.replace(a,b,1);p.write_text(s);subprocess.run(['cargo','fmt','--all'],check=True);print(hashlib.sha256(p.read_bytes()).hexdigest())
