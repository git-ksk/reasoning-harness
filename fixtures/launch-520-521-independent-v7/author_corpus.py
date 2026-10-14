"""Fresh v7 bounded grammar acceptance inputs; no implementation or root-case content read."""
import hashlib
import json
from pathlib import Path
OUT=Path(__file__).resolve().parent
cases=[]

def add(case_id,target,category,signals,positive=False,aliases=()):
    cases.append({
        'case_id':case_id,'category':category,'label':'positive' if positive else 'negative',
        'policy':{
            'policy_id':'independent-v7-bounded-acceptance','target_id':'v7-target-'+case_id,
            'target_question':'When was '+target+' launched?',
            'entity':{'canonical_id':'v7-canonical-'+case_id,'canonical_name':target,'aliases':list(aliases)},
            'relation':'change_or_launch','identity_requirement':'require_harness_anchor',
            'assessment_budget':{'max_model_attempts':2,'max_tokens':192,'max_elapsed_ms':90000},
        },
        'candidate':{'evidence_id':'v7-evidence-'+case_id,'source_id':'v7-source-'+case_id,
                     'signals':[{'kind':kind,'text':text} for kind,text in signals]},
        'proposal':{'target_binding':'exact','relation_binding':'exact'},
        'raw':{'identity_scope':'exact_target','relation_scope':'requested_relation','scope_risk':'none'},
        'expected':{'v17_relation_scope':'requested_relation' if positive else 'not_requested_relation',
                    'v30_disposition':'relevant' if positive else 'not_relevant','claim_scope':'source_local_only'},
    })

# Positive proof is direct, same-statement canonical/declared-alias ownership in existing launch/GA frames.
add('P701','Ensora Mote','passive_launch_neutral_installation_note', [('excerpt','Ensora Mote was launched on 4 May 2330. Installation notes follow the release record.')], True)
add('P702','Ulvessa Kite','has_been_launched_as_of_under_neutral_title', [('source_title','Published product timeline'), ('fact','Ulvessa Kite has been launched as of 5 May 2331.')], True)
add('P703','Aldori Pad','passive_introduction_neutral_heading', [('heading','Launch dates'), ('excerpt','Aldori Pad was introduced on 6 May 2332.')], True)
add('P704','Wesora Port','introduced_declared_alias_object', [('excerpt','Nindori Labs introduced WPort on 7 May 2333.')], True, ['WPort'])
add('P705','Delsora Watch','went_live_in_structured_metadata', [('structured_metadata','Delsora Watch went live on 8 May 2334.')], True)
add('P706','Elvori Spool','active_rollout_with_record_adjunct', [('fact','Elvori Spool rolled out on 9 May 2335, according to the release record.')], True)
add('P707','Kinsora Array','passive_rollout_as_of', [('excerpt','Kinsora Array was rolled out as of 10 May 2336.')], True)
add('P708','Pravessa Grid','is_now_generally_available', [('structured_metadata','Pravessa Grid is now generally available as of 11 May 2337.')], True)
add('P709','Tesora Board','became_generally_available_neutral_documentation_note', [('excerpt','Tesora Board became generally available on 12 May 2338. Its documentation has a separate index.')], True)
add('P710','Irdori Shelf','entered_general_availability_fact', [('fact','Irdori Shelf entered general availability on 13 May 2339.')], True)
add('P711','Avsora Loom','reached_ga_on_date', [('excerpt','Avsora Loom reached GA on 14 May 2340.')], True)
add('P712','Cendori Module','actor_has_made_object_generally_available', [('excerpt','Falvessa Systems has made Cendori Module generally available as of 15 May 2341.')], True)
add('P713','Belsora Crest','actor_launched_named_object_as_of', [('fact','Mavdori Devices launched Belsora Crest as of 16 May 2342.')], True)
add('P714','Iro.Dev Lumen 7.5','dotted_decimal_passive_introduction', [('excerpt','Iro.Dev Lumen 7.5 was introduced on 17 May 2343.')], True)
add('P715','Seldori Hub','declared_alias_has_been_launched_record_adjunct', [('fact','SRHub has been launched on 18 May 2344, according to the release record.')], True, ['SRHub'])
add('P716','Veressa Dock','neutral_title_heading_and_factual_proof', [('source_title','Version archive'), ('heading','Release timeline'), ('structured_metadata','Veressa Dock launched on 19 May 2345.')], True)
add('P717','Dorvessa Reel','separate_editor_feature_question', [('excerpt','Does Dorvessa Reel include an editor? Dorvessa Reel was launched on 20 May 2346.')], True)
add('P718','Kelvori Bay','has_an_property_adjacent_false_claim', [('excerpt','Kelvori Bay has an export menu. This claim is false. Kelvori Bay was introduced on 21 May 2347.')], True)
add('P719','Nelsora Ring','support_property_adjacent_denied_statement', [('excerpt','Nelsora Ring supports PDF export. That statement is denied. Nelsora Ring reached GA on 22 May 2348.')], True)
add('P720','Taldori Trace','included_property_adjacent_withdrawn_feature_claim', [('excerpt','Taldori Trace includes a timer. This feature claim was withdrawn. Taldori Trace was rolled out on 23 May 2349.')], True)
add('P721','Lavsora Route','closed_same_target_property_strike', [('excerpt','~~Lavsora Route offers offline mode.~~ Lavsora Route went live on 24 May 2350.')], True)
add('P722','Gelsora Field','crossed_has_an_property_then_named_launch', [('excerpt','❌ Gelsora Field has an offline editor. Gelsora Field was launched on 25 May 2351.')], True)
add('P723','Vensora Channel','fictional_heading_explicit_other_target_predicate', [('heading','Fictional release account: Peressa Brick launched on 24 May 2352.'), ('fact','Vensora Channel entered general availability on 26 May 2352.')], True)
add('P724','Zelvori Dial','url_navigation_without_authority', [('canonical_url','https://example.invalid/hypothetical/zelvori-dial'), ('navigation_or_footer','Retracted reports and fictional release dates'), ('excerpt','Zelvori Dial was introduced on 27 May 2353.')], True)
add('P725','Adessa Stack','other_target_denial_then_named_object_launch', [('excerpt','Orsora Kite was not launched in April. Veldori Systems launched Adessa Stack on 28 May 2354.')], True)

