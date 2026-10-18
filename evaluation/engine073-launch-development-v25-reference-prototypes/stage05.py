from pathlib import Path
import gzip,subprocess,hashlib
p=Path.cwd()/'crates/reasoning-harness-core/src/evidence_relevance.rs';s=p.read_text();Path('/private/tmp/engine073-source27-stage04-source.rs.gz').write_bytes(gzip.compress(p.read_bytes(),mtime=0))
a='''            let tail = tail.trim();
            [
                "does not exist",'''
b='''            let tail = tail.trim();
            let tail = tail.strip_prefix("itself ").unwrap_or(tail);
            let tail = tail.split_once(" and ").map_or(tail, |(description, predicate)| {
                let words = description.split_whitespace().collect::<Vec<_>>();
                if matches!(words.as_slice(), ["is", "a" | "an", "invented" | "fictional" | "imaginary", "product" | "service"]) { predicate } else { tail }
            });
            [
                "does not exist",'''
assert s.count(a)==1;s=s.replace(a,b,1)
needle='''        for (context, expected) in [
            (
                "The Subject is the opening-pattern illustration'''
new='''        check_current_role_context("The Subject is the opening-pattern illustration printed on the Silver Lens package. We retract that Subject. Silver Lens itself is an invented product and has never existed.", false);
        check_current_role_context("The Subject is the opening-pattern illustration printed on the Silver Lens package. We retract that Subject. Silver Lens itself has never existed.", false);
        for (context, expected) in [
            (
                "The Subject is the opening-pattern illustration'''
assert s.count(needle)==1;s=s.replace(needle,new,1);p.write_text(s);subprocess.run(['cargo','fmt','--all'],check=True);print(hashlib.sha256(p.read_bytes()).hexdigest())
