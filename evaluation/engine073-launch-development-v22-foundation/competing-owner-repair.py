from pathlib import Path
import hashlib,subprocess
p=Path('crates/reasoning-harness-core/src/evidence_relevance.rs');s=p.read_text()
assert 'fn v30_complete_claim_wrapper_preserves_only_proven_property_scope()' in s and 'FOUNDATION_DIAGNOSTIC' not in s
original=s
start=s.index('fn launch_owner_frame_role_v17(')
needle='    let words = label.split_whitespace().collect::<Vec<_>>();'
at=s.index(needle,start)+len(needle)
s=s[:at]+'''
    // Competing owner statements are declarations for the same role too.
    // Ignoring a later declaration must not make the first foreign owner unique.
    let statement_words = words
        .split_first()
        .filter(|(word, _)| {
            ["first", "second", "third", "additional", "another", "alternative", "competing"]
                .contains(word)
        })
        .map_or(words.as_slice(), |(_, rest)| rest);
    if let ["owner", "statement", "for", "the", role @ ("object" | "subject" | "item")] =
        statement_words
    {
        return Some(Some(role));
    }
'''+s[at:]
needle='''                "Owner of the Object: an unknown product.",'''
assert s.count(needle)==1
s=s.replace(needle,needle+'''
                "Owner of the Object: Maple Queue. Second owner statement for the Object: Silver Lens.",
                "Owner of the Object: Maple Queue. Additional owner statement for the Object: Amber Gate.",
                "Owner of the Object: Maple Queue. Competing owner statement for the Object: an unknown product.",''')
# A same owner repeated in an explicit statement remains a unique declaration;
# the declaration for another role is not permitted to override this one.
needle='''        let local = candidate(vec![(
            EvidenceRelevanceSignalKind::Excerpt,
            "Object: the launch claim of Silver Lens. Owner of the Object: Maple Queue. I retract the Object. Silver Lens was launched.",
        )]);'''
assert s.count(needle)==1
s=s.replace(needle,'''        for owners in [
            "Owner of the Object: Maple Queue. Second owner statement for the Object: Maple Queue.",
            "Owner statement for the Object: Maple Queue. Owner of the Subject: Silver Lens.",
        ] {
            let text = format!("Object: the launch claim. {owners} I retract the Object. Silver Lens was launched.");
            let local = candidate(vec![(EvidenceRelevanceSignalKind::Excerpt, &text)]);
            assert_eq!(materialize_evidence_relevance_v30(&policy, &local, Some(&proposal), Some(&raw)).unwrap().disposition, EvidenceRelevanceDisposition::Relevant, "{text}");
        }
'''+needle)
p.write_text(s);subprocess.run(['cargo','fmt','--all'],check=True)
print('role-specific competing owner declarations retained for uniqueness check',hashlib.sha256(p.read_bytes()).hexdigest())
