#!/usr/bin/env python3
"""V5 independent authoring: new discourse/notice compositions before observation."""
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parent
NAMES=['Altohaven Mosaic','Beechvale Rune','Caspian Trellis','Dawnridge Capsule','Equinox Coffer','Fablethorn Slate','Garnetwharf Lens','Harrowmere Beacon','Ivorybrook Axle','Jasperdale Loom','Kelpwood Ring','Lucentford Wedge','Marigoldhill Rail','Nimbuscove Prism','Orielwood Pivot','Petrelford Clasp','Quillhaven Tuner','Rosethorn Latch','Stonefern Knob','Tamariskfield Ruler','Umbralhaven Hook','Violetbay Panel','Whimbrelmere Rotor','Xericgrove Dial','Yellowfen Bridge','Zirconwood Coil','Adlerbay Valve','Briarwick Spool','Cedarstrand Bar','Dovehollow Crown']
OTHER='Granitepetal Chime';THIRD='Reedlantern Arc'
BUDGET={'max_model_attempts':2,'max_tokens':192,'max_elapsed_ms':90000}
def sg(kind,text):return {'kind':kind,'text':text}
def ex(text):return [sg('excerpt',text)]
cases=[]
for n,t in enumerate(NAMES,1):
    fact=sg('excerpt',f'{t} was launched.')
    if n==1:
        p=ex(f'Interview transcript. Interviewer: What do you retract? Publisher: I retract the torque-ceiling claim about {t}. My answer concerns torque.')
        q=ex(f'Interview transcript. Interviewer: What do you retract? Publisher: I retract the launch claim about {t}. My answer concerns launch.')
        axis='interview_explicit_retraction_answer';reason='The publisher answer itself explicitly identifies the torque or launch object, without relying on dialogue-role inference.'
    elif n==2:
        p=[sg('source_title','Current author reply preserved in the record'),sg('excerpt',f'Author: We retract the seam-width assertion for {t}; it is the seam-width assertion that this reply corrects.')]
        q=[sg('source_title','Current author reply preserved in the record'),sg('excerpt',f'Author: We retract the launch assertion for {t}; it is the launch assertion that this reply corrects.')]
        axis='current_reply_repeated_object';reason='The operative clause repeats a named non-launch property or launch, fixing scope directly.'
    elif n==3:
        p=ex(f'Letter from the author: I withdraw the statement, printed in the maintenance guide, that {t} requires oil every week. The subject of the withdrawn statement is the maintenance interval.')
        q=ex(f'Letter from the author: I withdraw the statement, printed in the release guide, that {t} was launched. The subject of the withdrawn statement is launch.')
        axis='author_letter_intervening_relative_phrase';reason='The full content clause names the same target and either its maintenance requirement or its launch; intervening guide text does not change that object.'
    elif n==4:
        p=[sg('heading','Current bulletin quotes the publisher directly'),sg('excerpt',f'The publisher writes: "We deny the statement that {t} has an enamel coating." The quoted denial concerns enamel coating.')]
        q=[sg('heading','Current bulletin quotes the publisher directly'),sg('excerpt',f'The publisher writes: "We deny the statement that {t} was launched." The quoted denial concerns launch.')]
        axis='attributed_present_quoted_denial';reason='The current attributed statement explicitly denies a coating claim or the target launch, rather than merely reporting an old unendorsed launch.'
    elif n==5:
        p=[sg('source_title','Print-proof marginal annotation'),sg('excerpt',f'The publisher adds the following current annotation: (I retract the assertion about the resin content of {t}.) The annotation concerns material composition.')]
        q=[sg('source_title','Print-proof marginal annotation'),sg('excerpt',f'The publisher adds the following current annotation: (I retract the assertion that {t} was launched.) The annotation concerns launch.')]
        axis='parenthesized_current_author_annotation';reason='Parentheses enclose an explicit first-person retraction with a clear property or launch object.'
    elif n==6:
        p=[sg('heading','Alt-text correction for the illustration'),sg('excerpt',f'Withdrawn text: "{t} has knurled edges." The author withdraws that knurled-edge statement.')]
        q=[sg('heading','Alt-text correction for the illustration'),sg('excerpt',f'Withdrawn text: "{t} was launched." The author withdraws that launch statement.')]
        axis='illustration_alt_text_withdrawn_object';reason='The exact withdrawn text and its following topic are supplied, so the object is not inferred from a numeric reference.'
    elif n==7:
        p=ex(f'Service-guide discussion. Customer: Is the service schedule accurate? Publisher reply: I deny that {t} requires weekly lubrication; this is a denial of that service-schedule claim.')
        q=ex(f'Release-guide discussion. Customer: Is the release account accurate? Publisher reply: I deny that {t} was launched; this is a denial of that launch claim.')
        axis='service_discussion_current_reply';reason='The publisher reply explicitly names the denied proposition; the customer question does not supply a new affirmative predicate.'
    elif n==8:
        p=ex(f'Editor: Which assertion is withdrawn? Author: We withdraw the hinge-angle range claim for {t}. Editor: The response is recorded as a correction of hinge-angle range.')
        q=ex(f'Editor: Which assertion is withdrawn? Author: We withdraw the launch claim for {t}. Editor: The response is recorded as a correction of launch.')
        axis='editor_author_exchange_corroborated_topic';reason='The author clause and editor topic echo identify the same explicit object, which is non-launch or launch.'
    elif n==9:
        p=[sg('source_title','Question-and-answer summary'),sg('excerpt',f'The following denial concerns the rotation-speed claim for {t}. We deny the rotation-speed claim for {t}.')]
        q=[sg('source_title','Question-and-answer summary'),sg('excerpt',f'The following denial concerns the launch claim for {t}. We deny the launch claim for {t}.')]
        axis='topic_then_repeated_full_denial';reason='Both sentences independently identify the denied object, without pronoun or entry-label dependence.'
    elif n==10:
        p=ex(f'We withdraw the claim, previously printed in the operating guide, that {t} supports magnetic mounting. The correction concerns mounting support.')
        q=ex(f'We withdraw the claim, previously printed in the release history, that {t} was launched. The correction concerns launch.')
        axis='intervening_attribution_withdraw_that_clause';reason='A direct that-clause makes the withdrawal object explicit despite the intervening attribution.'
    elif n==11:
        p=ex(f'{t}\'s acoustic-noise specification is hereby retracted by the author. The retracted specification concerns sound level.')
        q=ex(f'{t}\'s launch statement is hereby retracted by the author. The retracted statement concerns launch.')
        axis='possessive_hereby_retracted_subject';reason='The possessive phrase owns an explicit specification or launch statement; the second sentence confirms its topic.'
    elif n==12:
        p=[sg('heading','Author current correction'),sg('excerpt',f'{t} wasn\'t finished with chrome. I issue this as a correction of the chrome-finish claim.')]
        q=[sg('heading','Author current correction'),sg('excerpt',f'{t} wasn\'t launched. I issue this as a correction of the launch claim.')]
        axis='contracted_was_not_same_or_other_predicate';reason='The contraction is a denial of the existing was-launched proposition in the negative; chrome finish is a different property in the positive.'
    elif n==13:
        p=[sg('source_title','Author correction to the activity log'),sg('excerpt',f'The endurance test for {t} did not happen. This current correction concerns the endurance-test account.')]
        q=[sg('source_title','Author correction to the activity log'),sg('excerpt',f'The launch of {t} did not happen. This current correction concerns the launch account.')]
        axis='activity_log_did_not_happen';reason='The expressly absent activity is an endurance test or launch; this is not a new affirmative launch predicate.'
    elif n==14:
        p=ex(f'There has been no drop test of {t}. The publisher issues this statement about drop testing.')
        q=ex(f'There has been no launch of {t}. The publisher issues this statement about launch.')
        axis='existential_no_event_statement';reason='An explicit existential non-event denies drop testing or the target launch; it does not ask for support of a new affirmative relation.'
    elif n==15:
        p=[sg('heading','Publisher contact-information change'),sg('excerpt','The old mailing address was withdrawn. This notice concerns the address used for correspondence.')]
        q=[sg('heading','Publisher launch-statement change'),sg('excerpt',f'The launch statement for {t} was withdrawn. This notice concerns the content of that launch statement.')]
        axis='contact_route_vs_propositional_content';reason='Withdrawing a contact address is neutral about launch; withdrawing the target launch statement is not.'
    elif n==16:
        p=ex('We withdraw the download link associated with the spare-parts diagram. This action concerns that diagram link alone.')
        q=ex(f'We withdraw the launch statement associated with {t}. This action concerns that launch statement.')
        axis='diagram_link_vs_launch_statement';reason='The positive action removes a link artifact rather than a launch proposition; the negative explicitly withdraws the proposition.'
    elif n==17:
        p=[sg('source_title','Archive administrator message'),sg('excerpt','The table-of-contents rows were reordered. The administrator also changed the page-number typography.')]
        q=[sg('source_title','Author current message'),sg('excerpt',f'I retract the launch assertion printed here about {t}. This message concerns that launch assertion.')]
        axis='contents_reindexing_vs_author_retraction';reason='Contents order and typography are neutral; the current first-person retraction names the target launch.'
    elif n==18:
        p=ex('A navigation link for the warranty policy has been withdrawn. The notice concerns that navigation link, rather than a product-history statement.')
        q=ex(f'The launch assertion for {t} has been withdrawn. The notice concerns that product-history statement, rather than a navigation link.')
        axis='navigation_artifact_vs_history_assertion';reason='The direct withdrawal object distinguishes a navigation artifact from the target launch fact.'
    elif n==19:
        p=ex(f'The publisher of {OTHER} sends a current message: "I deny that {OTHER} was launched." The message is about {OTHER}.')
        q=ex(f'The publisher of {t} sends a current message: "I deny that {t} was launched." The message is about {t}.')
        axis='named_publisher_current_quote_owner';reason='The quoted denial itself names its product, independently of the publisher attribution.'
    elif n==20:
        p=ex(f'We retract {OTHER}\'s launch assertion, which appeared beside the record about {t}. The retracted assertion belongs to {OTHER}.')
        q=ex(f'We retract {t}\'s launch assertion, which appeared beside the record about {OTHER}. The retracted assertion belongs to {t}.')
        axis='possessive_owner_with_neighbor_reference';reason='The operative possessive explicitly identifies the launch owner; the neighboring record is context, not a reference-resolution requirement.'
    elif n==21:
        p=[sg('source_title','Insert containing the author current reply'),sg('excerpt',f'On the topic of {OTHER}\'s launch, I withdraw the previous statement. The withdrawn launch statement is about {OTHER}.')]
        q=[sg('source_title','Insert containing the author current reply'),sg('excerpt',f'On the topic of {t}\'s launch, I withdraw the previous statement. The withdrawn launch statement is about {t}.')]
        axis='topic_and_final_owner_in_author_reply';reason='Both the topic phrase and final sentence explicitly identify the owner of the withdrawn launch statement.'
    elif n==22:
        common=[sg('heading',f'Discussion covers {t}, {OTHER}, and {THIRD}')]
        p=common+ex(f'Editor: Which property is corrected? Author: We deny that {t} has a clockwise-only spindle. This denial concerns spindle direction.')
        q=common+ex('Editor: Which launch claim is corrected? Author: We deny one launch claim in this discussion. Our reply does not identify which product owns the denied claim.')
        axis='multi_product_reply_object_or_owner_missing';reason='The positive gives a direct property object for the canonical target; the negative expressly omits the launch claim owner among several products.'
    elif n==23:
        p=ex(f'I retract my temperature-calibration claim about {t}. This message revises the calibration claim; it is not a correction of a release-history event.')
        q=ex(f'I retract my launch claim about {t}. This message revises that launch claim; it is not a correction of a calibration value.')
        axis='personal_claim_and_explicit_exclusion';reason='The direct claim object and exclusion distinguish calibration from launch; no fact about calibration itself is approved.'
    elif n==24:
        p=ex(f'We deny the calibration claim about {t} discussed in this correspondence. The denial is limited to the calibration claim.')
        q=ex('We deny the launch claim discussed in this correspondence. The correspondence omits the product name of the denied launch claim.')
        axis='correspondence_denial_owner_absent';reason='The positive explicitly identifies a different property and its owner; the negative omits the launch claim owner.'
    elif n==25:
        p=[sg('source_title','Publisher letter listing enclosures'),sg('excerpt','Two enclosures accompany this letter: a glossary and a typesetting guide. The list describes the enclosed documents.')]
        q=[sg('source_title','Publisher letter listing a withdrawal'),sg('excerpt','A launch assertion in one enclosure is withdrawn. The letter and its enclosure list omit the product name for the withdrawn assertion.')]
        axis='enclosure_list_vs_unowned_withdrawal';reason='An enclosure list is neutral; an unowned launch withdrawal cannot authorize the target launch relation.'
    elif n==26:
        p=[sg('heading','Current author transcript'),sg('excerpt',f'Author: I retract the declaration about {t}\'s cord length. Transcript note: the corrected topic is cord length.')]
        q=[sg('heading','Current author transcript'),sg('excerpt','Author: I retract that launch claim. Transcript note: the product referent was lost from the recording.')]
        axis='transcript_explicit_property_or_lost_referent';reason='The positive object is explicit and non-launch; the negative explicitly records the lost product referent.'
    elif n==27:
        p=[sg('source_title','Signature page administration'),sg('excerpt','The signature page now displays the author initials. This update concerns the signature-page layout.')]
        q=[sg('source_title','Signature page current withdrawal'),sg('excerpt','I withdraw the launch announcement. This signature page omits the name of the product whose launch is withdrawn.')]
        axis='detached_signature_page_scope';reason='Signature-page layout is neutral; the present first-person launch withdrawal has an expressly omitted owner.'
    elif n==28:
        common=[sg('source_title',f'Current joint response about {t} and {OTHER}')]
        p=common+ex(f'We deny a rope-material claim for {OTHER}. This reply addresses the rope-material claim for {OTHER}.')
        q=common+ex('We retract one launch claim. Our reply does not identify which product owns that launch claim.')
        axis='joint_response_named_property_or_unknown_launch';reason='The positive explicitly names another product and property; the launch withdrawal in the negative is unassigned between products.'
    elif n==29:
        p=[sg('heading','Message about the attached illustration'),sg('excerpt','The illustration attachment now uses greyscale shading. The author message concerns illustration contrast.')]
        q=[sg('heading','Message about the attached withdrawal'),sg('excerpt','The launch statement in the attached notice is withdrawn by its author. The attachment does not identify the product of that launch statement.')]
        axis='attachment_contrast_vs_unidentified_launch';reason='Illustration contrast is neutral; an attached current launch withdrawal has no identified owner.'
    elif n==30:
        p=[sg('heading','Source-location footnote'),sg('excerpt','This footnote gives the shelf code and binding colour of the archived page. It is an indexer note.')]
        q=[sg('heading','Orphan correction footnote'),sg('excerpt','I retract the launch assertion mentioned here. The product name is omitted from this correction footnote.')]
        axis='indexer_footnote_vs_orphan_retraction';reason='The source-location note is neutral; an explicitly owner-missing launch retraction requires withholding.'
    for polarity,context in [('positive',p),('negative',q)]:
        headers=[x for x in context if x['kind']!='excerpt'];notes=[x for x in context if x['kind']=='excerpt']
        signals=headers+(notes+[fact] if n in (3,10,14) else [fact]+notes)
        cid=f'li5-{len(cases)+1:03d}'
        inp={'id':cid,'policy':{'policy_id':'independent-v5-launch-policy','target_id':f'v5-fictional-target-{n:02d}','target_question':f'Was {t} launched?','entity':{'canonical_id':f'v5-fictional-target-{n:02d}','canonical_name':t,'aliases':[]},'relation':'change_or_launch','identity_requirement':'require_harness_anchor','assessment_budget':BUDGET.copy()},'candidate':{'evidence_id':cid+'-evidence','source_id':cid+'-synthetic-source','signals':signals},'proposal':{'target_binding':'exact','relation_binding':'exact'},'raw':{'identity_scope':'exact_target','relation_scope':'requested_relation','scope_risk':'none'}}
        cases.append({'id':cid,'pair_id':f'v5-pair-{n:02d}','polarity':polarity,'axis':axis,'expected':{'v17_requested_relation':polarity=='positive','v30_relevant':polarity=='positive'},'rationale':reason,'input':inp})
for name,value in [('cases.json',{'schema':'engine073-launch-independent-v5','author':'independent evaluator /root/engine07_realworld_independent','cases':cases}),('inputs.json',[c['input'] for c in cases])]:
    with (ROOT/name).open('x') as f:json.dump(value,f,ensure_ascii=False,indent=2);f.write('\n')
print('newly_authored_cases',len(cases))
