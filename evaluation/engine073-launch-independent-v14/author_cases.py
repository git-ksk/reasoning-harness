#!/usr/bin/env python3
"""Fresh v14 correction-proposition, explanation, ownership and real-gap boundaries."""
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parent
NAMES=['Aldercrest Coupler','Burrhaven Gauge','Cormorantquay Panel','Dawnmere Rotor','Eglantinebank Clip','Flaxhaven Ring','Gannetquay Lever','Hennafield Platen','Isletmere Sensor','Jasminebank Wheel','Kestrelquay Seal','Larkspurhaven Board','Marigoldfield Latch','Nutmegquay Dock','Orchardmere Lens','Ploverbank Clip','Quarrystone Rail','Raspberryquay Hub','Sedgehaven Tray','Tawnyfield Spindle','Umberquay Gimbal','Vergefield Prism','Wisteriamere Coil','Xanthophyllbank Pedal','Yellowdockquay Valve','Zephyrhaven Cradle','Amaranthbank Clip','Bluebellquay Module','Citrinefield Scanner','Dunlinhaven Collar']
FOREIGN='Flintmere Prism'
SECOND='Larkquay Rotor'
BUDGET={'max_model_attempts':2,'max_tokens':192,'max_elapsed_ms':90000}
def s(k,t):return {'kind':k,'text':t}
def ex(t):return [s('excerpt',t)]
cases=[]
for n,t in enumerate(NAMES,1):
    if n==1:
        common=f'Object: the physical release-cover colour claim for {t}, specifically "the cover is not red". We retract this object. '
        p=ex(common+'The denial concerns that fully specified cover-colour proposition, including its colour negation.')
        q=ex(common+f'We also deny the launch claim for {t}. An explanatory note calls the notice foreign-only, but the additional denial explicitly names the target launch.')
        axis='negated_colour_proposition_vs_explicit_launch_denial_and_explanation';why='Negation inside a fully specified physical property is not target-launch negation. An extra explicit target denial is not waived by a foreign-only self-description.'
    elif n==2:
        p=ex(f'Subject: a specification claim. I deny this subject, namely the complete release-handle material statement for {t}. The adjacent explanation identifies only that material proposition.')
        q=ex(f'Subject: an event claim. I deny this subject, namely the launch statement for {t}. A later explanation says the correction is about {FOREIGN} only, without changing the explicit target-launch subject.')
        axis='local_complete_denial_apposition_vs_contradicting_foreign_explanation';why='A complete material apposition is safe. A foreign explanation cannot overwrite an explicit target-event apposition.'
    elif n==3:
        p=ex(f'Subject: an unowned launch claim. Owner supplement for the subject:\n{FOREIGN}\nObject: the printed release-label colour claim for {t}. We retract the subject and deny the object. Each role has a complete independent scope.')
        q=ex(f'Subject: an unowned launch claim. Owner supplement for the item:\n{FOREIGN}\nObject: the printed release-label colour claim for {t}. We retract the subject and deny the object. No owner supplement is assigned to the launch Subject; the Item label does not identify its owner.')
        axis='correct_same_role_owner_vs_unassigned_different_role_supplement';why='The positive supplement explicitly owns the launch Subject. A differently named role cannot fill an unknown launch Subject merely because the full foreign name is present.'
    elif n==4:
        p=ex(f'Item: the document page-count and heading-font claim for {t}. We withdraw the item and deny the document title-colour claim for {t}. Our scope explanation identifies only these complete document properties.')
        q=ex(f'Item: the document page-count and heading-font claim for {t}. We withdraw every assertion in this source, including the launch statement for {t}. A later scope note claims that only {FOREIGN} is affected; the explicit whole-source withdrawal remains in the record.')
        axis='complete_document_scope_vs_whole_source_with_foreign_only_note';why='A limited document-property correction is safe. Explicit whole-source withdrawal cannot be narrowed by a contradictory foreign-only note.'
    elif n==5:
        common=[s('source_title',f'Canonical product name: {t}')]
        p=common+ex(f'Object: the physical release-key width claim for {t}. We deny the object and withdraw the launch statement for {FOREIGN}. Naming note: the final token Clip belongs to the complete canonical name above.')
        q=common+ex(f'Object: the physical release-key width claim for {t}. We deny the object and withdraw an unowned launch statement. The paragraph identifying the withdrawn event\'s product owner is missing from this excerpt. Naming note: the final token Clip belongs to the complete canonical name above.')
        axis='canonical_clip_token_note_vs_actual_omitted_launch_owner';why='A complete canonical-name note is not clipping. The negative has a separate withdrawn launch event with actually omitted ownership and raw context_gap.'
    elif n==6:
        p=ex(f'Object: an expressly unowned launch statement. Owner supplement for the object:\n{FOREIGN}.\nWe deny this object while we retract the physical release-collar height claim for {t}. The single owner supplement supplies the complete foreign product name.')
        q=ex(f'Object: the launch statement for {t}. Owner supplement for the object:\n{FOREIGN}.\nWe deny this object while we retract the physical release-collar height claim for {t}. The supplement names a foreign product, but the Object already explicitly declares target launch.')
        axis='complete_owner_line_with_punctuation_vs_existing_target_owner';why='One designated full owner line fills only an unowned frame; punctuation does not confer authority to overwrite an existing target owner.'
    elif n==7:
        p=ex(f'Subject: the material of the physical launch lever on {t}. I retract the claim about this subject. Denial object: the complete launch-lever material proposition for {t}, not a product event.')
        q=ex(f'Subject: the material of the physical launch lever on {t}. I retract the claim about this subject. Separately, {t} does not exist. A note labels both corrections property-only, but the second statement denies the complete target itself.')
        axis='component_modifier_and_target_existence_not_overridden_by_property_note';why='A physical component containing the word launch is explicitly material-only. A separately named target-existence denial remains blocking despite a property-only note.'
    elif n==8:
        p=ex(f'Object: the printed font-size claim about the document title for {t}. We deny the object. Item: the printed heading-colour claim about the same document for {t}. We withdraw the item. Both complete objects are attributes of printed text.')
        q=ex(f'Object: the printed font-size claim about the document title for {t}. We deny the object. Item: the product launch statement for {t}, which appears in the document title. We withdraw the item. Its location in a title does not change the explicitly named product-event object.')
        axis='printed_title_heading_attributes_vs_launch_statement_carried_by_title';why='Typography and colour are non-launch attributes. A launch claim does not become a printed-property claim merely because it appears in a title.'
    elif n==9:
        p=ex(f'I deny that claim—the complete width-and-height specification for the physical release socket on {t}—and I withdraw the launch statement for {FOREIGN}. The apposition supplies one exact physical object.')
        q=ex(f'I deny that claim—either the width specification for the release socket on {t} or the launch statement for {t}—and I withdraw the launch statement for {FOREIGN}. The apposition leaves the first correction unselected; a note calling it foreign-only does not select an object.')
        axis='fully_selected_property_apposition_vs_explicitly_unselected_scope';why='The positive apposition is complete and uniquely physical. An unselected property/target-event correction is unknown despite an independently safe foreign event.'
    elif n==10:
        p=ex(f'Subject: the launch statement for {FOREIGN}. We retract that launch claim. The scope explanation repeats that this is the complete foreign-event Subject.')
        q=ex(f'Subject: the physical release-handle colour claim for {FOREIGN}. We retract that launch claim. Its product owner is not supplied. A scope explanation calls the launch correction foreign because {FOREIGN} is mentioned, but the typed Subject only supplies a colour property.')
        axis='foreign_event_referent_vs_self_claimed_foreign_property_loan';why='A full foreign-event Subject can bind a local launch claim. A foreign-only self-description does not turn a property Subject into the owner of an unidentified launch reference.'
    elif n==11:
        common=f'Item: the physical release-panel width claim for {t}. Author reply: "I deny the item." We report that we also retract the launch statement for '
        p=ex(common+f'{FOREIGN}. The report completely names the foreign event being withdrawn.')
        q=ex(common+f'{t}. The report completely names target launch; an editorial note nevertheless calls the report foreign-only.')
        axis='current_reported_withdrawal_not_overridden_by_editorial_self_label';why='The property Item is safe; an explicit reporting clause has its own full event owner and is not changed by a contradictory editorial label.'
    elif n==12:
        p=ex(f'Object: "The physical release button on {t} is not green." We withdraw the object, namely this complete button-colour statement. We also deny the printed button-label font claim for {t}.')
        q=ex(f'Object: "The physical release button on {t} is not green." We withdraw the object, namely this complete button-colour statement. We also deny the launch statement for {t}, although a trailing explanation says the notice only concerns button colour.')
        axis='quoted_negative_property_value_and_independent_target_event';why='A negative colour value stays inside a full property object. The trailing colour explanation cannot consume a separate complete target-launch denial.'
    elif n==13:
        p=ex(f'Object: the launch statement for {FOREIGN}. We withdraw the object because we deny the printed release-label font claim for {t}. The Object\'s complete owner is explicitly foreign.')
        q=ex(f'Object: the launch statement for {t}. We withdraw the object because we deny the printed release-label font claim for {t}. A subsequent explanation names {FOREIGN} as the intended owner, conflicting with the original complete target declaration.')
        axis='original_complete_event_owner_vs_later_conflicting_intended_owner';why='A later intended-owner explanation cannot overwrite a clearly target-owned launch Object. The positive is foreign-owned from its declaration.'
    elif n==14:
        p=ex(f'Subject: the document heading-font specification for {t}. We deny the subject. Excerpt note: this record contains the complete local subject and owner explanation; the correction is only heading typography.')
        q=ex('Subject: an unowned launch statement. We deny the subject. Excerpt note: the local owner sentence was omitted. A foreign owner is suggested but not identified or selected; the current correction has genuinely missing launch-owner context.')
        axis='complete_property_context_note_vs_genuine_omission_and_suggestion';why='A complete property context is safe. An omitted owner sentence and an unselected suggestion do not own a withdrawn/denied launch Subject; raw context_gap accompanies that semantic unknown.'
    elif n==15:
        p=ex(f'Object: the physical release-cover thickness claim for {t}. I withdraw this object and only this fully named thickness property. We also deny the launch claim for {FOREIGN}.')
        q=ex(f'Object: the physical release-cover thickness claim for {t}. I withdraw all statements in this source, including its target launch claim. We also deny the launch claim for {FOREIGN}. A final note says the withdrawal is limited to thickness, contradicting the explicit all-statements scope.')
        axis='explicitly_limited_property_scope_vs_all_statements_and_narrowing_note';why='A fully limited property withdrawal is positive. A contradictory final narrowing note cannot undo the recorded whole-source withdrawal.'
    elif n==16:
        p=[s('source_title',f'Complete product name: {t}'),s('heading',f'Object: the printed document title-font claim for {t}')]+ex(f'We deny the object and retract the launch statement for {FOREIGN}. Proper-name note: Clip is the last word of the complete product name, not omitted source context.')
        q=[s('source_title',f'{t} does not exist'),s('heading',f'Object: the printed document title-font claim for {t}')]+ex(f'We deny the object and retract the launch statement for {FOREIGN}. The canonical product name is {t}; Clip is part of that complete name. A property-only explanation does not remove the title\'s explicit target-existence denial.')
        axis='complete_clip_note_vs_actual_target_denial_in_source_title';why='A full-name note is neutral. An explicit target-existence denial in the source title remains source evidence and cannot be erased by a safe body correction.'
    elif n==17:
        p=ex(f'Item: an unowned launch statement. Current author reply: "I retract the item." Owner supplement for the item:\n{FOREIGN}\nThe supplement explicitly identifies this previously unowned Item.')
        q=ex(f'Item: the launch statement for {t}. Current author reply: "I retract the item." Owner supplement for the item:\n{FOREIGN}\nThe author calls the reply foreign-only, but its existing Item declaration explicitly names target launch.')
        axis='reply_tail_owner_supplement_fills_only_unowned_item';why='A same-role adjacent complete owner may fill a genuinely unowned frame, not replace a target-owned Item after its retraction.'
    elif n==18:
        p=ex(f'Subject: "The release-ring surface on {t} is blue." Author reply: "I deny the subject." Denial explanation: this complete ring-colour proposition for {t}.')
        q=ex(f'Subject: "The release-ring surface on {t} is blue." Author reply: "I deny the launch statement for {t}." A later narrative says the reply is about {FOREIGN} only, despite the reply\'s complete target-event object.')
        axis='quoted_same_role_property_reply_vs_explicit_target_reply_and_false_narrative';why='The positive same-role reply has a complete physical object. A later foreign narrative cannot change the explicitly quoted target-launch denial.'
    elif n==19:
        p=ex(f'Object: the paper-colour specification for the printed document about {t}. We deny that object. Property note: the paper colour is not specified; this note concerns only that complete paper-colour attribute and does not omit the launch record.')
        q=ex('Object: an unowned launch statement. We deny that object. Property note: the paper colour is not specified. The property note supplies no product owner for the launch Object, whose owner remains unrecorded.')
        axis='unknown_property_value_vs_unknown_launch_owner';why='An unspecified value of a fully identified document property is not missing launch context. An unowned launch Object remains unknown despite a separate paper-property note.'
    elif n==20:
        p=ex(f'Subject: the physical release-key height claim for {t}. We deny this subject. Item: the launch claim for {SECOND}. We withdraw the item. A scope note identifies each complete independent object.')
        q=ex(f'Subject: the physical release-key height claim for {t}. We deny this subject. Item: the launch claim for either {SECOND} or {t}, without a selection. We withdraw the item. A scope note calls the notice foreign-only but does not select between the declared Item owners.')
        axis='fully_owned_event_item_vs_mixed_owner_and_nonselecting_scope_note';why='A safe property Subject does not resolve a mixed launch Item. A foreign-only label that selects no owner is not a same-role complete owner supplement.'
    elif n==21:
        common=f'Object: the printed release-label font-size statement for {t}. We deny the object. The launch statement for '
        p=ex(common+f'{FOREIGN} is denied by us. This complete second sentence concerns a different product event.')
        q=ex(common+f'{t} is denied by us. A property-only note follows, but the second sentence directly denies the target event.')
        axis='passive_current_fully_named_event_denial_after_property';why='The first-person passive denial has a fully named event owner. Its clause scope is independent of the property Object and subsequent explanation.'
    elif n==22:
        p=ex(f'Object: an unowned launch statement. Owner supplement for the object: {FOREIGN}. Subject: "The physical release collar on {FOREIGN} is brass." We retract the object and deny the subject. The launch Object has its own explicit owner supplement.')
        q=ex(f'Object: an unowned launch statement. Subject: "The physical release collar on {FOREIGN} is brass." We retract the object and deny the subject. The launch Object\'s owner sentence is missing from this excerpt; the quoted property mentions a product but is not an Object-owner supplement.')
        axis='own_launch_owner_supplement_vs_foreign_name_only_in_property_quote';why='A foreign name inside a property quotation cannot fill a different unowned launch frame. The negative has actual omitted owner context and raw context_gap.'
    elif n==23:
        p=ex(f'Object: a physical specification. We deny this object, namely the complete release-pedal width statement for {t}. Subject: the launch statement for {FOREIGN}. We withdraw this subject. The explanation agrees with each full object.')
        q=ex(f'Object: a physical specification. We deny this object, namely the complete release-pedal width statement for {t}. Subject: the launch statement for {t}. We withdraw this subject. A general explanation says all corrections are foreign, contradicting the explicitly target-owned Subject.')
        axis='safe_local_property_apposition_does_not_shield_target_subject';why='A complete property apposition scopes only its own Object. A foreign-only general explanation cannot shield an independent target-owned launch Subject.'
    elif n==24:
        p=ex(f'Item: the printed font-style of the word "Launch" in the document title for {t}. We retract the item. Denial object: only the complete printed-word typography attribute in that title.')
        q=ex(f'Item: the product launch statement for {t}. We retract the item. A document note says the title prints the word "Launch", but that typography note does not change the explicitly target-owned product-event Item.')
        axis='printed_word_launch_attribute_vs_actual_product_event_item';why='A literal printed word within a complete font attribute is not a product launch claim. The negative directly retracts an event Item despite a typography note.'
    elif n==25:
        common=f'Object: the physical release-handle width claim for {t}. Subject: the document heading-colour claim for {FOREIGN}. We deny the object and retract the subject. '
        p=ex(common+'Both scopes are fully identified properties; the two product names belong to their own complete property objects.')
        q=ex(common+'We also withdraw that launch claim. Its product owner is not identified. An explanation calls it foreign because a foreign property owner appears above, but neither property frame supplies a launch-event referent.')
        axis='two_safe_properties_do_not_create_a_launch_referent';why='Two separately owned properties are safe. Their names cannot manufacture ownership or type for an additional unidentified launch withdrawal.'
    elif n==26:
        p=[s('heading',f'Owner supplement for the subject:\n{FOREIGN}')]+ex(f'Subject: an unowned launch statement. We retract the subject and deny the physical release-cover colour claim for {t}. The designated heading supplies the Subject\'s complete sole owner.')
        q=[s('heading',f'Owner supplement for the subject:\n{t}')]+ex(f'Subject: an unowned launch statement. We retract the subject and deny the physical release-cover colour claim for {t}. A narrative calls the notice foreign-only, but the designated full owner supplement explicitly names the target.')
        axis='heading_owner_target_assignment_vs_foreign_self_description';why='A designated same-role complete heading may assign the target or a foreign owner; a contradictory self-description cannot replace that assignment.'
    elif n==27:
        p=[s('source_title',f'Canonical entity name: {t}'),s('heading',f'Object: the physical release-socket width statement for {t}')]+ex(f'We deny the object and withdraw the launch claim for {SECOND}. Name annotation: the proper product name ends in Clip and is complete.')
        q=[s('source_title',f'We deny the launch statement for {t}'),s('heading',f'Object: the physical release-socket width statement for {t}')]+ex(f'We deny the object and withdraw the launch claim for {SECOND}. The canonical entity name is {t}; the proper name ends in Clip and is complete. A foreign-only explanation does not erase the source title\'s direct target-launch denial.')
        axis='canonical_clip_name_annotation_and_target_launch_denial_in_title';why='A limited complete-name annotation is neutral. A full target-launch denial in the title cannot be overwritten by safe body scopes or a foreign-only explanation.'
    elif n==28:
        common=f'Item: the physical release-button colour claim for {t}. Author reply: "I withdraw the item." We report that we also deny the launch statement for '
        p=ex(common+f'{FOREIGN}. The report identifies our additional correction with its complete foreign owner.')
        q=ex(common+f'{t}. A report-summary label says foreign-only, but the actual reported denial names target launch.')
        axis='actual_reported_denial_owner_over_summary_label';why='The complete reported denial controls its event scope; a summary label cannot turn a target denial into a foreign correction.'
    elif n==29:
        common=f'Object: the printed document title-colour claim for {t}. We deny the object. '
        p=ex(common+f'{FOREIGN} does not exist. We also withdraw the launch statement for {SECOND}. The two extra adverse statements fully name different products.')
        q=ex(common+f'{t} does not exist. We also withdraw the launch statement for {SECOND}. An explanation calls every extra statement foreign, despite the direct target-existence denial.')
        axis='fully_named_foreign_existence_denial_vs_target_itself';why='Fully named other-product existence/event corrections do not negate target launch. A direct target-existence denial remains blocking regardless of foreign labels or safe neighbors.'
    elif n==30:
        common_title=[s('source_title',f'Object: the printed document heading-font claim for {t}')]
        p=common_title+[s('heading',f'Subject: the launch statement for {FOREIGN}')]+ex(f'We deny the object and retract the subject. Item: an unowned launch statement. Owner supplement for the item:\n{SECOND}\nWe withdraw the item. Each role across title, heading and body has a complete independent scope.')
        q=common_title+[s('heading',f'Subject: the launch statement for {t}')]+ex(f'We deny the object and retract the subject. Owner supplement for the subject: {FOREIGN}. Item: an unowned launch statement. Owner supplement for the item:\n{SECOND}\nWe withdraw the item. A body explanation calls the whole record foreign-only, but the heading already owns the Subject as target launch.')
        axis='cross_signal_role_scopes_and_owned_heading_not_overwritten';why='Finite title/heading/body roles each have explicit scope. A later foreign Subject supplement or general explanation cannot rewrite a target-owned heading declaration.'
    for polarity,context in [('positive',p),('negative',q)]:
        headers=[v for v in context if v['kind']!='excerpt'];notes=[v for v in context if v['kind']=='excerpt'];fact=s('excerpt',f'{t} was launched.')
        signals=headers+(notes+[fact] if n in (2,4,8,11,15,18,23,28) else [fact]+notes)
        cid=f'li14-{len(cases)+1:03d}';risk='context_gap' if polarity=='negative' and n in (5,14,22) else 'none'
        inp={'id':cid,'policy':{'policy_id':'independent-v14-launch-policy','target_id':f'v14-fictional-target-{n:02d}','target_question':f'Was {t} launched?','entity':{'canonical_id':f'v14-fictional-target-{n:02d}','canonical_name':t,'aliases':[]},'relation':'change_or_launch','identity_requirement':'require_harness_anchor','assessment_budget':BUDGET.copy()},'candidate':{'evidence_id':cid+'-evidence','source_id':cid+'-synthetic-record','signals':signals},'proposal':{'target_binding':'exact','relation_binding':'exact'},'raw':{'identity_scope':'exact_target','relation_scope':'requested_relation','scope_risk':risk}}
        cases.append({'id':cid,'pair_id':f'v14-pair-{n:02d}','polarity':polarity,'axis':axis,'expected':{'v17_requested_relation':polarity=='positive','v30_relevant':polarity=='positive'},'rationale':why,'input':inp})
for name,value in [('cases.json',{'schema':'engine073-launch-independent-v14','author':'independent evaluator /root/engine07_realworld_independent','cases':cases}),('inputs.json',[c['input'] for c in cases])]:
    with (ROOT/name).open('x') as f:json.dump(value,f,ensure_ascii=False,indent=2);f.write('\n')
print('new_v14_cases',len(cases))
