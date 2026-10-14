"""Fresh independent v9 inputs within the existing bounded local launch contract."""
import hashlib
import json
from pathlib import Path
OUT=Path(__file__).resolve().parent
cases=[]

def add(case_id,target,category,signals,positive=False,aliases=()):
    cases.append({
        'case_id':case_id,'category':category,'label':'positive' if positive else 'negative',
        'policy':{
            'policy_id':'independent-v9-bounded-launch-acceptance','target_id':'v9-target-'+case_id,
            'target_question':'When was '+target+' launched?',
            'entity':{'canonical_id':'v9-canonical-'+case_id,'canonical_name':target,'aliases':list(aliases)},
            'relation':'change_or_launch','identity_requirement':'require_harness_anchor',
            'assessment_budget':{'max_model_attempts':2,'max_tokens':192,'max_elapsed_ms':90000},
        },
        'candidate':{'evidence_id':'v9-evidence-'+case_id,'source_id':'v9-source-'+case_id,
                     'signals':[{'kind':kind,'text':text} for kind,text in signals]},
        'proposal':{'target_binding':'exact','relation_binding':'exact'},
        'raw':{'identity_scope':'exact_target','relation_scope':'requested_relation','scope_risk':'none'},
        'expected':{'v17_relation_scope':'requested_relation' if positive else 'not_requested_relation',
                    'v30_disposition':'relevant' if positive else 'not_relevant','claim_scope':'source_local_only'},
    })

# Positives use only direct same-statement named/declared-alias proof and supported release predicates.
add('P901','Wenlora Gate','passive_launch_neutral_title', [('source_title','Published chronology'), ('fact','Wenlora Gate was launched on 1 May 2450.')], True)
add('P902','Asvella Rack','date_first_actor_object_launch', [('excerpt','On 2 May 2451, Nelmora Labs launched Asvella Rack. The release record includes a user guide.')], True)
add('P903','Teronda Leaf','has_been_launched_as_of', [('structured_metadata','Teronda Leaf has been launched as of 3 May 2452.')], True)
add('P904','Hovella Dial','passive_introduction_fact', [('fact','Hovella Dial was introduced on 4 May 2453.')], True)
add('P905','Marnora Cabin','introduced_alias_direct_object', [('excerpt','Zenvella Systems introduced MNCab on 5 May 2454.')], True, ['MNCab'])
add('P906','Dolonda Prism','went_live_as_of_neutral_followup', [('excerpt','Dolonda Prism went live as of 6 May 2455. The manual is available in the archive.')], True)
add('P907','Ivella Map','active_rollout_metadata', [('structured_metadata','Ivella Map rolled out on 7 May 2456.')], True)
add('P908','Kernora Circle','passive_rollout_record_adjunct', [('fact','Kernora Circle was rolled out on 8 May 2457, according to the release record.')], True)
add('P909','Molvella Spool','is_generally_available_as_of', [('excerpt','Molvella Spool is generally available as of 9 May 2458.')], True)
add('P910','Pernora Path','is_now_generally_available', [('fact','Pernora Path is now generally available as of 10 May 2459.')], True)
add('P911','Zovella Loom','became_generally_available_metadata', [('structured_metadata','Zovella Loom became generally available on 11 May 2460.')], True)
add('P912','Alnora Beam','actor_has_made_named_product', [('excerpt','Fervella Tools has made Alnora Beam generally available as of 12 May 2461.')], True)
add('P913','Osvella Hatch','entered_general_availability_record_adjunct', [('excerpt','Osvella Hatch entered general availability on 13 May 2462, according to the release record.')], True)
add('P914','Yernora Panel','reached_ga_neutral_heading', [('heading','Product history'), ('fact','Yernora Panel reached GA on 14 May 2463.')], True)
add('P915','Mora.Dev Unit 4.2','dotted_decimal_direct_subject', [('excerpt','Mora.Dev Unit 4.2 was launched on 15 May 2464.')], True)
add('P916','Senlora Port','alias_direct_object_launched', [('fact','Tarvella Devices launched SLPort on 16 May 2465.')], True, ['SLPort'])
add('P917','Noronda Array','separate_same_target_support_question', [('excerpt','Does Noronda Array support PDF export? Noronda Array was introduced on 17 May 2466.')], True)
add('P918','Velonda Box','has_a_property_adjacent_false_statement', [('excerpt','Velonda Box has a help panel. This statement is false. Velonda Box went live on 18 May 2467.')], True)
add('P919','Geronda Trace','separate_negated_included_feature', [('excerpt','Geronda Trace does not include a timer. Geronda Trace reached GA on 19 May 2468.')], True)
add('P920','Kelonda Watch','closed_strike_nonlaunch_property', [('excerpt','~~Kelonda Watch supports offline mode.~~ Kelonda Watch entered general availability on 20 May 2469.')], True)
add('P921','Alvella Crest','crossed_other_target_then_factual_proof', [('fact','❌ Maronda Shelf was introduced on 19 May 2470.'), ('excerpt','Alvella Crest was rolled out on 21 May 2470.')], True)
add('P922','Neronda Module','fictional_foreign_active_launch_heading', [('heading','Fictional claim: Barvella Tile launched on 20 May 2471.'), ('fact','Neronda Module was introduced on 22 May 2471.')], True)
add('P923','Zerlora Board','denied_foreign_went_live_title', [('source_title','Denied statement: Toronda Field went live on 21 May 2472.'), ('excerpt','Zerlora Board was launched on 23 May 2472.')], True)
add('P924','Jervella Grid','nonfactual_sources_cannot_govern', [('source_title','Release reference index'), ('canonical_url','https://example.invalid/withdrawn/jervella-grid'), ('navigation_or_footer','Fictional and hypothetical reports'), ('structured_metadata','Jervella Grid is generally available as of 24 May 2473.')], True)
add('P925','Dernora Light','property_question_heading_alias_ga_proof', [('heading','Does Dernora Light include an editor?'), ('fact','DNLight reached GA on 25 May 2474.')], True, ['DNLight'])

