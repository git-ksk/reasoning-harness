"""Fresh v4 input specification within the clarified direct statement ownership contract."""
import hashlib
import json
from pathlib import Path
OUT=Path(__file__).resolve().parent
cases=[]

def add(case_id,target,category,signals,positive=False,aliases=()):
    cases.append({
        'case_id':case_id,'category':category,'label':'positive' if positive else 'negative',
        'policy':{
            'policy_id':'independent-v4-direct-launch-policy','target_id':'v4-target-'+case_id,
            'target_question':'When was '+target+' launched?',
            'entity':{'canonical_id':'v4-canonical-'+case_id,'canonical_name':target,'aliases':list(aliases)},
            'relation':'change_or_launch','identity_requirement':'require_harness_anchor',
            'assessment_budget':{'max_model_attempts':2,'max_tokens':192,'max_elapsed_ms':90000},
        },
        'candidate':{'evidence_id':'v4-evidence-'+case_id,'source_id':'v4-source-'+case_id,
                     'signals':[{'kind':kind,'text':text} for kind,text in signals]},
        'proposal':{'target_binding':'exact','relation_binding':'exact'},
        'raw':{'identity_scope':'exact_target','relation_scope':'requested_relation','scope_risk':'none'},
        'expected':{'v17_relation_scope':'requested_relation' if positive else 'not_requested_relation',
                    'v30_disposition':'relevant' if positive else 'not_relevant','claim_scope':'source_local_only'},
    })

# Every positive owns its predicate in the same statement by canonical name or policy-declared alias.
add('P401','Orymena Compass','date_first_passive_launch', [('excerpt','On 12 May 2150, Orymena Compass was launched.')], True)
add('P402','Siventa Helix','active_launch_as_of', [('fact','Siventa Helix launched as of 13 May 2151.')], True)
add('P403','Avriona Desk','has_been_launched_record_adjunct', [('excerpt','Avriona Desk has been launched on 14 May 2152, according to the release record.')], True)
add('P404','Carvilo Horizon','introduced_neutral_leading_adjunct', [('excerpt','In the published chronology, Carvilo Horizon was introduced on 15 May 2153.')], True)
add('P405','Galventa Portal','introduced_direct_object', [('fact','Berori Devices introduced Galventa Portal on 16 May 2154.')], True)
add('P406','Mivrona Harbor','went_live_as_of', [('structured_metadata','Mivrona Harbor went live as of 17 May 2155.')], True)
add('P407','Fervena Scope','active_rollout', [('excerpt','Fervena Scope rolled out on 18 May 2156.')], True)
add('P408','Valrino Vessel','passive_rollout_as_of', [('fact','Valrino Vessel was rolled out as of 19 May 2157.')], True)
add('P409','Dorvena Gate','generally_available_as_of', [('excerpt','Dorvena Gate is generally available as of 20 May 2158.')], True)
add('P410','Zirona Pattern','now_generally_available', [('structured_metadata','Zirona Pattern is now generally available as of 21 May 2159.')], True)
add('P411','Olevra Panel','became_generally_available', [('fact','Olevra Panel became generally available on 22 May 2160.')], True)
add('P412','Varnilo Canvas','has_made_direct_object_as_of', [('excerpt','Pelora Systems has made Varnilo Canvas generally available as of 23 May 2161.')], True)
add('P413','Belvona Field','hyphenated_policy_alias_ga', [('fact','BV-Field entered general availability on 24 May 2162.')], True, ['BV-Field'])
add('P414','Cerona.Dev Kit 3.9','dotted_decimal_reached_ga', [('excerpt','Cerona.Dev Kit 3.9 reached GA on 25 May 2163.')], True)
add('P415','Morliva Dial','launched_direct_object', [('excerpt','Alvori Instruments launched Morliva Dial on 26 May 2164.')], True)
add('P416','Helvona Finch','same_target_why_feature_question', [('excerpt','Why does Helvona Finch use a dial? Helvona Finch was introduced on 27 May 2165.')], True)
add('P417','Telriva Kite','same_target_false_material_property', [('excerpt','The statement that Telriva Kite uses magnesium is false. Telriva Kite went live on 28 May 2166.')], True)
add('P418','Corlena Beacon','closed_property_strike_before_semicolon_launch', [('excerpt','~~Corlena Beacon offers a blue finish~~; Corlena Beacon was launched on 29 May 2167.')], True)
add('P419','Lurvona Shell','crossed_other_target_then_named_rollout', [('excerpt','❌ Raveli Disk was introduced on 28 May 2168.\nLurvona Shell was rolled out on 30 May 2168.')], True)
add('P420','Serniva Orchard','fiction_heading_explicit_other_target', [('heading','Fictional summary for Brevori Dial'), ('fact','Serniva Orchard reached GA on 31 May 2169.')], True)
add('P421','Mirvena Watch','denied_other_target_title', [('source_title','Correction: Govrila Deck was not introduced in April.'), ('excerpt','Mirvena Watch entered general availability on 1 May 2170.')], True)
add('P422','Averona Desk','retracted_language_property_heading', [('heading','Retracted claim: Averona Desk supports seven languages'), ('fact','Averona Desk was rolled out on 2 May 2171.')], True)
add('P423','Novrila Brick','url_marker_cannot_veto_alias_fact', [('canonical_url','https://release.example.invalid/withdrawn/novrila-brick'), ('fact','NBrick reached GA on 3 May 2172.')], True, ['NBrick'])
add('P424','Zalvena Cabin','navigation_not_governing_introduction', [('navigation_or_footer','¬ Fictional notices and denied rollout dates'), ('structured_metadata','Zalvena Cabin was introduced on 4 May 2173.')], True)
add('P425','Ermona Channel','heading_feature_question_not_launch_question', [('heading','Does Ermona Channel include a timer?'), ('excerpt','Ermona Channel was launched on 5 May 2174.')], True)
add('P426','Alriva.Tools Patch 4.8','metadata_direct_name_and_record_adjunct', [('source_title','Alriva.Tools Patch 4.8 archival index'), ('structured_metadata','Alriva.Tools Patch 4.8 was rolled out on 6 May 2175, according to the release record.')], True)

