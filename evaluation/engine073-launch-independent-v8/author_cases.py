#!/usr/bin/env python3
"""V8 new local-referent/owner/component control compositions, before semantics."""
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parent
NAMES=['Arborfall Prism','Bramblefen Dial','Celandine Dock','Dapplemere Coil','Elmwharf Cradle','Fernvale Spline','Gullhaven Marker','Honeysedge Wheel','Iriswharf Sleeve','Jadefield Link','Knotwood Frame','Laurelreach Roller','Mossquay Valve','Nuthaven Clip','Orchidford Gauge','Pearlmere Hub','Quincefield Bearing','Rowanshore Capsule','Silverfen Lever','Thornridge Scanner','Ulmushaven Pivot','Vinewood Coupler','Wheathaven Console','Xylobay Ring','Yarrowmere Bracket','Zebrafen Pedal','Ashmeadow Seal','Baythorn Gimbal','Cypressquay Plug','Dewfield Actuator']
OTHER='Larkstone Gasket'
BUDGET={'max_model_attempts':2,'max_tokens':192,'max_elapsed_ms':90000}
def sg(kind,text):return {'kind':kind,'text':text}
def ex(text):return [sg('excerpt',text)]
cases=[]
for n,t in enumerate(NAMES,1):
    fact=sg('excerpt',f'{t} was launched.')
    if n==1:
        p=ex(f'Local correction scope: the launch claim for {OTHER}. I retract this claim. The retracted claim belongs to {OTHER}.')
        q=ex(f'Local correction scope: the launch claim for {t}. I retract this claim. The retracted claim belongs to {t}.')
        axis='local_scope_this_claim_full_owner';why='Immediate scope and final owner both uniquely identify this claim as another product launch or the target launch.'
    elif n==2:
        p=ex(f'The corrected assertion is: "{t} has a detachable cooling fin." We withdraw that assertion. Its subject is the cooling-fin feature.')
        q=ex(f'The corrected assertion is: "{t} was launched." We withdraw that assertion. Its subject is the launch event.')
        axis='quoted_assertion_then_local_withdrawal';why='That assertion points to one complete immediately preceding quotation, a property or the existing launch proposition.'
    elif n==3:
        p=ex(f'The launch-indicator brightness claim for {t} is the topic of this note. That brightness claim is retracted by its author.')
        q=ex(f'The launch claim for {t} is the topic of this note. That launch claim is retracted by its author.')
        axis='local_property_head_repeated_in_reference';why='The reference repeats the property head or launch head, so a component brightness claim is not an event correction.'
    elif n==4:
        p=ex(f'This paragraph quotes one claim: "{OTHER} was launched." I retract the claim expressed by that quote. The retracted assertion is about {OTHER}.')
        q=ex(f'This paragraph quotes one claim: "{t} was launched." I retract the claim expressed by that quote. The retracted assertion is about {t}.')
        axis='single_local_quote_claim_with_final_owner';why='The complete quote and final owner uniquely identify the retracted assertion; removing a quotation artifact is not the tested action.'
    elif n==5:
        p=[sg('source_title',f'Local paragraph owner: {OTHER}'),sg('excerpt',f'The launch assertion for {OTHER} is the subject of this paragraph. I withdraw this assertion.')]
        q=[sg('source_title',f'Local paragraph owner: {t}'),sg('excerpt',f'The launch assertion for {t} is the subject of this paragraph. I withdraw this assertion.')]
        axis='local_title_and_adjacent_assertion_owner';why='Both title and immediate body supply the full owner; this assertion has a unique local object, not an arbitrary reference AST.'
    elif n==6:
        p=ex(f'I retract the following claim: "{t} has a titanium launch-spring casing." The following claim is a casing-material claim.')
        q=ex(f'I retract the following claim: "{t} was launched." The following claim is a launch-event claim.')
        axis='forward_local_content_after_colon';why='The following claim is provided in full after a colon and explicitly typed as casing material or launch.'
    elif n==7:
        p=ex(f'Correction regarding {OTHER}: the launch statement for {OTHER} is incorrect. I retract that statement, the launch statement for {OTHER}.')
        q=ex(f'Correction regarding {t}: the launch statement for {t} is incorrect. I retract that statement, the launch statement for {t}.')
        axis='local_statement_with_explicit_apposition';why='The final apposition repeats the full object and owner; no unspecified statement mapping is demanded.'
    elif n==8:
        p=ex(f'That claim—specifically the launch claim for {OTHER}—is withdrawn by the publisher. The named product is {OTHER}.')
        q=ex(f'That claim—specifically the launch claim for {t}—is withdrawn by the publisher. The named product is {t}.')
        axis='appositive_owner_inside_local_reference';why='The same sentence explicitly defines that claim and its complete owner inside the apposition.'
    elif n==9:
        p=ex(f'The launch-pedal stiffness statement for {t} is the statement being corrected. We withdraw that stiffness statement about the physical pedal.')
        q=ex(f'The launch statement for {t} is the statement being corrected. We withdraw that launch statement about the product event.')
        axis='component_stiffness_local_statement';why='The statement head and reference name a physical stiffness property or the historical event, not the truth of a new property.'
    elif n==10:
        p=ex(f'The release-trigger force claim for {t} is under correction. I retract that force claim. The trigger is a mechanical part.')
        q=ex(f'The launch assertion for {t} is under correction. I retract that launch assertion. The assertion concerns the product launch event.')
        axis='component_force_then_typed_local_retraction';why='That force claim is explicitly mechanical and non-event; that launch assertion is explicitly the target event.'
    elif n==11:
        p=[sg('source_title','Amendment to the physical launch-selector label'),sg('excerpt',f'Quoted statement: "{t} has an engraved launch-selector label." That label statement is withdrawn.')]
        q=[sg('source_title','Amendment to the product launch assertion'),sg('excerpt',f'Quoted statement: "{t} was launched." That launch statement is withdrawn.')]
        axis='selector_label_quote_local_withdrawal';why='The positive local quotation concerns an engraved physical label; the negative concerns the existing event proposition.'
    elif n==12:
        p=ex(f'We deny this property statement: "{t}\'s launch socket is nickel plated." The object of denial is socket plating.')
        q=ex(f'We deny this product-history statement: "{t} was launched." The object of denial is launch.')
        axis='this_statement_with_explicit_following_object';why='The full object immediately follows this statement, and its property/event type is repeated.'
    elif n==13:
        p=ex(f'The inspection of the physical launch lock on {t} never took place. The publisher issues this as a correction of the lock-inspection account.')
        q=ex(f'The launch of {t} never took place. The publisher issues this as a correction of the product-launch account.')
        axis='component_inspection_vs_product_non_event';why='The absent activity is explicitly inspection of a hardware lock or product launch; no preparation predicate is introduced.'
    elif n==14:
        p=ex(f'We withdraw the statement about the launch cradle\'s fastening torque on {t}. The statement concerns mechanical fastening torque.')
        q=ex(f'We withdraw the statement about the launch of {t}. The statement concerns the product launch event.')
        axis='nested_component_possessive_object';why='The physical fastening-torque object is explicitly scoped, unlike the target launch-event object.'
    elif n==15:
        p=ex(f'The statement that {t} was launched is unchanged. I retract the release-knob friction claim for {t}; that claim concerns a mechanical knob.')
        q=ex(f'The statement that {t} was launched is retracted. The release-knob friction claim for {t} is unchanged; that claim concerns a mechanical knob.')
        axis='reciprocal_unchanged_and_retracted_objects';why='The same two objects occur with opposite statuses; only retracting the launch object blocks the factual launch.'
    elif n==16:
        p=ex(f'I withdraw only the launch-port dimension claim about {t}. The withdrawn claim concerns the size of a physical aperture, not the occurrence of launch.')
        q=ex(f'I withdraw the launch-occurrence claim about {t}. The withdrawn claim concerns the occurrence of launch, not the size of a physical aperture.')
        axis='dimension_vs_occurrence_explicit_exclusion';why='An explicit physical-aperture dimension is non-launch; an explicit occurrence claim is launch.'
    elif n==17:
        p=ex(f'The active retraction applies to {OTHER}\'s launch statement. The owner of the retracted statement is {OTHER}, a different product from {t}.')
        q=ex(f'The active retraction applies to {t}\'s launch statement. The owner of the retracted statement is {t}, a different product from {OTHER}.')
        axis='active_retraction_repeated_complete_owner';why='Two clauses identify the complete retracted owner and distinguish products explicitly.'
    elif n==18:
        p=ex(f'We retract the launch claim for {OTHER}. The launch-button position claim for {t} is also withdrawn; that second claim is a physical-button-position specification.')
        q=ex(f'We retract the launch claim for {OTHER}. The launch claim for {t} is also withdrawn; that second claim is the target launch statement.')
        axis='other_owner_then_local_second_component_or_event';why='The second object is explicitly typed; another-owner correction cannot mask a target launch withdrawal.'
    elif n==19:
        p=ex(f'Current author reply: "We retract {OTHER}\'s assertion." The corrected assertion in this reply is {OTHER}\'s launch claim.')
        q=ex(f'Current author reply: "We retract {t}\'s assertion." The corrected assertion in this reply is {t}\'s launch claim.')
        axis='quoted_owner_with_following_claim_type';why='The quoted owner and immediately following launch type jointly define one local assertion.'
    elif n==20:
        p=[sg('source_title',f'Retraction about the launch claim for {OTHER}'),sg('excerpt',f'I withdraw this statement about {OTHER}. The launch statement for {OTHER} is the withdrawn statement.')]
        q=[sg('source_title',f'Retraction about the launch claim for {t}'),sg('excerpt',f'I withdraw this statement about {t}. The launch statement for {t} is the withdrawn statement.')]
        axis='title_scoped_this_statement_and_full_restating';why='The header and complete restatement supply the owner/type rather than leaving this statement unbound.'
    elif n==21:
        p=ex(f'The event under discussion is the launch of {OTHER}. That event never occurred. This is the publisher\'s current event correction.')
        q=ex(f'The event under discussion is the launch of {t}. That event never occurred. This is the publisher\'s current event correction.')
        axis='unique_local_event_then_nonexistence';why='That event has one immediate explicit event/owner; another-product non-event does not deny target launch.'
    elif n==22:
        p=ex(f'{OTHER} was not launched. What I deny is the affirmative launch claim addressed by this correction about {OTHER}; {t} is a different product.')
        q=ex(f'{t} was not launched. What I deny is the affirmative launch claim addressed by this correction about {t}; {OTHER} is a different product.')
        axis='direct_denial_with_complete_local_owner';why='The direct was-not-launched statement and owner clarification agree; no alias or new affirmative relation is required.'
    elif n==23:
        p=[sg('heading','Specification appendix layout note'),sg('excerpt','The appendix column widths were aligned. This note concerns document layout.')]
        q=[sg('heading','Local launch correction'),sg('excerpt','I retract this launch claim. The local note supplies no owner statement or product name for the claim.')]
        axis='neutral_layout_vs_ownerless_this_claim';why='Layout is neutral; this launch claim is expressly unbound and therefore a withholding negative.'
    elif n==24:
        p=ex(f'The release-selector legend on {t} is the item being corrected. We withdraw that legend-font claim; it is a printed component legend.')
        q=ex('This launch claim is withdrawn. The product referent was lost from the margin note, and this note does not restore it.')
        axis='unique_legend_item_vs_lost_local_referent';why='The positive explicitly identifies a printed component property; the negative explicitly loses the launch owner.'
    elif n==25:
        p=ex(f'The one claim quoted by this local notice is "{OTHER} was launched." I retract that quoted launch claim about {OTHER}.')
        q=ex(f'The two launch claims discussed here concern {t} and {OTHER}. I retract that claim. The note does not record which of those launch claims this reference selects.')
        axis='one_explicit_other_claim_vs_two_unselected_claims';why='The positive uniquely supplies another owner; the negative collection/reference is explicitly unresolved and is not forced positive.'
    elif n==26:
        p=ex(f'The corrected specification is the drive-belt width claim for {t}. I withdraw that width specification. This is the only specification addressed by this correction.')
        q=ex(f'The drive-belt width claim for {t} is corrected separately. I withdraw that launch statement as well. The product identity of the launch statement is not recorded.')
        axis='property_reference_vs_additional_unresolved_launch';why='A locally explicit width object is harmless; the additional launch reference is explicitly owner-missing despite a clear property cue.'
    elif n==27:
        p=[sg('source_title',f'Author reply about {OTHER}'),sg('excerpt',f'Author reply: "I deny its launch claim." The reply explicitly identifies its launch-claim owner as {OTHER}.')]
        q=[sg('source_title','Author reply without an identified product'),sg('excerpt','Author reply: "I deny its launch claim." The reply never identifies which product the possessive its refers to.')]
        axis='possessive_reply_resolved_or_unresolved_explicitly';why='The positive explicitly resolves the possessive to another complete owner; the negative explicitly leaves it unresolved.'
    elif n==28:
        p=ex('The local caption was moved to an appendix. This change concerns page layout and caption placement.')
        q=ex('The launch correction in this caption withdraws a launch claim. The caption omits the identity of the product whose claim is withdrawn.')
        axis='caption_move_vs_unidentified_local_launch';why='Moving caption placement is neutral; an owner-missing launch withdrawal cannot authorize the target relation.'
    elif n==29:
        p=ex(f'The corrected item is the seal-width claim about {t}. The retracted item is this seal-width claim, and its subject is physical seal width.')
        q=ex(f'The corrected items discussed here are launch claims for {t} and {OTHER}. One of them is retracted. The correction does not identify which product owns the retracted claim.')
        axis='unique_typed_item_vs_unassigned_plural_launch';why='The positive repeats one property object; the negative explicitly declines to assign a retracted launch item among products.'
    elif n==30:
        p=ex(f'I deny this statement about {t}: "The release handle is made of aluminium." The statement concerns handle material.')
        q=ex('Current orphan retraction: I withdraw that launch assertion. The quoted source fragment omits the product name, and the local note does not identify it.')
        axis='following_component_quote_vs_orphan_launch_assertion';why='The full positive quote is a material claim about a component; the negative local launch reference has no identified owner.'
    for polarity,context in [('positive',p),('negative',q)]:
        headers=[x for x in context if x['kind']!='excerpt'];notes=[x for x in context if x['kind']=='excerpt']
        signals=headers+(notes+[fact] if n in (6,13,21) else [fact]+notes)
        cid=f'li8-{len(cases)+1:03d}'
        inp={'id':cid,'policy':{'policy_id':'independent-v8-launch-policy','target_id':f'v8-fictional-target-{n:02d}','target_question':f'Was {t} launched?','entity':{'canonical_id':f'v8-fictional-target-{n:02d}','canonical_name':t,'aliases':[]},'relation':'change_or_launch','identity_requirement':'require_harness_anchor','assessment_budget':BUDGET.copy()},'candidate':{'evidence_id':cid+'-evidence','source_id':cid+'-synthetic-record','signals':signals},'proposal':{'target_binding':'exact','relation_binding':'exact'},'raw':{'identity_scope':'exact_target','relation_scope':'requested_relation','scope_risk':'none'}}
        cases.append({'id':cid,'pair_id':f'v8-pair-{n:02d}','polarity':polarity,'axis':axis,'expected':{'v17_requested_relation':polarity=='positive','v30_relevant':polarity=='positive'},'rationale':why,'input':inp})
for name,value in [('cases.json',{'schema':'engine073-launch-independent-v8','author':'independent evaluator /root/engine07_realworld_independent','cases':cases}),('inputs.json',[c['input'] for c in cases])]:
    with (ROOT/name).open('x') as f:json.dump(value,f,ensure_ascii=False,indent=2);f.write('\n')
print('new_v8_cases',len(cases))