# Negatives preserve governing scope over more than the last foreign claim and retain original marker/owner floors.
add('N901','Cerlora Ring','report_fiction_leading', [('excerpt','This report is fictional. Cerlora Ring was launched on 1 May 2480.')])
add('N902','Vernora Key','entire_passage_hypothesis_with_foreign_name', [('source_title','This entire passage is hypothetical. Gervella Loop was introduced on 2 May 2481.'), ('fact','Vernora Key went live on 3 May 2481.')])
add('N903','Avonda Stack','plural_launch_claims_retracted_after_foreign_statement', [('excerpt','Avonda Stack reached GA on 4 May 2482. Sorvella Module was launched on 5 May 2482. These launch claims were retracted.')])
add('N904','Merlora Channel','all_following_statements_false_two_products', [('heading','The following statements are false'), ('excerpt','Merlora Channel was introduced on 6 May 2483. Farlora Hub entered general availability on 7 May 2483.')])
add('N905','Irvella Mirror','report_denial_after_feature', [('excerpt','Irvella Mirror was rolled out on 8 May 2484. Irvella Mirror supports PDF export. This report is denied.')])
add('N906','Povella Patch','withdrawn_availability_heading', [('heading','Withdrawn availability statement'), ('structured_metadata','Povella Patch became generally available on 9 May 2485.')])
add('N907','Terlora Frame','named_launch_claim_denied', [('excerpt','The launch claim for Terlora Frame is denied. Terlora Frame has been launched on 10 May 2486.')])
add('N908','Orvella Slate','conditional_actor_introduced_object', [('excerpt','If the agreement was approved, Halnora Labs introduced Orvella Slate on 11 May 2487.')])
add('N909','Erlora Relay','passive_rollout_question', [('excerpt','Was Erlora Relay rolled out on 12 May 2488?')])
add('N910','Zarnora Wheel','unconfirmed_ga_rumor', [('excerpt','Unconfirmed rumor: Zarnora Wheel entered general availability on 13 May 2489.')])
add('N911','Nelonda Cove','actor_negated_has_made_object', [('fact','Virlora Systems has not made Nelonda Cove generally available on 14 May 2490.')])
add('N912','Beronda Studio','launch_actor_is_not_launched_target', [('excerpt','Beronda Studio launched Dolvella Desk on 15 May 2491.')])
add('N913','Kerlora Bay','heading_target_other_product_owner', [('heading','Kerlora Bay release record'), ('fact','Telvella Bay was introduced on 16 May 2492.')])
add('N914','Salnora Pad','named_record_pronoun_predicate', [('excerpt','The product named by this record is Salnora Pad. It reached GA on 17 May 2493.')])
add('N915','Caronda Disk','heading_only_affirmative_ga', [('heading','Caronda Disk is now generally available as of 18 May 2494.'), ('excerpt','Caronda Disk supports offline mode.')])
add('N916','Lernora Scope','title_only_launch_neutral_metadata', [('source_title','Lernora Scope was launched on 19 May 2495.'), ('structured_metadata','Lernora Scope manual format: PDF')])
add('N917','Nevlora Reel','url_footer_only_launch', [('canonical_url','https://example.invalid/Nevlora-Reel/launched/2496-05-20'), ('navigation_or_footer','Nevlora Reel was launched on 20 May 2496.')])
add('N918','Falvella Port','strike_spans_multiple_release_sentences', [('excerpt','~~Release record. Falvella Port went live on 21 May 2497. The archive includes a guide.~~')])
add('N919','Darlora Leaf','cross_prefix_ga', [('fact','❌ Darlora Leaf reached GA on 22 May 2498.')])
add('N920','Zeronda Dock','not_equal_launch_predicate', [('excerpt','Zeronda Dock ≠ launched on 23 May 2499.')])
add('N921','Rerlora Prism','logical_not_introduced', [('excerpt','¬ Rerlora Prism was introduced on 24 May 2500.')])
add('N922','Moronda Mote','ballot_cross_rollout', [('fact','✗ Moronda Mote was rolled out on 25 May 2501.')])
add('N923','Peronda Grid','heavy_cross_governing_heading', [('heading','✘ Peronda Grid launch claim'), ('excerpt','Peronda Grid has been launched on 26 May 2502.')])
claim='ZRMap entered general availability on 27 May 2503.'
add('N924','Zorlora Map','combining_strike_declared_alias_claim', [('excerpt',''.join(c+'\u0336' if not c.isspace() else c for c in claim))], aliases=['ZRMap'])
add('N925','Ternora Field','struck_target_launch_unstruck_features_and_foreign_launch', [('excerpt','Ternora Field includes a timer. ~~Ternora Field was launched on 28 May 2504.~~ Saronda Reel was introduced on 29 May 2504.')])

