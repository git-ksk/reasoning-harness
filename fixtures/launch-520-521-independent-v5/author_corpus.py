"""Fresh independent v5: bounded public contract verification, without implementation/root-test reads."""
import hashlib
import json
from pathlib import Path
OUT=Path(__file__).resolve().parent
cases=[]

def add(case_id,target,category,signals,positive=False,aliases=()):
    cases.append({
        'case_id':case_id,'category':category,'label':'positive' if positive else 'negative',
        'policy':{
            'policy_id':'independent-v5-bounded-launch-policy','target_id':'v5-target-'+case_id,
            'target_question':'When was '+target+' launched?',
            'entity':{'canonical_id':'v5-canonical-'+case_id,'canonical_name':target,'aliases':list(aliases)},
            'relation':'change_or_launch','identity_requirement':'require_harness_anchor',
            'assessment_budget':{'max_model_attempts':2,'max_tokens':192,'max_elapsed_ms':90000},
        },
        'candidate':{'evidence_id':'v5-evidence-'+case_id,'source_id':'v5-source-'+case_id,
                     'signals':[{'kind':kind,'text':text} for kind,text in signals]},
        'proposal':{'target_binding':'exact','relation_binding':'exact'},
        'raw':{'identity_scope':'exact_target','relation_scope':'requested_relation','scope_risk':'none'},
        'expected':{'v17_relation_scope':'requested_relation' if positive else 'not_requested_relation',
                    'v30_disposition':'relevant' if positive else 'not_relevant','claim_scope':'source_local_only'},
    })

# Positives: explicit same-statement canonical/declared-alias ownership and already declared predicates.
add('P501','Drenova Prism','passive_launch_with_neutral_followup', [('excerpt','Drenova Prism was launched on 8 May 2210. The release record also links a manual.')], True)
add('P502','Salmira Atlas','active_launch_factual_signal', [('fact','Salmira Atlas launched on 9 May 2211.')], True)
add('P503','Turvane Pad','launched_product_object', [('excerpt','Falmera Labs launched Turvane Pad on 10 May 2212.')], True)
add('P504','Vilmora Desk','introduced_alias_object', [('fact','Orvane Systems introduced VMD on 11 May 2213.')], True, ['VMD'])
add('P505','Kelmina Reach','passive_introduction_with_neutral_source_title', [('source_title','Published release record'), ('excerpt','Kelmina Reach was introduced on 12 May 2214.')], True)
add('P506','Garniva Tile','has_been_launched_record_adjunct', [('structured_metadata','Garniva Tile has been launched on 13 May 2215, according to the release record.')], True)
add('P507','Velmina Rack','went_live_as_of', [('fact','Velmina Rack went live as of 14 May 2216.')], True)
add('P508','Ravmira Gate','active_rollout_with_neutral_followup', [('excerpt','Ravmira Gate rolled out on 15 May 2217. The documentation is indexed by topic.')], True)
add('P509','Elvane Loop','date_first_passive_rollout', [('excerpt','On 16 May 2218, Elvane Loop was rolled out.')], True)
add('P510','Sernova Panel','generally_available_as_of', [('structured_metadata','Sernova Panel is generally available as of 17 May 2219.')], True)
add('P511','Pelmina Map','now_generally_available_with_neutral_heading', [('heading','Availability notes'), ('fact','Pelmina Map is now generally available as of 18 May 2220.')], True)
add('P512','Lorniva Field','became_generally_available', [('excerpt','Lorniva Field became generally available on 19 May 2221.')], True)
add('P513','Ulvane Track','entered_general_availability', [('fact','Ulvane Track entered general availability on 20 May 2222.')], True)
add('P514','Vorniva Cove','reached_ga_declared_alias', [('excerpt','VCove reached GA on 21 May 2223.')], True, ['VCove'])
add('P515','Nelmira Hatch','has_made_product_available', [('excerpt','Dervane Studio has made Nelmira Hatch generally available on 22 May 2224.')], True)
add('P516','Teri.Dev Slate 2.6','dotted_decimal_introduction', [('fact','Teri.Dev Slate 2.6 was introduced on 23 May 2225.')], True)
add('P517','Avmina Signal','same_target_support_question_then_launch', [('excerpt','Does Avmina Signal support exports? Avmina Signal was launched on 24 May 2226.')], True)
add('P518','Zerniva Beacon','same_target_language_question_then_ga', [('excerpt','How many languages does Zerniva Beacon support? Zerniva Beacon entered general availability on 25 May 2227.')], True)
add('P519','Mernova Badge','false_feature_claim_before_actual_launch', [('excerpt','Mernova Badge supports face unlock. That feature claim is false. Mernova Badge went live on 26 May 2228.')], True)
add('P520','Almina Port','negative_feature_statement_then_actual_rollout', [('excerpt','Almina Port does not include a scanner. Almina Port was rolled out on 27 May 2229.')], True)
add('P521','Irvane Brick','closed_strike_on_same_target_feature_only', [('excerpt','~~Irvane Brick includes a fingerprint reader.~~ Irvane Brick has been launched on 28 May 2230.')], True)
add('P522','Dravmira Ring','denied_different_target_title', [('source_title','Denied launch claim for Solmera Dock'), ('fact','Dravmira Ring became generally available on 29 May 2231.')], True)
add('P523','Quelmira Frame','struck_different_target_then_factual_introduction', [('excerpt','~~Hervane Switch was rolled out on 28 May 2232.~~ Quelmira Frame was introduced on 30 May 2232.')], True)
add('P524','Braniva Route','url_and_navigation_have_no_governing_authority', [('canonical_url','https://archive.example.invalid/fictional/braniva-route'), ('navigation_or_footer','Withdrawn claims and hypothetical release notes'), ('structured_metadata','Braniva Route reached GA on 31 May 2233.')], True)
add('P525','Eri.Tools Bay 5.7','declared_alias_metadata_under_neutral_title', [('source_title','Eri.Tools Bay 5.7 documentation index'), ('structured_metadata','ETBay was launched on 1 May 2234, according to the release record.')], True, ['ETBay'])

