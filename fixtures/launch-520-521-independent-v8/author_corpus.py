"""Independent v8: fresh bounded direct-owner and local/global context acceptance corpus."""
import hashlib
import json
from pathlib import Path
OUT=Path(__file__).resolve().parent
cases=[]

def add(case_id,target,category,signals,positive=False,aliases=()):
    cases.append({
        'case_id':case_id,'category':category,'label':'positive' if positive else 'negative',
        'policy':{
            'policy_id':'independent-v8-local-global-launch-policy','target_id':'v8-target-'+case_id,
            'target_question':'When was '+target+' launched?',
            'entity':{'canonical_id':'v8-canonical-'+case_id,'canonical_name':target,'aliases':list(aliases)},
            'relation':'change_or_launch','identity_requirement':'require_harness_anchor',
            'assessment_budget':{'max_model_attempts':2,'max_tokens':192,'max_elapsed_ms':90000},
        },
        'candidate':{'evidence_id':'v8-evidence-'+case_id,'source_id':'v8-source-'+case_id,
                     'signals':[{'kind':kind,'text':text} for kind,text in signals]},
        'proposal':{'target_binding':'exact','relation_binding':'exact'},
        'raw':{'identity_scope':'exact_target','relation_scope':'requested_relation','scope_risk':'none'},
        'expected':{'v17_relation_scope':'requested_relation' if positive else 'not_requested_relation',
                    'v30_disposition':'relevant' if positive else 'not_relevant','claim_scope':'source_local_only'},
    })

