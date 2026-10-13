"""Fresh offline synthetic corpus, authored without reading implementation bodies or prior corpora."""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'fixtures/launch-520-521-independent-v1'
cases = []

def add(case_id, target, category, signals, positive=False, aliases=()):
    cases.append({
        'case_id': case_id,
        'category': category,
        'label': 'positive' if positive else 'negative',
        'policy': {
            'policy_id': 'launch-independent-v1-policy',
            'target_id': 'target-' + case_id.lower(),
            'target_question': 'When was ' + target + ' launched?',
            'entity': {'canonical_id': 'entity-' + case_id.lower(), 'canonical_name': target, 'aliases': list(aliases)},
            'relation': 'change_or_launch',
            'identity_requirement': 'require_harness_anchor',
            'assessment_budget': {'max_model_attempts': 2, 'max_tokens': 192, 'max_elapsed_ms': 90000},
        },
        'candidate': {
            'evidence_id': 'evidence-' + case_id.lower(),
            'source_id': 'synthetic-source-' + case_id.lower(),
            'signals': [{'kind': k, 'text': t} for k, t in signals],
        },
        'proposal': {'target_binding': 'exact', 'relation_binding': 'exact'},
        'raw': {'identity_scope': 'exact_target', 'relation_scope': 'requested_relation', 'scope_risk': 'none'},
        'expected': {
            'v17_relation_scope': 'requested_relation' if positive else 'not_requested_relation',
            'v30_disposition': 'relevant' if positive else 'not_relevant',
            'source_local_only': True,
        },
    })

# Positive controls: actual source assertions; scope and markers for another target do not govern these claims.
add('P01', 'Lunavo Dock', 'plain_affirmative', [('excerpt', 'Lunavo Dock was launched on 14 March 2031.')], True)
add('P02', 'Cevra Mesh', 'ga_affirmative', [('excerpt', 'Cevra Mesh became generally available on 8 April 2032.')], True)
add('P03', 'Velorin Note', 'title_and_excerpt', [('source_title', 'Velorin Note launch announcement'), ('excerpt', 'Velorin Note was launched on 9 June 2030.')], True)
add('P04', 'Tavren Cloud', 'affirmative_heading', [('heading', 'Tavren Cloud was launched on 11 July 2033.'), ('excerpt', 'The release includes a collaboration workspace.')], True)
add('P05', 'Neruva Link', 'neutral_heading', [('heading', 'Release chronology'), ('excerpt', 'Neruva Link was launched on 12 August 2034.')], True)
add('P06', 'Sorvane Desk', 'fact_signal', [('fact', 'Sorvane Desk was launched on 13 September 2035.')], True)
add('P07', 'Eldavi Port', 'metadata_signal', [('structured_metadata', 'Eldavi Port was launched on 14 October 2036.')], True)
add('P08', 'Beluna 4.7', 'decimal_product', [('excerpt', 'Beluna 4.7 was launched on 15 November 2037.')], True)
add('P09', 'Vireo.Labs Aurora', 'dotted_vendor', [('excerpt', 'Vireo.Labs Aurora was launched on 16 December 2038.')], True)
add('P10', 'Kelvon Studio', 'alias', [('excerpt', 'KVS was launched on 17 January 2039.')], True, ['KVS'])
add('P11', 'Arliva.Core 2.8', 'dotted_decimal_product', [('fact', 'Arliva.Core 2.8 was launched on 18 February 2040.')], True)
add('P12', 'Valdora Sync', 'neutral_leading_prose', [('excerpt', 'The documentation site uses a compact layout. Valdora Sync was launched on 19 March 2041.')], True)
add('P13', 'Resuva Frame', 'neutral_trailing_prose', [('excerpt', 'Resuva Frame was launched on 20 April 2042. The archive includes installation notes.')], True)
add('P14', 'Leriva Forge', 'other_target_denial_before', [('excerpt', 'Osven Pilot was not launched in May. Leriva Forge was launched on 21 May 2043.')], True)
add('P15', 'Mavelo Path', 'other_target_symbol_separate_signal', [('fact', '❌ Dorsen Gate was launched on 3 June 2044.'), ('excerpt', 'Mavelo Path was launched on 22 June 2044.')], True)
add('P16', 'Senova Arc', 'other_target_strike_closed', [('excerpt', '~~Gavren Unit was launched on 2 July 2045.~~ Senova Arc was launched on 23 July 2045.')], True)
add('P17', 'Avrelis Hub', 'other_target_hypothetical_after', [('excerpt', 'Avrelis Hub was launched on 24 August 2046. In a hypothetical scenario, Jorven Map would launch next year.')], True)
add('P18', 'Quenari Wire', 'navigation_cannot_veto_fact', [('navigation_or_footer', 'Fictional releases and retracted entries'), ('excerpt', 'Quenari Wire was launched on 25 September 2047.')], True)
add('P19', 'Orvella Peak', 'url_cannot_veto_fact', [('canonical_url', 'https://archive.example.invalid/fictional/withdrawn/orvella-peak'), ('excerpt', 'Orvella Peak was launched on 26 October 2048.')], True)
add('P20', 'Devoria Leaf', 'fact_and_neutral_title', [('source_title', 'Monthly engineering bulletin'), ('fact', 'Devoria Leaf was launched on 27 November 2049.'), ('excerpt', 'The bulletin also covers system maintenance.')], True)
add('P21', 'Talvori Kit', 'ga_full_phrase', [('excerpt', 'Talvori Kit entered general availability on 28 December 2050.')], True)
add('P22', 'Cenora Grid', 'ga_short_form', [('excerpt', 'Cenora Grid reached GA on 29 January 2051.')], True)
add('P23', 'Avenro Lab', 'case_insensitive_identity', [('excerpt', 'AVENRO LAB was launched on 1 February 2052.')], True)
add('P24', 'Zeriva Stream', 'other_target_retraction_after', [('excerpt', 'Zeriva Stream was launched on 2 March 2053. The claim about Norvex Stack was retracted.')], True)

