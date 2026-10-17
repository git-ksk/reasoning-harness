#!/usr/bin/env python3
"""New v3 discourse/control cases, authored before any v3 semantic observation."""
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parent
NAMES=['Aerolune Slate','Berylfen Keypad','Calderbrook Meter','Duskmarsh Console','Eiderfall Badge','Flaxgarden Terminal','Gorsehill Reader','Hearthmere Switch','Irisbay Module','Jadegrove Speaker','Kitehaven Tracker','Lindenreach Frame','Mossfjord Scanner','Nettleford Dock','Opalmead Sensor','Pebblehaven Recorder','Quartzgrove Projector','Rillbrook Counter','Sorrelvale Receiver','Tansyfield Printer','Ulmusbay Charger','Velvetfen Monitor','Wrenmeadow Adapter','Xanthicove Clock','Yewgarden Router','Zephyrgrove Camera','Ashenford Pointer','Brackenmere Decoder','Chervilbay Filter','Driftwoodreach Timer']
OTHER='Pondstone Anchor'
THIRD='Riverglass Node'
BUDGET={'max_model_attempts':2,'max_tokens':192,'max_elapsed_ms':90000}

def sg(kind,text):return {'kind':kind,'text':text}
def ex(text):return [sg('excerpt',text)]
cases=[]
for n,t in enumerate(NAMES,1):
    a=f'{t} was launched on 14 March 2023.'
    b=f'{OTHER} was launched on 14 March 2023.'
    c=f'{THIRD} was launched on 14 March 2023.'
    if n==1:
        common=f'A — Release history: {a}\nB — A separate removable-panel claim for {t}.\nAuthor decision: '
        positive=ex(common+'I retract statement B, the removable-panel claim.')
        negative=ex(common+'I retract statement A, the launch claim.')
        axis='lettered_statement_scope';why='The author explicitly identifies the property statement versus the launch statement by both letter and subject.'
    elif n==2:
        common=f'Main entry: {a}\nCompanion entry: {b}\nCurrent publisher note: '
        positive=[sg('source_title','Two-product release ledger'),sg('excerpt',common+f'We deny the companion launch claim for {OTHER}.')]
        negative=[sg('source_title','Two-product release ledger'),sg('excerpt',common+f'We deny the main launch claim for {t}.')]
        axis='main_companion_entry';why='Both entries assert launch directly; the present denial names exactly which entry and product it concerns.'
    elif n==3:
        positive=[sg('excerpt',a),sg('excerpt',f'[Publisher amendment]\nAs the author, I withdraw the carrying-strap claim for {t}.')]
        negative=[sg('excerpt',a),sg('excerpt',f'[Publisher amendment]\nAs the author, I withdraw the claim that {t} was launched.')]
        axis='second_excerpt_amendment';why='A separate excerpt carries the same-publisher amendment; its object is either a strap property or the launch assertion.'
    elif n==4:
        positive=[sg('source_title',f'Retraction applies to the storage-format claim for {t}'),sg('heading','Release-history entry'),sg('excerpt',a)]
        negative=[sg('source_title',f'Retraction applies to the launch claim for {t}'),sg('heading','Release-history entry'),sg('excerpt',a)]
        axis='title_applies_to_object';why='The title expressly limits retraction to a named claim type for the same target.'
    elif n==5:
        positive=[sg('source_title','Accuracy bulletin'),sg('heading',f'Statement withdrawn: the oxygen-sensor specification for {t}'),sg('excerpt',a)]
        negative=[sg('source_title','Accuracy bulletin'),sg('heading',f'Statement withdrawn: the assertion that {t} was launched'),sg('excerpt',a)]
        axis='heading_statement_object';why='The heading gives the withdrawn statement object; a specification withdrawal does not deny launch.'
    elif n==6:
        common=f'Quoted item: "{a}"\nThis bulletin also states the fact directly: {a}\n'
        positive=ex(common+f'We deny the separate battery-life assertion for {t}.')
        negative=ex(common+'We deny that launch assertion.')
        axis='quoted_then_direct_assertion';why='Launch is asserted directly as well as quoted; the correction either names a separate property or points to that launch assertion.'
    elif n==7:
        common=a+'\nRegister administration: '
        positive=ex(common+'The entry was moved from cabinet 4 to cabinet 5. This note concerns filing.')
        negative=ex(common+'The archive owner now retracts the launch statement recorded here.')
        axis='administration_vs_present_retraction';why='A neutral filing action preserves launch; a current retraction of the statement recorded here does not.'
    elif n==8:
        common=f'Release-history line: {a}\nPresent correction: '
        positive=ex(common+f'The photoshoot for {t} did not occur.')
        negative=ex(common+f'The launch of {t} did not occur.')
        axis='did_not_occur_event_scope';why='The explicit non-event is either a photoshoot or the same target launch.'
    elif n==9:
        common=f'Register entry 9: {a}\nOur correction concerns the alleged event: '
        positive=ex(common+f'The museum tour for {t} never occurred.')
        negative=ex(common+f'The launch of {t} never occurred.')
        axis='never_occurred_event_scope';why='The separate museum-tour non-event leaves launch intact; a launch non-event directly contradicts it.'
    elif n==10:
        positive=ex(f'Current record: {a}\nLog annotation: No guided tour for {t} occurred.')
        negative=ex(f'Current record: {a}\nLog annotation: No launch for {t} occurred.')
        axis='no_event_log_annotation';why='The no-event annotation explicitly identifies the event type and target.'
    elif n==11:
        positive=ex(f'Today I deny the warranty claim concerning {t}.\nRelease history follows: {a}')
        negative=ex(f'Today I deny the launch claim concerning {t}.\nRelease history follows: {a}')
        axis='leading_today_author_denial';why='The present first-person denial names either warranty or launch before the direct launch statement.'
    elif n==12:
        common=f'Entry North: {a}\nEntry South: {b}\nSigned amendment: '
        positive=ex(common+f'I withdraw the launch claim for {OTHER}; this withdrawal concerns {OTHER} alone.')
        negative=ex(common+f'I withdraw the launch claim for {t}; this withdrawal concerns {t} alone.')
        axis='two_entries_exclusive_named_withdrawal';why='A signed withdrawal explicitly identifies one product and says that product alone is its scope.'
    elif n==13:
        positive=[sg('source_title',f'Entry D water-resistance claim for {t} is denied'),sg('heading','Entry C — Release history'),sg('excerpt',f'Entry C: {a}')]
        negative=[sg('source_title',f'Entry C launch claim for {t} is denied'),sg('heading','Entry C — Release history'),sg('excerpt',f'Entry C: {a}')]
        axis='cross_signal_entry_identifier';why='The title combines an entry identifier with an explicit property or launch claim object.'
    elif n==14:
        positive=ex(f'We now file this release-history entry under cabinet seven: {a}\nThe archive identifier is R-14.')
        negative=ex(f'We now retract this release-history entry: {a}\nThe archive identifier is R-14.')
        axis='colon_governed_entry_action';why='A colon introduces exactly the entry being filed or retracted; filing is neutral.'
    elif n==15:
        positive=ex(a+f' I deny only the phrase "{t} includes a stylus"; that phrase belongs to the accessories note.')
        negative=ex(a+f' I deny the phrase "{t} was launched on 14 March 2023"; that phrase belongs to this release note.')
        axis='quoted_denial_object';why='The quoted object of denial distinguishes an accessory claim from the very launch assertion.'
    elif n==16:
        positive=ex(a+f'\nFor accuracy, the publisher withdraws the fabric-cover claim about {t}.')
        negative=ex(a+f'\nFor accuracy, the publisher withdraws the launch claim about {t}.')
        axis='publisher_withdraws_claim_type';why='The current third-person publisher action has an explicit claim-type object.'
    elif n==17:
        positive=[sg('source_title','Editorial decision for the current register'),sg('heading',f'Current denial: the solar-cell claim for {t}'),sg('excerpt',a)]
        negative=[sg('source_title','Editorial decision for the current register'),sg('heading',f'Current denial: the launch claim for {t}'),sg('excerpt',a)]
        axis='heading_current_denial';why='The heading explicitly scopes the current denial to a solar-cell claim or the target launch.'
    elif n==18:
        common=f'Register Alpha contains two release assertions.\n- {a}\n- {b}\nPublisher notice: '
        positive=ex(common+'A separate spelling correction is pending.')
        negative=ex(common+'We retract a launch assertion above; the notice does not identify the entry.')
        axis='two_bullets_explicitly_unidentified';why='The negative explicitly withholds the identity of the retracted launch entry, so the target cannot be safely retained; the spelling notice is neutral.'
    elif n==19:
        common=f'Three launch entries are listed in this order.\nFirst: {a}\nSecond: {b}\nThird: {c}\nAuthor note: '
        positive=ex(common+f'I withdraw the cardboard-sleeve claim for {t} in the separate accessory list.')
        negative=ex(common+'I withdraw one launch claim in this list; this notice does not say which one.')
        axis='three_entries_one_unspecified_withdrawal';why='Explicitly unspecified withdrawal among three launch assertions is unsafe; a named accessory-list claim is a different property.'
    elif n==20:
        positive=[sg('source_title','Storage-location notice for this release-history entry'),sg('excerpt',a+' The record is held in drawer 20.')]
        negative=[sg('source_title','Retraction notice for an unspecified release-history entry'),sg('excerpt',a+' The notice provides no entry identifier.')]
        axis='title_unresolved_entry_notice';why='The negative retraction leaves its release-history referent unresolved; a storage-location notice is neutral.'
    elif n==21:
        common=f'Launch entries in the publisher register: {a} {b}\n'
        positive=ex(common+f'The publisher denies the power-cable specification for {OTHER}.')
        negative=ex(common+'The publisher denies its launch claim, without naming the product.')
        axis='publisher_possessive_unspecified_product';why='The negative explicitly omits which launch product is denied; the positive names a different property and product.'
    elif n==22:
        common=[sg('source_title','Two launch assertions in the current bulletin')]
        positive=common+[sg('heading','Cable specification withdrawn; the two release entries follow'),sg('excerpt',a+' '+b)]
        negative=common+[sg('heading','A launch statement is withdrawn; its referent was not recorded'),sg('excerpt',a+' '+b)]
        axis='heading_missing_referent';why='The negative heading explicitly records a missing launch-withdrawal referent; the cable specification is a separate property.'
    elif n==23:
        positive=[sg('source_title','Release-history register with one launch assertion'),sg('heading','We withdraw the typography note attached to this register'),sg('excerpt',a)]
        negative=[sg('source_title','Release-history register with one launch assertion'),sg('heading','We retract the single launch assertion in this register'),sg('excerpt',a)]
        axis='single_register_claim_reference';why='There is one explicit launch assertion, which the negative heading identifies; typography is a distinct editorial property.'
    elif n==24:
        positive=ex(f'Publisher record: {a}\nAs its publisher, I deny that {t} uses a lithium module.')
        negative=ex(f'Publisher record: {a}\nAs its publisher, I deny that {t} was launched.')
        axis='publisher_first_person_denies_that';why='The first-person publisher denies either the material/module property or the target launch itself.'
    elif n==25:
        positive=[sg('source_title',f'{t} cable-length correction'),sg('heading','We withdraw our cable-length statement'),sg('excerpt',a)]
        negative=[sg('source_title',f'{t} launch correction'),sg('heading','We withdraw our launch statement'),sg('excerpt',a)]
        axis='title_heading_joint_scope';why='Title and heading jointly name a cable-length statement or the launch statement for the canonical target.'
    elif n==26:
        common=f'Branch A concerns {t}: {a}\nBranch B concerns {OTHER}: {b}\nBranch correction: '
        positive=ex(common+f'We retract Branch B\'s launch claim about {OTHER}.')
        negative=ex(common+f'We retract Branch A\'s launch claim about {t}.')
        axis='branch_label_and_named_target';why='The branch label and product name jointly identify the retracted launch claim.'
    elif n==27:
        common=f'The register makes two statements. First: {a} Second: it lists a charging-socket claim for {t}.\n'
        positive=ex(common+'I retract the latter statement, the charging-socket claim.')
        negative=ex(common+'I retract the former statement, the launch claim.')
        axis='former_latter_with_explicit_claim_type';why='Both the ordinal reference and the explicit claim type distinguish the launch from the other property.'
    elif n==28:
        positive=ex(a+f'\nA reader asked about the headphones claim for {t}. The publisher now denies that headphones claim.')
        negative=ex(a+f'\nA reader asked about the launch claim for {t}. The publisher now denies that launch claim.')
        axis='reader_question_publisher_response_chain';why='The publisher response follows an explicit named-property or launch referent from the preceding sentence.'
    elif n==29:
        positive=ex(f'Release line 29: {a}\nThe invoice-template claim for {t} is hereby withdrawn by its author.')
        negative=ex(f'Release line 29: {a}\nThe launch claim for {t} is hereby withdrawn by its author.')
        axis='passive_hereby_author_withdrawal';why='The passive current withdrawal explicitly identifies the claim object and target.'
    elif n==30:
        common=[sg('source_title','Launch archive record'),sg('heading','Record status: indexed')]
        positive=common+[sg('excerpt',a+'\n\nNo revision is recorded for this launch assertion. The index entry is complete.')]
        negative=common+[sg('excerpt',a+'\n\nRevision notice: This launch assertion is withdrawn by the author. The index entry is complete.')]
        axis='neutral_no_revision_vs_withdrawal';why='No recorded revision is a neutral assertion about the record; the revision notice directly withdraws this launch assertion.'
    # Fixed before observation: the tested affirmative is always an unadorned
    # canonical-target-subject signal. Entry labels are contextual references,
    # never a required new launch-predicate syntax or a trusted reference AST.
    if n==6:
        positive=ex(f'Publisher annotation: The launch statement above appears in this record. We deny the separate battery-life assertion for {t}.')
        negative=ex('Publisher annotation: The launch statement above appears in this record. We deny that launch assertion.')
        axis='direct_assertion_with_back_reference'
        why='The standalone launch signal is factual; the annotation denies a separate property or that single launch assertion.'
    for polarity,signals in [('positive',positive),('negative',negative)]:
        headers=[dict(signal) for signal in signals if signal['kind']!='excerpt']
        context=[{'kind':'excerpt','text':signal['text'].replace(a,f'Launch assertion recorded for {t}.')} for signal in signals if signal['kind']=='excerpt']
        fact=sg('excerpt',a)
        signals=headers+(context+[fact] if n in (11,14) else [fact]+context)
        cid=f'li3-{len(cases)+1:03d}'
        inp={'id':cid,'policy':{'policy_id':'independent-v3-launch-policy','target_id':f'v3-fictional-target-{n:02d}','target_question':f'Was {t} launched on 14 March 2023?','entity':{'canonical_id':f'v3-fictional-target-{n:02d}','canonical_name':t,'aliases':[]},'relation':'change_or_launch','identity_requirement':'require_harness_anchor','assessment_budget':BUDGET.copy()},'candidate':{'evidence_id':cid+'-evidence','source_id':cid+'-synthetic-source','signals':signals},'proposal':{'target_binding':'exact','relation_binding':'exact'},'raw':{'identity_scope':'exact_target','relation_scope':'requested_relation','scope_risk':'none'}}
        cases.append({'id':cid,'pair_id':f'v3-pair-{n:02d}','polarity':polarity,'axis':axis,'expected':{'v17_requested_relation':polarity=='positive','v30_relevant':polarity=='positive'},'rationale':why,'input':inp})
for name,value in [('cases.json',{'schema':'engine073-launch-independent-v3','author':'independent evaluator /root/engine07_realworld_independent','cases':cases}),('inputs.json',[c['input'] for c in cases])]:
    with (ROOT/name).open('w') as f:json.dump(value,f,ensure_ascii=False,indent=2);f.write('\n')
print('newly_authored_cases',len(cases))
