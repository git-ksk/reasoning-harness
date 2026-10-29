"""Read-only saved-artifact RCA. No model, credential, network or scorer calls."""
import copy
import hashlib
import json
from pathlib import Path, PurePosixPath
from collections import Counter
import zipfile

ROOT = Path('/Users/sawadakousuke/x-code/reasoning-harness')
RAW = ROOT / 'target/private-evaluation-pending/cli-normal-real24-40873570944-raw'
OUT = ROOT / 'target/private-evaluation-pending/cli-normal-real24-40873570944-rca-independent'
ART = RAW / 'artifact'
MANAGED = Path('/Users/sawadakousuke/.codex/worktrees/engine073-retraction-safety/reasoning-harness')

def require(value, message):
    if not value:
        raise AssertionError(message)

def sha(data):
    return hashlib.sha256(data).hexdigest()

def read(path):
    return json.loads(path.read_bytes())

def compact_layout_only(text):
    result = []
    quoted = False
    escaped = False
    for char in text:
        if quoted:
            result.append(char)
            if escaped:
                escaped = False
            elif char == '\\':
                escaped = True
            elif char == '"':
                quoted = False
        elif char == '"':
            quoted = True
            result.append(char)
        elif char not in ' \t\r\n':
            result.append(char)
    require(not quoted and not escaped, 'unterminated JSON string')
    return ''.join(result)

