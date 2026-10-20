#!/usr/bin/env python3
"""V4 authoring stage only; new publication-notice/control constructions."""
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parent
NAMES=['Avenrock Dial','Brislen Meter','Cairnleaf Panel','Dolmere Gauge','Eshbrook Receiver','Falkspur Reader','Greyhaven Sensor','Hollinridge Module','Islewood Printer','Kelmor Speaker','Luneth Frame','Mirethorn Console','Norvale Scanner','Osmere Tracker','Prairieglass Switch','Quarryfern Adapter','Ruskfield Monitor','Solmere Recorder','Ternwood Dock','Uplandthorn Camera','Valewick Tablet','Westhaven Pointer','Yonderfen Counter','Zelbrook Projector','Ardenleaf Relay','Bramshore Terminal','Corvenhill Clock','Damarfen Badge','Elderquay Decoder','Fernbrook Filter']
OTHER='Marshglass Basin';THIRD='Copperwillow Case'
BUDGET={'max_model_attempts':2,'max_tokens':192,'max_elapsed_ms':90000}
def s(kind,text):return {'kind':kind,'text':text}
def body(text):return [s('excerpt',text)]
cases=[]
for n,t in enumerate(NAMES,1):
    fact=s('excerpt',f'{t} was launched in May 2022.')
    if n==1:
        p=[s('source_title','Signed erratum from the publisher'),s('heading','Correction scope: surface-finish description'),s('excerpt',f'As the author, I retract the glazed-surface statement about {t}. Signed: the publisher.')]
        q=[s('source_title','Signed erratum from the publisher'),s('heading','Correction scope: launch assertion'),s('excerpt',f'As the author, I retract the launch statement about {t}. Signed: the publisher.')]
        axis='signed_erratum_scope';reason='The signed erratum expressly names a surface-finish statement or the launch statement.'
    elif n==2:
        p=[s('source_title',f'Corrigendum concerning the optical coating of {t}'),s('heading','Current publisher correction'),s('excerpt',f'We withdraw the optical-coating claim for {t}; the correction is limited to optical coating.')]
        q=[s('source_title',f'Corrigendum concerning the launch of {t}'),s('heading','Current publisher correction'),s('excerpt',f'We withdraw the launch claim for {t}; the correction is limited to that launch claim.')]
        axis='journal_corrigendum_limited_object';reason='Title and current correction both limit the withdrawn object explicitly, without a numeric reference.'
    elif n==3:
        p=body(f'Publisher response to a press enquiry: We deny only the statement that {t} has a porcelain shell. This response concerns the shell composition.')
        q=body(f'Publisher response to a press enquiry: We deny the statement that {t} was launched. This response concerns the launch assertion.')
        axis='press_enquiry_publisher_response';reason='A publisher denial has a quoted-clause object concerning porcelain shell composition or launch.'
    elif n==4:
        p=[s('heading','Retraction for the composition table'),s('excerpt',f'The stainless-steel claim for {t} is withdrawn. This retraction concerns the composition table alone.')]
        q=[s('heading','Retraction for the release-history table'),s('excerpt',f'The launch claim for {t} is withdrawn. This retraction concerns the release-history statement alone.')]
        axis='composition_vs_release_table';reason='The table type and explicit claim object give the withdrawal scope.'
    elif n==5:
        p=body(f'Electrical-specification notice. The voltage-rating claim about {t} is retracted by the publisher. This notice is limited to voltage ratings.')
        q=body(f'Release-history notice. The launch claim about {t} is retracted by the publisher. This notice is limited to the launch claim.')
        axis='electrical_specification_notice';reason='The publisher expressly retracts a voltage specification or the launch claim; no property truth is requested.'
    elif n==6:
        p=[s('heading','Dimension correction'),s('excerpt',f'We retract the text describing {t} as eight millimetres thick. Our correction concerns thickness only.')]
        q=[s('heading','Launch correction'),s('excerpt',f'We retract the text stating that {t} was launched. Our correction concerns the launch statement.')]
        axis='described_dimension_text';reason='The object of retraction is a dimension description or the explicit launch statement.'
    elif n==7:
        p=body(f'Customer-question response, issued by the publisher: I deny the waterproof-rating claim for {t}. This response addresses the rating, not another specification.')
        q=body(f'Customer-question response, issued by the publisher: I deny the launch claim for {t}. This response addresses the launch claim.')
        axis='customer_question_rating_response';reason='The present first-person response names a waterproof rating or launch as the denied object.'
    elif n==8:
        p=[s('source_title','Manual errata sheet'),s('heading','Author-requested deletion'),s('excerpt',f'The sentence "{t} is sealed against dust" is withdrawn at the author\'s request. The deletion concerns dust sealing.')]
        q=[s('source_title','Manual errata sheet'),s('heading','Author-requested deletion'),s('excerpt',f'The sentence "{t} was launched in May 2022" is withdrawn at the author\'s request. The deletion concerns launch.')]
        axis='manual_errata_quoted_sentence';reason='The deleted sentence is quoted, making the different property or same launch object explicit.'
    elif n==9:
        p=[s('source_title','Lab-method annotation'),s('excerpt',f'The publisher adds this correction: No bench measurement of {t} occurred. The correction concerns measurement activity.')]
        q=[s('source_title','Release-history annotation'),s('excerpt',f'The publisher adds this correction: No launch of {t} ever occurred. The correction concerns the claimed launch.')]
        axis='measurement_vs_launch_non_event';reason='A measurement non-event is a different activity; a same-target launch non-event directly denies launch.'
    elif n==10:
        p=body(f'Catalogue export notice: The mass line for {t} has been withdrawn. The scope of our withdrawal is the mass specification alone.')
        q=body(f'Catalogue export notice: The launch line for {t} has been withdrawn. The scope of our withdrawal is the launch assertion alone.')
        axis='catalogue_export_line_scope';reason='The withdrawn line and express scope identify mass versus launch, independently of export mechanics.'
    elif n==11:
        p=[s('source_title','Limited retraction in the article abstract'),s('heading',f'{t}: infrared-transmission specification'),s('excerpt',f'We retract the infrared-transmission specification for {t}. The notice applies only to that physical specification.')]
        q=[s('source_title','Limited retraction in the article abstract'),s('heading',f'{t}: launch assertion'),s('excerpt',f'We retract the launch assertion for {t}. The notice applies to that launch assertion.')]
        axis='abstract_physical_spec_limit';reason='The abstract notice names the relevant physical specification or launch assertion directly.'
    elif n==12:
        p=[s('heading','Revised-page deletion notice'),s('excerpt',f'The author withdraws only this sentence: "{t} has a ceramic coating." The deletion concerns coating material.')]
        q=[s('heading','Revised-page deletion notice'),s('excerpt',f'The author withdraws this sentence: "{t} was launched in May 2022." The deletion concerns launch.')]
        axis='revision_deleted_sentence';reason='The author identifies the withdrawn sentence by its complete content and explicit topic.'
    elif n==13:
        p=[s('heading','Editor correction box'),s('excerpt',f'Affected product: {OTHER}. Affected claim: its launch. We retract the launch claim for {OTHER}.')]
        q=[s('heading','Editor correction box'),s('excerpt',f'Affected product: {t}. Affected claim: its launch. We retract the launch claim for {t}.')]
        axis='editor_box_named_product';reason='The final retraction clause repeats the product name, so no structured field interpretation or coreference is required.'
    elif n==14:
        p=[s('source_title',f'Syndicated correction about {OTHER}'),s('excerpt',f'This notice applies only to {OTHER}. The assertion that {OTHER} was launched is retracted by its publisher.')]
        q=[s('source_title',f'Syndicated correction about {t}'),s('excerpt',f'This notice applies only to {t}. The assertion that {t} was launched is retracted by its publisher.')]
        axis='syndicated_notice_explicit_owner';reason='The syndicated notice explicitly repeats and limits its owner to another product or the tested target.'
    elif n==15:
        common=f'This bulletin discusses {t}, {OTHER}, and {THIRD}. '
        p=body(common+f'Only the publisher statement about {OTHER} is corrected here: We deny that {OTHER} was launched.')
        q=body(common+f'Only the publisher statement about {t} is corrected here: We deny that {t} was launched.')
        axis='multi_product_bulletin_named_response';reason='Other names occur as context, but the denial itself names one product explicitly.'
    elif n==16:
        p=body(f'This correction concerns the capillary-channel design of {t}. I withdraw the capillary-channel design claim. The correction has that design as its entire scope.')
        q=body(f'This correction concerns the launch of {t}. I withdraw the launch claim. The correction has that launch as its entire scope.')
        axis='scope_paragraph_with_repeated_topic';reason='The paragraph repeatedly states a distinct design topic or the launch topic, without numbered mapping.'
    elif n==17:
        p=[s('source_title','Publication metadata correction'),s('excerpt','The article dateline was corrected from 9 June to 10 June. This notice concerns the publication timestamp alone.')]
        q=[s('source_title','Publication correction to the launch claim'),s('excerpt',f'The article statement that {t} was launched is retracted by the author. This notice concerns that launch statement.')]
        axis='publication_dateline_vs_claim';reason='An article timestamp correction is neutral about product launch; an explicit retraction of its launch assertion is not.'
    elif n==18:
        p=body('The reuse permission for this article is withdrawn. The product launch statement is unchanged; the notice concerns the permission to reuse the article.')
        q=body('The product launch statement in this article is withdrawn. The reuse permission is unchanged; the notice concerns the launch statement.')
        axis='article_permission_vs_content';reason='Withdrawing article reuse permission does not retract its launch fact; withdrawing the single launch statement does.'
    elif n==19:
        p=[s('source_title','Gallery caption amendment'),s('excerpt',f'The caption claim "{t} has a red indicator label" is retracted by the editor. The amendment concerns indicator colour.')]
        q=[s('source_title','Gallery caption amendment'),s('excerpt',f'The caption claim "{t} was launched in May 2022" is retracted by the editor. The amendment concerns launch.')]
        axis='gallery_caption_quoted_object';reason='The amended caption explicitly quotes the property or launch object being retracted.'
    elif n==20:
        p=[s('heading','Translation erratum'),s('excerpt',f'The publisher denies the statement that {t} uses an upward-facing connector. The erratum concerns connector orientation.')]
        q=[s('heading','Translation erratum'),s('excerpt',f'The publisher denies the statement that {t} was launched. The erratum concerns the launch assertion.')]
        axis='translation_connector_spec';reason='The statement object names connector orientation or launch; the test does not approve the connector claim.'
    elif n==21:
        p=[s('heading','Proof appendix: activity correction'),s('excerpt',f'An author notice records that no embossing trial for {t} took place. This is a correction of the embossing-trial account.')]
        q=[s('heading','Proof appendix: activity correction'),s('excerpt',f'An author notice records that no launch of {t} happened. This is a correction of the launch account.')]
        axis='proof_appendix_event_identity';reason='The non-event is explicitly embossing versus launch for the named target.'
    elif n==22:
        p=body(f'NOTICE\nPublisher statement: The shock-resistance claim for {t} is withdrawn.\nReason given: specification transcription error.')
        q=body(f'NOTICE\nPublisher statement: The launch claim for {t} is withdrawn.\nReason given: launch announcement error.')
        axis='publication_notice_reason_and_object';reason='The publisher statement gives an explicit object; the following reason corroborates its specification/launch scope.'
    elif n==23:
        p=body(f'This correction is addressed to the paragraph about {OTHER}, another product. We withdraw the assertion that {OTHER} was launched.')
        q=body(f'This correction is addressed to the paragraph about {t}, the product named in the launch fact. We withdraw the assertion that {t} was launched.')
        axis='addressed_paragraph_named_assertion';reason='The corrected paragraph subject and the withdrawal clause independently name its product.'
    elif n==24:
        p=[s('source_title','Publisher archival note'),s('excerpt','The archive now stores this page in a different folder. This notice records the storage location of the page.')]
        q=[s('source_title','Publisher withdrawal: product identity omitted'),s('excerpt','A launch claim has been withdrawn. The withdrawal notice does not name the product.')]
        axis='detached_notice_owner_missing';reason='The launch-withdrawal owner is explicitly absent, so withholding is required; a filing-location notice is neutral.'
    elif n==25:
        p=body('The editor publishes two corrections in this notice. One concerns typography; the other concerns the units in a dimension table. Both corrections are limited to those topics.')
        q=body('The editor publishes two corrections in this notice. One is a retracted launch claim without a named product; the other is typography. The owner of the launch claim is not identified.')
        axis='two_corrections_one_unidentified_launch';reason='The negative explicitly includes an unidentified launch retraction; both positive topics are plainly non-launch.'
    elif n==26:
        p=body(f'Report amendment: We retract the assertion about the anode specification of {t}. The amended assertion has that anode specification as its subject.')
        q=body('Report amendment: We retract an assertion in our release-history report. The withdrawn launch statement is not identified, and its product name is omitted.')
        axis='report_amendment_unidentified_statement';reason='An explicit anode object is outside launch; an unidentified launch withdrawal requires withholding.'
    elif n==27:
        p=[s('source_title',f'Joint publisher notice: {t} and {OTHER}'),s('excerpt','We retract one product\'s strap-length claim. Every withdrawal in this notice concerns strap length only.')]
        q=[s('source_title',f'Joint publisher notice: {t} and {OTHER}'),s('excerpt','We retract one product\'s launch claim. The product names were omitted from the withdrawal notice.')]
        axis='joint_notice_unknown_owner_known_topic';reason='An unknown owner still cannot affect launch when the topic is expressly strap length only; unknown launch owner is unsafe.'
    elif n==28:
        p=[s('heading','Publisher update'),s('excerpt','The catalogue file was reformatted. Its sections were reordered, and its page margins were widened.')]
        q=[s('heading','Publisher update'),s('excerpt','We withdraw a launch statement. The owner of the withdrawn launch statement is unknown in this notice.')]
        axis='publication_layout_vs_unknown_owner';reason='Layout edits are neutral; an expressly unknown launch-withdrawal owner is not a safe launch affirmation.'
    elif n==29:
        p=[s('source_title','Archive filing note'),s('excerpt',f'This record is indexed under {t}. Its topic category is product history.')]
        q=[s('source_title','Current author withdrawal'),s('excerpt','I withdraw the launch account associated with this record. The target name is omitted from this withdrawal account.')]
        axis='archive_index_vs_author_account';reason='An indexing note is neutral; a current first-person withdrawal of the associated launch account requires withholding.'
    elif n==30:
        p=[s('source_title','Proofreading note for a joint release bulletin'),s('heading','Changes limited to spacing and font size'),s('excerpt','All changes described in this notice concern document formatting.')]
        q=[s('source_title','Withdrawal from a joint release bulletin'),s('heading','Owner of the withdrawn claim not provided'),s('excerpt','We retract the launch claim. This notice lacks the product name for the retracted claim.')]
        axis='proofreading_vs_joint_withdrawal';reason='Formatting is neutral; the withdrawal expressly lacks the launch claim owner and cannot safely preserve launch.'
    for polarity,context in [('positive',p),('negative',q)]:
        headers=[x for x in context if x['kind']!='excerpt'];notes=[x for x in context if x['kind']=='excerpt']
        signals=headers+(notes+[fact] if n in (3,16,17) else [fact]+notes)
        cid=f'li4-{len(cases)+1:03d}'
        inp={'id':cid,'policy':{'policy_id':'independent-v4-launch-policy','target_id':f'v4-fictional-target-{n:02d}','target_question':f'Was {t} launched in May 2022?','entity':{'canonical_id':f'v4-fictional-target-{n:02d}','canonical_name':t,'aliases':[]},'relation':'change_or_launch','identity_requirement':'require_harness_anchor','assessment_budget':BUDGET.copy()},'candidate':{'evidence_id':cid+'-evidence','source_id':cid+'-synthetic-publication-source','signals':signals},'proposal':{'target_binding':'exact','relation_binding':'exact'},'raw':{'identity_scope':'exact_target','relation_scope':'requested_relation','scope_risk':'none'}}
        cases.append({'id':cid,'pair_id':f'v4-pair-{n:02d}','polarity':polarity,'axis':axis,'expected':{'v17_requested_relation':polarity=='positive','v30_relevant':polarity=='positive'},'rationale':reason,'input':inp})
for name,value in [('cases.json',{'schema':'engine073-launch-independent-v4','author':'independent evaluator /root/engine07_realworld_independent','cases':cases}),('inputs.json',[c['input'] for c in cases])]:
    with (ROOT/name).open('x') as f:json.dump(value,f,ensure_ascii=False,indent=2);f.write('\n')
print('new_cases',len(cases))
