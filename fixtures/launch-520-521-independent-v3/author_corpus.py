"""Independent v3: new constructions after fixed candidate identity; no runtime/root tests read."""
import hashlib
import json
from pathlib import Path

OUT = Path(__file__).resolve().parent
cases = []

def add(case_id, target, category, signals, positive=False, aliases=()):
    cases.append({
        'case_id': case_id,
        'category': category,
        'label': 'positive' if positive else 'negative',
        'policy': {
            'policy_id': 'independent-v3-launch-policy',
            'target_id': 'v3-target-' + case_id,
            'target_question': 'When was ' + target + ' launched?',
            'entity': {'canonical_id': 'v3-entity-' + case_id, 'canonical_name': target, 'aliases': list(aliases)},
            'relation': 'change_or_launch',
            'identity_requirement': 'require_harness_anchor',
            'assessment_budget': {'max_model_attempts': 2, 'max_tokens': 192, 'max_elapsed_ms': 90000},
        },
        'candidate': {
            'evidence_id': 'v3-evidence-' + case_id,
            'source_id': 'v3-source-' + case_id,
            'signals': [{'kind': kind, 'text': text} for kind, text in signals],
        },
        'proposal': {'target_binding': 'exact', 'relation_binding': 'exact'},
        'raw': {'identity_scope': 'exact_target', 'relation_scope': 'requested_relation', 'scope_risk': 'none'},
        'expected': {
            'v17_relation_scope': 'requested_relation' if positive else 'not_requested_relation',
            'v30_disposition': 'relevant' if positive else 'not_relevant',
            'claim_scope': 'source_local_only',
        },
    })

# Positives stay within the declared bounded launch/GA frames; each has actual factual-signal proof.
add('P301', 'Rhovena Workbench', 'launched_iso_date', [('fact', 'Rhovena Workbench was launched on 2031-04-17. Archive status: published.')], True)
add('P302', 'Embrilo Patch', 'has_been_launched', [('excerpt', 'Embrilo Patch has been launched; the release record gives 18 May 2032 as the release date.')], True)
add('P303', 'Kantora Canvas', 'introduced_month_day_date', [('excerpt', 'Kantora Canvas was introduced on June 19, 2033.')], True)
add('P304', 'Vespali Core', 'went_live', [('structured_metadata', 'Vespali Core went live on 20 May 2034.')], True)
add('P305', 'Draleno Loop', 'rolled_out', [('fact', 'Draleno Loop was rolled out on 21 May 2035.')], True)
add('P306', 'Pavessa Thread', 'became_generally_available', [('excerpt', 'Pavessa Thread became generally available on 22 May 2036.')], True)
add('P307', 'Volriso Board', 'is_now_generally_available', [('excerpt', 'Volriso Board is now generally available as of 23 May 2037.')], True)
add('P308', 'Zemrali Port', 'entered_ga_iso_date', [('structured_metadata', 'Zemrali Port entered general availability on 2038-05-24.')], True)
add('P309', 'Firvesa Clock', 'reached_ga_citation_adjunct', [('excerpt', 'Firvesa Clock reached GA on 25 May 2039, according to the release record.')], True)
add('P310', 'Orelvo Queue', 'actor_has_made_target_generally_available', [('excerpt', 'On 26 May 2040, Norlima Systems has made Orelvo Queue generally available.')], True)
add('P311', 'Arveola Shelf', 'alias_introduced', [('fact', 'AVS was introduced on 27 May 2041.')], True, ['AVS'])
add('P312', 'Orli.Dev Quanta 8.5', 'dotted_vendor_decimal_went_live', [('excerpt', 'Orli.Dev Quanta 8.5 went live on 28 May 2042.')], True)
add('P313', 'Zeromil 9.4', 'decimal_rolled_out', [('excerpt', 'Zeromil 9.4 was rolled out on 29 May 2043.')], True)
add('P314', 'Froveta Cabinet', 'other_target_denied_introduction', [('excerpt', 'Dalvero Capsule was never introduced in April. Froveta Cabinet was introduced on 30 May 2044.')], True)
add('P315', 'Lyrvano Loom', 'same_target_feature_question_before_launch', [('excerpt', 'How many connectors does Lyrvano Loom support? Lyrvano Loom was launched on 31 May 2045.')], True)
add('P316', 'Marveta Mirror', 'neutral_title_then_proof', [('source_title', 'Interviews about subscription plans'), ('fact', 'Marveta Mirror went live on 1 May 2046.')], True)
add('P317', 'Irvano Ledger', 'explicit_other_target_fiction_heading', [('heading', 'Fictional introduction account for Cormeta Flask'), ('excerpt', 'Irvano Ledger was introduced on 2 May 2047.')], True)
add('P318', 'Pelvano Capsule', 'other_target_retracted_heading_with_predicate', [('heading', 'Retracted: Sovreta Signal went live on 1 May 2048.'), ('structured_metadata', 'Pelvano Capsule was rolled out on 3 May 2048.')], True)
add('P319', 'Nevrola Cloud', 'same_target_crossed_feature_statement', [('excerpt', '✗ Nevrola Cloud offers a monthly plan. Nevrola Cloud became generally available on 4 May 2049.')], True)
add('P320', 'Kerveta Folio', 'same_target_logical_not_feature_statement', [('excerpt', '¬ (Kerveta Folio supports image export). Kerveta Folio was introduced on 5 May 2050.')], True)
add('P321', 'Rolvano Beam', 'same_target_withdrawn_battery_heading', [('heading', 'Withdrawn battery-life claim: Rolvano Beam lasts eight days'), ('fact', 'Rolvano Beam went live on 6 May 2051.')], True)
add('P322', 'Velmora Parcel', 'rolled_out_bounded_citation_adjunct', [('excerpt', 'Velmora Parcel was rolled out on 7 May 2052, according to the release record.')], True)
add('P323', 'Ilveta Gallery', 'clear_same_target_deictic_subject', [('excerpt', 'Ilveta Gallery is the product covered by this record. It was launched on 8 May 2053.')], True)
add('P324', 'Vormilo Relay', 'actor_proof_with_different_product_denial', [('excerpt', 'Helveta Systems has made Vormilo Relay generally available on 9 May 2054. Olvoro Systems has not made Garventa Feed generally available.')], True)
add('P325', 'Ralmora Cradle', 'navigation_markers_without_authority', [('navigation_or_footer', '❌ Fictional dispatches; withdrawn dates'), ('structured_metadata', 'Ralmora Cradle reached GA on 10 May 2055.')], True)
other='Nervoli Cell supports offline search.'
add('P326', 'Erveta Harbor', 'combining_strike_other_feature_then_launch', [('excerpt', ''.join(c+'\u0335' if not c.isspace() else c for c in other)+' Erveta Harbor was launched on 11 May 2056.')], True)
add('P327', 'Dravilo Thread', 'other_target_question_then_fact', [('excerpt', 'Was Querveta Kite introduced before April? Dravilo Thread entered general availability on 12 May 2057.')], True)
add('P328', 'Almivora Switch', 'same_target_withdrawn_feature_prose', [('excerpt', 'The earlier claim that Almivora Switch includes a scanner was withdrawn. Almivora Switch entered general availability on 13 May 2058.')], True)

