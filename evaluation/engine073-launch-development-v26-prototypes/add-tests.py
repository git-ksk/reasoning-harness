from pathlib import Path
import hashlib
p=Path('crates/reasoning-harness-core/src/evidence_relevance.rs');s=p.read_text();assert hashlib.sha256(p.read_bytes()).hexdigest()=='4f82b2e41614394f1bd528f01c817f0e962f1c546d7212f75f15c2f9c5787999'
tests=r'''
    #[test]
    fn v30_prefixed_physical_role_and_material_attributes_bind_locally() {
        for (context, expected) in [
            ("Workshop correction: The Object is the stated physical weight of the woven strap supplied with Silver Lens. I retract the assertion about this Object. My correction concerns the strap's weight.", true),
            ("Workshop correction: The Object is unknown. I retract the assertion about this Object. My correction concerns the strap's weight.", false),
            ("Workshop correction: The Object is the stated physical weight of the woven strap supplied with Silver Lens. I retract the assertion about this Subject.", false),
            ("Workshop correction: The Object is the stated physical weight of the woven strap supplied with Silver Lens. I retract the assertion about this Object, but we deny that Silver Lens was launched.", false),
            ("The Subject is the transparent material of the Silver Lens storage box. I retract the statement about this Subject. The assertion concerns the box's material, not the existence of the product.", true),
            ("The Subject is the transparent material of the Silver Lens storage box. I retract the statement about this Subject. Silver Lens itself has never existed.", false),
            ("The Subject is the transparent material of an unknown product's storage box. I retract the statement about this Subject.", false),
            ("The Subject is the claim that Silver Lens exists. I retract the statement about this Subject. The assertion concerns the box's material.", false),
        ] { check_current_role_context(context, expected); }
    }

    #[test]
    fn v30_printed_layout_attributes_keep_literal_event_words_inert() {
        for (open, close) in [("'", "'"), ("\"", "\""), ("‘", "’"), ("“", "”")] {
            let spacing=format!("The Object is the physical letter spacing in the printed sentence {open}Silver Lens was launched.{close} on a poster. I deny the assertion about this Object. This correction concerns letter spacing alone, not the event asserted by the printed words.");
            let position=format!("The Item is the position of the printed heading {open}launch date{close} on the Silver Lens label. We retract the assertion about that Item. The correction concerns only where the heading was printed, not a launch date or an event.");
            for context in [&spacing,&position] {
                check_current_role_context(context, true);
                check_current_role_context(&format!("{context} We deny that Silver Lens was launched."), false);
            }
            check_current_role_context(&format!("The Object is the truth of the printed sentence {open}Silver Lens was launched.{close} on a poster. I deny the assertion about this Object."),false);
        }
        check_current_role_context("The Object is the physical letter spacing in the printed sentence 'Silver Lens was launched. on a poster. I deny the assertion about this Object.",false);
        check_current_role_context("The Item is the position of the printed heading 'launch date' on an unknown label. We retract the assertion about that Item.",false);
    }

    #[test]
    fn v30_foreign_role_owner_complements_require_matching_declarations() {
        for (declaration,reference,expected) in [
            ("Violet Bell","Violet Bell",true),
            ("Silver Lens","Violet Bell",false),
            ("Violet Bell","Silver Lens",false),
            ("unknown","Violet Bell",false),
            ("Violet Bell and Silver Lens","Violet Bell",false),
        ] {
            let context=format!("Item owner: {declaration}. The Item is a launch claim. The Subject is the brushed finish on the Silver Lens mounting plate. We retract the Item owned by {reference} and deny the assertion about that Subject.");
            check_current_role_context(&context,expected);
        }
        check_current_role_context("Item owner: Violet Bell. The Item is a launch claim. The Subject is the brushed finish on the Silver Lens mounting plate. We retract the Item owned by Violet Bell and deny that Silver Lens was launched.",false);
    }

    #[test]
    fn v30_finite_foreign_claim_and_complete_owner_explanation_agree() {
        for (claim_owner,declared_owner,expected) in [
            ("Violet Bell","Violet Bell",true),
            ("Silver Lens","Silver Lens",false),
            ("Violet Bell","Silver Lens",false),
            ("Violet Bell","Cedar Basket",false),
            ("an unknown product","Violet Bell",false),
        ] {
            let context=format!("Object: claim that {claim_owner} was launched. We withdraw the assertion represented by this Object. The complete named owner of the Object is {declared_owner}.");
            check_current_role_context(&context,expected);
        }
        check_current_role_context("Object: claim that Violet Bell was launched. We withdraw the assertion represented by this Object. The complete named owner of the Subject is Violet Bell. Owner of the Object: Silver Lens.",false);
    }

    #[test]
    fn v30_adjacent_foreign_product_and_claim_references_stay_scoped() {
        for (owner,expected) in [("Violet Bell",true),("Silver Lens",false),("an unknown product",false)] {
            let context=format!("No launch took place for {owner}. We deny that product's launch claim. We also retract the Object, the silver print on the Silver Lens booklet spine.");
            check_current_role_context(&context,expected);
            let context=format!("The correction concerns the claim that {owner} was launched. I deny that claim. The claim's explicitly named product is {owner}.");
            check_current_role_context(&context,expected);
        }
        check_current_role_context("No launch took place for Violet Bell. We deny that product's launch claim and the launch of Silver Lens.",false);
        check_current_role_context("The correction concerns the claim that Violet Bell was launched. I deny that claim. The claim's explicitly named product is Silver Lens.",false);
        check_current_role_context("The correction concerns the claim that Violet Bell was launched and Silver Lens was launched. I deny that claim.",false);
    }

'''
anchor='    #[test]\n    fn v30_printed_title_literals_keep_outside_property_scope() {';assert s.count(anchor)==1;p.write_text(s.replace(anchor,tests+anchor,1));print('Five new public behavior groups added before semantic changes.')
