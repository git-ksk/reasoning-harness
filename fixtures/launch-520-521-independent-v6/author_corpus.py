"""Independent v6 authorship, constrained to public direct-owner and local-context rules."""
import hashlib
import json
from pathlib import Path
OUT=Path(__file__).resolve().parent
cases=[]

def add(case_id,target,category,signals,positive=False,aliases=()):
    cases.append({
        'case_id':case_id,'category':category,'label':'positive' if positive else 'negative',
        'policy':{
            'policy_id':'independent-v6-statement-launch-policy','target_id':'v6-target-'+case_id,
            'target_question':'When was '+target+' launched?',
            'entity':{'canonical_id':'v6-canonical-'+case_id,'canonical_name':target,'aliases':list(aliases)},
            'relation':'change_or_launch','identity_requirement':'require_harness_anchor',
            'assessment_budget':{'max_model_attempts':2,'max_tokens':192,'max_elapsed_ms':90000},
        },
        'candidate':{'evidence_id':'v6-evidence-'+case_id,'source_id':'v6-source-'+case_id,
                     'signals':[{'kind':kind,'text':text} for kind,text in signals]},
        'proposal':{'target_binding':'exact','relation_binding':'exact'},
        'raw':{'identity_scope':'exact_target','relation_scope':'requested_relation','scope_risk':'none'},
        'expected':{'v17_relation_scope':'requested_relation' if positive else 'not_requested_relation',
                    'v30_disposition':'relevant' if positive else 'not_relevant','claim_scope':'source_local_only'},
    })