# Negative controls: the exact/exact raw proposal is deliberately optimistic; deterministic checks must withhold.
add('N01', 'Solvani Dock', 'hypothetical_title', [('source_title', 'Hypothetical release chronology'), ('excerpt', 'Solvani Dock was launched on 3 April 2031.')])
add('N02', 'Merava Loom', 'fictional_heading', [('heading', 'Fictional release story'), ('excerpt', 'Merava Loom was launched on 4 May 2032.')])
add('N03', 'Tovari Desk', 'false_heading', [('heading', 'False claim'), ('excerpt', 'Tovari Desk was launched on 5 June 2033.')])
add('N04', 'Ralven Port', 'denied_heading', [('heading', 'Denied launch claim'), ('excerpt', 'Ralven Port was launched on 6 July 2034.')])
add('N05', 'Covari Path', 'retracted_title', [('source_title', 'Retracted launch announcement'), ('excerpt', 'Covari Path was launched on 7 August 2035.')])
add('N06', 'Nelvori Edge', 'withdrawn_heading', [('heading', 'Withdrawn release statement'), ('fact', 'Nelvori Edge was launched on 8 September 2036.')])
add('N07', 'Veldani Wave', 'fictional_leading_sentence', [('excerpt', 'This is a fictional release story. Veldani Wave was launched on 9 October 2037.')])
add('N08', 'Koravi Tide', 'fictional_trailing_sentence', [('excerpt', 'Koravi Tide was launched on 10 November 2038. This release story is fictional.')])
add('N09', 'Cevoni Park', 'hypothetical_leading_sentence', [('excerpt', 'The following is a hypothetical release scenario. Cevoni Park was launched on 11 December 2039.')])
add('N10', 'Ardeli Beam', 'hypothetical_trailing_sentence', [('excerpt', 'Ardeli Beam was launched on 12 January 2040. This is only a hypothetical scenario.')])
add('N11', 'Luneri Axis', 'false_leading_sentence', [('excerpt', 'The following launch statement is false. Luneri Axis was launched on 13 February 2041.')])
add('N12', 'Jovari Canvas', 'false_trailing_sentence', [('excerpt', 'Jovari Canvas was launched on 14 March 2042. That launch statement is false.')])
add('N13', 'Evarni Link', 'denied_leading_sentence', [('excerpt', 'The following launch claim was denied. Evarni Link was launched on 15 April 2043.')])
add('N14', 'Revalis Core', 'denied_trailing_sentence', [('excerpt', 'Revalis Core was launched on 16 May 2044. The publisher denied that launch claim.')])
add('N15', 'Orenvi Lens', 'withdrawn_leading_sentence', [('excerpt', 'This release statement has been withdrawn. Orenvi Lens was launched on 17 June 2045.')])
add('N16', 'Tervani Wire', 'retracted_trailing_sentence', [('excerpt', 'Tervani Wire was launched on 18 July 2046. The publisher retracted this release statement.')])
add('N17', 'Melvori Grid', 'hypothetical_multi_signal', [('heading', 'Hypothetical scenario'), ('excerpt', 'The project uses a modular architecture.'), ('excerpt', 'Melvori Grid was launched on 19 August 2047.')])
add('N18', 'Cavrani Leaf', 'false_title_metadata', [('source_title', 'False launch report'), ('structured_metadata', 'Cavrani Leaf was launched on 20 September 2048.')])
add('N19', 'Zarvoni Forge', 'fictional_title_neutral_heading_fact', [('source_title', 'Fictional technology chronicle'), ('heading', 'Product timeline'), ('fact', 'Zarvoni Forge was launched on 21 October 2049.')])
add('N20', 'Eldora Sync', 'url_only', [('canonical_url', 'https://releases.example.invalid/Eldora-Sync-was-launched-on-22-November-2050')])
add('N21', 'Vereni Map', 'navigation_only', [('navigation_or_footer', 'Vereni Map was launched on 23 December 2051.')])
add('N22', 'Lorvani Kit', 'url_and_navigation_no_factual_proof', [('canonical_url', 'https://example.invalid/Lorvani-Kit/launched/2052-01-24'), ('navigation_or_footer', 'Lorvani Kit release timeline')])
add('N23', 'Kervani Cloud', 'wrong_target', [('excerpt', 'Dervalo Cloud was launched on 25 February 2053.')])
add('N24', 'Raveni Dock', 'conditional', [('excerpt', 'If financing had arrived, Raveni Dock would have launched on 26 March 2054.')])
add('N25', 'Toveni Hub', 'rumor', [('excerpt', 'An unconfirmed rumor says Toveni Hub was launched on 27 April 2055.')])
add('N26', 'Alveri Note', 'question', [('excerpt', 'Was Alveri Note launched on 28 May 2056?')])
add('N27', 'Mervani Arc', 'markdown_strike_single_sentence', [('excerpt', '~~Mervani Arc was launched on 29 June 2057.~~')])
add('N28', 'Solmeri Peak', 'markdown_strike_cross_sentence', [('excerpt', '~~The timeline is listed below. Solmeri Peak was launched on 1 July 2058.~~')])
add('N29', 'Kavren Studio', 'markdown_strike_cross_multiple_sentences', [('excerpt', '~~The release note was drafted. The team held an internal rehearsal. Kavren Studio was launched on 2 August 2059. The package includes a guide.~~')])
add('N30', 'Zelvori Beam', 'cross_mark_prefix', [('excerpt', '❌ Zelvori Beam was launched on 3 September 2060.')])
add('N31', 'Tarnovi Frame', 'not_equal_operator', [('excerpt', 'Tarnovi Frame ≠ launched on 4 October 2061.')])
add('N32', 'Delvari Gate', 'logical_not_prefix', [('excerpt', '¬ Delvari Gate was launched on 5 November 2062.')])
add('N33', 'Lervani Desk', 'ballot_x_prefix', [('excerpt', '✗ Lervani Desk was launched on 6 December 2063.')])
add('N34', 'Norlavi Port', 'heavy_ballot_x_prefix', [('excerpt', '✘ Norlavi Port was launched on 7 January 2064.')])
strike_claim = 'Sevori Cloud was launched on 8 February 2065.'
add('N35', 'Sevori Cloud', 'combining_long_stroke_full_claim', [('excerpt', ''.join(c + '\u0336' if not c.isspace() else c for c in strike_claim))])
add('N36', 'Kelravi Loom', 'combining_short_stroke_launch_predicate', [('excerpt', 'Kelravi Loom was l\u0335a\u0335u\u0335n\u0335c\u0335h\u0335e\u0335d\u0335 on 9 March 2066.')])
add('N37', 'Valrani Wire', 'cross_mark_suffix', [('excerpt', 'Valrani Wire was launched on 10 April 2067. ❌')])
add('N38', 'Ovelri Edge', 'strike_heading_governs_excerpt', [('heading', '~~Ovelri Edge launch announcement~~'), ('excerpt', 'Ovelri Edge was launched on 11 May 2068.')])
add('N39', 'Javrani Path', 'explicit_no_prefix', [('excerpt', 'No: Pavren news is incorrect. Javrani Path was not launched on 12 June 2069.')])
add('N40', 'Dovrani Mesh', 'target_denied_other_affirmative', [('excerpt', 'Dovrani Mesh was not launched on 13 July 2070. Sarvilo Mesh was launched on that date.')])
add('N41', 'Calravi Unit', 'cancelled_plan', [('excerpt', 'Calravi Unit was scheduled to launch on 14 August 2071, but the launch was cancelled.')])
add('N42', 'Merlavi Grid', 'quoted_false_claim', [('excerpt', 'The statement "Merlavi Grid was launched on 15 September 2072" is false.')])