def main():
    run = read(RAW / 'run.json')
    artifact = read(RAW / 'artifact.json')
    zip_sha = sha((RAW / 'artifact.zip').read_bytes())
    require(zip_sha == '9100a6d26d9ab115ed6ceeac9d53de96753e1f0479e78a899cc26a87f86c4459'
            and artifact['digest'] == 'sha256:' + zip_sha, 'ZIP digest mismatch')
    require(run['id'] == artifact['workflow_run']['id'] == 40873570944
            and artifact['id'] == 12896743028
            and run['head_sha'] == artifact['workflow_run']['head_sha']
            == '7073471d7490c64841200ae9ddaf252317c34721'
            and run['event'] == 'push' and run['run_attempt'] == 1
            and run['head_branch'] == 'cli-normal-real24-development-v1'
            and run['conclusion'] == 'failure', 'run/artifact identity mismatch')
    with zipfile.ZipFile(RAW / 'artifact.zip') as archive:
        names = [x.filename for x in archive.infolist() if not x.is_dir()]
        require(len(names) == len(set(names)), 'duplicate ZIP member')
        require(set(names) == {str(p.relative_to(ART)) for p in ART.rglob('*') if p.is_file()},
                'extracted artifact inventory mismatch')
        for name in names:
            path = PurePosixPath(name)
            require(not path.is_absolute() and '..' not in path.parts, 'unsafe ZIP member')
            require(archive.read(name) == (ART / name).read_bytes(), 'extracted bytes changed')
    plan = read(ART / 'plan.json')
    summary = read(ART / 'lane/summary.json')
    state = read(ART / 'lane/state.json')
    rows = summary['rows']
    require(len(rows) == len(plan['jobs']) == 48
            and [r['id'] for r in rows] == [j['id'] for j in plan['jobs']], 'row plan mismatch')
    counts = {'complete': sum(r['complete'] is True for r in rows),
              'failed': sum(r['admission']['allowed'] and not r['complete'] for r in rows),
              'suppressed': sum(not r['admission']['allowed'] for r in rows)}
    require(counts == {'complete': 2, 'failed': 1, 'suppressed': 45}, 'unexpected cohort outcome')
    require(rows[2]['id'] == 'rw-02-candidate' and not summary['complete']
            and state['charged_reservations'] == summary['charged_reservations'] == 3 * 28096,
            'failure/reservation mismatch')
    latch = 'typed_operational_failure:usage_budget_unmeasurable'
    require(state['guard']['latch'] == summary['latch'] == latch, 'latch mismatch')
    for row in rows[3:]:
        require({p.name for p in (ART / 'lane' / row['id']).iterdir()} == {'row.json'}
                and row['admission']['reason'] == latch, 'suppressed row executed')
    journals = {}
    failed_events = None
    for row in rows[:3]:
        directory = ART / 'lane' / row['id']
        events = [json.loads(line) for line in (directory / 'model-journal.jsonl').read_text().splitlines()]
        kinds = Counter(e['kind'] for e in events)
        responses = [e for e in events if e['kind'] == 'model_response']
        wire = [e for e in events if e['kind'] == 'wire_http_response']
        require(all(e['status'] == 200 for e in wire), 'non-200 HTTP response')
        usage = []
        for event in responses:
            response = event['response']
            u = response['usage']
            require(all(type(u[k]) is int and u[k] >= 0 for k in ['input_tokens', 'output_tokens', 'total_tokens'])
                    and u['total_tokens'] == u['input_tokens'] + u['output_tokens']
                    and response['provider_attempts'] == 1, 'unknown/inconsistent prefix usage')
            usage.append({'generation': event['generation'], **u})
        require(kinds['wire_request'] == len(wire) == len(responses), 'wire/response binding mismatch')
        journals[row['id']] = {'kind_counts': dict(kinds), 'HTTP_statuses': [e['status'] for e in wire],
                               'known_usage': usage,
                               'input_tokens': sum(u['input_tokens'] for u in usage),
                               'output_tokens': sum(u['output_tokens'] for u in usage)}
        if row['id'] == 'rw-02-candidate':
            failed_events = events
    failed = ART / 'lane/rw-02-candidate'
    trace = read(failed / 'trace.json')
    trace_requests = [e for e in trace['events'] if e['kind'] == 'model_request']
    final = [e for e in trace_requests if e['phase'] == 'final_render']
    require(len(final) == 1 and len(trace_requests) == 9, 'final render trace missing')
    request = final[0]['request']
    refusals = [e for e in failed_events if e['kind'] == 'pre_request_refusal']
    require(len(refusals) == 1 and refusals[0]['completed_or_reserved_generations'] == 8,
            'unexpected refusal generation')
    encode = lambda v: json.dumps(v, ensure_ascii=False, separators=(',', ':')).encode()
    original_size = len(encode(request))
    require(original_size == refusals[0]['request_bytes'] == 13365, 'request byte reproduction mismatch')
    accepted = [e for e in failed_events if e['kind'] == 'model_request']
    require([e['request'] for e in accepted] == [e['request'] for e in trace_requests[:8]],
            'trace/journal request mismatch')
    require([e['generation'] for e in accepted] == list(range(1, 9)), 'accepted generation gap')
    require(all(e.get('generation', 0) <= 8 for e in failed_events), 'ninth HTTP/model dispatch found')
    marker = 'Verified reasoning artifact:\n'
    task = request['task']
    require(task.count(marker) == 1, 'artifact marker ambiguous')
    start = task.index(marker) + len(marker)
    value, end = json.JSONDecoder().raw_decode(task[start:])
    original_artifact = task[start:start + end]
    compact = compact_layout_only(original_artifact)
    require(json.loads(compact) == value, 'JSON values changed')
    replacement = copy.deepcopy(request)
    replacement['task'] = task[:start] + compact + task[start + end:]
    require(replacement['task'][:start] == task[:start]
            and replacement['task'][start + len(compact):] == task[start + end:]
            and {k:v for k,v in request.items() if k != 'task'}
            == {k:v for k,v in replacement.items() if k != 'task'}, 'outside artifact changed')
    compact_size = len(encode(replacement))
    usage = journals['rw-02-candidate']
    require(usage['input_tokens'] == 6301 and usage['output_tokens'] == 3118
            and compact_size == 11186 and request['max_tokens'] == 978, 'compaction/budget mismatch')
    observations = MANAGED / 'target/ordinary-run40873570944-analysis/observations'
    index = read(observations / 'observation-index.json')
    observation_rows = [read(p) for p in observations.glob('*.json')
                        if p.name not in ['verified-run.json', 'observation-index.json']
                        and p.stem == read(p).get('row_id')]
    operations = dict(Counter(r['operation'] for r in observation_rows))
    require(len(observation_rows) == index['observations'] == 48 and operations ==
            {'complete': 2, 'incomplete': 1, 'not_run': 45}
            and index['actual_answer_f1'] is None and index['binding_failures'] == [],
            'observation completeness conflated')
    checkpoint = read(MANAGED / 'evaluation/cli-normal-real24-development-v1-results/execution-checkpoint.json')
    require(checkpoint['full_F1'] is None and checkpoint['first_aggregate_score_started'] is False,
            'checkpoint scoring scope changed')
    report = {'artifact_id': artifact['id'], 'run_id': run['id'], 'ZIP_sha256': zip_sha,
              'artifact_file_count': len(names), 'all_extracted_members_match_ZIP': True,
              'cohort': counts, 'journals': journals,
              'actual_instrumented_HTTP_total': sum(j['kind_counts']['wire_request'] for j in journals.values()),
              'failed_job': 'rw-02-candidate', 'failure_phase': 'final_render',
              'failed_job_HTTP': 8, 'ninth_wire_dispatches': 0, 'refusal': refusals[0],
              'original_request_bytes': original_size, 'original_input_reservation': 6301 + original_size + 512,
              'compact_request_bytes': compact_size, 'compact_input_reservation': 6301 + compact_size + 512,
              'input_cap': 24000, 'output_reservation': 3118 + request['max_tokens'], 'output_cap': 4096,
              'per_request_byte_cap': 12000, 'only_structural_JSON_whitespace_removed': True,
              'artifact_values_equal': True, 'outside_artifact_and_other_request_fields_unchanged': True,
              'original_request_sha256': sha(encode(request)), 'diagnostic_compact_request_sha256': sha(encode(replacement)),
              'canonical_failed_outcome': rows[2]['outcome'], 'canonical_summary_unknown_usage_rows': summary['unknown_usage_rows'],
              'canonical_known_tokens': summary['known_total_tokens'], 'canonical_known_attempts': summary['known_provider_attempts'],
              'canonical_reservation_charge': summary['charged_reservations'], 'canonical_latch': summary['latch'],
              'observation_operations': operations, 'restored_observations': 48,
              'observation_index_sha256': sha((observations / 'observation-index.json').read_bytes()),
              'observation_provenance_binding': index['execution_provenance_sha256'],
              'full_F1': None, 'first_aggregate_score_started': False,
              'external_HTTP_added': 0, 'model_or_scorer_executed': False,
              'counterfactual_completion_or_quality_claim': False}
    with (OUT / 'independent-calculation.json').open('x') as stream:
        json.dump(report, stream, indent=2)
        stream.write('\n')
    print(json.dumps({k:v for k,v in report.items() if k != 'journals'}))

if __name__ == '__main__':
    main()
