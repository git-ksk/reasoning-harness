"""Fresh v2 author specification; no implementation or root repair content consulted."""
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
            'policy_id': 'launch-independent-v2-policy',
            'target_id': 'entity-for-' + case_id,
            'target_question': 'When was ' + target + ' launched?',
            'entity': {'canonical_id': 'canonical-for-' + case_id, 'canonical_name': target, 'aliases': list(aliases)},
            'relation': 'change_or_launch',
            'identity_requirement': 'require_harness_anchor',
            'assessment_budget': {'max_model_attempts': 2, 'max_tokens': 192, 'max_elapsed_ms': 90000},
        },
        'candidate': {
            'evidence_id': 'independent-v2-evidence-' + case_id,
            'source_id': 'independent-v2-source-' + case_id,
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

# New positive constructions, with factual proof restricted to excerpt/fact/structured_metadata.
add('P201', 'Brisano Vault', 'launch_day_before_may', [('excerpt', 'Brisano Vault was launched on 7 May 2081, according to the release record.')], True)
add('P202', 'Pelvoro Pulse', 'entered_general_availability_may', [('fact', 'Pelvoro Pulse entered general availability on 6 May 2082.')], True)
add('P203', 'Nuvessa Board', 'reached_ga_may', [('structured_metadata', 'Nuvessa Board reached GA on 5 May 2083.')], True)
add('P204', 'Rindavo Shelf', 'neutral_sentence_before_proof', [('excerpt', 'The release archive has moved to a new address. Rindavo Shelf was launched on 4 May 2084.')], True)
add('P205', 'Carvessa Gate', 'neutral_sentence_after_ga', [('excerpt', 'Carvessa Gate entered general availability on 3 May 2085. The manual is divided into eight chapters.')], True)
add('P206', 'Sorvilo Relay', 'factual_metadata_with_neutral_heading', [('heading', 'Availability history'), ('structured_metadata', 'Sorvilo Relay reached GA on 2 May 2086.')], True)
add('P207', 'Kivrano Panel', 'dotted_alias', [('fact', 'Kiv.Panel entered general availability on 1 May 2087.')], True, ['Kiv.Panel'])
add('P208', 'Zovella 6.3', 'decimal_ga', [('excerpt', 'Zovella 6.3 reached GA on 19 May 2088.')], True)
add('P209', 'Cirri.Labs Nera', 'dotted_vendor_entered_ga', [('excerpt', 'Cirri.Labs Nera entered general availability on 18 May 2089.')], True)
add('P210', 'Danvoro.Route 5.6', 'dotted_decimal_launch', [('excerpt', 'The archive covers two product families. Danvoro.Route 5.6 was launched on 17 May 2090. A separate appendix lists contributors.')], True)
add('P211', 'Falvessa Bay', 'neutral_question_in_preceding_sentence', [('excerpt', 'Where is the support guide? Falvessa Bay was launched on 16 May 2091.')], True)
add('P212', 'Hesvoro Rack', 'other_subject_negative_before_may', [('excerpt', 'Bralune Frame was not launched in April. Hesvoro Rack was launched on 15 May 2092.')], True)
add('P213', 'Volmessa Rail', 'other_subject_withdrawn_title', [('source_title', 'Withdrawn claim about Ruvani Cove'), ('fact', 'Volmessa Rail was launched on 14 May 2093.')], True)
add('P214', 'Penvaro Lane', 'other_subject_false_heading', [('heading', 'False claim about Doravi Deck'), ('excerpt', 'Penvaro Lane entered general availability on 13 May 2094.')], True)
add('P215', 'Zanvessa Key', 'other_subject_hypothetical_heading', [('heading', 'Hypothetical launch scenario for Elviro Scope'), ('fact', 'Zanvessa Key reached GA on 12 May 2095.')], True)
add('P216', 'Talmero Stack', 'other_subject_fiction_sentence', [('excerpt', 'The story about Veyrani Lock is fictional. Talmero Stack entered general availability on 11 May 2096.')], True)
add('P217', 'Kinvaro Pad', 'other_subject_denial_after_ga', [('excerpt', 'Kinvaro Pad reached GA on 10 May 2097. Halvuna Bridge was not launched that month.')], True)
add('P218', 'Mesvoro Lane', 'closed_strike_other_subject_before', [('excerpt', '~~Juvrani Hatch entered general availability on 8 May 2098.~~ Mesvoro Lane was launched on 9 May 2098.')], True)
add('P219', 'Bervessa Ring', 'cross_other_subject_different_signal', [('fact', '❌ Gavloro Shell reached GA on 7 May 2099.'), ('excerpt', 'Bervessa Ring entered general availability on 8 May 2099.')], True)
add('P220', 'Covrano Mark', 'not_equal_other_subject', [('excerpt', 'Osrani Jet ≠ launched on 5 May 2100. Covrano Mark was launched on 7 May 2100.')], True)
add('P221', 'Larvessa Tile', 'logical_not_other_subject', [('excerpt', '¬ (Fravilo Point was launched on 4 May 2101). Larvessa Tile was launched on 6 May 2101.')], True)
other = 'Pavnori Chart was launched on 3 May 2102.'
add('P222', 'Desvoro Byte', 'combining_strike_other_subject', [('excerpt', ''.join(c+'\u0336' if not c.isspace() else c for c in other) + ' Desvoro Byte reached GA on 5 May 2102.')], True)
add('P223', 'Tavmessa Wave', 'other_subject_named_denial_trailing', [('excerpt', 'Tavmessa Wave entered general availability on 4 May 2103. The publisher denied the launch report about Xavrani Cone.')], True)
add('P224', 'Nermero Deck', 'nonfactual_url_context_cannot_govern', [('canonical_url', 'https://release.example.invalid/hypothetical/nermero-deck'), ('heading', 'Project chronology'), ('excerpt', 'Nermero Deck was launched on 3 May 2104.')], True)
add('P225', 'Varnessa Cove', 'nonfactual_footer_cannot_govern', [('navigation_or_footer', 'Retractions and fictional release scenarios'), ('fact', 'Varnessa Cove reached GA on 2 May 2105.')], True)
add('P226', 'Belmero Dock', 'title_anchor_and_alias_factual_proof', [('source_title', 'Belmero Dock release timeline'), ('excerpt', 'BMD was launched on 1 May 2106.')], True, ['BMD'])
add('P227', 'Osvessa Pond', 'active_intransitive_launch', [('excerpt', 'Osvessa Pond launched on 21 May 2107. Its user guide was published separately.')], True)
add('P228', 'Firro.Tools Sera 7.2', 'dotted_vendor_decimal_ga_with_title', [('source_title', 'Firro.Tools Sera 7.2 availability notes'), ('fact', 'Firro.Tools Sera 7.2 reached GA on 20 May 2108.')], True)

add('P229', 'Kelmora Tap', 'same_target_strike_on_nonlaunch_statement', [('excerpt', '~~Kelmora Tap includes an offline editor.~~ Kelmora Tap was launched on 22 May 2109.')], True)
add('P230', 'Almora Plane', 'same_target_cross_on_nonlaunch_statement', [('excerpt', '❌ Almora Plane supports XML export. Almora Plane entered general availability on 23 May 2109.')], True)
add('P231', 'Jermoro Sink', 'same_target_false_heading_nonlaunch_statement', [('heading', 'False claim: Jermoro Sink has an offline mode'), ('fact', 'Jermoro Sink reached GA on 24 May 2109.')], True)

# New negative constructions: governing context, original marker scope, bounded GA frames and authority.
add('N201', 'Auvessa Crate', 'false_named_title_governs', [('source_title', 'False: Auvessa Crate launch report'), ('excerpt', 'Auvessa Crate was launched on 19 May 2110.')])
add('N202', 'Darmero Cone', 'denial_sentence_heading_governs_fact', [('heading', 'This launch claim is denied'), ('fact', 'Darmero Cone was launched on 18 May 2111.')])
add('N203', 'Culvoro Loop', 'withdrawn_title_governs_excerpt', [('source_title', 'Release statement withdrawn'), ('excerpt', 'Culvoro Loop entered general availability on 17 May 2112.')])
add('N204', 'Zelmero Jet', 'retraction_notice_governs_metadata', [('heading', 'Retraction notice'), ('structured_metadata', 'Zelmero Jet reached GA on 16 May 2113.')])
add('N205', 'Nivessa Scope', 'fictional_heading_governs_ga', [('heading', 'Fictional scenario: Nivessa Scope rollout'), ('excerpt', 'Nivessa Scope reached GA on 15 May 2114.')])
add('N206', 'Savrano Plate', 'hypothetical_title_governs_ga', [('source_title', 'Hypothetical availability timeline for Savrano Plate'), ('excerpt', 'Savrano Plate entered general availability on 14 May 2115.')])
add('N207', 'Jevessa Rack', 'fictional_leading_without_release_noun', [('excerpt', 'What follows is fictional. Jevessa Rack was launched on 13 May 2116.')])
add('N208', 'Morvoro Point', 'withdrawn_preceding_sentence_reference', [('excerpt', 'Morvoro Point was launched on 12 May 2117. The preceding launch sentence was withdrawn.')])
add('N209', 'Kelmessa Coin', 'false_correction_trailing', [('excerpt', 'Kelmessa Coin was launched on 11 May 2118. Correction: this launch report is false.')])
add('N210', 'Tesrano Pool', 'denied_above_ga_reference', [('excerpt', 'Tesrano Pool reached GA on 10 May 2119. The above GA claim was denied.')])
add('N211', 'Duvessa Pipe', 'fictional_account_trailing_ga', [('excerpt', 'Duvessa Pipe entered general availability on 9 May 2120. This account is fictional.')])
add('N212', 'Rulmoro Board', 'hypothetical_leading_availability', [('excerpt', 'This is a hypothetical availability timeline. Rulmoro Board entered general availability on 8 May 2121.')])
add('N213', 'Casvessa Drum', 'strike_governs_two_sentences', [('excerpt', '~~Availability record: Casvessa Drum reached GA on 7 May 2122. This record supersedes the earlier draft.~~')])
add('N214', 'Permero Reel', 'cross_heading_governs_fact', [('heading', '❌ Permero Reel release record'), ('fact', 'Permero Reel was launched on 6 May 2123.')])
add('N215', 'Wavessa Moor', 'not_equal_launch_predicate', [('excerpt', 'Wavessa Moor ≠ launched on 5 May 2124.')])
add('N216', 'Hulvoro Pod', 'logical_not_parenthesized_ga', [('excerpt', '¬ (Hulvoro Pod entered general availability on 4 May 2125).')])
add('N217', 'Lovessa Stack', 'ballot_cross_suffix_ga', [('excerpt', 'Lovessa Stack reached GA on 3 May 2126 ✗')])
add('N218', 'Vesmero Bin', 'heavy_cross_prefix_ga', [('excerpt', '✘ Vesmero Bin entered general availability on 2 May 2127.')])
claim = 'Zulvoro Trace reached GA on 1 May 2128.'
add('N219', 'Zulvoro Trace', 'combining_long_stroke_ga', [('excerpt', ''.join(c+'\u0336' if not c.isspace() else c for c in claim))])
add('N220', 'Lanvessa Sort', 'strike_scope_across_newline_and_sentence', [('excerpt', '~~Obsolete publication draft.\nThe archive was prepared for the press. Lanvessa Sort was launched on 21 May 2129.~~')])
add('N221', 'Quivoro Fold', 'entered_ga_question', [('excerpt', 'Did Quivoro Fold enter general availability on 20 May 2130?')])
add('N222', 'Balmessa Spur', 'entered_ga_counterfactual', [('excerpt', 'Balmessa Spur would have entered general availability on 19 May 2131 if approval had arrived.')])
add('N223', 'Remvoro Shard', 'reached_ga_if_condition', [('excerpt', 'If the regulator approved the project, Remvoro Shard reached GA on 18 May 2132.')])
add('N224', 'Zenvessa Shelf', 'never_entered_ga', [('excerpt', 'Zenvessa Shelf never entered general availability on 17 May 2133.')])
add('N225', 'Kovmero Rise', 'not_reached_ga', [('excerpt', 'Kovmero Rise has not reached GA on 16 May 2134.')])
add('N226', 'Dilmessa Curl', 'ga_misattributed_other_subject', [('heading', 'Dilmessa Curl availability archive'), ('excerpt', 'Gelmoro Curl entered general availability on 15 May 2135.')])
add('N227', 'Felmessa Box', 'unconfirmed_ga_rumor', [('excerpt', 'It is an unconfirmed rumor that Felmessa Box reached GA on 14 May 2136.')])
add('N228', 'Tanmoro Fin', 'rumor_labeled_launch', [('excerpt', 'Rumor: Tanmoro Fin was launched on 13 May 2137.')])
add('N229', 'Carmessa Tile', 'wrong_target_excerpt_heading_anchor', [('heading', 'Carmessa Tile project notes'), ('excerpt', 'Jalmoro Tile was launched on 12 May 2138.')])
add('N230', 'Silvessa Grove', 'heading_affirmative_without_factual_proof', [('heading', 'Silvessa Grove entered general availability on 11 May 2139.'), ('excerpt', 'The archive lists three documentation formats.')])
add('N231', 'Mulmero Bond', 'title_affirmative_without_factual_proof', [('source_title', 'Mulmero Bond reached GA on 10 May 2140.'), ('structured_metadata', 'Archive language: English; section: availability')])
add('N232', 'Bolvessa Patch', 'url_claim_without_factual_proof', [('canonical_url', 'https://news.example.invalid/Bolvessa-Patch-entered-general-availability-on-9-May-2141'), ('navigation_or_footer', 'Bolvessa Patch availability')])
add('N233', 'Nalmoro Orbit', 'footer_affirmative_without_factual_proof', [('navigation_or_footer', 'Nalmoro Orbit reached GA on 8 May 2142.')])
add('N234', 'Hervessa Gate', 'title_proof_and_factual_question', [('source_title', 'Hervessa Gate was launched on 7 May 2143.'), ('excerpt', 'When was Hervessa Gate launched?')])
add('N235', 'Polmero Glide', 'planned_launch_only', [('fact', 'Polmero Glide is scheduled to launch on 6 May 2144.')])
add('N236', 'Yalmessa Park', 'topic_mention_no_assertion', [('excerpt', 'This index discusses the launch date of Yalmessa Park and links to a separate announcement.')])

payload = {
    'schema_version': 2,
    'author_identity': '/root/launch_072_independent',
    'candidate_commit': 'de47d3ae4573bdfeaebae45be56c3595c06f683c',
    'candidate_source_path': 'crates/reasoning-harness-core/src/evidence_relevance.rs',
    'candidate_source_sha256': 'c2f119030c718af66f872f913bf697951c33eace9067cec9bbcdbd5bca0d4f17',
    'authority_contract': 'Only excerpt, fact and structured_metadata furnish factual proof. Heading/source_title govern; URL/navigation cannot prove or govern factual claims.',
    'cases': cases,
}
raw = (json.dumps(payload, ensure_ascii=False, indent=2) + '\n').encode()
(OUT/'corpus.json').write_bytes(raw)
manifest = {
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
(OUT/'manifest.json').write_text(json.dumps(manifest, indent=2)+'\n')
print(json.dumps(manifest, indent=2))
