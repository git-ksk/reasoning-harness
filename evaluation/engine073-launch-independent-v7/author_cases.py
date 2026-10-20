#!/usr/bin/env python3
"""V7 new complete-name ownership and event-word property scope controls."""
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parent
PAIRS=[('Alderhelm Lens','Alderhelm Rail'),('Birchhaven Valve','Birchhaven Coupler'),('Citrusmoor Plate','Citrusmoor Shim'),('Dovewick Relay','Dovewick Sensor'),('Elderfield Hub','Elderfield Knob'),('Flintmarsh Dial','Flintmarsh Nozzle'),('Goldhaven Loom','Goldhaven Capsule'),('Heatherquay Ring','Heatherquay Lever'),('Ivyshore Gauge','Ivyshore Coffer'),('Jadewood Crown','Jadewood Cradle'),('Kestrelbrook Panel','Kestrelbrook Cap'),('Lindenbay Spindle','Lindenbay Bracket'),('Meadowrift Frame','Meadowrift Key'),('Nutmegwood Console','Nutmegwood Tuner'),('Oakenford Dock','Oakenford Slider'),('Ploverhaven Switch','Ploverhaven Pointer'),('Quillmeadow Terminal','Quillmeadow Filter'),('Reedhaven Board','Reedhaven Meter'),('Sedgewharf Reader','Sedgewharf Drum'),('Tawnybrook Recorder','Tawnybrook Scanner'),('Umberquay Seal','Umberquay Handle'),('Violetmere Rotor','Violetmere Valve'),('Windgrove Chassis','Windgrove Panel'),('Xanthicfield Bearing','Xanthicfield Rack'),('Yewhaven Clip','Yewhaven Base'),('Zinniaquay Sprocket','Zinniaquay Harness'),('Amberlake Flange','Amberlake Socket'),('Brookhaven Ferrule','Brookhaven Spool'),('Copperford Pedal','Copperford Rail'),('Dawnmeadow Actuator','Dawnmeadow Loom')]
BUDGET={'max_model_attempts':2,'max_tokens':192,'max_elapsed_ms':90000}
def sg(kind,text):return {'kind':kind,'text':text}
def ex(text):return [sg('excerpt',text)]
cases=[]
for n,(t,o) in enumerate(PAIRS,1):
    fact=sg('excerpt',f'{t} was launched.')
    distinct=f'{t} and {o} are different products. Their shared first word is not an alias declaration.'
    if n==1:
        p=ex(distinct+f' We retract the launch claim for {o}. The retraction concerns {o} alone.')
        q=ex(distinct+f' We retract the launch claim for {t}. The retraction concerns {t} alone.')
        axis='shared_first_token_full_owner_retraction';why='Complete product names are expressly distinct; the operative retraction names a different product or the exact target.'
    elif n==2:
        p=[sg('source_title',f'Launch denial for {o}'),sg('heading',distinct),sg('excerpt',f'We deny that {o} was launched. This denial names the full product {o}.')]
        q=[sg('source_title',f'Launch denial for {t}'),sg('heading',distinct),sg('excerpt',f'We deny that {t} was launched. This denial names the full product {t}.')]
        axis='similar_names_title_heading_full_denial';why='The title and full operative clause identify the denied owner; a shared name token does not replace complete ownership.'
    elif n==3:
        p=[sg('heading','Current publisher words'),sg('excerpt',distinct+f' Publisher of {o}: "We withdraw the launch claim for {o}."')]
        q=[sg('heading','Current publisher words'),sg('excerpt',distinct+f' Publisher of {t}: "We withdraw the launch claim for {t}."')]
        axis='neighbor_name_current_quote_owner';why='Publisher attribution and the quote itself both give the complete named owner of the launch withdrawal.'
    elif n==4:
        p=ex(distinct+f' We retract the launch statement about {o}, which was printed beside the record for {t}. The withdrawn statement is about {o}.')
        q=ex(distinct+f' We retract the launch statement about {t}, which was printed beside the record for {o}. The withdrawn statement is about {t}.')
        axis='similar_names_neighbor_record_object';why='The direct object and final sentence identify one owner explicitly despite the neighboring record and shared first token.'
    elif n==5:
        p=[sg('source_title',f'Current correction accompanying the {t} record'),sg('excerpt',distinct+f' This launch denial belongs to {o}: we deny the launch assertion for {o}.')]
        q=[sg('source_title',f'Current correction accompanying the {t} record'),sg('excerpt',distinct+f' This launch denial belongs to {t}: we deny the launch assertion for {t}.')]
        axis='record_title_vs_explicit_denial_owner';why='Being attached to the target record does not change an explicitly different owner; the negative owns the target directly.'
    elif n==6:
        p=ex(distinct+f' {o}\'s launch assertion is withdrawn. Separately, the shaft-diameter claim for {t} is withdrawn.')
        q=ex(distinct+f' {t}\'s launch assertion is withdrawn. Separately, the shaft-diameter claim for {t} is withdrawn.')
        axis='similar_possessive_owner_with_property_cue';why='The positive has only another owner\'s launch correction plus a target property correction; the negative includes target launch.'
    elif n==7:
        p=ex(distinct+f' We deny that {o} was launched. {t} is the separate product described by the intact launch entry.')
        q=ex(distinct+f' We deny that {t} was launched. {o} is the separate product described by another launch entry.')
        axis='full_named_denial_and_separate_entry';why='The operative that-clause has an explicit complete product name; the entry description is additional ownership context.'
    elif n==8:
        p=ex(distinct+f' Scope of correction: {o} only. I retract the launch claim for {o}. This correction does not name {t} as its object.')
        q=ex(distinct+f' Scope of correction: {t} only. I retract the launch claim for {t}. This correction does not name {o} as its object.')
        axis='full_scope_owner_and_excluded_neighbor';why='Both the scope and retraction directly name the owner; an exclusion reinforces the complete-name distinction.'
    elif n==9:
        p=ex(distinct+f' The launch of {o} did not happen. The publisher issues this as a correction of the {o} launch claim.')
        q=ex(distinct+f' The launch of {t} did not happen. The publisher issues this as a correction of the {t} launch claim.')
        axis='shared_token_named_non_event';why='A launch non-event is expressly assigned to another fully named product or the canonical target.'
    elif n==10:
        p=[sg('heading','Complete-name event correction'),sg('excerpt',distinct+f' No launch of {o} occurred. The omitted event is the launch of {o}.')]
        q=[sg('heading','Complete-name event correction'),sg('excerpt',distinct+f' No launch of {t} occurred. The omitted event is the launch of {t}.')]
        axis='same_prefix_full_name_no_launch';why='Repeated complete owner names distinguish the non-event; no aliases, family hierarchy or fuzzy identity is inferred.'
    elif n==11:
        p=ex(f'We deny the physical launch-button specification for {t}. This denial concerns the hardware button, not the product-history event.')
        q=ex(f'We deny the launch claim for {t}. This denial concerns the product-history event, not a hardware button.')
        axis='launch_button_specification_vs_event';why='Launch modifies an explicit hardware-button specification in the positive; the negative denies historical product launch.'
    elif n==12:
        p=[sg('heading','Front-panel correction'),sg('excerpt',f'We retract the claim that {t} has a launch icon printed on its front panel. The retracted object is the printed icon claim.')]
        q=[sg('heading','Product-history correction'),sg('excerpt',f'We retract the claim that {t} was launched. The retracted object is the product launch claim.')]
        axis='printed_launch_icon_claim';why='A printed icon is a physical/display property with an explicit object, not the occurrence of launch.'
    elif n==13:
        p=ex(f'The release-lever alloy claim for {t} is withdrawn. This withdrawal concerns the alloy in a mechanical lever.')
        q=ex(f'The launch claim for {t} is withdrawn. This withdrawal concerns the product launch event.')
        axis='release_lever_alloy_vs_launch';why='Release is part of a component name and an alloy specification in the positive; event ownership is explicit in the negative.'
    elif n==14:
        p=ex(f'We withdraw the launch-switch voltage claim for {t}. The claim concerns an electrical rating of the physical switch.')
        q=ex(f'We withdraw the launch statement for {t}. The statement concerns the product launch event.')
        axis='launch_switch_voltage_scope';why='The explicit electrical-rating object is outside the historical launch relation despite its launch word.'
    elif n==15:
        p=ex(f'I deny that {t}\'s release latch is brass. This is a denial about the material of a mechanical latch.')
        q=ex(f'I deny that {t} was launched. This is a denial about the historical launch event.')
        axis='release_latch_material_that_clause';why='The positive clause denies material of a named component; only the negative denies the existing was-launched proposition.'
    elif n==16:
        p=[sg('source_title','Correction to the launch-key legend'),sg('excerpt',f'We retract the font-size claim for the launch-key legend on {t}. The claim concerns printed typography on a physical key.')]
        q=[sg('source_title','Correction to the product launch claim'),sg('excerpt',f'We retract the launch claim for {t}. The claim concerns product history.')]
        axis='launch_key_legend_typography';why='Both the title and clause identify a physical key label rather than an event in the positive.'
    elif n==17:
        p=ex(f'Current publisher decision: I withdraw the release-button diameter claim for {t}. My withdrawal concerns a button dimension.')
        q=ex(f'Current publisher decision: I withdraw the launch claim for {t}. My withdrawal concerns the launch event.')
        axis='release_button_dimension';why='A button dimension is explicitly a different property; the negative directly withdraws target launch.'
    elif n==18:
        p=ex(f'The launcher-housing finish specification for {t} is retracted. The specification describes the surface finish of a housing component.')
        q=ex(f'The launch statement for {t} is retracted. The statement describes the product launch event.')
        axis='launcher_housing_finish';why='The physical housing finish is the retracted object, not the historical event; property truth is not being approved.'
    elif n==19:
        p=[sg('heading','Indicator correction'),sg('excerpt',f'We deny the launch-indicator LED colour claim for {t}. The denied claim concerns an indicator\'s light colour.')]
        q=[sg('heading','Historical-event correction'),sg('excerpt',f'We deny the launch claim for {t}. The denied claim concerns the historical event.')]
        axis='launch_indicator_led_colour';why='An explicitly named indicator/light-colour object must not be confused with the product launch event.'
    elif n==20:
        p=ex(f'We withdraw the release-coating specification for {t}. The withdrawn specification concerns a manufacturing surface coating.')
        q=ex(f'We withdraw the launch claim for {t}. The withdrawn claim concerns the launch event.')
        axis='release_coating_manufacturing_spec';why='Release describes an explicit surface-coating specification, not a different affirmative launch predicate.'
    elif n==21:
        p=[sg('source_title',f'Notice accompanying the {t} record'),sg('excerpt',distinct+f' The old contact route for {o} is withdrawn. This is an administrative contact-route change for {o}.')]
        q=[sg('source_title',f'Notice accompanying the {t} record'),sg('excerpt',distinct+f' The launch statement for {t} is withdrawn. This is a product-history correction for {t}.')]
        axis='similar_name_contact_route_vs_claim';why='Administrative withdrawal for a fully named different product is neutral about target launch; the negative withdraws that launch.'
    elif n==22:
        p=ex(distinct+f' We retract the archive-filename label claim for {o}. The corrected object is the label attached to an archive file.')
        q=ex(distinct+f' We retract the launch claim for {t}. The corrected object is the target launch statement.')
        axis='other_name_archive_label';why='The positive explicitly identifies another owner and an administrative label; the negative identifies target launch.'
    elif n==23:
        p=[sg('source_title',f'{t} launch bulletin correction'),sg('excerpt',f'We retract the page-count claim for the {t} launch bulletin. The correction concerns document length only.')]
        q=[sg('source_title',f'{t} launch bulletin correction'),sg('excerpt',f'We retract the launch claim for {t}. The correction concerns the launch statement in the bulletin.')]
        axis='launch_bulletin_length_vs_claim';why='The launch word in a document title does not turn its page-count property into the launch event.'
    elif n==24:
        p=ex(f'We withdraw the launch-banner colour claim for {t}. This notice concerns the printed banner\'s colour.')
        q=ex(f'We withdraw the launch claim for {t}. This notice concerns the product launch event.')
        axis='launch_banner_colour';why='The complete object identifies a printed banner property versus the event itself.'
    elif n==25:
        p=ex(f'We retract the release-section pagination claim for {t}. The corrected claim concerns page numbering in the catalogue.')
        q=ex(f'We retract the launch claim for {t}. The corrected claim concerns product history.')
        axis='release_section_pagination';why='Page numbering is an explicit neutral document property even though the section name contains release.'
    elif n==26:
        common=[sg('heading',f'Joint record of {t} and {o}'),sg('excerpt',distinct+f' We withdraw the launch-button width claim for {t}. That claim concerns a physical button.')]
        p=common+ex(f'We withdraw the launch claim for {o}. The withdrawn launch owner is explicitly {o}.')
        q=common+ex('We withdraw another launch claim in this joint record. The product owning that withdrawn launch claim is not named.')
        axis='component_cue_then_neighbor_or_unknown_launch';why='The positive has a clear component scope and another complete owner; the negative retains an explicitly unidentified launch addendum.'
    elif n==27:
        p=ex(distinct+f' I retract the spring-stiffness claim for {t}. This correction concerns the spring specification.')
        q=ex(distinct+f' I retract a launch claim for one product sharing the word {t.split()[0]}. The complete product name was omitted.')
        axis='shared_short_name_unresolved_full_owner';why='The positive gives the full owner and property. A prefix-only launch reference is explicitly unresolved, not a forced alias resolution.'
    elif n==28:
        p=ex(distinct+f' We retract the launch-button travel claim for {t}. The assertion concerns physical button movement.')
        q=ex(distinct+f' We retract a launch assertion. The notice does not say whether its product is {t} or {o}.')
        axis='button_travel_vs_explicit_two_owner_ambiguity';why='Physical button travel is clearly non-launch; the negative explicitly leaves the launch owner between two products unspecified.'
    elif n==29:
        p=[sg('source_title','Publisher current reply'),sg('excerpt',distinct+f' I withdraw the launch claim for {o}. I identify the withdrawn owner as {o}.')]
        q=[sg('source_title','Publisher current reply'),sg('excerpt',distinct+' I withdraw one product\'s launch record. This reply does not specify which of these different products owns it.')]
        axis='two_similar_names_named_or_unassigned_record';why='Complete repeated ownership keeps the positive unambiguous; the negative collection reference is explicitly unassigned.'
    elif n==30:
        common=[sg('source_title',f'Release-note correction for {t.split()[0]}')]
        p=common+ex(f'We withdraw the pulse-width specification for {t}. The correction is limited to that electrical specification.')
        q=common+ex('We withdraw the launch assertion described by this release note. The full product name is not provided, so the launch assertion owner is not identified.')
        axis='short_header_with_full_property_or_missing_launch_owner';why='A short header does not require identity inference: the positive clause supplies a full property owner, while the negative explicitly lacks its launch owner.'
    for polarity,context in [('positive',p),('negative',q)]:
        headers=[x for x in context if x['kind']!='excerpt'];notes=[x for x in context if x['kind']=='excerpt']
        signals=headers+(notes+[fact] if n in (9,15,20) else [fact]+notes)
        cid=f'li7-{len(cases)+1:03d}'
        inp={'id':cid,'policy':{'policy_id':'independent-v7-launch-policy','target_id':f'v7-fictional-target-{n:02d}','target_question':f'Was {t} launched?','entity':{'canonical_id':f'v7-fictional-target-{n:02d}','canonical_name':t,'aliases':[]},'relation':'change_or_launch','identity_requirement':'require_harness_anchor','assessment_budget':BUDGET.copy()},'candidate':{'evidence_id':cid+'-evidence','source_id':cid+'-synthetic-record','signals':signals},'proposal':{'target_binding':'exact','relation_binding':'exact'},'raw':{'identity_scope':'exact_target','relation_scope':'requested_relation','scope_risk':'none'}}
        cases.append({'id':cid,'pair_id':f'v7-pair-{n:02d}','polarity':polarity,'axis':axis,'expected':{'v17_requested_relation':polarity=='positive','v30_relevant':polarity=='positive'},'rationale':why,'input':inp})
for name,value in [('cases.json',{'schema':'engine073-launch-independent-v7','author':'independent evaluator /root/engine07_realworld_independent','cases':cases}),('inputs.json',[c['input'] for c in cases])]:
    with (ROOT/name).open('x') as f:json.dump(value,f,ensure_ascii=False,indent=2);f.write('\n')
print('new_v7_cases',len(cases))
