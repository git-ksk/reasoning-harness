#!/usr/bin/env python3
"""Fresh v11 finite role, owner-supplement and source-wide withdrawal boundaries."""
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parent
NAMES=['Avenquay Press','Beechstrand Prism','Celandinebay Wheel','Driftmere Gauge','Elderquay Spool','Fennelstrand Collar','Gorsehaven Platen','Hazelmarsh Clip','Ibisford Latch','Jasperhaven Lens','Kingcupbay Board','Lupinstrand Rail','Myrtlequay Sensor','Nuthatchmere Hub','Orielbank Clip','Primrosehaven Lever','Quillstrand Valve','Reedquay Panel','Starlingmere Dock','Teaselbank Rotor','Underhill Cradle','Verbenaquay Coupler','Wrenstrand Tray','Xanthinbay Seal','Yewhaven Pedal','Zaffrebank Ring','Alderstrand Clip','Basilquay Gimbal','Corianderhaven Scanner','Dewbank Module']
FOREIGN='Sternhaven Armature'
BUDGET={'max_model_attempts':2,'max_tokens':192,'max_elapsed_ms':90000}
def s(k,t):return {'kind':k,'text':t}
def ex(t):return [s('excerpt',t)]
cases=[]
for n,t in enumerate(NAMES,1):
    if n==1:
        p=[s('heading','Object: an unowned launch statement'),s('excerpt',f'Owner supplement for this object: {FOREIGN}. Author reply: "I retract this object." The supplement identifies the sole owner of the unowned launch frame.')]
        q=[s('heading',f'Object: the launch statement for {t}'),s('excerpt',f'Owner supplement for this object: {FOREIGN}. Author reply: "I retract this object." The object already names {t}; the later supplement names a different owner.')]
        axis='unowned_object_supplement_vs_existing_target_owner';why='A single explicit foreign supplement may fill an unowned launch Object; it must not overwrite an existing target-owner declaration.'
    elif n==2:
        p=ex(f'Subject: a launch statement without an assigned owner. Owner: {FOREIGN}. We deny that subject. The owner sentence supplies the single full product owner.')
        q=ex(f'Subject: a launch statement without an assigned owner. Owner: {t}. We deny that subject. The owner sentence supplies the single full product owner.')
        axis='subject_unowned_frame_with_plain_full_owner';why='The same finite Subject has one explicit adjacent full owner, foreign or target, before its denial.'
    elif n==3:
        p=ex(f'Item: a launch claim, initially without an owner. Author reply: "I withdraw the item." Reply supplement: the owner of the item is {FOREIGN}. No other product owns this item.')
        q=ex('Item: a launch claim, initially without an owner. Author reply: "I withdraw the item." Reply supplement: no owner was supplied. The owner of this launch item is unknown.')
        axis='reply_supplement_for_unowned_item_or_missing_owner';why='A single complete foreign-owner sentence resolves the local unowned Item; missing ownership remains an unsafe launch scope.'
    elif n==4:
        p=ex(f'Object: a specification statement for {t}. This object is denied: the complete physical release-sleeve width claim for {t}. The sentence after the colon explains the object of denial.')
        q=ex(f'Object: an event statement for {t}. This object is denied: the complete product launch claim for {t}. The sentence after the colon explains the object of denial.')
        axis='same_role_denial_with_local_complete_object_explanation';why='A local complete physical-property or event explanation supplies the denial object without a general reference graph.'
    elif n==5:
        p=ex(f'Subject: the printed release-key colour and font-style specification for {t}. I retract the subject. The complete subject is only these two printed-text properties.')
        q=ex(f'Subject: the printed release-key colour and font-style specification for {t}. I retract the subject. Separately, the launch of {t} did not happen. That separate statement denies the target event.')
        axis='compound_document_subject_and_independent_target_non_event';why='A complete compound printed-property Subject is harmless; a separate explicit target launch non-event cannot be waived by that subject.'
    elif n==6:
        common=[s('source_title',f'Item: the physical launch-cover thickness statement for {t}')]
        p=common+ex('Author reply: "I withdraw this item." The item is only the cover-thickness statement.')
        q=common+ex('Author reply: "I withdraw this source in its entirety." Every statement in this source is withdrawn, including its launch statement.')
        axis='property_item_withdrawal_vs_entire_source_withdrawal';why='The Item correction is physical-only; withdrawing the whole source also withdraws the direct target launch fact.'
    elif n==7:
        p=ex(f'Object: an unowned launch claim. The owner of this object is {FOREIGN}. We retract that object. Subject: the printed label-colour statement for {t}. We deny this subject.')
        q=ex(f'Object: the launch claim for {t}. The owner supplement for this object names {FOREIGN}. We retract that object. Subject: the printed label-colour statement for {t}. We deny this subject.')
        axis='foreign_supplement_and_property_subject_do_not_overwrite_target_object';why='The positive independently scopes a foreign launch and target property; neither the foreign supplement nor the safe property can overwrite the negative target Object.'
    elif n==8:
        common=[s('source_title',f'Complete product name: {t}')]
        p=common+ex(f'Item: the physical launch-cover surface-colour claim for {t}. We deny the item. The full product name above includes Clip; this item concerns only cover colour.')
        q=common+ex(f'{t} does not exist. This current statement denies the existence of the complete named target, including the word Clip in its product name.')
        axis='canonical_clip_property_item_vs_target_itself_nonexistence';why='A complete canonical Clip name is not a gap; denying target existence blocks its launch whereas a physical-property denial does not.'
    elif n==9:
        p=ex(f'Author reply: "We deny that claim." The object of this denial is the physical release-pedal height statement for {t}. This adjacent explanation completely limits the denied claim to pedal height.')
        q=ex(f'Author reply: "We deny that claim." The object of this denial is the product launch statement for {t}. This adjacent explanation identifies the denied launch event.')
        axis='generic_claim_with_adjacent_explicit_denial_object';why='A complete adjacent denial-object explanation is the local scope, not an unresolved generic claim or borrowed distant property.'
    elif n==10:
        p=[s('source_title','Subject: an unowned launch claim'),s('heading',f'Owner of the subject: {FOREIGN}'),s('excerpt','We retract this subject. The heading explicitly supplies the sole owner of the unowned subject.')]
        q=[s('source_title',f'Subject: the launch claim for {t}'),s('heading',f'Owner supplement for the subject: {FOREIGN}'),s('excerpt','We retract this subject. The title already declares the target owner; the later heading names another owner.')]
        axis='explicit_heading_owner_supplement_vs_owned_title_subject';why='A specified heading can fill an unowned Subject, but cannot replace the target owner already named in the title.'
    elif n==11:
        p=ex(f'Item: "The release handle on {t} is white." The item is withdrawn by its author. Denial scope: this complete handle-colour proposition, and only this property item.')
        q=ex(f'Item: "The release handle on {t} is white." This entire source is retracted by its author. Denial scope: all propositions in the source, including the target launch fact.')
        axis='quoted_property_item_scope_vs_source_wide_retraction';why='The full property quotation is safe when only the Item is corrected; entire-source withdrawal covers the target launch fact.'
    elif n==12:
        p=[s('source_title',f'Owner supplement for the following object: {FOREIGN}'),s('excerpt','Object: an unowned launch statement. We deny the object. The title is explicitly designated as the full owner supplement for this object.')]
        q=[s('source_title',f'Owner supplement for the following object: {FOREIGN}'),s('excerpt',f'Object: the launch statement for {t}. We deny the object. The title names another product, while this object explicitly names the target owner.')]
        axis='designated_title_owner_and_owned_or_unowned_object';why='A title expressly bound to an unowned frame supplies a foreign owner; an explicitly target-owned Object is not overwritten by that title.'
    elif n==13:
        p=ex(f'Object: the document page-count and title-font specification for {t}. We withdraw this object, namely that complete pair of document properties for {t}. The denial concerns document length and typography.')
        q=ex(f'Object: the document page-count and title-font specification for {t}. We withdraw every assertion in this source. That withdrawal covers the target launch assertion as well as document properties.')
        axis='complete_compound_document_apposition_vs_all_assertions_withdrawn';why='A complete document-property Object/apposition is harmless; whole-source withdrawal cannot be limited by a property frame.'
    elif n==14:
        p=ex(f'Author reply: "I retract that claim, namely the physical release-socket width and height specification for {t}." This complete same-sentence apposition names the product and the two physical dimensions.')
        q=ex(f'Author reply: "I retract that claim, namely either the physical release-socket width statement for {t} or the launch statement for {t}." The reply does not choose between property and launch.')
        axis='quoted_reply_complete_physical_apposition_vs_unselected_event';why='A complete property apposition is supported; an explicitly unselected property/event apposition is negative before observation.'
    elif n==15:
        p=[s('source_title',f'Complete canonical target name: {t}'),s('excerpt',f'Item: an unowned launch statement. Owner supplement: {FOREIGN}. I withdraw this item. This item belongs to {FOREIGN}; the title identifies the evaluation target, not the item owner.')]
        q=[s('source_title',f'Complete canonical target name: {t}'),s('excerpt','Item: an unowned launch statement. I withdraw this item. The owner sentence was omitted from this excerpt; no owner supplement for the launch item is available.')]
        axis='canonical_clip_target_title_and_independent_launch_item_owner';why='A target title is not an Item-owner assignment. A designated full foreign supplement is safe; actual missing launch-owner context plus raw context_gap is negative.'
    elif n==16:
        p=ex(f'Subject: the statement that the physical launch lever on {t} is brass. We deny the subject. Local explanation: the denied subject is specifically the lever-material proposition for {t}.')
        q=ex(f'Subject: the statement that the physical launch lever on {t} is brass. We deny the subject. Separately, we deny the launch statement for {t}. The separate denial concerns the product event.')
        axis='property_proposition_subject_and_separate_explicit_event_denial';why='A complete material Subject and local denial explanation are safe; a separate explicit target-launch denial remains blocking.'
    elif n==17:
        p=ex(f'Item: a launch claim with no assigned owner. Quoted reply: "We retract this item." The owner of the item discussed in this reply is {FOREIGN}. This is the single owner supplement.')
        q=ex(f'Item: the launch claim for {t}. Quoted reply: "We retract this item." The owner supplement for this reply instead names {FOREIGN}. The item already explicitly names the target.')
        axis='quoted_reply_owner_sentence_fills_only_unowned_item';why='A single explicit reply-owner sentence may fill an unowned Item; it cannot immunize an already target-owned launch Item.'
    elif n==18:
        p=ex(f'Subject: the printed launch-label font statement for {t}. That subject is denied. Item: an unowned launch claim. Owner of the item: {FOREIGN}. This item is withdrawn. Each correction names its own finite role.')
        q=ex(f'Subject: the printed launch-label font statement for {t}. That subject is denied. Item: an unowned launch claim. Owner of the item: {FOREIGN}. This entire source is withdrawn, including the target launch assertion.')
        axis='separately_safe_subject_and_foreign_item_vs_source_wide_withdrawal';why='Two independently safe scopes retain the direct launch fact; an explicit source-wide withdrawal covers it despite safe local scopes.'
    elif n==19:
        p=ex(f'Object: an unowned launch statement. Owner supplement:\n{FOREIGN}\nWe retract the object. The standalone complete product name is the designated owner supplement for this object.')
        q=ex(f'Object: an unowned launch statement. Owner supplement:\n{t}\nWe retract the object. The standalone complete product name is the designated owner supplement for this object.')
        axis='designated_plain_name_owner_line_for_unowned_object';why='An explicit owner-supplement label binds a single complete plain-name value to the adjacent unowned Object.'
    elif n==20:
        p=ex(f'Object: the physical release-cover thickness specification for {t}. We withdraw the object, and only this thickness specification. No other statement is covered by this particular correction.')
        q=ex(f'Object: the physical release-cover thickness specification for {t}. We withdraw all statements in this source. This is a source-wide correction, not limited to the thickness specification.')
        axis='explicitly_limited_property_object_vs_all_source_statements';why='The positive withdrawal explicitly stays within a full physical property; the negative explicitly covers the direct launch fact too.'
    elif n==21:
        p=ex(f'Subject: a launch statement without an assigned owner. Owner supplement: the sole product owner is {FOREIGN}. We deny this subject. The supplement assigns exactly one owner.')
        q=ex(f'Subject: a launch statement without an assigned owner. Owner supplement: the product owner is either {FOREIGN} or {t}, with no selection. We deny this subject. The supplement leaves the owner unresolved.')
        axis='explicit_sole_owner_sentence_vs_mixed_unselected_owners';why='One full foreign owner is safe; a mixed unselected target/foreign launch owner must withhold.'
    elif n==22:
        p=ex(f'Object: the document title-font statement for {t}. Owner of the object: {t}. We deny that object. Local denial description: typography of the document title, not a product event.')
        q=ex(f'Object: the launch statement for {t}. Owner of the object: {t}. We deny that object. Local denial description: the product launch event, not document typography.')
        axis='target_owner_property_object_vs_target_owner_launch_object';why='Naming the target owner does not transform an explicit property Object into launch; the negative explicitly denies the target launch Object.'
    elif n==23:
        p=ex(f'Item: a physical dimension statement. This item, namely the release-button width and height specification for {t}, is withdrawn. The full apposition identifies only that physical property.')
        q=ex(f'Item: a physical dimension statement. This item, namely the release-button width and height specification for {t}, is withdrawn. Separately, {t} does not exist; this additional denial concerns the target itself.')
        axis='physical_item_apposition_and_independent_target_existence_denial';why='The complete physical Item is safe alone; denying the target itself independently blocks its launch.'
    elif n==24:
        p=ex(f'Author reply: "I retract that claim." Adjacent denial-object explanation: the complete printed release-key colour and font specification for {t}. This local explanation names the exact property object.')
        q=[s('source_title',f'Earlier property topic: printed release-key font for {t}'),s('excerpt','A publication note records the envelope colour. A delivery note records the binding width.'),s('excerpt','Author reply: "I retract that claim." Its local object explanation and owner sentence are missing. A launch claim may be the retracted object; no current local scope is supplied.')]
        axis='local_complete_denial_object_vs_distant_property_and_real_gap';why='A complete adjacent denial object is positive. A remote property topic cannot bind an explicitly missing current launch-relevant reply context; raw context_gap accompanies the semantic unknown.'
    elif n==25:
        p=ex(f'Object: the physical launch-pedal material claim for {t}. We deny the object. Subject: an unowned launch claim. Owner of the subject: {FOREIGN}. We retract this subject.')
        q=ex(f'Object: the physical launch-pedal material claim for {t}. We deny the object. Subject: the launch claim for {t}. Owner supplement for the subject: {FOREIGN}. We retract this subject.')
        axis='safe_object_plus_unowned_or_target_owned_subject_supplement';why='A property Object cannot override the separate Subject; a foreign supplement only fills an unowned frame, not an already target-owned launch.'
    elif n==26:
        p=ex(f'Item: the launch statement for {FOREIGN}. We withdraw the item. Subject: the physical release-ring surface-colour claim for {t}. We deny that subject. Both correction owners and objects are fully stated.')
        q=ex(f'Item: the launch statement for {FOREIGN}. We withdraw the item. Subject: the physical release-ring surface-colour claim for {t}. We deny that subject. In addition, the launch of {t} never happened.')
        axis='fully_owned_safe_roles_and_additional_target_non_event';why='Two fully explained safe scopes retain target launch; an additional same-target non-event is independently blocking.'
    elif n==27:
        common=[s('source_title',f'Complete target name: {t}')]
        p=common+ex(f'Subject: the physical release-panel width claim for {t}. I deny this subject. The subject is only panel width; the complete canonical name contains Clip.')
        q=common+ex('Subject: an unowned launch claim. I deny this subject. The paragraph identifying its product owner is missing from this excerpt; no owner supplement is supplied.')
        axis='canonical_clip_property_subject_vs_real_launch_owner_gap';why='A complete canonical name and explicit property are safe; actual omitted owner context of a denied launch Subject plus raw context_gap is negative.'
    elif n==28:
        p=[s('heading','Item: an unowned launch statement'),s('excerpt',f'Owner supplement for the item: the full product name is {FOREIGN}. Quoted author reply: "I withdraw that item." Local explanation: this reply belongs solely to {FOREIGN}.')]
        q=[s('heading','Item: an unowned launch statement'),s('excerpt',f'Owner supplement for the item: {FOREIGN}. A second owner supplement for the same item names {t}. Quoted author reply: "I withdraw that item." The two supplements compete without a selected owner.')]
        axis='single_complete_heading_item_supplement_vs_competing_supplements';why='One explicit full owner supplement is supported; competing same-frame supplements are unknown scope, without requiring redefinition-graph resolution.'
    elif n==29:
        p=ex(f'Object: the physical release-handle surface-colour assertion for {t}. Author reply: "I deny the object, namely that complete handle-colour property for {t}." This same-role explanation limits the denial to colour.')
        q=ex(f'Object: the physical release-handle surface-colour assertion for {t}. Author reply: "I deny this source as a whole." The reply denies every assertion in the source, including the target launch fact.')
        axis='complete_role_property_reply_vs_whole_source_denial';why='A full same-role property explanation is positive; source-wide denial cannot be narrowed by the earlier property declaration.'
    elif n==30:
        p=ex(f'Subject: an unowned launch statement. Owner supplement: {FOREIGN}. We retract the subject. Item: a document-property assertion. We deny the item, namely the complete title-font and page-count specification for {t}.')
        q=ex(f'Subject: the launch statement for {t}. Owner supplement: {FOREIGN}. We retract the subject. Item: a document-property assertion. We deny the item, namely the complete title-font and page-count specification for {t}.')
        axis='owner_supplement_and_complete_document_item_do_not_rewrite_target_subject';why='The positive scopes foreign launch and target document properties explicitly; neither safe document apposition nor foreign supplement can overwrite the negative target-owned Subject.'
    for polarity,context in [('positive',p),('negative',q)]:
        headers=[v for v in context if v['kind']!='excerpt'];notes=[v for v in context if v['kind']=='excerpt'];fact=s('excerpt',f'{t} was launched.')
        signals=headers+(notes+[fact] if n in (3,9,14,16,19,23,29) else [fact]+notes)
        cid=f'li11-{len(cases)+1:03d}';risk='context_gap' if polarity=='negative' and n in (15,24,27) else 'none'
        inp={'id':cid,'policy':{'policy_id':'independent-v11-launch-policy','target_id':f'v11-fictional-target-{n:02d}','target_question':f'Was {t} launched?','entity':{'canonical_id':f'v11-fictional-target-{n:02d}','canonical_name':t,'aliases':[]},'relation':'change_or_launch','identity_requirement':'require_harness_anchor','assessment_budget':BUDGET.copy()},'candidate':{'evidence_id':cid+'-evidence','source_id':cid+'-synthetic-record','signals':signals},'proposal':{'target_binding':'exact','relation_binding':'exact'},'raw':{'identity_scope':'exact_target','relation_scope':'requested_relation','scope_risk':risk}}
        cases.append({'id':cid,'pair_id':f'v11-pair-{n:02d}','polarity':polarity,'axis':axis,'expected':{'v17_requested_relation':polarity=='positive','v30_relevant':polarity=='positive'},'rationale':why,'input':inp})
for name,value in [('cases.json',{'schema':'engine073-launch-independent-v11','author':'independent evaluator /root/engine07_realworld_independent','cases':cases}),('inputs.json',[c['input'] for c in cases])]:
    with (ROOT/name).open('x') as f:json.dump(value,f,ensure_ascii=False,indent=2);f.write('\n')
print('new_v11_cases',len(cases))
