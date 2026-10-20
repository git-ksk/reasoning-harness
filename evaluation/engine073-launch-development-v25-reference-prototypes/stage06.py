from pathlib import Path
import gzip,subprocess,hashlib
p=Path.cwd()/'crates/reasoning-harness-core/src/evidence_relevance.rs';s=p.read_text();Path('/private/tmp/engine073-source27-stage05-source.rs.gz').write_bytes(gzip.compress(p.read_bytes(),mtime=0))
a='''                && value[end..]
                    .trim_start()
                    .split_once(':')
                    .is_some_and(|(label, _)| {
                        launch_owner_reply_label_v17(label)
                            || launch_owner_frame_role_v17(&normalized(label)).is_some()
                    })'''
b='''                && (value[end..].trim_start().chars().next().is_some_and(char::is_uppercase)
                    || value[end..].trim_start().split_once(':').is_some_and(|(label, _)| {
                        launch_owner_reply_label_v17(label)
                            || launch_owner_frame_role_v17(&normalized(label)).is_some()
                    }))'''
assert s.count(a)==1;s=s.replace(a,b,1)
a='''    fn v30_printed_title_literals_keep_outside_property_scope() {
'''
b=a+'''        check_current_role_context("Withdrawn text: \\\"Silver Lens has knurled edges.\\\" The author withdraws that knurled-edge statement.", true);
'''
assert s.count(a)==1;s=s.replace(a,b,1);p.write_text(s);subprocess.run(['cargo','fmt','--all'],check=True);print(hashlib.sha256(p.read_bytes()).hexdigest())