# New direct-owner positives, including explicitly local references to unrelated non-launch statements.
add('P601','Cirlume Portal','passive_launch_as_of', [('fact','Cirlume Portal was launched as of 2 May 2270.')], True)
add('P602','Nolvira Stack','date_first_active_launch', [('excerpt','On 3 May 2271, Nolvira Stack launched.')], True)
add('P603','Barlumi Shelf','introduced_with_neutral_followup', [('excerpt','Barlumi Shelf was introduced on 4 May 2272. The reference manual has a glossary.')], True)
add('P604','Torvena Dock','introduced_direct_object_with_title', [('source_title','Product introduction record'), ('fact','Zelmuri Labs introduced Torvena Dock on 5 May 2273.')], True)
add('P605','Velumi Badge','went_live_as_of_in_metadata', [('structured_metadata','Velumi Badge went live as of 6 May 2274.')], True)
add('P606','Avlira Reel','active_rollout_on_date', [('fact','Avlira Reel rolled out on 7 May 2275.')], True)
add('P607','Rovlumi Channel','passive_rollout_citation_adjunct', [('excerpt','Rovlumi Channel was rolled out on 8 May 2276, according to the release record.')], True)
add('P608','Cerumi Cradle','generally_available_as_of', [('excerpt','Cerumi Cradle is generally available as of 9 May 2277.')], True)
add('P609','Helvira Scope','actor_made_named_product_available', [('fact','Morlumi Systems has made Helvira Scope generally available on 10 May 2278.')], True)
add('P610','Pervumi Canvas','became_generally_available_with_heading', [('heading','Released product timeline'), ('excerpt','Pervumi Canvas became generally available on 11 May 2279.')], True)
add('P611','Erlumi Stone','entered_general_availability_metadata', [('structured_metadata','Erlumi Stone entered general availability on 12 May 2280.')], True)
add('P612','Sovlira Cabin','reached_ga_neutral_followup', [('excerpt','Sovlira Cabin reached GA on 13 May 2281. The release record includes a checksum list.')], True)
add('P613','Ari.Dev Thread 6.2','dotted_declared_alias_subject', [('fact','Ari.Thread was launched on 14 May 2282.')], True, ['Ari.Thread'])
add('P614','Lorumi Patch 8.1','declared_alias_direct_object', [('excerpt','Nelvumi Devices launched LP81 on 15 May 2283.')], True, ['LP81'])
add('P615','Zorlira Watch','has_been_launched_as_of', [('excerpt','Zorlira Watch has been launched as of 16 May 2284.')], True)
add('P616','Darlumi Board','adjacent_feature_claim_denied', [('excerpt','Darlumi Board includes a navigation panel. That claim was denied. Darlumi Board was introduced on 17 May 2285.')], True)
add('P617','Varlira Bay','adjacent_feature_statement_retracted', [('excerpt','Varlira Bay supports PDF export. This statement was retracted. Varlira Bay entered general availability on 18 May 2286.')], True)
add('P618','Nerumi Finch','adjacent_property_claim_false', [('excerpt','Nerumi Finch has a red finish. This property claim is false. Nerumi Finch went live on 19 May 2287.')], True)
add('P619','Rilvumi Leaf','adjacent_feature_statement_hypothetical', [('excerpt','Rilvumi Leaf includes a help overlay. That statement is hypothetical. Rilvumi Leaf was rolled out on 20 May 2288.')], True)
add('P620','Ferlira Module','separate_same_target_export_question', [('excerpt','Does Ferlira Module support PDF export? Ferlira Module is now generally available as of 21 May 2289.')], True)
add('P621','Olvumi Pulse','withdrawn_other_target_heading', [('heading','Withdrawn release statement for Dalvuri Trace'), ('fact','Olvumi Pulse reached GA on 22 May 2290.')], True)
add('P622','Qerlumi Atlas','same_target_property_closed_strike', [('excerpt','~~Qerlumi Atlas offers offline search.~~ Qerlumi Atlas was launched on 23 May 2291.')], True)
add('P623','Alvumi Shelf','marked_nonlaunch_fact_then_named_launch_signal', [('fact','❌ Alvumi Shelf supports face unlock.'), ('excerpt','Alvumi Shelf went live on 24 May 2292.')], True)
add('P624','Velvuri Harbor','nonfactual_context_without_governing_authority', [('source_title','Release archive index'), ('canonical_url','https://example.invalid/retracted/velvuri-harbor'), ('navigation_or_footer','Fictional reports'), ('fact','Velvuri Harbor was introduced on 25 May 2293.')], True)
add('P625','Berlira Spindle','feature_question_heading_direct_object_metadata', [('heading','Does Berlira Spindle include a timer?'), ('structured_metadata','Torvumi Labs introduced Berlira Spindle on 26 May 2294.')], True)

