"""Create-only M1 and authorship closure. Static corpus validation only; no Engine or scorer."""
from pathlib import Path
from collections import Counter
from datetime import datetime, timezone
import hashlib
import json
import os
import re
import sys

NOTES = Path(__file__).resolve().parent
ROOT = Path('/private/tmp/engine073-realworld-independent-v1-input')
M0 = Path('/private/tmp/engine073-realworld-M0-v1-staging/receipt.json')
EXPECTED_M0 = '4023eb552ee964e0e99550a61a9f4c2d2800d2d994b7e42c439cf3cf7669d4d2'
CONTRACT = Path('/private/tmp/engine073-complete-M0-independent-review-v1/runtime-code-v2/repo/evaluation/engine073-realworld-evaluator-v1')
FEATURE = Path('/Users/sawadakousuke/.codex/worktrees/engine073-retraction-safety/reasoning-harness')
OLD_PRESERVATION = Path('/private/tmp/engine073-complete-M0-independent-review-v1/preservation-verification.json')

def sha(blob):
    return hashlib.sha256(blob).hexdigest()

def inventory(root, exclude=()):
    return {str(p.relative_to(root)): sha(p.read_bytes()) for p in sorted(root.rglob('*')) if p.is_file() and str(p.relative_to(root)) not in exclude}

def save(name, value, root=NOTES):
    p = root / name
    blob = (json.dumps(value, ensure_ascii=False, indent=2) + '\n').encode()
    with p.open('xb') as f:
        f.write(blob)
        f.flush()
        os.fsync(f.fileno())
    return sha(blob)

def utc():
    return datetime.now(timezone.utc).isoformat()

assert not (ROOT/'manifest.json').exists(), 'M1 already exists; never overwrite or refreeze'
assert not (NOTES/'artifact-manifest.json').exists(), 'already closed'
assert sha(M0.read_bytes()) == EXPECTED_M0
sys.path.insert(0, str(CONTRACT))
from corpus import validate, verify_provenance

data, gold, policy = validate(ROOT)
verify_provenance(ROOT, M0)
preparation = json.loads((ROOT/'preparation.json').read_bytes())
start = json.loads((NOTES/'authoring-start.json').read_bytes())
static = json.loads((NOTES/'static-preflight-final.json').read_bytes())
reviews = json.loads((NOTES/'full-context-review.json').read_bytes())
assert static['all_request_budgets_pass'] and static['schema_and_provenance_pass']
assert reviews['complete'] is True and len(reviews['document_reviews']) == 12
assert all(x['complete'] is True for x in reviews['document_reviews'])
assert preparation['model_or_Engine_accuracy_observations_before_freeze'] == 0
assert preparation['provider_model_Oracle_calls'] == 0
assert start['authorship_started_unix_seconds'] == preparation['authorship_started_unix_seconds']

documents = data['documents']
sources = data['sources']
cases = data['cases']
gold_by_key = {(x['case_id'], x['track']): x for x in gold['records']}
assert len(cases) == 24 and len(sources) == 48 and len(gold_by_key) == 48
assert {sid for c in cases for sid in c['source_ids']} == set(sources)
assert {s['document_id'] for s in sources.values()} == set(documents)
assert len({x['product'] for x in documents.values()}) == 6
for d in documents.values():
    assert re.fullmatch(r'https://raw\.githubusercontent\.com/[^/]+/[^/]+/[0-9a-f]{40}/.+', d['url'])
    assert d['url'] == d['immutable_capture_url']
for c in cases:
    for track in ['e2e_normal', 'helper_opt_in']:
        row = gold_by_key[(c['id'], track)]
        assert row['full_source_rationale'] and row['scope_time_origin_review']
        for group in row['groups']:
            assert group['required_supported_information']
            assert set(group['supporting_source_ids']) <= set(c['source_ids'])
        if row['label'] == 'negative':
            assert row['answerability'] == 'none' and not row['groups'] and row['unresolved_facets']
    helper = gold_by_key[(c['id'], 'helper_opt_in')]
    assert policy[c['id']]['controlled_reference_authorized'] == (helper['label'] == 'positive')
    if helper['label'] == 'positive':
        assert len({sources[s]['source_version'] for s in c['source_ids']}) == 1