# Negative statements exercise target ownership, deictic ownership, original markers and governing authority.
add('N301', 'Sevrila Bridge', 'fictional_dispatch_heading_introduction', [('heading', 'Fictional dispatch'), ('excerpt', 'Sevrila Bridge was introduced on 14 May 2060.')])
add('N302', 'Pelmora Switch', 'bare_retraction_title_went_live', [('source_title', 'Retraction'), ('fact', 'Pelmora Switch went live on 15 May 2061.')])
add('N303', 'Corvilo Spindle', 'denied_account_leading_rollout', [('excerpt', 'The following is a denied launch account. Corvilo Spindle was rolled out on 16 May 2062.')])
add('N304', 'Derveta Parcel', 'introduction_record_retracted_trailing', [('excerpt', 'Derveta Parcel was introduced on 17 May 2063. This introduction record has been retracted.')])
add('N305', 'Falrino Gallery', 'fictional_record_after_citation', [('excerpt', 'Falrino Gallery was launched on 18 May 2064, according to the release record. Correction: that release record is fictional.')])
add('N306', 'Ovriela Pond', 'hypothetical_dispatch_leading', [('excerpt', 'This dispatch is hypothetical. Ovriela Pond became generally available on 19 May 2065.')])
add('N307', 'Lervilo Ring', 'struck_launch_then_unstruck_feature', [('excerpt', 'Publication draft. ~~Lervilo Ring went live on 20 May 2066. The record was prepared for distribution.~~ Lervilo Ring includes a help viewer.')])
add('N308', 'Herveta Flask', 'struck_title_governs_introduction', [('source_title', '~~Herveta Flask introduction record~~'), ('fact', 'Herveta Flask was introduced on 21 May 2067.')])
add('N309', 'Kavriela Route', 'cross_suffix_introduction', [('excerpt', 'Kavriela Route was introduced on 22 May 2068 ❌')])
add('N310', 'Norveta Frame', 'logical_not_went_live', [('excerpt', '¬ Norveta Frame went live on 23 May 2069.')])
add('N311', 'Zavriela Note', 'not_equal_generally_available', [('excerpt', 'Zavriela Note ≠ generally available on 24 May 2070.')])
add('N312', 'Telvilo Trace', 'heavy_cross_rollout', [('fact', '✘ Telvilo Trace was rolled out on 25 May 2071.')])
intro='introduced'
add('N313', 'Relveta Studio', 'combining_strike_introduced_predicate', [('excerpt', 'Relveta Studio was '+''.join(c+'\u0335' for c in intro)+' on 26 May 2072.')])
add('N314', 'Auvrino Shelf', 'crossed_fact_neutral_metadata', [('fact', '✗ Auvrino Shelf has been launched on 27 May 2073.'), ('structured_metadata', 'Auvrino Shelf documentation format: HTML')])
add('N315', 'Kelmori Cab', 'introduction_question', [('excerpt', 'Was Kelmori Cab introduced on 28 May 2074?')])
add('N316', 'Merveta Prism', 'went_live_conditional', [('excerpt', 'If the final approval arrived, Merveta Prism went live on 29 May 2075.')])
add('N317', 'Lorvilo Rill', 'rollout_negation', [('excerpt', 'Lorvilo Rill was not rolled out on 30 May 2076.')])
add('N318', 'Seraveta Labs', 'actor_not_launched_object_is_other_product', [('excerpt', 'Seraveta Labs has made Pervilo Tab generally available on 31 May 2077.')])
add('N319', 'Kavrino Patch', 'deictic_subject_is_explicitly_other_product', [('heading', 'Kavrino Patch comparison notes'), ('excerpt', 'Zerveta Slate is the product covered by this record. It was launched on 1 May 2078.')])
add('N320', 'Barvilo Grove', 'unconfirmed_introduction_rumor', [('excerpt', 'An unconfirmed rumor reports that Barvilo Grove was introduced on 2 May 2079.')])
add('N321', 'Olveta Systems', 'introducing_actor_is_not_introduced_product', [('excerpt', 'Olveta Systems introduced Tervilo Drop on 3 May 2080.')])
add('N322', 'Elvori Dial', 'heading_only_went_live', [('heading', 'Elvori Dial went live on 4 May 2081.'), ('fact', 'The release record has an appendix with Elvori Dial system requirements.')])
add('N323', 'Danveta Cove', 'url_footer_only_rollout', [('canonical_url', 'https://example.invalid/Danveta-Cove-was-rolled-out-2082-05-05'), ('navigation_or_footer', 'Danveta Cove was rolled out on 5 May 2082.')])
add('N324', 'Peravilo Pod', 'quoted_launch_explicitly_denied', [('excerpt', 'The record says "Peravilo Pod was launched on 6 May 2083". This claim is denied.')])
add('N325', 'Jerveta Span', 'conditional_with_citation_adjunct', [('excerpt', 'If the contract was signed, Jerveta Span was launched on 7 May 2084, according to the release record.')])
add('N326', 'Molvori Flag', 'actor_has_not_made_target_available', [('excerpt', 'Dervilo Systems has not made Molvori Flag generally available on 8 May 2085.')])
add('N327', 'Pavriela Hatch', 'unstruck_feature_struck_launch_same_target', [('excerpt', 'Pavriela Hatch includes a timeline view. ~~Pavriela Hatch was launched on 9 May 2086.~~')])
add('N328', 'Kolveta Beam', 'false_heading_same_target_generally_available', [('heading', 'False claim: Kolveta Beam became generally available'), ('excerpt', 'Kolveta Beam was launched on 10 May 2087.')])

