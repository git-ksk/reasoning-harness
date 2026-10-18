from pathlib import Path
p=Path('crates/reasoning-harness-core/src/evidence_relevance.rs');s=p.read_text();anchor='mod tests {\n    use super::*;';assert s.count(anchor)==1
module=r'''
    fn check_current_role_context(context: &str, expected: bool) {
        let policy = launch_policy_for_v28();
        let proposal = EvidenceRelevanceBindingProposal { target_binding: EvidenceRelevanceBinding::Exact, relation_binding: EvidenceRelevanceBinding::Exact };
        let raw = EvidenceLocalQualificationV6 { identity_scope: EvidenceLocalIdentityScope::ExactTarget, relation_scope: EvidenceLocalRelationScope::RequestedRelation, scope_risk: EvidenceLocalBlockingReason::None };
        let local = candidate(vec![(EvidenceRelevanceSignalKind::Excerpt, "Silver Lens was launched."), (EvidenceRelevanceSignalKind::Excerpt, context)]);
        assert_eq!(derive_effective_evidence_local_qualification_v17(&policy,&local,Some(&proposal),Some(&raw)).unwrap().relation_scope == EvidenceLocalRelationScope::RequestedRelation,expected,"v17: {context}");
        assert_eq!(materialize_evidence_relevance_v30(&policy,&local,Some(&proposal),Some(&raw)).unwrap().disposition == EvidenceRelevanceDisposition::Relevant,expected,"v30: {context}");
    }

    #[test]
    fn v30_copular_roles_preserve_property_and_target_denial_boundaries() {
        for declaration in ["Object: ","The Object is ","Object is "] {
            for (value,expected) in [("the green color of the tamper seal on Silver Lens",true),("the claim that Silver Lens was launched",false),("an unknown launch claim",false)] {
                check_current_role_context(&format!("{declaration}{value}. We deny the claim about this Object."),expected);
            }
        }
        check_current_role_context("The Object is the green color of the tamper seal on Silver Lens. We deny the claim represented by this Object.",true);
        check_current_role_context("The Object is the claim that Silver Lens was launched. We deny the claim represented by this Object.",false);
    }

    #[test]
    fn v30_copular_role_owners_cannot_be_borrowed_or_conflicted() {
        for (context,expected) in [
            ("The Object is a launch claim. Object owner: Violet Bell. We withdraw the claim represented by this Object.",true),
            ("The Object is a launch claim. Object owner: Silver Lens. We withdraw the claim represented by this Object.",false),
            ("The Object is a launch claim. Object owner: unknown. We withdraw the claim represented by this Object.",false),
            ("The Object is a launch claim. Object owner: Violet Bell. Second Object owner: Cedar Basket. We withdraw this Object.",false),
            ("The Object is a launch claim. Subject owner: Violet Bell. We withdraw this Object.",false),
            ("The Object is the claim that Violet Bell was launched. Object owner: Cedar Basket. We withdraw this Object.",false),
            ("The Object is the claim that Violet Bell was launched. Object owner: Silver Lens. We withdraw this Object.",false),
            ("The Object is the claim that Violet Bell was launched. Object owner: Violet Bell. We withdraw this Object.",true),
        ] { check_current_role_context(context,expected); }
    }

    #[test]
    fn v30_immediate_and_competing_role_replies_keep_same_owner_checks() {
        for (context,expected) in [
            ("The Object is a launch claim. Immediate author reply: Object owner is Violet Bell. We deny that Object.",true),
            ("The Object is a launch claim. Immediate author reply: Object owner is Violet Bell. Second author reply: Object owner is Silver Lens. We deny that Object.",false),
            ("The Object is a launch claim. Immediate author reply: Subject owner is Violet Bell. We deny that Object.",false),
            ("Author statement: The Subject is the green color of the tamper seal on Silver Lens. Author reply: We withdraw the assertion about this Subject.",true),
        ] { check_current_role_context(context,expected); }
    }

    #[test]
    fn v30_gerund_target_denials_cannot_inherit_copular_property_scope() {
        for clause in ["while denying that Silver Lens was launched", "before denying the claim that Silver Lens was launched"] {
            check_current_role_context(&format!("The Object is the green color of the tamper seal on Silver Lens. We withdraw that Object {clause}."),false);
        }
    }
'''
s=s.replace(anchor,anchor+module,1);p.write_text(s)