payload={
 'schema_version':9,'author_identity':'/root/launch_072_independent',
 'candidate_commit':'f70d1ada8bd56767af4256c82122b01eb60a9d23',
 'candidate_source_path':'crates/reasoning-harness-core/src/evidence_relevance.rs',
 'candidate_source_sha256':'6b32d628cf731977b0f637b99646485d7f57d8fc3d9f9fa62b9a5432d09a34a6',
 'positive_owner_rule':'same-statement canonical or declared alias direct subject/object in supported bounded predicates',
 'cases':cases,
}
raw=(json.dumps(payload,ensure_ascii=False,indent=2)+'\n').encode();(OUT/'corpus.json').write_bytes(raw)
manifest={
 'author_identity':payload['author_identity'],'candidate_commit':payload['candidate_commit'],
 'candidate_source_sha256':payload['candidate_source_sha256'],'corpus_sha256':hashlib.sha256(raw).hexdigest(),
 'total_cases':len(cases),'positive_cases':sum(c['label']=='positive' for c in cases),'negative_cases':sum(c['label']=='negative' for c in cases),
 'case_ids':[c['case_id'] for c in cases],
 'safety_gate':'zero negative RequestedRelation; zero false Relevant; zero API errors',
 'positive_recall_gate':'zero positive misses; zero API errors',
}
(OUT/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n');print(json.dumps(manifest,indent=2))