payload = {
    'schema_version': 3,
    'author_identity': '/root/launch_072_independent',
    'candidate_commit': 'fdd630e1085ab3994e3397de0646741313cfd15c',
    'candidate_source_path': 'crates/reasoning-harness-core/src/evidence_relevance.rs',
    'candidate_source_sha256': '63493029537f8e66762332e118090abbbce5fdf1481e22b7df57124ab12b2c32',
    'authority_contract': 'Factual proof only from excerpt/fact/structured_metadata; heading/title govern only; URL/navigation cannot prove or govern.',
    'cases': cases,
}
raw=(json.dumps(payload, ensure_ascii=False, indent=2)+'\n').encode()
(OUT/'corpus.json').write_bytes(raw)
manifest={
    'author_identity': payload['author_identity'],
    'candidate_commit': payload['candidate_commit'],
    'candidate_source_sha256': payload['candidate_source_sha256'],
    'corpus_sha256': hashlib.sha256(raw).hexdigest(),
    'total_cases': len(cases),
    'positive_cases': sum(c['label']=='positive' for c in cases),
    'negative_cases': sum(c['label']=='negative' for c in cases),
    'case_ids': [c['case_id'] for c in cases],
    'safety_gate': 'zero negative RequestedRelation; zero false Relevant; zero API errors',
    'positive_recall_gate': 'zero positive misses; zero API errors',
}
(OUT/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(json.dumps(manifest,indent=2))