assert all(s['synthetic_mutation'] is False for s in sources.values())

old_map = json.loads(OLD_PRESERVATION.read_bytes())['old928_including_prep5']
assert len(old_map) == 928
old_diffs = [name for name, digest in old_map.items() if sha((FEATURE/name).read_bytes()) != digest]
assert not old_diffs, old_diffs
old_digest = sha(json.dumps(old_map, sort_keys=True, separators=(',', ':')).encode())

counts = {track: dict(Counter(x['label'] for x in gold['records'] if x['track'] == track)) for track in ['e2e_normal', 'helper_opt_in']}
families = {track: dict(Counter(x['family'] for x in gold['records'] if x['track'] == track)) for track in ['e2e_normal', 'helper_opt_in']}
assert all(len(v) == 8 and set(v.values()) == {3} for v in families.values())
maximum_request = max(c['native_shape_ModelRequest_UTF8_bytes'] for c in static['checks'])
scope_review = {
    'identity': data['identity'], 'reviewer': start['author_id'], 'reviewed_utc': utc(),
    'stage': 'pre-M1 prospective independent source/gold review; zero accuracy observations',
    'result': 'PASS_FOR_STATIC_M1_FREEZE', 'cases': 24, 'gold_records': 48,
    'source_records': 48, 'documents': 12, 'products': sorted({d['product'] for d in documents.values()}),
    'publisher_repository_origins': sorted({s['origin_group'] for s in sources.values()}),
    'gold_counts': counts, 'family_counts': families,
    'normal_positive_partial_cases': [x['case_id'] for x in gold['records'] if x['track']=='e2e_normal' and x['label']=='positive' and x['answerability']=='partial'],
    'all_full_context_reviews_complete': True, 'full_context_review_sha256': sha((NOTES/'full-context-review.json').read_bytes()),
    'quote_bytes_hash_scope_version_retrieval_binding': 'PASS via frozen corpus/check_source contract',
    'all_gold_group_witnesses_case_local': True, 'all_documents_used': True,
    'immutable_40hex_raw_capture_urls': True, 'controlled_reference_authorized_positive8_only': True,
    'synthetic_mutations': 0, 'maximum_native_shape_ModelRequest_UTF8_bytes': maximum_request,
    'frozen_ModelRequest_limit': 3584, 'reserved_input_upper_bound_maximum': maximum_request+512,
    'labels_rationales_and_gold_not_in_helper_packet': True,
    'pre_freeze_changes': 'Initial bytes, static checks, exact diffs and rationale preserved outside corpus. Questions/labels/answerability/families unchanged; no result-based repair.',
    'limits': [
        '12 complete official documents from six repository origins are reused across24 questions, not24 independent documents.',
        'Conflict family tests apparent tensions with explicit option/mode/recipe conditions; does not fabricate contradictory upstream statements.',
        'Untrusted-control family uses genuine upstream warnings/example commands/code as inert data, without synthetic prompt injection.',
        'No inference from a duplicate capture to independent publisher corroboration.',
        'Curated host reference_facts are bounded extractions reviewed against full documents, not raw extraction performance or universal/human-approved truth.',
        'Scoped API documentation does not establish unobserved private configuration, live state, current support or different identifier namespaces.',
        'Helper version hashes remain distinct where original documents differ; positive equivalence is limited to authorized same-version propositions.',
        'Semantic labels are prospective independent-agent annotations; no candidate model/Engine output was observed.'
    ],
    'Engine_actual_scorer_model_provider_Oracle_calls': 0,
    'old_cohort_or_parent_case_bodies_read_for_authorship': False,
}
save('scope-review-final.json', scope_review)
preservation = {'verified_utc': utc(), 'method': 'hash-only; no old case/author/inputs/raw contents decoded',
    'old_reference_count_including_prep5': 928, 'old_source_references': 923, 'separate_prep_references': 5,
    'old_differences': old_diffs, 'old_map_canonical_sha256': old_digest, 'old928_including_prep5': old_map,
    'M0_receipt_sha256': sha(M0.read_bytes()), 'M0_receipt_unchanged': True}
