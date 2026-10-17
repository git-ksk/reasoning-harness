#!/usr/bin/env python3
"""Fresh v10 finite typed-reference and independently scoped correction controls."""
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parent
NAMES=['Amberwick Latch','Broomhaven Gauge','Cedarhollow Lens','Duneford Platen','Elmharbour Clip','Fallowmere Wheel','Goldwharf Panel','Heatherlake Loom','Irisquay Valve','Juniperbank Clip','Kestrelford Tray','Larchhaven Pedal','Mallowbridge Sensor','Nettleharbour Clip','Oatmeadow Hub','Pinewharf Seal','Quartzford Collar','Rosehaven Spindle','Saffronmere Lever','Thistlebay Rotor','Uplandquay Frame','Violetford Cradle','Willowbank Coupler','Xanthicmead Dock','Yarrowhaven Coil','Zirconwharf Ring','Acornfield Clip','Bramblequay Board','Cowslipford Scanner','Dogrosehaven Prism']
FOREIGN='Sablemere Gimbal'
BUDGET={'max_model_attempts':2,'max_tokens':192,'max_elapsed_ms':90000}
def s(k,t):return {'kind':k,'text':t}
def ex(t):return [s('excerpt',t)]
cases=[]
for n,t in enumerate(NAMES,1):
    if n==1:
        p=[s('heading',f'Object: the physical launch-panel width statement for {t}'),s('excerpt','Editorial correction: that object is denied. The width specification is the object of this notice. Publisher note: this notice was typeset in two columns.')]
        q=[s('heading',f'Object: the product launch statement for {t}'),s('excerpt','Editorial correction: that object is denied. The product launch statement is the object of this notice. Publisher note: this notice was typeset in two columns.')]
        axis='heading_object_denial_and_neutral_publication_note';why='The same typed Object is explicitly width or target launch; an independent neutral publication note cannot change its scope.'
    elif n==2:
        p=ex(f'Specification erratum. Subject: "The launch collar on {t} is copper." This subject is withdrawn. The withdrawal concerns the quoted collar-material specification.')
        q=ex(f'Specification erratum. Subject: "{t} was launched." This subject is withdrawn. The withdrawal concerns the quoted product launch statement.')
        axis='quoted_subject_with_same_type_withdrawal';why='A complete adjacent quoted Subject supplies material or target launch; this Subject refers to the same type.'
    elif n==3:
        p=ex(f'Item: a physical dimension assertion for {t}. We deny the item, namely the height statement for the release button on {t}. This is a button-height correction.')
        q=ex(f'Item: a launch assertion for {t}. We deny the item, namely the product launch statement for {t}. This is a launch-event correction.')
        axis='item_with_complete_owner_and_dimension_apposition';why='The complete same-sentence apposition defines the Item without a numbered or distant reference.'
    elif n==4:
        p=ex(f'Publisher routing card. Object: the launch claim belonging to {FOREIGN}. The object is retracted. Routing note: the full product owner of the retracted object is {FOREIGN}.')
        q=ex(f'Publisher routing card. Object: a launch claim belonging to either {t} or {FOREIGN}. The object is retracted. Routing note: no choice between the two product owners was recorded.')
        axis='same_object_article_reference_and_selected_owner';why='The positive Object has one fully named foreign owner; the negative explicitly leaves the launch owner unselected.'
    elif n==5:
        p=[s('source_title',f'Product identifier: {t}'),s('excerpt',f'Subject: the physical release-socket width specification for {t}. We deny this subject. This is a socket-width property correction; {t} is the complete product name.')]
        q=[s('source_title',f'Product identifier: {t}'),s('excerpt',f'The launch of {t} did not happen. This current notice denies the launch event; {t} is the complete product name.')]
        axis='complete_clip_name_property_vs_explicit_non_event';why='Clip is part of an explicitly complete canonical name, not a context-gap marker. The negative directly denies the target event.'
    elif n==6:
        p=ex(f'Item: a launch statement. Author reply: "I deny the item." Owner supplement immediately after the reply: the owner of this item is {FOREIGN}. The item concerns {FOREIGN}\'s launch.')
        q=ex('Item: a launch statement. Author reply: "I deny the item." Owner supplement immediately after the reply: absent. The owner of this launch item was not supplied.')
        axis='item_reply_with_following_complete_owner_supplement';why='An adjacent complete foreign-owner supplement makes the positive local Item unambiguous; the negative explicitly lacks its launch owner.'
    elif n==7:
        p=ex(f'Object: the printed launch-label font statement for {t}. I withdraw this object. Subject: the physical release-handle width statement for {t}. I deny the subject. These are separate printed-font and physical-width corrections.')
        q=ex(f'Object: the printed launch-label font statement for {t}. I withdraw this object. Subject: an additional launch statement whose product owner is not recorded. I deny the subject. This is a separate launch correction with an unidentified owner.')
        axis='separately_typed_property_cues_vs_additional_unknown_launch';why='Two separately named property types are safe in the positive; the property Object cannot waive the independent ownerless launch Subject.'
    elif n==8:
        p=ex(f'Author correction: I retract that claim—the complete surface-material statement for the physical launch cover on {t}. The apposition identifies only this physical material claim.')
        q=ex(f'Author correction: I retract that claim—either the surface-material statement for the launch cover on {t} or the product launch statement for {t}. The apposition does not choose which claim is retracted.')
        axis='complete_property_apposition_vs_unselected_event_apposition';why='The positive complete same-sentence apposition is supported; the negative expressly leaves property versus launch scope unselected.'
    elif n==9:
        p=[s('heading',f'Subject: the document title-font specification for {t}'),s('excerpt','The subject is denied by its author. Footer: the erratum has a wide left margin. The denial concerns the document title font.')]
        q=[s('heading',f'Subject: the product launch statement for {t}'),s('excerpt','The subject is denied by its author. Footer: the erratum has a wide left margin. The denial concerns the product launch.')]
        axis='document_subject_denial_with_layout_footer';why='The footer is neutral in both arms; the same explicit Subject type determines property versus event denial.'
    elif n==10:
        p=[s('heading',f'Complete product name: {t}'),s('excerpt',f'Object: the release-button surface-colour specification for {t}. That object is withdrawn. The object is only the button-colour specification.')]
        q=[s('heading',f'Complete product name: {t}'),s('excerpt','Object: a launch statement. That object is withdrawn. The paragraph naming the product is missing from this excerpt; the owner of the withdrawn launch object is unavailable.')]
        axis='canonical_clip_complete_context_vs_actual_owner_gap';why='A complete canonical Clip name plus a physical property is positive; the negative has actual missing owner context and a launch withdrawal, with explicit raw context_gap.'
    elif n==11:
        p=[s('heading',f'Owner supplement for the following reply: {FOREIGN}'),s('excerpt','Object: the launch statement of the product named in that owner supplement. Author reply: "I retract the object." The reply belongs to the fully named owner above.')]
        q=[s('heading',f'Owner supplement for the following reply: {t}'),s('excerpt','Object: the launch statement of the product named in that owner supplement. Author reply: "I retract the object." The reply belongs to the fully named owner above.')]
        axis='adjacent_heading_owner_for_typed_object_reply';why='One adjacent heading completely names the owner of a singular launch Object; no alias or reference graph is requested.'
    elif n==12:
        p=ex(f'Object: the launch statement for {FOREIGN}. We retract this object. Subject: the physical launch-wheel diameter statement for {t}. We deny this subject. Each correction has its own complete product owner.')
        q=ex(f'Object: the launch statement for {FOREIGN}. We retract this object. Subject: the launch statement for {t}. We deny this subject. Each correction has its own complete product owner.')
        axis='foreign_object_and_separately_owned_subject';why='A fully named foreign launch and target physical property are safe together; a foreign Object cannot mask the separate target-launch Subject.'
    elif n==13:
        p=ex(f'Item: the launch-indicator colour claim for {t}. The item is denied. Object: the launch claim for {FOREIGN}. The object is withdrawn. The first correction is indicator colour and the second belongs to {FOREIGN}.')
        q=ex(f'Item: the launch-indicator colour claim for {t}. The item is denied. Object: the launch claim for {t}. The object is withdrawn. The first correction is indicator colour and the second is the target launch.')
        axis='property_item_before_foreign_or_target_launch_object';why='Separate explicit typed corrections retain target launch only if the launch Object is fully foreign, never merely because the Item is a property.'
    elif n==14:
        p=[s('source_title',f'Complete canonical product name: {t}'),s('heading','Erratum about physical properties'),s('excerpt',f'Item: the material specification for the release latch on {t}. I retract this item. The item is latch material, and {t} is the full product name.')]
        q=[s('source_title',f'Complete canonical product name: {t}'),s('heading','Erratum about a launch statement'),s('excerpt',f'Item: the launch statement for {t}. I retract this item. The item is the product launch, and {t} is the full product name.')]
        axis='canonical_clip_title_and_typed_item_context';why='The exact complete canonical name must not itself imply clipping; the typed Item is explicitly material or target launch.'
    elif n==15:
        p=ex(f'Subject: the printed launch-key font-size assertion for {t}. The subject is withdrawn. Author clarification: this subject describes the font size of printed key text, not a product event.')
        q=ex(f'Subject: the launch assertion for {t}. The subject is withdrawn. Author clarification: this subject describes the product event, not printed key text.')
        axis='same_subject_with_explicit_printed_type_clarification';why='The full Subject and adjacent type clarification agree; physical component modifiers are not event predicates.'
    elif n==16:
        p=ex(f'Quotation supplied with the correction: "The release lever on {t} is black." Item: this complete lever-colour quotation. Author response: "We deny the item." The object of denial is lever colour.')
        q=ex(f'Quotation supplied with the correction: "{t} was launched." Item: this complete launch quotation. Author response: "We deny the item." The object of denial is the product launch.')
        axis='quotation_then_typed_item_then_same_type_reply';why='One adjacent complete quote plus matching Item type binds the reply to colour or target launch.'
    elif n==17:
        p=[s('heading',f'Subject: the physical launch-cover thickness claim for {t}'),s('excerpt',f'Quoted claim: "The launch cover on {t} is thin." Author reply: "I withdraw the subject." The subject is the cover-thickness property.')]
        q=[s('heading',f'Subject: the launch claim for {t}'),s('excerpt',f'Quoted claim: "The launch cover on {t} is thin." Author reply: "I withdraw the subject." The subject is the target launch even though the quotation concerns cover thickness.')]
        axis='subject_heading_and_quote_type_agreement_or_conflict';why='The positive heading and quote consistently identify a physical Subject; competing target-event Subject in the negative requires withholding.'
    elif n==18:
        p=ex(f'Object: the launch claim for {FOREIGN}. Author reply: "I withdraw this object." Owner supplement: {FOREIGN}, the sole product owner of this object.')
        q=[s('source_title',f'Earlier document-property note: launch-label font for {t}'),s('excerpt','Publication record: pages were stapled at the top. Delivery record: the envelope was sealed.'),s('excerpt','Author reply: "I withdraw that claim." The local sentence identifying the claim is missing. The reply may concern a launch claim; its owner and type are not supplied.')]
        axis='locally_bound_object_vs_distant_property_and_missing_reply_context';why='A local explicit foreign Object is positive. A remote property title cannot resolve a separate reply whose local launch/type/owner context is explicitly missing; raw context_gap is supplied.'
    elif n==19:
        p=ex(f'Item: a physical specification claim for {t}. We retract this item, namely the width and height specification for the release pedal on {t}. Both dimensions concern that physical pedal.')
        q=ex(f'Item: a product event claim for {t}. We retract this item, namely the launch statement for {t}. The claim concerns that product launch event.')
        axis='fully_explained_compound_property_item_apposition';why='A complete physical width-and-height apposition is one non-launch Item; the negative complete apposition is target launch.'
    elif n==20:
        p=ex(f'Object: a launch statement. Author response: "I deny that object." Adjacent owner supplement: {FOREIGN} owns the denied launch statement. No second owner is assigned.')
        q=ex(f'Object: a launch statement. Author response: "I deny that object." Adjacent owner supplement: the statement may belong to {t} or {FOREIGN}; the owner is not selected.')
        axis='post_reply_owner_supplement_selected_or_mixed';why='The positive supplement names one full foreign owner; the negative explicitly leaves target-versus-foreign ownership unresolved.'
    elif n==21:
        p=ex(f'Object: the release-panel material claim for {t}. This object is retracted. Subject: the document page-count statement for {t}. That subject is denied. Both corrections are independently typed non-launch properties.')
        q=ex(f'Object: the release-panel material claim for {t}. This object is retracted. Subject: a launch statement with an unidentified product owner. That subject is denied. The second correction is independently a launch claim with unknown ownership.')
        axis='two_local_typed_properties_vs_ownerless_second_subject';why='Each type has its own adjacent reference; a clear property Object cannot excuse an independent ownerless launch Subject.'
    elif n==22:
        p=ex(f'Quoted statement: "The launch-selector text on {t} uses a large font." Author reply: "We retract that claim." The adjacent quotation is the printed-font statement being retracted.')
        q=ex(f'Quoted statement: "{t} was launched." Author reply: "We retract that claim." The adjacent quotation is the product launch statement being retracted.')
        axis='generic_claim_with_adjacent_complete_font_or_event_quote';why='The generic claim reference has one adjacent complete quotation and explicit scope restatement, rather than borrowing a distant property type.'
    elif n==23:
        p=[s('source_title','Publisher correction ledger'),s('heading',f'Item: the launch statement for {FOREIGN}'),s('excerpt',f'The item is retracted. Confirmation: {FOREIGN} is the complete owner of this item. Layout note: the ledger uses a narrow right margin.')]
        q=[s('source_title','Publisher correction ledger'),s('heading',f'Item: a launch statement involving either {t} or {FOREIGN}'),s('excerpt','The item is retracted. Confirmation: the owner of this item was not chosen. Layout note: the ledger uses a narrow right margin.')]
        axis='ledger_item_heading_with_foreign_or_unselected_confirmation';why='Neutral ledger/layout context does not alter one explicit foreign Item or resolve the unselected owner of the negative Item.'
    elif n==24:
        p=ex(f'Subject: the launch-ring surface-colour assertion for {t}. We deny the subject. Object: the printed release-key font assertion for {t}. We withdraw the object. Both are specifications, separately named and typed.')
        q=ex(f'Subject: the launch assertion for {t}. We deny the subject. Object: the printed release-key font assertion for {t}. We withdraw the object. The first correction is product launch and the second is typography.')
        axis='target_event_subject_before_safe_property_object';why='A later explicit property Object cannot erase an earlier fully named target-launch Subject denial; positive corrections are both property-only.'
    elif n==25:
        p=[s('source_title',f'Owner supplement: {FOREIGN}'),s('heading','Item: the named owner\'s launch statement'),s('excerpt',f'Author reply: "I retract that item." The title names {FOREIGN}, the full owner of the retracted launch item.')]
        q=[s('source_title',f'Owner supplement: {t}'),s('heading','Item: the named owner\'s launch statement'),s('excerpt',f'Author reply: "I retract that item." The title names {t}, the full owner of the retracted launch item.')]
        axis='complete_title_owner_with_same_item_reply';why='The adjacent title/heading/reply identify one complete owner of the typed launch Item; no alias or arbitrary graph is needed.'
    elif n==26:
        p=ex(f'The claim about {t} is denied—specifically, the physical thickness statement for the launch panel on {t}. This complete apposition limits the denial to panel thickness.')
        q=ex(f'The claim about {t} is denied—specifically, the product launch statement for {t}. This complete apposition identifies the launch event.')
        axis='generic_target_claim_with_complete_property_or_event_apposition';why='The generic claim is fully explained by an adjacent same-sentence apposition; no unresolved generic reference is forced positive.'
    elif n==27:
        p=[s('source_title',f'Full product name: {t}'),s('excerpt',f'Object: the physical launch-button height claim for {t}. We deny the object. The height of this physical button is the only correction here.')]
        q=[s('source_title',f'Full product name: {t}'),s('excerpt','Object: a launch statement. We deny the object. This excerpt omits the owner sentence; the product owner of the denied launch object is unknown.')]
        axis='full_clip_name_vs_explicitly_missing_launch_owner_sentence';why='The positive Clip name is complete and the property scope explicit; the negative has a real missing owner sentence and denied launch Object plus raw context_gap.'
    elif n==28:
        p=ex(f'Item: the launch statement for {FOREIGN}. Source quotation: "{FOREIGN} was launched." I withdraw this item. The item belongs solely to {FOREIGN}.')
        q=ex(f'Item: the launch statement for {t}. Source quotation: "{FOREIGN} was launched." I withdraw this item. The item declaration names the target while the quotation names another product; their scopes conflict.')
        axis='typed_item_and_quoted_owner_agreement_or_conflict';why='One fully foreign owner is safe; a target-launch Item competing with a foreign quote must not inherit foreign immunity.'
    elif n==29:
        p=ex(f'Object: the document page-count claim for {t}. We retract that object. Subject: the release-cover colour statement for {t}. We deny this subject. Printer note: both corrections are printed on cream paper.')
        q=ex(f'Object: the document page-count claim for {t}. We retract that object. Subject: the launch statement for {t}. We deny this subject. Printer note: both corrections are printed on cream paper.')
        axis='document_property_object_and_second_subject_with_neutral_printer_note';why='The independently typed second Subject can be physical colour or target launch; neither the document Object nor neutral printer note can mask it.'
    elif n==30:
        p=ex(f'Subject: the printed launch-label colour statement for {t}. That subject is denied. Item: the launch statement for {FOREIGN}. This item is withdrawn. The first correction is printed colour and the second launch owner is {FOREIGN}.')
        q=ex(f'Subject: the printed launch-label colour statement for {t}. That subject is denied. Item: the launch statement for {t}. This item is withdrawn. The first correction is printed colour and the second launch owner is {t}.')
        axis='safe_subject_before_fully_owned_foreign_or_target_item';why='The first property Subject is safe in both arms; the separately typed launch Item must be evaluated by its complete owner.'
    for polarity,context in [('positive',p),('negative',q)]:
        headers=[v for v in context if v['kind']!='excerpt'];notes=[v for v in context if v['kind']=='excerpt'];fact=s('excerpt',f'{t} was launched.')
        signals=headers+(notes+[fact] if n in (2,6,16,19,22,26) else [fact]+notes)
        cid=f'li10-{len(cases)+1:03d}';risk='context_gap' if polarity=='negative' and n in (10,18,27) else 'none'
        inp={'id':cid,'policy':{'policy_id':'independent-v10-launch-policy','target_id':f'v10-fictional-target-{n:02d}','target_question':f'Was {t} launched?','entity':{'canonical_id':f'v10-fictional-target-{n:02d}','canonical_name':t,'aliases':[]},'relation':'change_or_launch','identity_requirement':'require_harness_anchor','assessment_budget':BUDGET.copy()},'candidate':{'evidence_id':cid+'-evidence','source_id':cid+'-synthetic-record','signals':signals},'proposal':{'target_binding':'exact','relation_binding':'exact'},'raw':{'identity_scope':'exact_target','relation_scope':'requested_relation','scope_risk':risk}}
        cases.append({'id':cid,'pair_id':f'v10-pair-{n:02d}','polarity':polarity,'axis':axis,'expected':{'v17_requested_relation':polarity=='positive','v30_relevant':polarity=='positive'},'rationale':why,'input':inp})
for name,value in [('cases.json',{'schema':'engine073-launch-independent-v10','author':'independent evaluator /root/engine07_realworld_independent','cases':cases}),('inputs.json',[c['input'] for c in cases])]:
    with (ROOT/name).open('x') as f:json.dump(value,f,ensure_ascii=False,indent=2);f.write('\n')
print('new_v10_cases',len(cases))
