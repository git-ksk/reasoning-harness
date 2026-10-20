#!/usr/bin/env python3
"""Fresh v13 multiple-adverse-clause, same-role owner and property/event controls."""
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parent
NAMES=['Avocetbank Press','Bellflowerquay Rail','Cloudmere Gauge','Dulsehaven Lens','Egretstrand Wheel','Fescuebank Module','Garnetquay Clip','Harebellmere Tray','Ivoryhaven Collar','Juncusbank Panel','Kitequay Spindle','Lindenmere Dock','Madderhaven Rotor','Nightjarbank Clip','Ochrequay Coupler','Petrelmere Lever','Quillhaven Hub','Rowanbank Sensor','Samphirequay Board','Ternmere Platen','Ulmusbank Cradle','Violetquay Pedal','Waxwinghaven Clip','Xylitebank Seal','Yarrowquay Gimbal','Zinnoberhaven Valve','Asterbank Ring','Berylquay Scanner','Cloverhaven Latch','Dahliafield Prism']
FOREIGN='Coralwick Frame'
SECOND='Maplewharf Dial'
BUDGET={'max_model_attempts':2,'max_tokens':192,'max_elapsed_ms':90000}
def s(k,t):return {'kind':k,'text':t}
def ex(t):return [s('excerpt',t)]
cases=[]
for n,t in enumerate(NAMES,1):
    if n==1:
        common=f'Subject: the physical release-handle width specification for {t}. We deny this subject because we deny the launch claim for {FOREIGN}, and we also retract the launch statement for '
        p=ex(common+f'{SECOND}. The three corrections have a complete physical scope and two explicitly named foreign event owners.')
        q=ex(common+f'{t}. The last clause separately withdraws the target launch, despite two earlier safe corrections.')
        axis='three_adverse_clauses_property_and_two_explicit_events';why='Every positive adverse clause is explicitly outside target launch. The final target-event clause in the negative cannot borrow safety from earlier clauses.'
    elif n==2:
        common=f'Object: the document title-font specification for {t}. We retract the object although we deny the document heading-colour claim for {t}, and we withdraw '
        p=ex(common+f'the heading-font-size statement for {t}. All three corrections concern complete printed-document properties.')
        q=ex(common+f'the launch statement for {t}. The third correction concerns the product event rather than document typography.')
        axis='three_document_corrections_with_one_event_substitution';why='The Object exemption is local; each although/and correction needs its own complete property or event scope.'
    elif n==3:
        p=ex(f'Item: an unowned launch statement. Owner supplement for the item:\n{FOREIGN}\nWe deny this item as we report that we retract the printed release-label colour claim for {t}. The next-line name is the sole full owner supplement for this Item.')
        q=ex(f'Item: the launch statement for {t}. Owner supplement for the item:\n{FOREIGN}\nWe deny this item as we report that we retract the printed release-label colour claim for {t}. The Item already declares the target owner, which the next-line supplement cannot replace.')
        axis='same_item_next_line_owner_and_explicit_as_reporting_clause';why='A full next-line same-role owner fills an unowned launch frame only. A safe reporting property clause cannot overwrite an existing target-owned Item.'
    elif n==4:
        common=f'Item: "The release collar on {t} is blue." We retract the item, and we report that we also withdraw the launch claim for '
        p=ex(common+f'{FOREIGN}. The Item is physical colour and the reporting clause names the foreign event owner completely.')
        q=ex(common+f'{t}. The reporting clause explicitly describes our withdrawal of the target launch claim.')
        axis='quoted_property_item_and_reported_current_event_withdrawal';why='The explicit first-person reporting clause has its own full event owner; the quoted property Item cannot exempt it.'
    elif n==5:
        common=f'Subject: the printed release-key colour assertion for {t}. We deny the subject (and we deny the launch statement for {FOREIGN}); '
        p=ex(common+f'we also withdraw the launch claim for {SECOND}. Both event clauses have complete foreign owners.')
        q=ex(common+'we also withdraw a launch claim whose product owner is not identified. This last correction is a separate unknown launch claim.')
        axis='parenthesized_foreign_event_and_additional_unknown_launch';why='The positive has one property and two fully foreign events. A later unidentified launch withdrawal is unsafe even after safe parenthesized correction.'
    elif n==6:
        p=ex(f'Object: the launch statement for {FOREIGN}. Before we deny the physical release-wheel diameter claim for {t}, we withdraw the object and we deny the document title-font claim for {t}. All current corrections are fully scoped.')
        q=ex(f'Object: the launch statement for {t}. Before we deny the physical release-wheel diameter claim for {t}, we withdraw the object and we deny the document title-font claim for {t}. The Object explicitly belongs to target launch.')
        axis='before_clause_safe_properties_and_owned_launch_object';why='No chronology inference is needed: the first-person Object withdrawal has one complete owner, foreign or target, regardless of property clauses.'
    elif n==7:
        common=[s('source_title',f'Complete canonical name: {t}')]
        p=common+ex(f'Object: the physical release-button height claim for {t}. We retract the object while we deny the button-width specification for {t}, and we withdraw the printed button-label colour assertion for {t}. Note: Clip is part of this complete product name.')
        q=common+ex(f'Object: the physical release-button height claim for {t}. We retract the object while we deny the button-width specification for {t}, and we withdraw a launch claim whose owner sentence is missing from this excerpt. Note: Clip is part of this complete product name, but the separate launch owner is unknown.')
        axis='complete_clip_note_three_properties_vs_real_unknown_launch_gap';why='A complete canonical Clip name/note is not a gap. The negative has an independent ownerless launch withdrawal with actual omitted context and raw context_gap.'
    elif n==8:
        p=ex(f'Object: the launch statement for {FOREIGN}. We deny this object, and we report that we retract that launch claim, while we deny the printed release-label font statement for {t}. The local launch reference is the fully owned foreign event Object.')
        q=ex(f'Object: the physical release-cover material statement for {FOREIGN}. We deny this object, and we report that we retract that launch claim, while we deny the printed release-label font statement for {t}. The separate launch reference has no assigned owner; the Object is a property.')
        axis='and_reporting_launch_reference_cannot_borrow_property_frame';why='Explicit foreign-event type can support its local launch reference. A foreign property and another safe property cannot supply an unidentified launch owner.'
    elif n==9:
        common=f'Object: the physical release-cover colour claim for {t}. We deny the object. Subject: the launch claim for {FOREIGN}. We retract the subject as we report that '
        p=ex(common+f'we deny the launch statement for {SECOND}. Each event in the reporting/current clauses has a complete foreign owner.')
        q=ex(common+f'{t} does not exist. The explicit reporting clause denies the complete target itself.')
        axis='property_object_foreign_subject_and_reported_target_existence';why='A safe Object and foreign Subject cannot waive a separate explicitly reported target-existence denial.'
    elif n==10:
        common=[s('heading',f'Owner supplement for the object:\n{FOREIGN}')]
        p=common+ex(f'Object: an unowned launch statement. We retract the object because we deny the launch claim for {SECOND}, and we deny the physical release-sleeve width claim for {t}. The heading explicitly supplies this unowned Object\'s full owner.')
        q=common+ex(f'Object: the launch statement for {t}. We retract the object because we deny the launch claim for {SECOND}, and we deny the physical release-sleeve width claim for {t}. The Object already names target launch; the foreign heading cannot overwrite it.')
        axis='designated_heading_next_line_owner_and_owned_object';why='A designated complete heading owner fills an unowned same-role Object; it cannot overwrite the target owner already declared in a frame.'
    elif n==11:
        common=f'Item: the document title-colour and heading-font-size claim for {t}. We deny the item and we report that '
        p=ex(common+f'we withdraw the document page-count claim for {t}. Both full objects are printed/document properties.')
        q=ex(common+'we withdraw all assertions in this source. The report explicitly includes withdrawal of its target launch statement.')
        axis='compound_document_item_and_reported_source_wide_withdrawal';why='A complete property Item does not narrow an explicitly reported whole-source withdrawal to property scope.'
    elif n==12:
        common=f'I retract that claim, namely the complete release-knob width and height specification for {t}, although we deny the launch claim for {FOREIGN}, and we withdraw '
        p=ex(common+f'the launch statement for {SECOND}. The full local apposition is physical and both later event owners are foreign.')
        q=ex(common+f'the launch statement for {t}. The final event clause explicitly withdraws the target launch.')
        axis='complete_physical_apposition_and_two_separately_owned_event_clauses';why='A complete local property apposition scopes one clause; two subsequent event clauses must each independently have a safe full owner.'
    elif n==13:
        p=ex(f'Item: an unowned launch statement. Owner supplement for the item: {FOREIGN}. We withdraw the item, and we deny that launch claim while we retract the physical release-handle thickness claim for {t}. The launch reference is the fully foreign launch Item.')
        q=ex(f'Item: the physical release-handle height claim for {FOREIGN}. Owner of the item: {FOREIGN}. We withdraw the item, and we deny that launch claim while we retract the physical release-handle thickness claim for {t}. No owner is supplied for the separate launch claim; this Item is only a dimension statement.')
        axis='owned_property_item_does_not_supply_reported_launch_owner';why='The owner of a fully typed property is not automatically the owner of an independent launch reference; an explicitly supplemented foreign launch frame is different.'
    elif n==14:
        common=[s('heading',f'Complete target name: {t}')]
        p=common+ex(f'Subject: the printed release-label font-style claim for {t}. We deny the subject while we withdraw the launch statement for {FOREIGN}, and we retract the physical dial-colour claim for {SECOND}. Limited name note: Clip belongs to the complete canonical target name.')
        q=common+ex(f'Subject: the printed release-label font-style claim for {t}. We deny the subject while we withdraw the launch statement for {FOREIGN}, and {t} does not exist. Limited name note: Clip belongs to the complete canonical target name.')
        axis='canonical_clip_note_multiple_safe_clauses_vs_target_nonexistence';why='The complete Clip note is neutral; a current target-existence denial in the final clause blocks launch despite earlier safe scopes.'
    elif n==15:
        p=ex(f'Object: an unowned launch statement. Owner supplement for the object:\n{FOREIGN}\nWe withdraw that object as we report that we deny the launch statement for {SECOND}. Each launch correction has one full foreign owner.')
        q=ex(f'Object: an unowned launch statement. Owner supplement for the object:\nnot supplied\nPublication note: the envelope is grey. A later unassigned note names {FOREIGN}, but the Object\'s omitted owner sentence is unavailable. We withdraw that object as we report that we deny the launch statement for {SECOND}.')
        axis='immediate_full_owner_line_vs_missing_owner_and_later_unassigned_name';why='Only the immediately designated same-role full owner fills the frame. An explicitly unassigned later name cannot repair an actual missing owner, with raw context_gap.'
    elif n==16:
        common=f'Subject: "The physical release ring on {t} is brass." I deny this subject, namely the complete ring-material property for {t}, and I report that '
        p=ex(common+f'we withdraw the launch claim for {FOREIGN}. The complete local explanation is material and the other event owner is foreign.')
        q=ex(common+'we withdraw this source as a whole. The reported withdrawal includes every assertion, including target launch.')
        axis='quoted_property_same_subject_explanation_and_reporting_scope';why='A full same-role material explanation scopes one denial; a separate reported entire-source withdrawal remains broad.'
    elif n==17:
        common=f'Object: the release-cover width statement for {t}. We deny the object. Subject: an unowned launch statement. Owner of the subject: {FOREIGN}. We retract this subject. '
        p=ex(common+f'Item: the launch claim for {SECOND}. We withdraw the item. Each role has a complete independent object and owner.')
        q=ex(common+f'Item: the launch claim for {t}. Owner supplement for the item: {SECOND}. We withdraw the item. The Item already explicitly names the target owner.')
        axis='three_finite_roles_and_owned_item_not_rewritten_by_supplement';why='Every positive role is separately safe. A third target-owned launch Item cannot be overwritten by a foreign supplement or earlier safe roles.'
    elif n==18:
        common=f'I deny that claim, namely the complete document heading-font and title-colour specification for {t}, because we deny the launch statement for '
        p=ex(common+f'{SECOND}, while we retract the launch claim for {FOREIGN}. The two extra event clauses have full foreign owners.')
        q=ex(common+f'{t}, while we retract the launch claim for {FOREIGN}. The reason clause explicitly denies target launch despite the other safe clause.')
        axis='document_apposition_because_and_while_event_owner_checks';why='Each because/while event needs independent scope; the complete document-property apposition does not consume the reason clause.'
    elif n==19:
        common=[s('source_title','Object: launch statement'),s('heading',f'Owner supplement for the object:\n{FOREIGN}')]
        p=common+ex(f'The Object above has no preassigned owner; the designated heading supplies its sole full owner. Because we deny the launch statement for {SECOND}, we withdraw the object and deny the printed release-label colour claim for {t}.')
        q=[s('source_title',f'Object: the launch statement for {t}'),s('heading',f'Owner supplement for the object:\n{FOREIGN}')]+ex(f'The Object above already explicitly names the target owner. Because we deny the launch statement for {SECOND}, we withdraw the object and deny the printed release-label colour claim for {t}.')
        axis='owned_title_object_and_heading_supplement_with_leading_reason';why='A complete designated heading may fill a clearly unowned title Object, but cannot replace an existing target-owner declaration.'
    elif n==20:
        common=f'Subject: the printed release-key colour claim for {t}. We deny this subject. Item: an unowned launch statement. '
        p=ex(common+f'Owner of the item: {SECOND}. We withdraw this item as we report that we retract the launch claim for {FOREIGN}. Both event owners are complete and foreign.')
        q=ex(common+f'Owner of the item: either {SECOND} or {t}, without a selection. We withdraw this item as we report that we retract the launch claim for {FOREIGN}. The Item owner is explicitly unresolved.')
        axis='reported_foreign_event_does_not_resolve_separate_mixed_item';why='A fully foreign reporting clause cannot select between a mixed target/foreign owner in a separate withdrawn launch Item.'
    elif n==21:
        common=f'Subject: the document title-font-size and heading-font-style specification for {t}. We deny the subject. Object: the launch statement for {FOREIGN}. We retract the object before we withdraw '
        p=ex(common+f'the launch claim for {SECOND}. The document scope and both current event owners are explicit.')
        q=ex(common+'a launch claim whose local owner paragraph is missing from the excerpt. The last withdrawn event has an unidentified owner.')
        axis='compound_title_heading_properties_and_before_unknown_launch_gap';why='The before clause states a current explicit withdrawal, not event-order truth. Missing ownership of that last launch cannot borrow a prior safe owner; raw context_gap is supplied.'
    elif n==22:
        common=f'Item: a physical dimension assertion for {t}. Author reply: "I deny the item, namely the complete release-pedal width claim for {t}." We also report that we retract the launch statement for '
        p=ex(common+f'{FOREIGN}, and we deny the printed pedal-label colour claim for {t}. Each statement has a full property or foreign event object.')
        q=ex(common+f'{t}, and we deny the printed pedal-label colour claim for {t}. The reporting clause withdraws target launch independently of both safe properties.')
        axis='complete_item_denial_explanation_and_and_reporting_clause';why='A local quoted denial explanation is property-only; an explicit and/reporting clause retains its own complete event owner.'
    elif n==23:
        common=[s('source_title',f'Full canonical target name: {t}')]
        p=common+ex(f'Object: the physical release-panel width claim for {t}. We deny the object. Subject: the printed panel-label colour claim for {t}. We withdraw this subject and report that we retract the document heading-font claim for {t}. Name note: Clip is part of the complete target name.')
        q=common+ex(f'Object: the physical release-panel width claim for {t}. We deny the object. Subject: the printed panel-label colour claim for {t}. We withdraw this subject and report that we retract this entire source, including target launch. Name note: Clip is part of the complete target name.')
        axis='canonical_clip_multiple_properties_and_reported_whole_source';why='Complete-name notes are not gaps; a reported source-wide withdrawal is unsafe despite multiple prior property corrections.'
    elif n==24:
        p=ex(f'Subject: an unowned launch statement. Owner supplement for the subject:\n{FOREIGN}\nObject: the physical release-collar width claim for {t}. We deny the object before we withdraw the subject. The only owner supplement explicitly belongs to the launch Subject.')
        q=ex(f'Subject: an unowned launch statement. Object: the physical release-collar width claim for {t}. Owner supplement for the object:\n{FOREIGN}\nWe deny the object before we withdraw the subject. No owner supplement belongs to the launch Subject, whose owner remains unknown.')
        axis='same_role_owner_supplement_cannot_fill_different_role';why='A full supplement must explicitly name the same role as the unowned frame; a property Object supplement cannot own the unknown launch Subject.'
    elif n==25:
        p=ex(f'Subject: "{FOREIGN} was launched." We deny the subject, and we report that we retract that launch claim. We also deny the document title-colour claim for {t}. The quoted Subject is a fully foreign event.')
        q=ex(f'Subject: "The release lever on {FOREIGN} is blue." We deny the subject, and we report that we retract that launch claim. We also deny the document title-colour claim for {t}. The separate launch owner is not identified; the Subject is only lever colour.')
        axis='quoted_foreign_event_vs_property_and_reported_launch_reference';why='A complete foreign-event quote may bind a local launch reference; a property quote and another safe property cannot supply that missing launch owner.'
    elif n==26:
        common=f'Object: the launch claim for {FOREIGN}. We retract the object. Subject: the physical release-sleeve width claim for {t}. We deny this subject while '
        p=ex(common+f'the launch of {SECOND} never happened. The explicit non-event names another complete foreign product.')
        q=ex(common+f'the launch of {t} never happened. The final while clause explicitly denies the target event.')
        axis='safe_roles_and_explicit_fully_owned_non_event_clause';why='The non-event clause has its own full owner and cannot be exempted by the property Subject or earlier foreign Object.'
    elif n==27:
        p=ex(f'Item: an unowned launch statement. Owner supplement for the item:\n{FOREIGN}\nBecause we deny the physical release-key material claim for {t}, we withdraw the item although we deny the launch statement for {SECOND}. Every clause is fully scoped outside target launch.')
        q=ex(f'Item: the launch statement for {t}. Owner supplement for the item:\n{FOREIGN}\nBecause we deny the physical release-key material claim for {t}, we withdraw the item although we deny the launch statement for {SECOND}. The Item owner was already the target before the supplement.')
        axis='next_line_owner_property_reason_and_foreign_contrast';why='Safe reason/contrast clauses and a foreign next-line name cannot overwrite an existing target-owned launch Item.'
    elif n==28:
        p=ex(f'Object: an unowned launch statement. Owner supplement for the object:\n{FOREIGN}\nWe retract the object and deny the document page-count claim for {t}, while we withdraw the printed release-label font-size claim for {t}.')
        q=ex(f'Object: an unowned launch statement. Owner supplement for the object: either {FOREIGN} or {t}, with no selection. We retract the object and deny the document page-count claim for {t}, while we withdraw the printed release-label font-size claim for {t}.')
        axis='single_next_line_owner_vs_mixed_owner_among_property_clauses';why='The entire positive launch frame has one designated full foreign owner. Other safe properties do not resolve a mixed owner in the negative.'
    elif n==29:
        common=f'Subject: the physical release-knob colour claim for {t}. We deny the subject (and we retract the launch claim for {FOREIGN}, '
        p=ex(common+f'while we deny the printed knob-label font claim for {t}). All parenthesized corrections have complete foreign/property scopes.')
        q=ex(common+'while we retract every assertion in this source, including target launch). The parenthesized last correction is explicitly source-wide.')
        axis='parenthesized_multiple_corrections_and_whole_source_last_clause';why='Every parenthesized adverse clause needs independent scope. Parentheses and earlier safe corrections do not narrow a final whole-source withdrawal.'
    elif n==30:
        p=ex(f'Object: the document title-font claim for {t}. We deny the object. Subject: the launch statement for {FOREIGN}. We retract this subject and report that we deny that launch claim. Item: the launch statement for {SECOND}. We withdraw the item. Each launch reference is a complete foreign event.')
        q=ex(f'Object: the document title-font claim for {t}. We deny the object. Subject: the physical release-button width claim for {FOREIGN}. We retract this subject and report that we deny that launch claim. Item: the launch statement for {SECOND}. We withdraw the item. The reported launch claim has no identified owner; the Subject is a physical dimension.')
        axis='three_roles_and_one_reported_unknown_launch_reference';why='A safe document Object and a separate foreign launch Item cannot supply ownership for a launch reference following a property Subject.'
    for polarity,context in [('positive',p),('negative',q)]:
        headers=[v for v in context if v['kind']!='excerpt'];notes=[v for v in context if v['kind']=='excerpt'];fact=s('excerpt',f'{t} was launched.')
        signals=headers+(notes+[fact] if n in (4,6,11,12,16,18,22,26,29) else [fact]+notes)
        cid=f'li13-{len(cases)+1:03d}';risk='context_gap' if polarity=='negative' and n in (7,15,21) else 'none'
        inp={'id':cid,'policy':{'policy_id':'independent-v13-launch-policy','target_id':f'v13-fictional-target-{n:02d}','target_question':f'Was {t} launched?','entity':{'canonical_id':f'v13-fictional-target-{n:02d}','canonical_name':t,'aliases':[]},'relation':'change_or_launch','identity_requirement':'require_harness_anchor','assessment_budget':BUDGET.copy()},'candidate':{'evidence_id':cid+'-evidence','source_id':cid+'-synthetic-record','signals':signals},'proposal':{'target_binding':'exact','relation_binding':'exact'},'raw':{'identity_scope':'exact_target','relation_scope':'requested_relation','scope_risk':risk}}
        cases.append({'id':cid,'pair_id':f'v13-pair-{n:02d}','polarity':polarity,'axis':axis,'expected':{'v17_requested_relation':polarity=='positive','v30_relevant':polarity=='positive'},'rationale':why,'input':inp})
for name,value in [('cases.json',{'schema':'engine073-launch-independent-v13','author':'independent evaluator /root/engine07_realworld_independent','cases':cases}),('inputs.json',[c['input'] for c in cases])]:
    with (ROOT/name).open('x') as f:json.dump(value,f,ensure_ascii=False,indent=2);f.write('\n')
print('new_v13_cases',len(cases))