# Negative controls have optimistic model claims but no valid ungoverned same-statement factual proof.
add('N401','Pavrona Arc','fictional_briefing_leading', [('excerpt','This briefing is fictional. Pavrona Arc was introduced on 7 May 2180.')])
add('N402','Delvena Light','hypothetical_bulletin_trailing', [('excerpt','Delvena Light went live on 8 May 2181. This bulletin is hypothetical.')])
add('N403','Berona Route','false_heading_actor_launch', [('heading','False report'), ('excerpt','Nolori Devices launched Berona Route on 9 May 2182.')])
add('N404','Ivrila Cone','named_denied_title_governs_ga', [('source_title','Denied: Ivrila Cone availability claim'), ('structured_metadata','Ivrila Cone reached GA on 10 May 2183.')])
add('N405','Rolvona Badge','retraction_following_statement_heading', [('heading','Retraction of the following statement'), ('fact','Rolvona Badge was rolled out on 11 May 2184.')])
add('N406','Uvrina Layer','withdrawn_description_trailing', [('excerpt','Uvrina Layer entered general availability on 12 May 2185. This description was withdrawn.')])
add('N407','Jelvena Grid','hypothetical_next_paragraph', [('excerpt','The next paragraph is hypothetical. Jelvena Grid has been launched on 13 May 2186.')])
add('N408','Yarona Chart','fictional_plural_entries', [('heading','These entries are fictional'), ('fact','Yarona Chart became generally available on 14 May 2187.')])
add('N409','Civrila Signal','cross_suffix_before_period', [('excerpt','Civrila Signal was launched on 15 May 2188 ❌.')])
add('N410','Kovrena Lens','logical_not_alias_introduction', [('fact','¬ KVL was introduced on 16 May 2189.')], aliases=['KVL'])
add('N411','Malvona Unit','not_equal_reached_ga', [('excerpt','Malvona Unit ≠ reached GA on 17 May 2190.')])
add('N412','Tervina Rack','ballot_cross_heading_claim', [('heading','✗ Tervina Rack launch record'), ('fact','Tervina Rack launched on 18 May 2191.')])
add('N413','Gelmona Deck','heavy_cross_title_claim', [('source_title','✘ Gelmona Deck rollout notice'), ('excerpt','Gelmona Deck was rolled out on 19 May 2192.')])
claim='Osriva Ring is now generally available as of 20 May 2193.'
add('N414','Osriva Ring','combining_full_claim_stroke', [('excerpt',''.join(c+'\u0336' if not c.isspace() else c for c in claim))])
add('N415','Zevrona Stack','strike_spans_fact_and_following_sentence', [('excerpt','~~Release entry: Zevrona Stack entered general availability on 21 May 2194. The archive then listed a migration guide.~~')])
add('N416','Nevlina Scroll','unclosed_original_markdown_strike', [('excerpt','~~Nevlina Scroll was introduced on 22 May 2195.')])
add('N417','Farvona Trace','rollout_if_condition', [('excerpt','If the supplier accepted the contract, Farvona Trace was rolled out on 23 May 2196.')])
add('N418','Virena Loom','now_generally_available_question', [('excerpt','Is Virena Loom now generally available as of 24 May 2197?')])
add('N419','Carona Folder','has_made_negation', [('excerpt','Volriva Labs has not made Carona Folder generally available as of 25 May 2198.')])
add('N420','Lelmona Capsule','reached_ga_unconfirmed_rumor', [('excerpt','Unconfirmed rumor: Lelmona Capsule reached GA on 26 May 2199.')])
add('N421','Nervona Industries','launching_actor_is_not_launched_target', [('excerpt','Nervona Industries launched Hilriva Field on 27 May 2200.')])
add('N422','Solrena Relay','cross_sentence_pronoun_with_named_record', [('excerpt','This release record names Solrena Relay as its subject. It was rolled out on 28 May 2201.')])
add('N423','Gavrila Wave','affirmative_heading_without_factual_claim', [('heading','Gavrila Wave is now generally available as of 29 May 2202.'), ('fact','Gavrila Wave installation guide: 12 pages.')])
add('N424','Elvona Vessel','affirmative_title_url_navigation_only', [('source_title','Elvona Vessel went live on 30 May 2203.'), ('canonical_url','https://example.invalid/elvona-vessel/went-live'), ('navigation_or_footer','Elvona Vessel availability chronology')])
add('N425','Pervina Circle','url_and_footer_only_named_claim', [('canonical_url','https://example.invalid/Pervina-Circle/launched/2204-05-31'), ('navigation_or_footer','Pervina Circle has been launched on 31 May 2204.')])
add('N426','Telmona Rail','target_property_other_product_introduction', [('excerpt','Telmona Rail has a matte finish. Rovrila Grove was introduced on 1 May 2205.')])

