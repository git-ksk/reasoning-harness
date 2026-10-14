"""Fresh independent v10 corpus: bounded owner proof and fail-closed adverse-cue scope."""
import hashlib
import json
from pathlib import Path
OUT=Path(__file__).resolve().parent
cases=[]

def add(case_id,target,category,signals,positive=False,aliases=()):
    cases.append({
        'case_id':case_id,'category':category,'label':'positive' if positive else 'negative',
        'policy':{
            'policy_id':'independent-v10-bounded-scope-acceptance','target_id':'v10-target-'+case_id,
            'target_question':'When was '+target+' launched?',
            'entity':{'canonical_id':'v10-canonical-'+case_id,'canonical_name':target,'aliases':list(aliases)},
            'relation':'change_or_launch','identity_requirement':'require_harness_anchor',
            'assessment_budget':{'max_model_attempts':2,'max_tokens':192,'max_elapsed_ms':90000},
        },
        'candidate':{'evidence_id':'v10-evidence-'+case_id,'source_id':'v10-source-'+case_id,
                     'signals':[{'kind':kind,'text':text} for kind,text in signals]},
        'proposal':{'target_binding':'exact','relation_binding':'exact'},
        'raw':{'identity_scope':'exact_target','relation_scope':'requested_relation','scope_risk':'none'},
        'expected':{'v17_relation_scope':'requested_relation' if positive else 'not_requested_relation',
                    'v30_disposition':'relevant' if positive else 'not_relevant','claim_scope':'source_local_only'},
    })

# Explicitly owned factual proof; clear foreign/property-local adverse scope must not erase it.
add('P1001','Kavento Vale','passive_launch_neutral_followup', [('excerpt','Kavento Vale was launched on 1 May 2510. The manual contains installation instructions.')], True)
add('P1002','Orelina Frame','date_first_actor_object_launch', [('fact','On 2 May 2511, Navelto Labs launched Orelina Frame.')], True)
add('P1003','Tivento Leaf','actor_introduced_named_object', [('excerpt','Belinto Systems introduced Tivento Leaf on 3 May 2512.')], True)
add('P1004','Marelina Gate','declared_alias_direct_launch_subject', [('fact','MGate launched on 4 May 2513.')], True, ['MGate'])
add('P1005','Dovento Desk','passive_introduction_metadata', [('structured_metadata','Dovento Desk was introduced on 5 May 2514.')], True)
add('P1006','Elinto Prism','has_been_launched_as_of', [('excerpt','Elinto Prism has been launched as of 6 May 2515.')], True)
add('P1007','Parelina Loop','went_live_as_of_neutral_title', [('source_title','Availability timeline'), ('fact','Parelina Loop went live as of 7 May 2516.')], True)
add('P1008','Zivento Patch','active_rollout_metadata', [('structured_metadata','Zivento Patch rolled out on 8 May 2517.')], True)
add('P1009','Larelina Bay','passive_rollout_record_adjunct', [('excerpt','Larelina Bay was rolled out on 9 May 2518, according to the release record.')], True)
add('P1010','Servento Dial','generally_available_as_of', [('fact','Servento Dial is generally available as of 10 May 2519.')], True)
add('P1011','Novento Ring','now_generally_available_metadata', [('structured_metadata','Novento Ring is now generally available as of 11 May 2520.')], True)
add('P1012','Carinto Track','became_generally_available_neutral_followup', [('excerpt','Carinto Track became generally available on 12 May 2521. The release record links a guide.')], True)
add('P1013','Varelina Slate','actor_has_made_named_product', [('excerpt','Yervento Tools has made Varelina Slate generally available on 13 May 2522.')], True)
add('P1014','Fervento Shelf','entered_general_availability_fact', [('fact','Fervento Shelf entered general availability on 14 May 2523.')], True)
add('P1015','Arel.Dev Board 5.3','dotted_decimal_reached_ga', [('excerpt','Arel.Dev Board 5.3 reached GA on 15 May 2524.')], True)
add('P1016','Narelina Cove','declared_alias_under_neutral_source_structure', [('source_title','Reference archive'), ('heading','Product release record'), ('structured_metadata','NCove was introduced on 16 May 2525.')], True, ['NCove'])
add('P1017','Torinto Mote','separate_feature_support_question', [('excerpt','Does Torinto Mote support PDF export? Torinto Mote entered general availability on 17 May 2526.')], True)
add('P1018','Alvento Port','has_a_property_adjacent_false_claim', [('excerpt','Alvento Port has a help panel. That property claim is false. Alvento Port was launched on 18 May 2527.')], True)
add('P1019','Serlina Key','separate_negative_feature', [('excerpt','Serlina Key does not include an editor. Serlina Key went live on 19 May 2528.')], True)
add('P1020','Darelina Module','closed_strike_same_target_feature_only', [('excerpt','~~Darelina Module includes a timer.~~ Darelina Module reached GA on 20 May 2529.')], True)
add('P1021','Ivinto Watch','crossed_foreign_launch_then_object_introduction', [('excerpt','❌ Morento Desk was launched on 19 May 2530. Farelina Labs introduced Ivinto Watch on 21 May 2530.')], True)
add('P1022','Urelina Wheel','fictional_foreign_active_launch_heading', [('heading','Fictional statement: Zarelto Bay launched on 20 May 2531.'), ('fact','Urelina Wheel was rolled out on 22 May 2531.')], True)
add('P1023','Kelinto Spool','denied_foreign_passive_introduction_title', [('source_title','Denied claim: Vervento Gate was introduced on 21 May 2532.'), ('excerpt','Kelinto Spool became generally available on 23 May 2532.')], True)
add('P1024','Ravento Grid','url_footer_no_governing_authority', [('canonical_url','https://example.invalid/retracted/ravento-grid'), ('navigation_or_footer','Fictional release records'), ('fact','Ravento Grid has been launched on 24 May 2533, according to the release record.')], True)
add('P1025','Orelto Light','feature_question_heading_actor_object_ga', [('heading','Does Orelto Light include a timer?'), ('structured_metadata','Tarinto Systems has made Orelto Light generally available as of 25 May 2534.')], True)

