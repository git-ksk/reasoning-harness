from pathlib import Path
import subprocess,hashlib
p=Path('crates/reasoning-harness-core/src/evidence_relevance.rs');s=p.read_text();a=s.index('    fn v30_quote_truth_status_is_not_a_printed_format_property()');b=s.index('\n    #[test]',a);i=s.rfind('\n    }',a,b);assert i>a
s=s[:i]+r'''
        for body in ["the legend \"Silver Lens was launched\" on the Cedar Basket instrument diagram has a dotted underline","the quotation \"Silver Lens was launched\" in the Cedar Basket guide is underlined","the label reading \"Silver Lens was launched\" on the Cedar Basket package has a scalloped edge"] {
            let text=format!("Object: the claim that {body}. I withdraw the Object.");let local=candidate(vec![(EvidenceRelevanceSignalKind::Excerpt,&text)]);
            assert_ne!(derive_effective_evidence_local_qualification_v17(&policy,&local,Some(&proposal),Some(&raw)).unwrap().relation_scope,EvidenceLocalRelationScope::RequestedRelation,"printed text alone: {text}");
            assert_ne!(materialize_evidence_relevance_v30(&policy,&local,Some(&proposal),Some(&raw)).unwrap().disposition,EvidenceRelevanceDisposition::Relevant,"printed text alone: {text}");
        }
'''+s[i:];p.write_text(s);subprocess.run(['cargo','fmt','--all'],check=True);print(hashlib.sha256(p.read_bytes()).hexdigest())