payload={
    'schema_version':4,'author_identity':'/root/launch_072_independent',
    'candidate_commit':'48ac1ba8aba655aa73d192962d7996bcc997c5e0',
    'candidate_source_path':'crates/reasoning-harness-core/src/evidence_relevance.rs',
    'candidate_source_sha256':'2b32a1e4a721cb169b15420e113ad91808476380ff912c12b46c029f8f81684d',
    'positive_owner_rule':'policy canonical name or declared alias owns predicate in the same actual statement; no cross-sentence pronoun bridging',
    'cases':cases,
}
raw=(json.dumps(payload,ensure_ascii=False,indent=2)+'\n').encode()
(OUT/'corpus.json').write_bytes(raw)
manifest={
    'author_identity':payload['author_identity'],'candidate_commit':payload['candidate_commit'],
    'candidate_source_sha256':payload['candidate_source_sha256'],'corpus_sha256':hashlib.sha256(raw).hexdigest(),
    'total_cases':len(cases),'positive_cases':sum(c['label']=='positive' for c in cases),'negative_cases':sum(c['label']=='negative' for c in cases),
    'case_ids':[c['case_id'] for c in cases],
    'safety_gate':'zero negative RequestedRelation; zero false Relevant; zero API errors',
    'positive_recall_gate':'zero positive misses; zero API errors',
}
(OUT/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(json.dumps(manifest,indent=2))