save('preservation-verification.json', preservation)

files = inventory(ROOT)
assert len(files) == 28
assert set(files) == {'corpus.json','gold.json','host-policy.json','preparation.json'} | {d[k] for d in documents.values() for k in ['raw_path','normalized_path']}
m1 = {
    'identity': data['identity'], 'stage': 'M1_COMPLETE_PROSPECTIVE_FROZEN', 'frozen_utc': utc(),
    'author_id': start['author_id'], 'authoring_started_utc': start['started_utc'],
    'M0_receipt_sha256': EXPECTED_M0, 'files_self_excluding_manifest': len(files),
    'files': files, 'normalization': 'utf8-plain-whitespace-v1',
    'accuracy_observations_before_freeze': 0, 'actual_scorer_processes': 0,
    'scope_review_sha256': sha((NOTES/'scope-review-final.json').read_bytes()),
    'self_exclusion': 'manifest.json is excluded from files; its SHA256 is the external M1 anchor.'
}
m1_sha = save('manifest.json', m1, ROOT)
validate(ROOT, m1_sha)
verify_provenance(ROOT, M0)
assert inventory(ROOT, ('manifest.json',)) == files
validation = {
    'validated_utc': utc(), 'mode': 'STATIC_ONLY', 'command': 'corpus.validate(ROOT, M1_sha256); corpus.verify_provenance(ROOT, final_M0_receipt)',
    'frozen_validator_corpus_py_sha256': sha((CONTRACT/'corpus.py').read_bytes()),
    'M1_sha256': m1_sha, 'M0_receipt_sha256': EXPECTED_M0, 'schema': 'PASS', 'manifest': 'PASS', 'provenance': 'PASS',
    'corpus_files_self_excluding_manifest': 28, 'corpus_files_including_manifest': 29,
    'counts': counts, 'family_counts': families, 'helper_maximum_request_bytes': maximum_request,
    'Engine_actual_scorer_model_provider_Oracle_calls': 0,
    'old928_hash_only_unchanged': True,
}
save('M1-static-validation.json', validation)
receipt = dict(validation, state='CLOSED_AUTHORSHIP_M1_FROZEN_AWAITING_FIRST_EVALUATION',
    author_id=start['author_id'], authoring_started_utc=start['started_utc'],
    source_full_context_review_documents=12, documents=12, products=6, source_records=48, cases=24, gold_track_records=48,
    gold_annotation_authority='Independent-agent annotations; no human review or universal truth claim.',
    accuracy_evaluation_started=False, first_run_or_score_marker_created=False,
    next_action='Parent may statically verify frozen M1 and create fixed first-run plans in the same runtime. Independent author does not launch evaluation.',
    acquisition_history='Initial sandbox DNS failures and one upstream404 retained; authorized public-source acquisition then completed. No model/provider call.',
    remaining_acceptance='Actual first model observations, complete visible-answer/span annotation, first scorer, provider-specific F1, citation/usefulness/safety/cost gates are unperformed. Groq remains deferred and overall full3-provider GO is not established.')
