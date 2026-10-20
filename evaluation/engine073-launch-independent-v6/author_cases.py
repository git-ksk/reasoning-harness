#!/usr/bin/env python3
"""V6 new multi-cue interference/control compositions, pre-observation only."""
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parent
NAMES=['Acornhaven Plinth','Bouldermere Kite','Cobaltbrook Chute','Deltawick Crank','Ebonridge Pylon','Frostmead Ferrule','Greenquay Lever','Hawthornreach Drum','Indigowood Coupler','Jutefield Roller','Kornelmere Rudder','Lilacshore Pedal','Mistvale Gimbal','Northfen Bracket','Ochrebay Couplet','Poplarhaven Sprocket','Quicksedge Chassis','Ravenford Stud','Sprucefield Seal','Tealmeadow Actuator','Umbershore Beam','Vesperwood Holder','Windpetal Ramp','Xenialbrook Nozzle','Yamfield Shim','Zesthaven Anvil','Auburnquay Bolt','Bluewillow Skid','Cranehaven Cleat','Duskgarden Hub']
OTHER='Pearlreed Socket';THIRD='Maplecrest Sleeve'
BUDGET={'max_model_attempts':2,'max_tokens':192,'max_elapsed_ms':90000}
def sg(kind,text):return {'kind':kind,'text':text}
def ex(text):return [sg('excerpt',text)]
cases=[]
for n,t in enumerate(NAMES,1):
    fact=sg('excerpt',f'{t} was launched.')
    if n==1:
        p=ex(f'We retract the ductility claim for {t}. We separately retract the bend-radius claim for {t}.')
        q=ex(f'We retract the ductility claim for {t}. We separately retract the launch claim for {t}.')
        axis='two_independent_retractions_same_owner';why='Two explicit property objects are harmless; replacing one with the same-target launch object requires withholding despite the other property cue.'
    elif n==2:
        p=ex(f'We withdraw both the thread-pitch claim about {t} and the friction-coefficient claim about {t}. Both withdrawn objects are named in this sentence.')
        q=ex(f'We withdraw both the thread-pitch claim about {t} and the launch claim about {t}. Both withdrawn objects are named in this sentence.')
        axis='both_named_objects_one_withdrawal';why='Both objects have explicit property types and owner names; no opaque collection reference is required. A listed launch object cannot be exempted by the other property.'
    elif n==3:
        p=[sg('excerpt',f'The coolant-flow assertion for {t} is retracted.'),sg('excerpt',f'Current author statement: I deny the bearing-clearance assertion for {t}.')]
        q=[sg('excerpt',f'The coolant-flow assertion for {t} is retracted.'),sg('excerpt',f'Current author statement: I deny the launch assertion for {t}.')]
        axis='separate_excerpt_correction_objects';why='The second separate excerpt explicitly denies a different property or the target launch; the first cannot mask it.'
    elif n==4:
        common=[sg('source_title',f'We retract the humidity-tolerance claim for {t}')]
        p=common+[sg('heading',f'We deny the dielectric-strength claim for {t}')]
        q=common+[sg('heading',f'We deny the launch claim for {t}')]
        axis='title_property_and_heading_object';why='Two header cues each explicitly name an object; a property-scoped title does not erase a launch-scoped heading.'
    elif n==5:
        p=ex(f'The corrosion-class claim for {t} is withdrawn. Additional current correction: the impact-endurance claim for {t} is withdrawn.')
        q=ex(f'The corrosion-class claim for {t} is withdrawn. Additional current correction: the launch claim for {t} is withdrawn.')
        axis='additional_correction_after_property';why='The additional correction is independent and explicitly scoped; any launch withdrawal must override an irrelevant first cue.'
    elif n==6:
        p=ex(f'We withdraw the sentence "{t} has milled slots." We also withdraw the sentence "{t} has anodized rails."')
        q=ex(f'We withdraw the sentence "{t} has milled slots." We also withdraw the sentence "{t} was launched."')
        axis='two_complete_quoted_withdrawn_sentences';why='Complete quoted objects make both property scopes, or the included launch scope, explicit without numbered references.'
    elif n==7:
        p=ex(f'We retract the launch claim for {OTHER}. We also retract the launch claim for {THIRD}. The record about {t} appears on this page.')
        q=ex(f'We retract the launch claim for {OTHER}. We also retract the launch claim for {t}. The record about {THIRD} appears on this page.')
        axis='two_explicit_launch_owners';why='The positive retracts only two fully named other owners. The negative explicitly includes the canonical target among the retracted owners.'
    elif n==8:
        p=ex(f'We deny the lead-content claim for {t}. Independently, we withdraw the launch claim for {OTHER}.')
        q=ex(f'We deny the launch claim for {t}. Independently, we withdraw the launch claim for {OTHER}.')
        axis='property_or_target_denial_with_other_launch';why='The other-owner launch withdrawal is irrelevant to the target but cannot cancel a separate target launch denial.'
    elif n==9:
        common=[sg('source_title',f'We retract the launch claim for {OTHER}')]
        p=common+ex(f'Additional author decision: I retract the thermal-conductivity claim for {t}.')
        q=common+ex(f'Additional author decision: I retract the launch claim for {t}.')
        axis='other_owner_title_and_additional_target_object';why='An explicit other-owner title cannot globally exempt a target launch retraction in the body.'
    elif n==10:
        p=ex(f'We deny the shielding-thickness claim for {t}. At the end of this update, I retract the load-capacity claim for {t}.')
        q=ex(f'We deny the shielding-thickness claim for {t}. At the end of this update, I retract the launch claim for {t}.')
        axis='leading_property_and_trailing_author_object';why='The trailing first-person action has its own full object, either another property or target launch.'
    elif n==11:
        p=ex(f'We deny the surface-hardness claim for {t}. The lubrication-cycle claim for {t} is separately retracted.')
        q=ex(f'We deny the launch claim for {t}. The lubrication-cycle claim for {t} is separately retracted.')
        axis='leading_target_or_property_with_trailing_property';why='A later unrelated property retraction cannot erase the leading same-target launch denial.'
    elif n==12:
        common=[sg('source_title','Consolidated independent author corrections'),sg('excerpt',f'Material note: the flexural-strength claim for {t} is withdrawn.')]
        p=common+ex(f'Event note: We deny the launch claim for {OTHER}.')
        q=common+ex(f'Event note: We deny the launch claim for {t}.')
        axis='material_note_and_event_note';why='The event-note denial explicitly identifies its owner independently of the material-note scope.'
    elif n==13:
        p=ex(f'Current corrections:\n- We retract the groove-depth claim for {t}.\n- We retract the insulation-class claim for {t}.')
        q=ex(f'Current corrections:\n- We retract the groove-depth claim for {t}.\n- We retract the launch claim for {t}.')
        axis='unnumbered_bullets_with_full_objects';why='Each bullet is a complete explicit statement; no trusted list AST or numeric resolution is assumed.'
    elif n==14:
        common=[sg('source_title',f'Author correction sheet discussing {t} and {OTHER}'),sg('excerpt',f'We withdraw the launch claim for {OTHER}.')]
        p=common+ex(f'We deny the seal-pressure claim for {t}.')
        q=common+ex(f'We deny the launch claim for {t}.')
        axis='other_owner_excerpt_then_target_object';why='A second explicit target object must be considered separately from the other-owner first excerpt.'
    elif n==15:
        p=ex(f'{t}\'s porosity claim is withdrawn. {t}\'s solder-composition claim is also withdrawn.')
        q=ex(f'{t}\'s porosity claim is withdrawn. {t}\'s launch claim is also withdrawn.')
        axis='two_possessive_claim_subjects';why='Both possessive claim subjects directly identify the owner and property/launch type.'
    elif n==16:
        p=ex(f'The author denies the tensile-strength claim for {t}; independently, the author retracts the launch claim for {OTHER}.')
        q=ex(f'The author denies the tensile-strength claim for {t}; independently, the author retracts the launch claim for {t}.')
        axis='independent_semicolon_claims';why='A semicolon separates explicit objects; a non-launch first object cannot waive a target launch second object.'
    elif n==17:
        p=ex(f'Most of this note concerns cable properties. I withdraw the fuse-rating claim for {t}. This is the author\'s current decision.')
        q=ex(f'Most of this note concerns cable properties. I withdraw the launch claim for {t}. This is the author\'s current decision.')
        axis='general_topic_then_specific_current_object';why='The specific operative object controls; a general non-exclusive topic cue does not override an explicit launch withdrawal.'
    elif n==18:
        p=ex(f'I retract the drainage-groove claim about {t}. I withdraw the separate launch statement about {OTHER}.')
        q=ex(f'I retract the drainage-groove claim about {t}. I withdraw the separate launch statement about {t}.')
        axis='personal_property_then_separate_launch_owner';why='The second launch statement explicitly names its owner and is independent of the preceding property.'
    elif n==19:
        body=ex(f'The granularity claim for {t} is withdrawn. The surface-roughness claim for {t} is retracted.')
        p=[sg('source_title',f'We retract the compressibility claim for {t}')]+body
        q=[sg('source_title',f'We retract the launch claim for {t}')]+body
        axis='critical_title_with_property_only_body';why='Property-only body corrections do not erase a title that explicitly retracts the same target launch.'
    elif n==20:
        common=[sg('heading','Two current publisher responses quoted in full')]
        p=common+ex(f'Publisher response: "We deny the shear-strength claim for {t}." Separate publisher response: "We withdraw the resonance-frequency claim for {t}."')
        q=common+ex(f'Publisher response: "We deny the shear-strength claim for {t}." Separate publisher response: "We withdraw the launch claim for {t}."')
        axis='two_current_quoted_correction_objects';why='Each quote is current and has a full named object; the second launch withdrawal is not neutralized by the first property.'
    elif n==21:
        p=ex(f'We withdraw the thermal-expansion claim for {t}. The thermal trial for {t} did not take place.')
        q=ex(f'We withdraw the thermal-expansion claim for {t}. The launch of {t} did not take place.')
        axis='property_withdrawal_then_event_nonexistence';why='A distinct trial non-event preserves launch; explicit launch nonexistence requires withholding despite the first property cue.'
    elif n==22:
        p=ex(f'We withdraw the launch claim for {OTHER}. No actual durability test of {t} occurred.')
        q=ex(f'We withdraw the launch claim for {OTHER}. No actual launch of {t} occurred.')
        axis='other_owner_launch_then_named_non_event';why='The second non-event explicitly owns either a different activity or the tested target launch; the first other-owner cue cannot exempt it.'
    elif n==23:
        p=[sg('excerpt',f'{t} was not made with silver. This is the publisher\'s current material correction.'),sg('excerpt',f'We retract the launch claim for {OTHER}.')]
        q=[sg('excerpt',f'{t} was not launched. This is the publisher\'s current launch correction.'),sg('excerpt',f'We retract the launch claim for {OTHER}.')]
        axis='direct_target_denial_and_other_owner_retraction';why='The target predicate is a material property or existing launch predicate; the following other-owner retraction cannot overwrite it.'
    elif n==24:
        common=ex('An archived facsimile file has been withdrawn. This administrative note concerns the file copy alone.')
        p=common+ex(f'The service benchmark for {t} never happened.')
        q=common+ex(f'The launch of {t} never happened.')
        axis='artifact_withdrawal_then_explicit_non_event';why='An administrative file action is neutral; the separate second excerpt must still block a target launch non-event.'
    elif n==25:
        common=ex(f'We withdraw the width claim for {t}.')
        p=common+ex(f'We separately retract the height claim for {t}.')
        q=common+ex('We separately retract a launch statement. The owner of that launch statement is omitted from the notice.')
        axis='property_then_unidentified_launch_statement';why='A fully scoped property correction does not resolve or waive an expressly unidentified launch retraction.'
    elif n==26:
        common=ex(f'We deny the launch claim for {OTHER}.')
        p=common+ex(f'We withdraw the launch claim for {THIRD}. Both owners are explicitly named in the operative clauses.')
        q=common+ex('We withdraw another launch claim. The notice does not identify which product owns this additional withdrawn claim.')
        axis='named_other_owner_then_additional_unknown_owner';why='Two fully named other owners preserve the target; the second negative owner is explicitly unresolved and cannot inherit the first owner.'
    elif n==27:
        common=[sg('heading',f'We withdraw the paint-colour claim for {t}')]
        p=common+ex(f'We retract the mass-distribution claim for {t}.')
        q=common+ex('We retract a launch assertion. The product referent for this launch assertion is missing.')
        axis='property_heading_then_missing_launch_referent';why='An explicit property header does not bind the owner of an additional missing-reference launch retraction.'
    elif n==28:
        common=[sg('source_title',f'Joint property review of {t} and {OTHER}')]
        p=common+ex(f'We withdraw the spindle-diameter claim for {t}. We retract the paint-viscosity claim for {OTHER}.')
        q=common+ex(f'We withdraw the spindle-diameter claim for {t}. We retract one product\'s launch claim; the review does not say which product.')
        axis='joint_property_review_with_unknown_launch_addendum';why='All positive objects and owners are explicit. The launch addendum has an unresolved owner despite a property-scoped review title.'
    elif n==29:
        common=ex(f'This record covers {t}, {OTHER}, and {THIRD}. {OTHER} was launched. {THIRD} was launched.')
        p=common+ex(f'We withdraw the launch claim for {OTHER}. The withdrawn owner is explicitly {OTHER}.')
        q=common+ex('We withdraw a launch claim in this record. The notice does not identify which of the three products owns the withdrawn claim.')
        axis='three_product_record_named_or_unknown_withdrawal';why='The positive identifies another owner directly; the negative explicitly leaves ownership among all three record products unresolved.'
    elif n==30:
        common=[sg('heading','Current two-part author notice'),sg('excerpt','The page margins were widened. The indexer note concerns document layout.')]
        p=common+ex(f'I withdraw the engraving-alignment claim for {t}.')
        q=common+ex('I withdraw a launch assertion. The corrected launch claim and its owner are not identified in this notice.')
        axis='neutral_part_then_property_or_unresolved_launch';why='Neutral layout text cannot scope away a distinct current first-person launch withdrawal whose owner is expressly missing.'
    for polarity,context in [('positive',p),('negative',q)]:
        headers=[x for x in context if x['kind']!='excerpt'];notes=[x for x in context if x['kind']=='excerpt']
        signals=headers+(notes+[fact] if n in (11,17,23) else [fact]+notes)
        cid=f'li6-{len(cases)+1:03d}'
        inp={'id':cid,'policy':{'policy_id':'independent-v6-launch-policy','target_id':f'v6-fictional-target-{n:02d}','target_question':f'Was {t} launched?','entity':{'canonical_id':f'v6-fictional-target-{n:02d}','canonical_name':t,'aliases':[]},'relation':'change_or_launch','identity_requirement':'require_harness_anchor','assessment_budget':BUDGET.copy()},'candidate':{'evidence_id':cid+'-evidence','source_id':cid+'-synthetic-record','signals':signals},'proposal':{'target_binding':'exact','relation_binding':'exact'},'raw':{'identity_scope':'exact_target','relation_scope':'requested_relation','scope_risk':'none'}}
        cases.append({'id':cid,'pair_id':f'v6-pair-{n:02d}','polarity':polarity,'axis':axis,'expected':{'v17_requested_relation':polarity=='positive','v30_relevant':polarity=='positive'},'rationale':why,'input':inp})
for name,value in [('cases.json',{'schema':'engine073-launch-independent-v6','author':'independent evaluator /root/engine07_realworld_independent','cases':cases}),('inputs.json',[c['input'] for c in cases])]:
    with (ROOT/name).open('x') as f:json.dump(value,f,ensure_ascii=False,indent=2);f.write('\n')
print('new_multi_cue_cases',len(cases))