# Known adverse cues with unresolved scope fail closed; broad/collection cues are not narrowed to foreign owner.
add('N1001','Barelina Field','whole_report_fiction', [('excerpt','This report is fictional. Barelina Field was introduced on 1 May 2540.')])
add('N1002','Dervento Crest','entire_passage_hypothesis_with_foreign_subject', [('source_title','This entire passage is hypothetical. Calinto Frame went live on 2 May 2541.'), ('fact','Dervento Crest reached GA on 3 May 2541.')])
add('N1003','Nerinto Array','plural_retraction_after_two_release_claims', [('excerpt','Nerinto Array was launched on 4 May 2542. Salvento Rack entered general availability on 5 May 2542. These statements were retracted.')])
add('N1004','Parinto Cabin','unresolved_subset_false_claims', [('excerpt','Parinto Cabin went live on 6 May 2543. Elvento Shelf was introduced on 7 May 2543. Some of these claims are false.')])
add('N1005','Vervina Beam','unresolved_retracted_reference', [('excerpt','Vervina Beam entered general availability on 8 May 2544. Qarelina Hub was launched on 9 May 2544. Something above was retracted.')])
add('N1006','Alrinto Reel','unresolved_one_fictional_statement', [('excerpt','Alrinto Reel was rolled out on 10 May 2545. Harvento Cove reached GA on 11 May 2545. One of the statements is fictional.')])
add('N1007','Zarelina Board','named_launch_claim_denied_before_proof', [('excerpt','The launch claim for Zarelina Board is denied. Zarelina Board has been launched on 12 May 2546.')])
add('N1008','Lervento Ring','withdrawn_availability_title', [('source_title','Withdrawn availability statement'), ('structured_metadata','Lervento Ring is now generally available as of 13 May 2547.')])
add('N1009','Cerinto Pattern','conditional_actor_launch_object', [('excerpt','If the approval was granted, Jolvento Labs launched Cerinto Pattern on 14 May 2548.')])
add('N1010','Nelinto Dock','entered_general_availability_question', [('excerpt','Did Nelinto Dock enter general availability on 15 May 2549?')])
add('N1011','Garvento Loop','unconfirmed_rollout_rumor', [('excerpt','Unconfirmed rumor: Garvento Loop rolled out on 16 May 2550.')])
add('N1012','Porinto Channel','actor_has_not_made_object_available', [('fact','Serento Systems has not made Porinto Channel generally available on 17 May 2551.')])
add('N1013','Yarelina Labs','introducing_actor_is_not_introduced_target', [('excerpt','Yarelina Labs introduced Terinto Stone on 18 May 2552.')])
add('N1014','Corinto Desk','heading_target_other_product_factual_owner', [('heading','Corinto Desk release notes'), ('fact','Wervento Desk was launched on 19 May 2553.')])
add('N1015','Mervento Prism','named_record_pronoun_only_predicate', [('excerpt','The product named in this record is Mervento Prism. It was introduced on 20 May 2554.')])
add('N1016','Zerinto Port','heading_only_ga_with_feature_excerpt', [('heading','Zerinto Port reached GA on 21 May 2555.'), ('excerpt','Zerinto Port has a help panel.')])
add('N1017','Arelina Scope','title_only_launch_with_neutral_metadata', [('source_title','Arelina Scope was launched on 22 May 2556.'), ('structured_metadata','Arelina Scope documentation format: Markdown')])
add('N1018','Velinto Relay','url_navigation_only_positive_claim', [('canonical_url','https://example.invalid/Velinto-Relay/introduced/2557-05-23'), ('navigation_or_footer','Velinto Relay was introduced on 23 May 2557.')])
add('N1019','Kerinto Tile','markdown_strike_multi_sentence_scope', [('excerpt','~~Release record. Kerinto Tile entered general availability on 24 May 2558. The guide lists migration steps.~~')])
add('N1020','Harelina Bay','cross_prefix_launched', [('fact','❌ Harelina Bay was launched on 25 May 2559.')])
add('N1021','Tervina Pulse','not_equal_general_availability', [('excerpt','Tervina Pulse ≠ generally available as of 26 May 2560.')])
add('N1022','Merinto Frame','logical_not_rollout', [('excerpt','¬ Merinto Frame was rolled out on 27 May 2561.')])
add('N1023','Larelto Field','ballot_cross_ga_suffix', [('fact','Larelto Field reached GA on 28 May 2562 ✗')])
add('N1024','Rervento Module','heavy_cross_governing_heading', [('heading','✘ Rervento Module launch record'), ('excerpt','Rervento Module has been launched on 29 May 2563.')])
claim='FD45 went live on 30 May 2564.'
add('N1025','Fora.Dev Desk 4.5','combining_strike_declared_alias', [('excerpt',''.join(c+'\u0336' if not c.isspace() else c for c in claim))], aliases=['FD45'])

payload={
 'schema_version':10,'author_identity':'/root/launch_072_independent',
 'candidate_commit':'2f12d4c61546e60d31026d9b691c9eb369c37957',
 'candidate_source_path':'crates/reasoning-harness-core/src/evidence_relevance.rs',
 'candidate_source_sha256':'5343e722618721e58ad6a24b12c55c1272eb3c581bb3ada26b24d8596dfebbf5',
 'positive_owner_rule':'same-statement canonical or declared alias direct subject/object in supported bounded predicates',
 'adverse_scope_rule':'known negative/fiction/retraction cue with unresolved scope withholds; explicit foreign/property-local scope remains local',
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
