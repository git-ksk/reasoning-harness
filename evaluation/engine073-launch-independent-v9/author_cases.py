#!/usr/bin/env python3
"""New v9 typed-object/quote/apposition/owner-boundary controls, before semantics."""
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parent
NAMES=['Apricotmere Compass','Birchwharf Tray','Crescentfen Sensor','Dillhaven Coupler','Eidermead Prism','Foxglovequay Rail','Gingermere Loom','Hollyfield Wheel','Inkbay Pedal','Jonquilford Coil','Knapweedshore Module','Lavenderwharf Gauge','Mulberryreach Dock','Nectarhaven Ring','Oxalisbay Panel','Poppyford Seal','Quailmeadow Spindle','Ravensedge Valve','Snowberryquay Lever','Tansybrook Frame','Umberlake Key','Vervainford Rotor','Whitecloverbay Hub','Xylemfield Clip','Yolkmeadow Cradle','Zinnialake Scanner','Ashwharf Board','Brackenfield Slide','Cornflowerhaven Gimbal','Dogwoodquay Recorder']
OTHER='Rivermint Handle'
BUDGET={'max_model_attempts':2,'max_tokens':192,'max_elapsed_ms':90000}
def sg(kind,text):return {'kind':kind,'text':text}
def ex(text):return [sg('excerpt',text)]
cases=[]
for n,t in enumerate(NAMES,1):
    fact=sg('excerpt',f'{t} was launched.')
    if n==1:
        p=ex(f'Correction ticket. Object: the physical launch-button width claim for {t}. I withdraw that object. The object is a button-width specification.')
        q=ex(f'Correction ticket. Object: the launch claim for {t}. I withdraw that object. The object is the product launch statement.')
        axis='object_declaration_then_that_object';why='The typed object and repeated type uniquely identify button width or launch; the reference requires no general graph.'
    elif n==2:
        p=ex(f'Subject: the launch-indicator colour statement for {t}. We deny the statement about this subject. The subject is an indicator-colour property.')
        q=ex(f'Subject: the launch statement for {t}. We deny the statement about this subject. The subject is the launch event.')
        axis='subject_declaration_and_subject_statement';why='The subject type and full owner are explicit and adjacent to the denial.'
    elif n==3:
        p=ex(f'Item: "The release handle on {t} is brass." I retract that item. It is the quoted handle-material statement.')
        q=ex(f'Item: "{t} was launched." I retract that item. It is the quoted launch statement.')
        axis='quoted_item_and_explicit_item_type';why='One complete quote defines the singular item; material and event objects are not confused.'
    elif n==4:
        p=ex(f'Owner supplement: {OTHER}. Author reply: "I retract its launch claim." The reply identifies its owner as {OTHER}.')
        q=ex(f'Owner supplement: {t}. Author reply: "I retract its launch claim." The reply identifies its owner as {t}.')
        axis='owner_supplement_before_current_reply';why='The adjacent supplement and final full-name restatement both assign the singular launch claim owner.'
    elif n==5:
        p=ex(f'Item: a singular claim. That claim, namely the release-lever material statement for {t}, is withdrawn. The item concerns lever material.')
        q=ex(f'Item: a singular claim. That claim, namely the launch statement for {t}, is withdrawn. The item concerns the product launch event.')
        axis='singular_item_apposition';why='The same-sentence apposition fully defines the singular claim and owner rather than leaving an unbound item.'
    elif n==6:
        p=ex(f'Object: the printed launch-label statement for {t}. Subject: typography of that printed launch label. We retract this typography statement.')
        q=ex(f'Object: the launch statement for {t}. Subject: typography of a printed launch label. We retract the launch statement for {t}.')
        axis='object_subject_consistency_vs_explicit_event';why='The positive roles consistently describe a printed property. A typography subject cannot erase the negative explicit target launch retraction.'
    elif n==7:
        p=ex(f'Author reply: "I withdraw that claim." Object: the physical launch-switch voltage claim for {t}. The reply withdraws that electrical-rating object.')
        q=ex(f'Author reply: "I withdraw that claim." Object: the launch claim for {t}. The reply withdraws that product-launch object.')
        axis='reply_followed_by_adjacent_object';why='One adjacent complete typed object resolves the singular reply, with an explicit electrical/event head.'
    elif n==8:
        p=ex(f'Subject: the launch statement for {OTHER}. We deny that statement. The full owner of the denied statement is {OTHER}.')
        q=ex(f'Subject: the launch statement for {t}. We deny that statement. The full owner of the denied statement is {t}.')
        axis='subject_with_repeated_complete_launch_owner';why='The subject and final owner uniquely agree on another product or the target.'
    elif n==9:
        p=[sg('heading',f'Item: the release-button width statement for {t}'),sg('excerpt','Current author reply: "I deny this width statement." The item is a physical-button-width specification.')]
        q=[sg('heading',f'Item: the launch statement for {t}'),sg('excerpt','Current author reply: "I deny this launch statement." The item is the product launch statement.')]
        axis='item_heading_and_typed_reply';why='The heading supplies a full owner/object and the adjacent reply repeats its property/event head.'
    elif n==10:
        p=ex(f'Quoted object: "The release handle on {t} is blue." We deny this colour claim. Object type: handle colour.')
        q=ex(f'Quoted object: "{t} was launched." We deny this launch claim. Object type: product launch.')
        axis='concrete_quote_with_object_type';why='The quoted concrete property and object-type restatement are explicit; no arbitrary property truth inference is requested.'
    elif n==11:
        p=ex(f'Subject: surface material of {t}\'s launch socket. The claim about that subject is retracted. This is the socket-material claim.')
        q=ex(f'Subject: the launch of {t}. The claim about that subject is retracted. This is the product-launch claim.')
        axis='subject_property_possessive_or_event';why='The subject head material is physical despite the component modifier launch; the event subject belongs to target launch.'
    elif n==12:
        p=ex(f'Object: the launch claim for {OTHER}. Author reply: "I retract this claim." The adjacent owner supplement names {OTHER} as the owner of the retracted launch claim.')
        q=ex(f'Object: the launch claim for {OTHER}. Author reply: "I retract this claim." The adjacent owner supplement instead names {t} as the owner of the retracted launch claim. The two owner assignments conflict.')
        axis='object_and_owner_supplement_agree_or_conflict';why='The positive uniquely assigns another owner. Conflicting complete owner assignments in the negative are unresolved and must withhold.'
    elif n==13:
        p=ex(f'Subject: the launch-selector label-font claim for {t}. Author reply: "We deny the launch-selector label-font claim for {t}." The subject is printed typography.')
        q=ex(f'Subject: the launch-selector label-font claim for {t}. Author reply: "We deny the launch claim for {t}." The reply explicitly denies the product launch event.')
        axis='property_subject_vs_reply_event_override';why='The positive object repeats the typed property; a property subject cannot waive the explicit target event denial in the negative.'
    elif n==14:
        common=[sg('source_title',f'Item: the launch-banner colour claim for {t}')]
        p=common+ex(f'We withdraw this banner-colour item. Separately, we deny the launch-banner font-size claim for {t}. Both corrections concern printed banner properties.')
        q=common+ex(f'We withdraw this banner-colour item. Separately, we deny the launch claim for {t}. The separate denial concerns the product event.')
        axis='item_and_separate_explicit_correction';why='Both positive corrections have clear non-launch objects and one owner; the separate negative target launch denial must not be masked.'
    elif n==15:
        common=[sg('heading',f'Object: the printed legend-font claim for {t}')]
        p=common+ex(f'We retract the legend-font claim for {t}. The claim concerns printed text.')
        q=common+ex(f'The launch of {t} never happened. This current correction concerns the launch event.')
        axis='property_object_with_independent_non_event';why='A typed property object does not erase a separate explicit same-target launch non-event.'
    elif n==16:
        p=[sg('source_title',f'Object: the physical launch-port width claim for {t}'),sg('excerpt',f'I withdraw that width claim for {t}. This is an aperture-width correction.')]
        q=[sg('source_title',f'Object: the launch claim for {OTHER}'),sg('excerpt',f'I withdraw the launch claim for {t}. This reply explicitly names {t}, while the header names another product.')]
        axis='clear_property_vs_mixed_owner_header_reply';why='The positive is one clear property correction. The negative includes an explicit target launch withdrawal and conflicting header ownership.'
    elif n==17:
        common=[sg('source_title',f'Item: the document page-count claim for {t}')]
        p=common+ex(f'I withdraw that item—the document page-count claim for {t}. The item is a document-length property.')
        q=common+ex(f'I withdraw that item—the launch claim for {t}. The apposition identifies a launch event even though the header identifies a document property.')
        axis='item_header_and_apposition_type_conflict';why='A full property apposition preserves launch; an explicit target launch apposition requires withholding despite the competing property header.'
    elif n==18:
        p=ex(f'Subject: the launch claim for {OTHER}. Author reply: "We retract this claim." Owner supplement: {OTHER}, the full owner of this launch claim.')
        q=ex('Subject: a launch claim. Author reply: "We retract this claim." Owner supplement: not recorded. The product of this launch claim is unidentified.')
        axis='subject_reply_with_owner_present_or_missing';why='The positive names another complete owner twice; the negative explicitly omits the owner and is not forced positive.'
    elif n==19:
        p=ex(f'Object: the following quoted colour property. "The launch-button cap on {t} is red." I withdraw this colour-property object.')
        q=ex(f'Object: the following quoted launch statement. "{t} was launched." I withdraw this launch-statement object.')
        axis='forward_quoted_object_with_complete_type';why='One concrete quotation directly supplies a physical colour object or the existing launch proposition.'
    elif n==20:
        p=ex(f'Subject: the launch-key legend-font statement for {t}. Owner: {t}. We retract this font statement. The object is printed typography.')
        q=ex(f'Subject: a launch statement. Owner assignments: {t} and {OTHER}, with no selection. We retract this launch statement. The correction does not identify which product owns the retracted statement.')
        axis='one_property_owner_vs_mixed_launch_assignments';why='One complete property owner is unambiguous; mixed unselected launch-owner assignments are a withholding negative.'
    elif n==21:
        common=ex(f'Object: the physical release-button height claim for {t}. I withdraw that height claim.')
        p=common+ex('Footer: the page margins were widened. This footer records a layout change.')
        q=common+ex('Footer correction: I retract another launch claim. The owner of this additional launch claim is missing from the footer.')
        axis='typed_property_and_footer_unknown_launch';why='Neutral layout text is harmless; an additional owner-missing launch withdrawal cannot inherit or be waived by the property object.'
    elif n==22:
        p=ex(f'Owner supplement: {OTHER}. Quoted author reply: "I deny its launch claim." The owner of that launch claim is explicitly {OTHER}.')
        q=ex(f'Owner supplement: {t}. Quoted author reply: "I deny its launch claim." The owner of that launch claim is explicitly {t}.')
        axis='adjacent_owner_and_possessive_quote';why='Both adjacent owner declarations uniquely resolve the singular possessive to another product or the target.'
    elif n==23:
        p=ex(f'Item: the quoted statement about {OTHER}\'s launch. "{OTHER} was launched." I withdraw that item. The item owner is {OTHER}.')
        q=ex(f'Item: one launch statement concerning either {t} or {OTHER}. I withdraw that item. The item owner is not selected or recorded.')
        axis='quoted_item_full_owner_vs_either_owner';why='The positive complete quotation and owner agree; an unselected either-owner launch item is unresolved.'
    elif n==24:
        p=ex(f'Subject: release-knob material on {t}. Author reply: "I retract the claim about this subject." Owner supplement: {t}; the subject is mechanical-knob material.')
        q=ex('Subject: a product launch. Author reply: "I retract the claim about this subject." Owner supplement: absent; the product is not identified.')
        axis='subject_property_quote_vs_absent_launch_owner';why='The positive subject/type/owner are explicit and non-launch; the negative lacks the owner of a launch correction.'
    elif n==25:
        p=ex(f'Object: the release-latch material claim for {t}. Item: the same release-latch material claim for {t}. I withdraw that material claim.')
        q=ex(f'Object: the release-latch material claim for {t}. Item: the launch claim for {t}. I withdraw that claim. The reference does not select between the property object and launch item.')
        axis='repeated_property_roles_vs_unselected_competing_types';why='The positive roles fully repeat one property object; the negative expressly leaves the reference between property and launch unselected.'
    elif n==26:
        common=[sg('source_title',f'Subject: the printed launch-icon colour claim for {t}')]
        p=common+ex(f'Item: the printed launch-icon colour claim for {t}. I withdraw that colour item. It is a display/printed-colour property.')
        q=common+ex(f'Author reply: "I withdraw the launch claim for {t}." The reply concerns the product event rather than the printed-icon property.')
        axis='property_subject_item_vs_explicit_launch_reply';why='Consistent property roles are harmless; an explicit target launch withdrawal is not waived by a property title.'
    elif n==27:
        p=ex(f'Quoted text: "The release handle on {t} is aluminium." Object: that handle-material claim for {t}. We retract this object.')
        q=ex(f'Quoted text: "The release handle on {t} is aluminium." Object: the launch claim for {t}. We retract this object. The typed object and quoted property disagree.')
        axis='quote_object_agreement_or_launch_conflict';why='The positive quote and typed material object agree; the negative explicitly includes target launch as the retracted typed object.'
    elif n==28:
        p=ex(f'Subject: the launch claim for {OTHER}. Owner supplement: {OTHER}. I retract this subject claim; it belongs to {OTHER}.')
        q=ex(f'Subject: the launch claim for either {t} or {OTHER}. I retract this subject claim. Its product owner is not selected.')
        axis='subject_full_other_owner_vs_either_owner';why='The positive repeatedly names another owner; the negative either-owner subject is explicitly unresolved.'
    elif n==29:
        p=ex(f'Item: the launch-button width claim for {t}. This item—and only this physical width claim—is withdrawn by its author.')
        q=ex('Item: an unspecified launch claim. This item is withdrawn by its author. No product owner is recorded for the item.')
        axis='appositive_physical_item_vs_ownerless_launch_item';why='The singular positive apposition fixes width scope; the negative item explicitly omits its launch owner.'
    elif n==30:
        common=[sg('heading',f'Object: the launch-selector font-style claim for {t}')]
        p=common+ex(f'Question: Which statement is withdrawn? Author reply: "I withdraw the launch-selector font-style claim for {t}." The reply identifies typography as its object.')
        q=common+ex(f'Question: Which statement is withdrawn? Author reply: "I withdraw the launch claim for {t}." The reply identifies the target launch as its object.')
        axis='typed_object_and_reply_full_proposition';why='The operative reply itself repeats a full property or event object, so no arbitrary dialogue-role parser is needed.'
    for polarity,context in [('positive',p),('negative',q)]:
        headers=[x for x in context if x['kind']!='excerpt'];notes=[x for x in context if x['kind']=='excerpt']
        signals=headers+(notes+[fact] if n in (7,11,19) else [fact]+notes)
        cid=f'li9-{len(cases)+1:03d}'
        inp={'id':cid,'policy':{'policy_id':'independent-v9-launch-policy','target_id':f'v9-fictional-target-{n:02d}','target_question':f'Was {t} launched?','entity':{'canonical_id':f'v9-fictional-target-{n:02d}','canonical_name':t,'aliases':[]},'relation':'change_or_launch','identity_requirement':'require_harness_anchor','assessment_budget':BUDGET.copy()},'candidate':{'evidence_id':cid+'-evidence','source_id':cid+'-synthetic-record','signals':signals},'proposal':{'target_binding':'exact','relation_binding':'exact'},'raw':{'identity_scope':'exact_target','relation_scope':'requested_relation','scope_risk':'none'}}
        cases.append({'id':cid,'pair_id':f'v9-pair-{n:02d}','polarity':polarity,'axis':axis,'expected':{'v17_requested_relation':polarity=='positive','v30_relevant':polarity=='positive'},'rationale':why,'input':inp})
for name,value in [('cases.json',{'schema':'engine073-launch-independent-v9','author':'independent evaluator /root/engine07_realworld_independent','cases':cases}),('inputs.json',[c['input'] for c in cases])]:
    with (ROOT/name).open('x') as f:json.dump(value,f,ensure_ascii=False,indent=2);f.write('\n')
print('new_v9_cases',len(cases))
