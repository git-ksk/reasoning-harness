from pathlib import Path
import hashlib,subprocess
p=Path('crates/reasoning-harness-core/src/evidence_relevance.rs');s=p.read_text();assert hashlib.sha256(p.read_bytes()).hexdigest()=='4daf64b918532a9004dea9807b8001ec743244b20b5cd1892c7f3a7ce316d1ef'
tests=r'''
    #[test]
    fn v30_printed_nominal_watermark_keeps_complete_physical_owner() {
        let property="The Subject is the pale green watermark printed along the Silver Lens booklet page edge. We deny the assertion about that Subject. Only the printed watermark's color is disputed.";
        check_current_role_context(property,true);
        check_current_role_context(&format!("{property} We deny that Silver Lens was launched."),false);
        check_current_role_context("The Subject is the pale green watermark printed along an unknown product's booklet page edge. We deny the assertion about that Subject.",false);
        check_current_role_context("The Subject is the pale green watermark printed along the Silver Lens booklet page edge and the launch claim. We deny the assertion about that Subject.",false);
    }

    #[test]
    fn v30_parenthetical_correction_coverage_requires_same_property_role() {
        let property="The Object is the silver finish of the Silver Lens metal clip. I retract the claim about this Object (the withdrawal covers the clip-finish description only).";
        check_current_role_context(property,true);
        check_current_role_context(&format!("{property} We deny that Silver Lens was launched."),false);
        check_current_role_context("The Object is the silver finish of the Silver Lens metal clip. I retract the claim about this Object (the withdrawal covers the launch description only).",false);
        check_current_role_context("The Object is the silver finish of the Silver Lens metal clip. I retract the claim about this Subject (the withdrawal covers the clip-finish description only).",false);
        check_current_role_context("The Object is unknown. I retract the claim about this Object (the withdrawal covers the clip-finish description only).",false);
    }

    #[test]
    fn v30_lettering_of_quoted_word_is_an_owned_printed_property() {
        for (open, close) in [("'", "'"), ("\"", "\""), ("‘", "’"), ("“", "”")] {
            let property=format!("The Item is the uppercase lettering of the word {open}WITHDRAWN{close} printed on the Silver Lens storage tag. We withdraw the assertion about that Item. This corrects the lettering, not a launch statement.");
            check_current_role_context(&property,true);
            check_current_role_context(&format!("{property} We withdraw all launch statements."),false);
            check_current_role_context(&format!("The Item is the truth of the word {open}LAUNCHED{close} printed on the Silver Lens storage tag. We withdraw the assertion about that Item."),false);
        }
        check_current_role_context("The Item is the uppercase lettering of the word 'WITHDRAWN' printed on an unknown tag. We withdraw the assertion about that Item.",false);
    }

    #[test]
    fn v30_generic_quoted_claim_requires_unique_same_role_owner() {
        for (open, close) in [("'", "'"), ("\"", "\""), ("‘", "’"), ("“", "”")] {
            for (owner,expected) in [("Violet Bell",true),("Silver Lens",false),("unknown",false)] {
                let context=format!("The Item is the quoted claim: {open}The product was launched.{close} Immediate reply: Item owner is {owner}. We withdraw this Item, the claim about {owner}'s launch.");
                check_current_role_context(&context,expected);
            }
        }
        check_current_role_context("The Item is the quoted claim: 'The product was launched.' Immediate reply: Subject owner is Violet Bell. We withdraw this Item, the claim about Violet Bell's launch.",false);
        check_current_role_context("The Item is the quoted claim: 'The product was launched.' Immediate reply: Item owner is Violet Bell. Another reply: Item owner is Silver Lens. We withdraw this Item, the claim about Violet Bell's launch.",false);
    }

    #[test]
    fn v30_plural_claim_wrapper_coordinates_only_complete_excluded_roles() {
        let property="The Object is the claim that Violet Bell was launched. The Subject is the embossed border of the Silver Lens badge. The Item is the printed title alignment of the Cedar Basket booklet. We deny this Object and retract the assertions about that Subject and this Item.";
        check_current_role_context(property,true);
        check_current_role_context(&format!("{property} Silver Lens was not launched."),false);
        check_current_role_context("The Subject is the embossed border of the Silver Lens badge. The Item is unknown. We retract the assertions about that Subject and this Item.",false);
        check_current_role_context("The Subject is the embossed border of the Silver Lens badge. The Item is the launch of Silver Lens. We retract the assertions about that Subject and this Item.",false);
        check_current_role_context("The Subject is the embossed border of the Silver Lens badge. The Item is the printed title alignment of the Cedar Basket booklet. We retract the assertions about that Subject or this Item.",false);
    }

    #[test]
    fn v30_optional_article_printed_heading_requires_outside_owner() {
        for article in ["", "the "] {
            let property=format!("Object: {article}printed heading 'Launch inventory' on the Silver Lens paper insert. We retract the assertion about this Object. The correction concerns what heading was printed on the insert, not a launch event or an inventory of real events.");
            check_current_role_context(&property,true);
            check_current_role_context(&format!("{property} We deny that Silver Lens was launched."),false);
            check_current_role_context(&format!("Object: {article}printed heading 'Launch inventory' on an unknown insert. We retract the assertion about this Object."),false);
        }
    }

'''
anchor='    #[test]\n    fn v30_printed_title_literals_keep_outside_property_scope() {';assert s.count(anchor)==1
p.write_text(s.replace(anchor,tests+anchor,1));subprocess.run(['cargo','fmt','--all'],check=True)
print('six public behavior regression groups added before semantic change')