payload = {
    'schema_version': 1,
    'author_identity': '/root/launch_072_independent',
    'purpose': 'Independent offline synthetic evaluation of launch context and original negation markers for Engine #520/#521',
    'candidate_commit': '049be6542d95539e275d9a44a5bac89835b43834',
    'candidate_source_path': 'crates/reasoning-harness-core/src/evidence_relevance.rs',
    'candidate_source_sha256': 'c9d18f43633665191b18af5c5d79cc535b7914be66e91651ea4788bd18a41736',
    'cases': cases,
}
corpus_bytes = (json.dumps(payload, ensure_ascii=False, indent=2) + '\n').encode()
(OUT / 'corpus.json').write_bytes(corpus_bytes)
manifest = {
    'author_identity': payload['author_identity'],
    'candidate_commit': payload['candidate_commit'],
    'candidate_source_path': payload['candidate_source_path'],
    'candidate_source_sha256': payload['candidate_source_sha256'],
    'corpus_sha256': hashlib.sha256(corpus_bytes).hexdigest(),
    'total_cases': len(cases),
    'positive_cases': sum(c['label'] == 'positive' for c in cases),
    'negative_cases': sum(c['label'] == 'negative' for c in cases),
    'case_ids': [c['case_id'] for c in cases],
    'safety_gate': 'zero negative cases with RequestedRelation or Relevant',
    'positive_recall_gate': 'zero positive cases missing RequestedRelation or Relevant',
}
(OUT / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
print(json.dumps(manifest, indent=2))