# Negatives: ordinary governing context, plural references, original markers and owner/authority boundaries.
add('N501','Jarniva Grid','fictional_heading_governs_launch', [('heading','Fictional release record'), ('fact','Jarniva Grid was launched on 2 May 2240.')])
add('N502','Melvane Shard','plural_fictional_claims_title', [('source_title','The following claims are fictional'), ('excerpt','Melvane Shard was introduced on 3 May 2241.')])
add('N503','Dalmina Line','plural_hypothetical_launch_claims_leading', [('excerpt','These launch claims are hypothetical. Dalmina Line went live on 4 May 2242.')])
add('N504','Corvane Byte','plural_statements_retracted_trailing', [('excerpt','Corvane Byte was rolled out on 5 May 2243. The release record lists a migration guide. These statements were retracted.')])
add('N505','Narmira Stack','false_release_statement_leading', [('excerpt','The following release statement is false. Narmira Stack entered general availability on 6 May 2244.')])
add('N506','Filmina Cabin','denied_launch_statement_trailing', [('excerpt','Filmina Cabin has been launched on 7 May 2245. This launch statement is denied.')])
add('N507','Remvane Channel','withdrawn_heading_governs_ga', [('heading','Withdrawn claim'), ('structured_metadata','Remvane Channel reached GA on 8 May 2246.')])
add('N508','Zalmira Wheel','hypothetical_heading_neutral_title', [('source_title','Availability record'), ('heading','Hypothetical release statement'), ('fact','Zalmira Wheel is now generally available as of 9 May 2247.')])
add('N509','Celmira Hub','cross_prefix_on_launch', [('excerpt','❌ Celmira Hub launched on 10 May 2248.')])
add('N510','Ormina Trace','logical_not_introduction', [('fact','¬ Ormina Trace was introduced on 11 May 2249.')])
add('N511','Dervina Fold','not_equal_alias_rollout', [('excerpt','DFold ≠ rolled out on 12 May 2250.')], aliases=['DFold'])
add('N512','Mavmina Peak','ballot_cross_suffix_ga', [('excerpt','Mavmina Peak became generally available on 13 May 2251 ✗')])
add('N513','Belvane Veil','heavy_cross_heading_governs_fact', [('heading','✘ Belvane Veil availability claim'), ('fact','Belvane Veil reached GA on 14 May 2252.')])
claim='Lermira Module was launched on 15 May 2253.'
add('N514','Lermira Module','combining_short_stroke_full_launch_claim', [('excerpt',''.join(c+'\u0335' if not c.isspace() else c for c in claim))])
add('N515','Zovmina Beam','strike_across_sentences_other_unstruck_product', [('excerpt','~~Release record. Zovmina Beam entered general availability on 16 May 2254.~~ Ravmera Flag was launched on 17 May 2254.')])
add('N516','Tarvane Scope','unstruck_feature_but_struck_launch', [('excerpt','Tarvane Scope supports exports. ~~Tarvane Scope was rolled out on 18 May 2255.~~')])
add('N517','Yelmira Cloud','has_made_target_negation', [('excerpt','Pelmane Labs has not made Yelmira Cloud generally available on 19 May 2256.')])
add('N518','Garmira Axis','introduction_condition', [('excerpt','If the contract was approved, Garmira Axis was introduced on 20 May 2257.')])
add('N519','Nelvane Kernel','reached_ga_question', [('excerpt','Did Nelvane Kernel reach GA on 21 May 2258?')])
add('N520','Alvmina Reel','unconfirmed_launch_rumor', [('excerpt','Unconfirmed rumor: Alvmina Reel was launched on 22 May 2259.')])
add('N521','Pervane Studio','introducing_actor_not_product', [('fact','Pervane Studio introduced Delmera Core on 23 May 2260.')])
add('N522','Zelmira Shell','target_heading_other_factual_owner', [('heading','Zelmira Shell release notes'), ('excerpt','Corvena Shell went live on 24 May 2261.')])
add('N523','Savmina Archive','pronoun_launch_after_named_record', [('excerpt','The product named in this record is Savmina Archive. It entered general availability on 25 May 2262.')])
add('N524','Kelmane Pad','url_navigation_only_affirmative', [('canonical_url','https://example.invalid/Kelmane-Pad/reached-ga/2263-05-26'), ('navigation_or_footer','Kelmane Pad reached GA on 26 May 2263.')])
add('N525','Velniva Disk','source_title_only_launch_assertion', [('source_title','Velniva Disk was launched on 27 May 2264.'), ('excerpt','The Velniva Disk manual contains a setup checklist.')])

payload={
 'schema_version':5,'author_identity':'/root/launch_072_independent',
 'candidate_commit':'c6e23e13f66503508c3055a0d0c528b38f72e920',
 'candidate_source_path':'crates/reasoning-harness-core/src/evidence_relevance.rs',
 'candidate_source_sha256':'3cfb333a44b8924511d4794f537665d723b9512f66861bfd268f47ea92fae127',
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