# Direct same-statement owner proof within declared predicates, plus genuinely local non-launch/foreign contexts.
add('P801','Evanto Beam','passive_launch_record_adjunct_neutral_followup', [('excerpt','Evanto Beam was launched on 1 May 2390, according to the release record. The archive includes the manual.')], True)
add('P802','Evranto Shelf','date_first_actor_launches_named_object', [('fact','On 2 May 2391, Calbora Labs launched Evranto Shelf.')], True)
add('P803','Kibora Ring','passive_introduction_factual_signal', [('fact','Kibora Ring was introduced on 3 May 2392.')], True)
add('P804','Savanto Tile','introduced_declared_alias_object', [('excerpt','Belbora Systems introduced SVTile on 4 May 2393.')], True, ['SVTile'])
add('P805','Feranto Clock','has_been_launched_date', [('structured_metadata','Feranto Clock has been launched on 5 May 2394.')], True)
add('P806','Nivanto Gate','went_live_as_of_neutral_heading', [('heading','Availability dates'), ('excerpt','Nivanto Gate went live as of 6 May 2395.')], True)
add('P807','Parbora Map','active_rollout_fact', [('fact','Parbora Map rolled out on 7 May 2396.')], True)
add('P808','Ravanto Cradle','passive_rollout_neutral_followup', [('excerpt','Ravanto Cradle was rolled out on 8 May 2397. The manual lists system requirements.')], True)
add('P809','Selbora Leaf','is_generally_available_as_of', [('structured_metadata','Selbora Leaf is generally available as of 9 May 2398.')], True)
add('P810','Varanto Grid','is_now_generally_available', [('excerpt','Varanto Grid is now generally available as of 10 May 2399.')], True)
add('P811','Ovbora Pulse','became_generally_available_record_adjunct', [('fact','Ovbora Pulse became generally available on 11 May 2400, according to the release record.')], True)
add('P812','Melbora Trace','actor_has_made_direct_object', [('excerpt','Tervanto Labs has made Melbora Trace generally available on 12 May 2401.')], True)
add('P813','Irvanto Board','entered_general_availability_with_neutral_title', [('source_title','Product archive'), ('excerpt','Irvanto Board entered general availability on 13 May 2402.')], True)
add('P814','Nerbora Port','reached_ga_in_metadata', [('structured_metadata','Nerbora Port reached GA on 14 May 2403.')], True)
add('P815','Avo.Tools Loom 9.2','dotted_decimal_declared_alias_launch', [('fact','ATL92 was launched on 15 May 2404.')], True, ['ATL92'])
add('P816','Nolbora Desk','separate_feature_count_question', [('excerpt','How many timers does Nolbora Desk include? Nolbora Desk went live on 16 May 2405.')], True)
add('P817','Elbora Prism','has_an_property_adjacent_false_statement', [('excerpt','Elbora Prism has an export panel. That property statement is false. Elbora Prism was introduced on 17 May 2406.')], True)
add('P818','Dervanto Cabin','separate_negated_feature_before_ga', [('excerpt','Dervanto Cabin does not support PDF export. Dervanto Cabin entered general availability on 18 May 2407.')], True)
add('P819','Volbora Field','struck_same_target_included_feature', [('excerpt','~~Volbora Field includes a navigation panel.~~ Volbora Field reached GA on 19 May 2408.')], True)
add('P820','Tavanto Key','marked_other_target_launch_then_target_proof', [('excerpt','❌ Rilbora Array was launched on 18 May 2409. Tavanto Key was launched on 20 May 2409.')], True)
add('P821','Avbora Relay','fictional_foreign_passive_introduction_title', [('source_title','Fictional statement: Serlano Prism was introduced on 19 May 2410.'), ('fact','Avbora Relay became generally available on 21 May 2410.')], True)
add('P822','Jervanto Hatch','denied_foreign_active_launch_heading', [('heading','Denied launch claim: Merlino Kite launched on 20 May 2411.'), ('excerpt','Jervanto Hatch was rolled out on 22 May 2411.')], True)
add('P823','Velbora Wheel','hypothetical_foreign_active_rollout_title', [('source_title','Hypothetical statement: Torlano Grid rolled out on 21 May 2412.'), ('fact','Velbora Wheel went live on 23 May 2412.')], True)
add('P824','Kervanto Box','url_and_footer_without_governing_authority', [('canonical_url','https://example.invalid/fictional/kervanto-box'), ('navigation_or_footer','Denied launch claims'), ('structured_metadata','Kervanto Box has been launched as of 24 May 2413.')], True)
add('P825','Alvbora Module','feature_question_heading_and_object_proof', [('heading','Does Alvbora Module support PDF export?'), ('fact','Nervanto Labs launched Alvbora Module on 25 May 2414.')], True)