# New negatives: broad governing report/passage scope, local denial of launch, original marks and ownership.
add('N601','Jerlumi Forge','broad_report_fiction_leading', [('excerpt','This report is fictional. Jerlumi Forge was introduced on 1 May 2300.')])
add('N602','Farvuri Pattern','broad_passage_hypothesis_after_feature', [('excerpt','Farvuri Pattern includes a help viewer. This passage is hypothetical. Farvuri Pattern was launched on 2 May 2301.')])
add('N603','Molumi Drum','broad_report_retraction_after_feature', [('excerpt','Molumi Drum entered general availability on 3 May 2302. Molumi Drum supports exports. This report has been retracted.')])
add('N604','Terlira Key','governing_false_heading_with_feature_interposed', [('heading','False launch claim'), ('excerpt','Terlira Key has a matte finish. Terlira Key went live on 4 May 2303.')])
add('N605','Calvumi Tide','adjacent_launch_claim_false', [('excerpt','Calvumi Tide was rolled out on 5 May 2304. That claim is false.')])
add('N606','Dervuri Slate','adjacent_launch_statement_withdrawn', [('excerpt','Dervuri Slate reached GA on 6 May 2305. This statement was withdrawn.')])
add('N607','Galumi Field','adjacent_launch_statement_fictional_after_feature', [('excerpt','Galumi Field includes an editor. Galumi Field was introduced on 7 May 2306. That statement is fictional.')])
add('N608','Sarlira Beam','broad_passage_hypothetical_title', [('source_title','This passage is hypothetical'), ('fact','Sarlira Beam is now generally available as of 8 May 2307.')])
add('N609','Kevlumi Brick','launch_if_condition', [('excerpt','If the launch license was granted, Kevlumi Brick was launched on 9 May 2308.')])
add('N610','Nelvuri Circle','direct_object_introduction_question', [('excerpt','Did Arlumi Labs introduce Nelvuri Circle on 10 May 2309?')])
add('N611','Vorlumi Parcel','generally_available_unconfirmed_rumor', [('excerpt','Unconfirmed rumor: Vorlumi Parcel became generally available on 11 May 2310.')])
add('N612','Alrlumi Deck','actor_did_not_launch_target', [('fact','Belvuri Systems did not launch Alrlumi Deck on 12 May 2311.')])
add('N613','Yerlumi Systems','actor_target_made_other_product_available', [('excerpt','Yerlumi Systems has made Rolvuri Hub generally available on 13 May 2312.')])
add('N614','Helvuri Board','heading_target_other_product_factual_owner', [('heading','Helvuri Board availability notes'), ('fact','Varlumi Board reached GA on 14 May 2313.')])
add('N615','Kelumi Ring','named_record_then_unbound_pronoun_launch', [('excerpt','This record concerns Kelumi Ring. It was launched on 15 May 2314.')])
add('N616','Cerlira Switch','source_title_only_affirmative_feature_body', [('source_title','Cerlira Switch was introduced on 16 May 2315.'), ('excerpt','Cerlira Switch includes an export menu.')])
add('N617','Zerlumi Layer','heading_only_ga_neutral_metadata', [('heading','Zerlumi Layer reached GA on 17 May 2316.'), ('structured_metadata','Zerlumi Layer manual format: Markdown')])
add('N618','Perlira Light','url_navigation_only_available_claim', [('canonical_url','https://example.invalid/Perlira-Light/general-availability/2317-05-18'), ('navigation_or_footer','Perlira Light is now generally available as of 18 May 2317.')])
add('N619','Lelvumi Scope','markdown_strike_across_three_sentences', [('excerpt','~~Publication draft. Lelvumi Scope was rolled out on 19 May 2318. A migration guide was attached.~~')])
add('N620','Morvuri Note','cross_prefix_general_availability', [('fact','❌ Morvuri Note entered general availability on 20 May 2319.')])
add('N621','Rerlumi Dial','not_equal_launch_relation', [('excerpt','Rerlumi Dial ≠ launched on 21 May 2320.')])
add('N622','Tolumi Gate','logical_not_reached_ga', [('excerpt','¬ Tolumi Gate reached GA on 22 May 2321.')])
add('N623','Narlira Loop','ballot_cross_introduction_suffix', [('fact','Narlira Loop was introduced on 23 May 2322 ✗')])
add('N624','Velumi Flask','heavy_cross_launch_heading', [('heading','✘ Velumi Flask launch claim'), ('excerpt','Velumi Flask has been launched on 24 May 2323.')])
claim='Sorlumi Dock went live on 25 May 2324.'
add('N625','Sorlumi Dock','combining_strike_target_other_product_unstruck', [('excerpt',''.join(c+'\u0336' if not c.isspace() else c for c in claim)+' Parlumi Anchor was launched on 26 May 2324.')])

payload={
 'schema_version':6,'author_identity':'/root/launch_072_independent',
 'candidate_commit':'0cf240469f5b81ccc93ecb9d8dec53e85f3cff8b',
 'candidate_source_path':'crates/reasoning-harness-core/src/evidence_relevance.rs',
 'candidate_source_sha256':'e0b4f91248ee6d4746835ecc19a9df8549b3de95733db754978b3de6f9826434',
 'positive_owner_rule':'same-statement policy canonical name or declared alias as direct subject/object; no pronoun bridging',
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