# Negatives: report-wide authority, scoped original marks, and proof/ownership boundaries.
add('N701','Birvessa Cloud','broad_report_fiction_leading', [('excerpt','This report is fictional. Birvessa Cloud reached GA on 1 May 2360.')])
add('N702','Ondori Tile','plural_hypothetical_statements_heading', [('heading','These statements are hypothetical'), ('fact','Ondori Tile was introduced on 2 May 2361.')])
add('N703','Darsora Finch','broad_passage_fiction_trailing', [('excerpt','Darsora Finch was launched on 3 May 2362. This passage is fictional.')])
add('N704','Celdori Gate','whole_report_retraction_after_feature', [('excerpt','Celdori Gate went live on 4 May 2363. Celdori Gate includes a timer. The whole report was retracted.')])
add('N705','Parsora Axis','false_claim_title_governs_object_introduction', [('source_title','False introduction claim'), ('fact','Javessa Labs introduced Parsora Axis on 5 May 2364.')])
add('N706','Fendori Board','named_launch_claim_denied_leading', [('excerpt','The launch claim for Fendori Board is denied. Fendori Board launched on 6 May 2365.')])
add('N707','Yelsora Pulse','withdrawn_release_heading_governs_ga', [('heading','Withdrawn release statement'), ('structured_metadata','Yelsora Pulse became generally available on 7 May 2366.')])
add('N708','Nelvessa Leaf','closed_strike_full_rollout', [('excerpt','~~Nelvessa Leaf was rolled out on 8 May 2367.~~')])
add('N709','Ildori Scope','unclosed_strike_after_neutral_label', [('excerpt','Archive entry: ~~Ildori Scope was introduced on 9 May 2368.')])
add('N710','Rensora Wheel','cross_prefix_launch', [('fact','❌ Rensora Wheel was launched on 10 May 2369.')])
add('N711','Kaldori Shelf','not_equal_available_predicate', [('excerpt','Kaldori Shelf ≠ generally available as of 11 May 2370.')])
add('N712','Bervori Cabin','logical_not_went_live', [('excerpt','¬ Bervori Cabin went live on 12 May 2371.')])
add('N713','Jelsora Studio','ballot_cross_reached_ga', [('fact','✗ Jelsora Studio reached GA on 13 May 2372.')])
add('N714','Torvessa Path','heavy_cross_governing_heading', [('heading','✘ Torvessa Path launch record'), ('excerpt','Torvessa Path has been launched on 14 May 2373.')])
claim='Gindori Atlas entered general availability on 15 May 2374.'
add('N715','Gindori Atlas','combining_strike_full_ga_claim', [('excerpt',''.join(c+'\u0335' if not c.isspace() else c for c in claim))])
add('N716','Lersora Patch','feature_unstruck_launch_struck', [('excerpt','Lersora Patch includes a timer. ~~Lersora Patch went live on 16 May 2375.~~')])
add('N717','Nardori Loop','target_launch_question', [('excerpt','Was Nardori Loop launched on 17 May 2376?')])
add('N718','Elsora Light','object_launch_condition', [('excerpt','If the agreement was approved, Vardori Labs launched Elsora Light on 18 May 2377.')])
add('N719','Velvessa Panel','passive_introduction_negation', [('fact','Velvessa Panel was not introduced on 19 May 2378.')])
add('N720','Pindori Prism','unconfirmed_rollout_rumor', [('excerpt','Unconfirmed rumor: Pindori Prism was rolled out on 20 May 2379.')])
add('N721','Neldori Systems','actor_target_has_made_other_product_available', [('excerpt','Neldori Systems has made Corvessa Bay generally available on 21 May 2380.')])
add('N722','Pelsora Module','cross_sentence_named_record_pronoun', [('excerpt','This record names Pelsora Module. It has been launched on 22 May 2381.')])
add('N723','Eldori Port','title_only_positive_with_feature_body', [('source_title','Eldori Port entered general availability on 23 May 2382.'), ('heading','Documentation'), ('excerpt','Eldori Port supports PDF export.')])
add('N724','Sarsora Key','url_and_footer_only_affirmative', [('canonical_url','https://example.invalid/Sarsora-Key/launched/2383-05-24'), ('navigation_or_footer','Sarsora Key was launched on 24 May 2383.')])
add('N725','Veldori Frame','heading_target_other_named_ga_owner', [('heading','Veldori Frame general availability notes'), ('fact','Dervessa Frame reached GA on 25 May 2384.')])

payload={
 'schema_version':7,'author_identity':'/root/launch_072_independent',
 'candidate_commit':'19a8faaa582c1d4af7a68972c494b2e29f72d80e',
 'candidate_source_path':'crates/reasoning-harness-core/src/evidence_relevance.rs',
 'candidate_source_sha256':'1e6293feab2c5eb7428a4427c7de9dc2d635a01fe9de14036db64085ffdde315',
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
