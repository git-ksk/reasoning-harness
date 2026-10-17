#!/usr/bin/env python3
"""New v12 finite correction-clause, event/property and owner boundaries."""
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parent
NAMES=['Adlerhaven Rail','Bramleyquay Spool','Cressmeadow Rotor','Dapplebank Gauge','Ermineford Hub','Firsedge Tray','Galewharf Clip','Hedgerowbay Prism','Indigomere Panel','Jadequay Lever','Kilmere Collar','Lilacbank Wheel','Mossquay Lens','Nacreharbour Clip','Ospreyford Board','Pearlstrand Gimbal','Quincebay Sensor','Rookmere Platen','Sagequay Latch','Tamariskbank Dock','Urchinford Ring','Valerianquay Coil','Wheatmere Clip','Xericbank Coupler','Yellowhornbay Valve','Zosterford Seal','Anisequay Pedal','Boragebank Cradle','Carminehaven Scanner','Dovemere Module']
FOREIGN='Fernwharf Bridge'
BUDGET={'max_model_attempts':2,'max_tokens':192,'max_elapsed_ms':90000}
def s(k,t):return {'kind':k,'text':t}
def ex(t):return [s('excerpt',t)]
cases=[]
for n,t in enumerate(NAMES,1):
    if n==1:
        common=f'Object: the physical release-lever width statement for {t}. '
        p=ex(common+f'We retract this object because we deny the launch statement for {FOREIGN}. The reason clause names the other product completely.')
        q=ex(common+f'We retract this object because we deny the launch statement for {t}. The reason clause names the target event completely.')
        axis='property_object_because_separate_fully_owned_event_denial';why='A property retraction may be safe, but its exclusion cannot consume a separate reason-clause target-launch denial. The foreign reason is explicitly owned.'
    elif n==2:
        common=f'Subject: the document heading-font specification for {t}. '
        p=ex(common+f'I withdraw the subject although we also deny the printed heading-colour specification for {t}. Both corrections are printed-document properties.')
        q=ex(common+f'I withdraw the subject although we also deny the launch statement for {t}. The contrast clause separately denies the product event.')
        axis='document_subject_although_property_or_event_clause';why='The Subject is typography in both cases. A separate although clause is independently property-only or explicitly target launch.'
    elif n==3:
        common=f'Item: "The release knob on {t} is red." '
        p=ex(common+f'This item is retracted while we deny the physical knob-height statement for {t}. The two statements concern colour and height.')
        q=ex(common+f'This item is retracted while we deny the launch statement for {t}. The while clause concerns the target event.')
        axis='quoted_item_while_current_property_or_launch_denial';why='While separates current correction clauses; no date/order inference is required, and the target-event denial is not part of the colour item.'
    elif n==4:
        p=ex(f'Object: the launch statement for {FOREIGN}. We retract this object because we deny that launch claim. The complete foreign event declaration is the adjacent launch referent.')
        q=ex(f'Object: the physical release-button width statement for {FOREIGN}. We retract this object because we deny that launch claim. The product owner of the separate launch claim is not identified.')
        axis='foreign_event_object_vs_property_object_for_launch_reference';why='An explicit foreign launch frame can bind that launch claim; a foreign property frame cannot supply the owner/type of a separate unidentified launch reference.'
    elif n==5:
        common=[s('heading',f'Object: the physical release-collar material claim for {t}')]
        p=common+ex(f'Because we deny the launch statement for {FOREIGN}, we withdraw that object. This sentence has a fully named foreign reason and a material correction.')
        q=common+ex(f'Because we deny the launch statement for {t}, we withdraw that object. This sentence has a fully named target-launch denial before the material correction.')
        axis='leading_because_clause_before_same_object_reference';why='The preceding reason clause must be assessed before or independently of the harmless typed Object correction.'
    elif n==6:
        common=f'Subject: the printed launch-key font-size claim for {t}. '
        p=ex(common+f'I retract this subject (and I also deny the printed launch-key colour claim for {t}). The parenthesized correction is another text property.')
        q=ex(common+f'I retract this subject (and I also deny the launch statement for {t}). The parenthesized correction is a separate product-event denial.')
        axis='same_subject_with_parenthesized_separate_denial';why='Parentheses contain an explicit additional clause, not a new AST requirement. Its full property/event object remains independent of the Subject.'
    elif n==7:
        common=[s('source_title',f'Complete canonical product name: {t}')]
        p=common+ex(f'Object: the physical launch-cover width claim for {t}. We retract the object; we also deny the cover-colour statement for {t}. The complete name includes Clip and both corrections concern physical properties.')
        q=common+ex(f'Object: the physical launch-cover width claim for {t}. We retract the object; we also retract a launch claim whose owner sentence is missing from this excerpt. The separate launch owner is unknown.')
        axis='complete_clip_name_and_semicolon_actual_unknown_launch';why='The full Clip name and two properties are safe. A separate withdrawn ownerless launch with actual missing context is negative, with explicit raw context_gap.'
    elif n==8:
        common=[s('source_title',f'Subject: the release-label font-style statement for {t}')]
        p=common+ex(f'Although we deny the launch claim for {FOREIGN}, we withdraw the subject. The title describes a font property and the opening denial names another owner.')
        q=common+ex(f'Although we deny the launch claim for {t}, we withdraw the subject. The opening denial names the target launch despite the font-property title.')
        axis='leading_although_event_denial_and_property_title';why='A property title cannot exempt a separate leading target launch denial; a fully named foreign event does not negate target launch.'
    elif n==9:
        p=ex(f'Object: an unowned launch statement. Owner of the object: {FOREIGN}. We retract the object because we deny the printed release-label colour claim for {t}.')
        q=ex(f'Object: the launch statement for {t}. Owner supplement for the object: {FOREIGN}. We retract the object because we deny the printed release-label colour claim for {t}.')
        axis='reason_property_does_not_override_owned_launch_frame';why='One full foreign owner may fill an unowned Object; a property reason and foreign supplement cannot overwrite an existing target launch owner.'
    elif n==10:
        common=f'Item: the physical release-sleeve thickness statement for {t}. '
        p=ex(common+f'We withdraw the item although we deny the physical sleeve-colour specification for {t}. Each correction is limited to a complete physical property.')
        q=ex(common+'We withdraw this entire source although the item itself is a physical property. All assertions in the source, including its target launch assertion, are withdrawn.')
        axis='although_property_item_vs_entire_source_withdrawal';why='The negative explicitly withdraws the whole source; the although property clause cannot narrow that scope.'
    elif n==11:
        common=f'Item: the printed launch-selector font statement for {t}. '
        p=ex(common+f'Quoted author reply: "I withdraw that item because we deny the printed selector-colour claim for {t}." The reply corrects two printed properties.')
        q=ex(common+f'Quoted author reply: "I withdraw that item because we deny the launch claim for {t}." The reply contains a separate explicit target-event reason clause.')
        axis='quoted_author_reply_correction_and_reason_clause';why='A quoted current reply still contains separate complete correction/reason clauses; typed Item exclusion is not source-wide.'
    elif n==12:
        common=f'Subject: an unowned launch claim. The sole product owner of the subject is {FOREIGN}. '
        p=ex(common+f'We retract this subject while we deny the physical release-handle material claim for {t}. The while clause concerns handle material.')
        q=ex(common+f'We retract this subject while {t} does not exist. The while clause explicitly denies the target itself.')
        axis='foreign_supplemented_subject_and_target_itself_denial';why='A safe foreign launch Subject cannot erase a separate target-existence denial in a current while clause.'
    elif n==13:
        common=f'I deny that claim, namely the complete document page-count specification for {t}, '
        p=ex(common+f'whereas we retract the launch statement for {FOREIGN}. The apposition is document length and the other event owner is explicit.')
        q=ex(common+f'whereas we retract the launch statement for {t}. The apposition is document length but the other clause withdraws the target event.')
        axis='complete_property_apposition_before_whereas_launch_retraction';why='The complete local property apposition scopes only its own denial clause, not the separately owned whereas event retraction.'
    elif n==14:
        common=[s('heading',f'Full canonical product name: {t}')]
        p=common+ex(f'Object: the physical release-pedal colour statement for {t}. I retract this object because we deny the pedal-height specification for {t}. The full name includes Clip; both objects are physical properties.')
        q=common+ex(f'Object: the physical release-pedal colour statement for {t}. I retract this object because {t} does not exist. This reason denies the complete target itself.')
        axis='canonical_clip_property_reason_vs_target_nonexistence';why='Clip is explicitly part of the complete canonical name, while direct target nonexistence in the separate reason clause blocks launch.'
    elif n==15:
        p=ex(f'Item: an unowned launch statement. Owner supplement:\n{FOREIGN}\nWe retract that launch claim. The complete name supplies the sole owner of this launch Item.')
        q=ex(f'Item: the physical release-button height statement for {FOREIGN}. We retract that launch claim. The product owner of this separate launch claim is not supplied; the Item above describes only a physical dimension.')
        axis='plain_full_owner_launch_item_vs_foreign_property_launch_reference';why='A designated single plain-name supplement fills a launch Item; the owner of a foreign property is not automatically the owner of an unidentified launch reference.'
    elif n==16:
        p=ex(f'Object: the printed release-key colour specification for {t}. We deny this object. Subject: an unowned launch claim. Owner of the subject: {FOREIGN}. We retract the subject as we correct the printed font specification for {t}.')
        q=ex(f'Object: the printed release-key colour specification for {t}. We deny this object. Subject: the launch claim for {t}. Owner supplement for the subject: {FOREIGN}. We retract the subject as we correct the printed font specification for {t}.')
        axis='safe_property_clauses_and_supplemented_or_owned_subject';why='Explicit safe property clauses do not overwrite the separate launch Subject; supplements only fill an unowned frame.'
    elif n==17:
        common=f'Object: the printed release-label font-style claim for {t}. '
        p=ex(common+f'We withdraw the object since we deny the printed label-colour statement for {t}. Both current corrections concern printed text.')
        q=ex(common+f'We withdraw the object since the launch of {t} never happened. The since clause explicitly denies the target event.')
        axis='since_clause_complete_property_vs_explicit_launch_non_event';why='Since is a clause boundary only; no temporal inference is needed. An explicit target non-event cannot be swallowed by property exclusion.'
    elif n==18:
        common=f'Item: the physical release-wheel diameter statement for {t}. '
        p=ex(common+f'We deny the launch claim for {FOREIGN} before we withdraw the item. The current denial names another full product owner.')
        q=ex(common+f'We deny the launch claim for {t} before we withdraw the item. The current denial names the target launch before the property correction.')
        axis='before_clause_property_correction_and_current_launch_denial';why='The current first-person denial has a full owner regardless of the before clause; the test does not infer chronology or future event truth.'
    elif n==19:
        common=f'Subject: the document heading-colour statement for {t}. '
        p=ex(common+f'We retract the subject, while we also deny the heading-font claim for {t}. Each clause has a complete printed-property object.')
        q=ex(common+'We retract the subject, while we also withdraw this source as a whole. The second clause withdraws every assertion, including the launch fact.')
        axis='comma_while_second_property_or_whole_source_scope';why='The second clause is independently a property correction or whole-source withdrawal, not part of the safe Subject.'
    elif n==20:
        p=ex(f'Object: an unowned launch statement. The owner of the object is {FOREIGN}. We withdraw the object although we deny the physical release-ring colour claim for {t}.')
        q=ex(f'Object: an unowned launch statement. The owner of the object is either {FOREIGN} or {t}, with no selection. We withdraw the object although we deny the physical release-ring colour claim for {t}.')
        axis='single_foreign_owner_vs_mixed_owner_despite_safe_contrast_clause';why='One full foreign owner is safe; an explicitly unselected target/foreign owner is unresolved despite the additional property clause.'
    elif n==21:
        p=ex(f'Subject: "{FOREIGN} was launched." We deny the subject. That launch claim is withdrawn by us. The complete foreign-event quotation is the launch referent.')
        q=ex(f'Subject: "The release socket on {FOREIGN} is brass." We deny the subject. That launch claim is withdrawn by us. Its product-owner sentence is missing from this excerpt; the quoted Subject supplies only socket material.')
        axis='quoted_foreign_event_subject_vs_property_quote_and_actual_gap';why='A foreign-event quotation binds launch reference; a property quotation cannot fill an omitted launch owner. The negative has actual context loss and raw context_gap.'
    elif n==22:
        common=f'Object: a physical specification for {t}. We deny this object, namely the complete release-lever width statement for {t}, '
        p=ex(common+f'but we also deny the lever-height specification for {t}. Both clauses concern physical dimensions.')
        q=ex(common+f'but we also deny the launch statement for {t}. The but clause independently denies the product event.')
        axis='complete_object_apposition_but_independent_denial';why='A complete same-role apposition scopes one physical denial clause; but does not transfer that exemption to the target-launch clause.'
    elif n==23:
        common=[s('source_title',f'Full target product name: {t}')]
        p=common+ex(f'Object: the physical release-cover colour claim for {t}. We retract the object. Item: the launch statement for {FOREIGN}. We withdraw this item because we deny the printed cover-font claim for {t}. The complete target name includes Clip.')
        q=common+ex(f'Object: the physical release-cover colour claim for {t}. We retract the object. Item: the launch statement for {FOREIGN}. We withdraw this item because {t} does not exist. The complete target name includes Clip.')
        axis='canonical_clip_safe_roles_and_separate_target_existence_reason';why='A complete Clip name and two safe typed roles are positive; target nonexistence in a separate reason remains blocking.'
    elif n==24:
        p=ex(f'Subject: the launch statement for {FOREIGN}. Object: the physical release-knob material claim for {t}. We retract the subject, namely the launch claim for {FOREIGN}, as we deny the object. The same-role reference and complete foreign-event apposition identify the launch owner.')
        q=ex(f'Subject: the physical release-key colour claim for {FOREIGN}. Object: the physical release-knob material claim for {t}. We retract that launch claim as we deny the object. The separate launch claim has no assigned owner; both typed declarations are properties.')
        axis='two_typed_roles_do_not_loan_property_type_to_launch_reference';why='A same-role Subject reference with a complete foreign-event apposition is unambiguous; two safe properties cannot manufacture the owner of the unidentified launch reference in the negative.'
    elif n==25:
        common=f'Item: the printed launch-label colour statement for {t}. '
        p=ex(common+f'We withdraw the item when we deny the printed label-font specification for {t}. The current correction clauses both concern printed properties.')
        q=ex(common+f'We withdraw the item when we deny the launch statement for {t}. The current when clause explicitly denies the target event.')
        axis='when_clause_current_property_or_target_event_denial';why='When separates explicit current first-person clauses; no general chronological reasoning or new launch predicate is required.'
    elif n==26:
        common=f'Subject: the document page-count claim for {t}. '
        p=ex(common+f'Author reply: "I retract the subject, namely the complete document page-count statement for {t}." The reply limits withdrawal to document length.')
        q=ex(common+'Author reply: "I retract this document in its entirety." Every assertion in this source document, including its launch statement, is withdrawn.')
        axis='quoted_complete_document_property_reply_vs_document_wide_retraction';why='The positive complete property explanation is local; the negative explicitly withdraws the whole source document rather than just its page count.'
    elif n==27:
        common=f'Object: "The release handle on {t} is black." '
        p=ex(common+f'We deny the launch statement for {FOREIGN} while we retract the object. The launch denial is fully foreign and the object is handle colour.')
        q=ex(common+f'We deny the launch statement for {FOREIGN} while we retract the object. We also retract all claims in this source, including the launch claim for {t}.')
        axis='foreign_event_clause_and_quoted_property_plus_source_wide_addition';why='Foreign event denial and target colour correction are safe; a subsequent explicit whole-source addition is not waived by them.'
    elif n==28:
        p=ex(f'Object: an unowned launch statement. The owner of this object is {FOREIGN}. We withdraw this object because the page layout was revised. The reason is neutral document layout.')
        q=ex('Object: an unowned launch statement. We withdraw this object because its owner context was omitted. The paragraph supplying the product owner is missing from the excerpt, and no owner supplement is supplied.')
        axis='single_full_owner_vs_actual_missing_owner_with_neutral_reason';why='A complete foreign owner resolves the positive launch frame; the negative independently withdraws an ownerless launch with actual missing context and raw context_gap.'
    elif n==29:
        common=f'Subject: the physical release-panel thickness claim for {t}. '
        p=ex(common+f'We deny this subject. We retract the launch statement for {FOREIGN}. Each sentence states one complete correction scope.')
        q=ex(common+f'We deny this subject. We retract the launch statement for {t}. Each sentence states one complete correction scope.')
        axis='separate_sentence_control_property_then_fully_owned_event';why='Separate sentences are a control for clause-boundary behavior; the independently owned event is foreign or target, without chronology or implicit references.'
    elif n==30:
        common=f'I retract that claim, namely the complete printed release-button font-size statement for {t}. '
        p=ex(common+f'Item: an unowned launch statement. Owner of the item: {FOREIGN}. We deny the item although we withdraw the physical button-height claim for {t}.')
        q=ex(common+f'Item: the launch statement for {t}. Owner supplement for the item: {FOREIGN}. We deny the item although we withdraw the physical button-height claim for {t}.')
        axis='complete_property_apposition_and_owner_frame_not_rewritten_by_safe_clause';why='The positive has a complete property apposition and one foreign-filled launch frame. Safe surrounding properties and a foreign supplement cannot overwrite a target-owned Item.'
    for polarity,context in [('positive',p),('negative',q)]:
        headers=[v for v in context if v['kind']!='excerpt'];notes=[v for v in context if v['kind']=='excerpt'];fact=s('excerpt',f'{t} was launched.')
        signals=headers+(notes+[fact] if n in (3,10,13,18,22,26,30) else [fact]+notes)
        cid=f'li12-{len(cases)+1:03d}';risk='context_gap' if polarity=='negative' and n in (7,21,28) else 'none'
        inp={'id':cid,'policy':{'policy_id':'independent-v12-launch-policy','target_id':f'v12-fictional-target-{n:02d}','target_question':f'Was {t} launched?','entity':{'canonical_id':f'v12-fictional-target-{n:02d}','canonical_name':t,'aliases':[]},'relation':'change_or_launch','identity_requirement':'require_harness_anchor','assessment_budget':BUDGET.copy()},'candidate':{'evidence_id':cid+'-evidence','source_id':cid+'-synthetic-record','signals':signals},'proposal':{'target_binding':'exact','relation_binding':'exact'},'raw':{'identity_scope':'exact_target','relation_scope':'requested_relation','scope_risk':risk}}
        cases.append({'id':cid,'pair_id':f'v12-pair-{n:02d}','polarity':polarity,'axis':axis,'expected':{'v17_requested_relation':polarity=='positive','v30_relevant':polarity=='positive'},'rationale':why,'input':inp})
for name,value in [('cases.json',{'schema':'engine073-launch-independent-v12','author':'independent evaluator /root/engine07_realworld_independent','cases':cases}),('inputs.json',[c['input'] for c in cases])]:
    with (ROOT/name).open('x') as f:json.dump(value,f,ensure_ascii=False,indent=2);f.write('\n')
print('new_v12_cases',len(cases))