save('receipt.json', receipt)
readme = f'''# Fresh real-document24 prospective input — M1 frozen

M1 SHA256: `{m1_sha}`
M0 SHA256: `{EXPECTED_M0}`

Authorship began {start['started_utc']} after M0 completion. Twenty-four questions use12 complete official documents from curl8.10.1, Git2.47.0, CMake3.31.0, OpenSSL3.0.15, SQLite3.47.2 and Redis7.2.5. All12 full texts were read, pinned to immutable40hex raw URLs, and bound to raw/normalized hashes and UTF8 quote spans. Six repository origins and12 documents are reused; this is not24 independent documents.

Gold has48 track records: normal17 positive/7 negative, helper8 positive/16 negative. Each of eight families has3 cases pertrack. Two normal positives explicitly preserve unresolved project/build facets. All labels/rationales precede observations and are outside Engine/model inputs. Genuine documented commands/examples remain inert source data; no artificial source mutation or execution occurred. Host facts are controlled extractions, not raw extraction or universal/human truth. Captured historical policy does not become present-day state.

Frozen corpus/provenance/inventory validation PASS, with28 self-excluding files/29 including manifest. Native-shape helper ModelRequest maximum {maximum_request} UTF8 bytes is below3584; reserved input upper bound {maximum_request+512} is below4096. Static validation imported the frozen corpus contract only; Engine, model, provider, actual scorer and Oracle calls are0. No evaluation result or F1 exists yet.

The initial authoring draft, first static result, pre-M1 exact diffs/reasons and final static result remain here outside corpus. Pre-freeze cleanup removed case-specific visible hints, clarified witnesses and removed an unasked extra requirement without changing questions, labels, answerability or family assignments. No observation-dependent repair occurred.

All928 prior references, including separate prep5, were rehashed without reading old cohort bodies and remained unchanged. Final M0 receipt remained byte identical. The corpus is read-only. This notes directory is separately self-excluding hash-closed; no notes/scripts/extra files appear inside the corpus.

Next: parent same-runtime static/M1/plan verification, then separately authorized first-only execution. No Engine/model/scorer execution was launched by this author. Groq is still deferred; real-answer F1, citation/unsafe/efficiency gates and full3-provider acceptance remain unperformed.
'''
with (NOTES/'README.md').open('x') as f:
    f.write(readme); f.flush(); os.fsync(f.fileno())

for p in ROOT.rglob('*'):
    if p.is_file():
        with p.open('rb') as f: os.fsync(f.fileno())
        p.chmod(0o444)
for p in sorted([x for x in ROOT.rglob('*') if x.is_dir()], key=lambda x: len(x.parts), reverse=True): p.chmod(0o555)
ROOT.chmod(0o555)
assert sha((ROOT/'manifest.json').read_bytes()) == m1_sha
assert inventory(ROOT, ('manifest.json',)) == files
assert sha(M0.read_bytes()) == EXPECTED_M0
assert all(sha((FEATURE/name).read_bytes()) == digest for name,digest in old_map.items())
note_files = inventory(NOTES, ('artifact-manifest.json',))
anchor = save('artifact-manifest.json', {
    'identity':'engine073-realworld-independent-v1-authoring-closure', 'closed_utc':utc(),
    'files_self_excluding_manifest':len(note_files), 'files':note_files,
    'corpus_root':str(ROOT), 'M1_sha256':m1_sha, 'corpus_files_including_manifest':dict(files, **{'manifest.json':m1_sha}),
    'old928_hash_only_unchanged':True, 'M0_receipt_unchanged':True,
    'state':'CLOSED_AUTHORSHIP_ONLY_AWAITING_FIRST_EVALUATION', 'Engine_actual_scorer_model_provider_Oracle_calls':0,
    'self_exclusion':'artifact-manifest.json is excluded from files; external SHA256 is the notes anchor.'
})
for p in NOTES.rglob('*'):
    if p.is_file():
        with p.open('rb') as f: os.fsync(f.fileno())
        p.chmod(0o444)
for p in sorted([x for x in NOTES.rglob('*') if x.is_dir()], key=lambda x:len(x.parts), reverse=True): p.chmod(0o555)
NOTES.chmod(0o555)
assert inventory(NOTES, ('artifact-manifest.json',)) == note_files
print(json.dumps({'M1_sha256':m1_sha, 'notes_final_anchor':anchor, 'corpus_files':29, 'notes_files_including_manifest':len(note_files)+1, 'counts':counts,'max_native_ModelRequest_bytes':maximum_request,'static_provenance':'PASS','old928_unchanged':True,'Engine_actual_scorer_model_provider_Oracle_calls':0},indent=2))