# Negatives: global authority survives a foreign name; original markers and strict owner/proof floors remain intact.
add('N801','Laranto Path','all_following_fiction_governs_despite_foreign_subject', [('heading','All following launch statements are fictional: Corlano Ring launched on 1 May 2420.'), ('excerpt','Laranto Path was introduced on 2 May 2420.')])
add('N802','Nelbora Bay','whole_report_hypothesis_with_foreign_launch_sentence', [('source_title','This entire report is hypothetical. Calvanto Board went live on 2 May 2421.'), ('fact','Nelbora Bay reached GA on 3 May 2421.')])
add('N803','Rilvanto Panel','plural_statements_retracted_after_two_products', [('excerpt','Rilvanto Panel was launched on 4 May 2422. Sorbora Cove was introduced on 5 May 2422. These statements were retracted.')])
add('N804','Toranto Signal','false_launch_heading_direct_proof', [('heading','False launch statement'), ('fact','Toranto Signal has been launched on 6 May 2423.')])
add('N805','Kalbora Hatch','named_denied_launch_before_affirmative', [('excerpt','The launch claim about Kalbora Hatch is denied. Kalbora Hatch went live on 7 May 2424.')])
add('N806','Gervanto Spool','report_fiction_after_launch_and_feature', [('excerpt','Gervanto Spool became generally available on 8 May 2425. Gervanto Spool supports exports. This report is fictional.')])
add('N807','Dalbora Brick','withdrawn_claim_heading', [('heading','Withdrawn availability claim'), ('structured_metadata','Dalbora Brick entered general availability on 9 May 2426.')])
add('N808','Viranto Mote','cross_prefix_introduction', [('fact','❌ Viranto Mote was introduced on 10 May 2427.')])
add('N809','Berbora Thread','not_equal_reached_ga', [('excerpt','Berbora Thread ≠ reached GA on 11 May 2428.')])
add('N810','Talvanto Array','logical_not_has_been_launched', [('excerpt','¬ Talvanto Array has been launched on 12 May 2429.')])
add('N811','Serbora Cabin','ballot_cross_rollout_suffix', [('fact','Serbora Cabin was rolled out on 13 May 2430 ✗')])
add('N812','Olivanto Patch','heavy_cross_governs_heading_claim', [('heading','✘ Olivanto Patch general availability record'), ('excerpt','Olivanto Patch reached GA on 14 May 2431.')])
add('N813','Pavbora Mirror','strike_scope_across_release_sentences', [('excerpt','~~Draft entry. Pavbora Mirror is now generally available as of 15 May 2432. The guide has a troubleshooting appendix.~~')])
claim='Devanto Crest was introduced on 16 May 2433.'
add('N814','Devanto Crest','combining_strike_full_introduction', [('excerpt',''.join(c+'\u0336' if not c.isspace() else c for c in claim))])
add('N815','Ulvbora Frame','struck_launch_unstruck_same_target_property', [('excerpt','~~Ulvbora Frame was launched on 17 May 2434.~~ Ulvbora Frame has an export panel.')])
add('N816','Roranto Finch','actor_object_introduction_question', [('excerpt','Did Zalbora Labs introduce Roranto Finch on 18 May 2435?')])
add('N817','Elvanto Line','conditional_actor_has_made_target_available', [('excerpt','If the license was approved, Melvanto Systems has made Elvanto Line generally available on 19 May 2436.')])
add('N818','Harbora Scope','unconfirmed_introduction_rumor', [('excerpt','Unconfirmed rumor: Harbora Scope was introduced on 20 May 2437.')])
add('N819','Zeranto Reel','actor_did_not_launch_object', [('fact','Darlano Devices did not launch Zeranto Reel on 21 May 2438.')])
add('N820','Varlano Labs','actor_target_introduces_other_product', [('excerpt','Varlano Labs introduced Terbora Tile on 22 May 2439.')])
add('N821','Alranto Beam','heading_target_other_direct_factual_owner', [('heading','Alranto Beam release archive'), ('fact','Morvanto Beam was rolled out on 23 May 2440.')])
add('N822','Perbora Dial','named_record_then_pronoun_only_predicate', [('excerpt','The record names Perbora Dial as the product. It went live on 24 May 2441.')])
add('N823','Fervanto Shelf','heading_only_ga_feature_excerpt', [('heading','Fervanto Shelf entered general availability on 25 May 2442.'), ('excerpt','Fervanto Shelf includes a timer.')])
add('N824','Yelbora Rack','title_only_launch_neutral_metadata', [('source_title','Yelbora Rack was launched on 26 May 2443.'), ('structured_metadata','Yelbora Rack documentation format: HTML')])
add('N825','Cerlano Note','url_navigation_only_release_assertion', [('canonical_url','https://example.invalid/Cerlano-Note/rolled-out/2444-05-27'), ('navigation_or_footer','Cerlano Note was rolled out on 27 May 2444.')])

payload={
 'schema_version':8,'author_identity':'/root/launch_072_independent',
 'candidate_commit':'bcf65d9b2f0edb88c05780e4322a633c720d8a8a',
 'candidate_source_path':'crates/reasoning-harness-core/src/evidence_relevance.rs',
 'candidate_source_sha256':'69f3e3116ae0a887cf5b83401d2355485edc5dd877a983c94587c3f1cbb00322',
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
