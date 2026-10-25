"""One-shot tag admission from *unfiltered*, calendar-windowed GitHub push history.

The GitHub Actions workflow-specific endpoint can 404 before first registration.
A branch=tag filter is not adequate evidence of all previous tag usages.
The repository-wide endpoint also caps results near 1000 even if total_count is
larger, so query date partitions and abort if any partition saturates the cap.
This is a visible-history guard, not proof against administrative history deletion.
"""
import argparse
from datetime import date, timedelta, datetime, timezone
import hashlib
import json
from pathlib import Path
import re
import subprocess

WORKFLOW_PATH='.github/workflows/cli-repair-paired-development.yml'
TAG='cli-repair-paired-real24-development-v1'

def windows_since(first,last):
    if not isinstance(first,date) or not isinstance(last,date) or first>last:
        raise ValueError('invalid_history_time_range')
    windows=[];cursor=first
    while cursor<=last:
        next_month=(cursor.replace(day=28)+timedelta(days=4)).replace(day=1)
        end=min(last,next_month-timedelta(days=1))
        windows.append((cursor.isoformat(),end.isoformat()))
        cursor=end+timedelta(days=1)
    return windows

def single_first_push(history,tag,source_sha,run_id):
    if tag!=TAG or not re.fullmatch(r'[0-9a-f]{40}',str(source_sha)):
        raise ValueError('tag_or_source_identity_changed')
    if not isinstance(history,dict):raise ValueError('history_snapshot_not_dict')
    try:
        start=date.fromisoformat(history['repository_created_date_utc'])
        stop=date.fromisoformat(history['checked_through_date_utc'])
        expected=windows_since(start,stop)
    except (TypeError,ValueError,KeyError) as error:
        raise ValueError('history_range_invalid') from error
    batches=history.get('windows')
    if not isinstance(batches,list) or len(batches)!=len(expected):
        raise ValueError('history_windows_incomplete')
    runs=[];totals=[]
    for batch,(first,last) in zip(batches,expected):
        if not isinstance(batch,dict) or (batch.get('start'),batch.get('end'))!=(first,last):
            raise ValueError('history_window_gap_or_overlap')
        pages=batch.get('pages')
        if not isinstance(pages,list) or not pages:raise ValueError('history_window_not_available')
        section=[]
        for page in pages:
            if not isinstance(page,dict) or not isinstance(page.get('workflow_runs'),list):
                raise ValueError('history_page_invalid')
            if len(page['workflow_runs'])>100:raise ValueError('history_page_over_limit')
            section.extend(page['workflow_runs'])
        # GitHub Actions run search silently caps at 1000 results, even when
        # total_count reports more. Refuse anything at or beyond the limit.
        if len(section)>=1000 or any(type(p.get('total_count')) is not int or
                                     p['total_count']>=1000 or
                                     p['total_count']!=len(section) for p in pages):
            raise ValueError('history_month_incomplete_or_at_api_cap')
        for run in section:
            if not isinstance(run,dict) or run.get('event')!='push':
                raise ValueError('history_not_push_only')
            timestamp=run.get('created_at','')
            if not isinstance(timestamp,str) or not first<=timestamp[:10]<=last:
                raise ValueError('history_run_outside_window')
        runs.extend(section);totals.append(len(section))
    ids=[r.get('id') for r in runs]
    if any(type(x) is not int for x in ids) or len(set(ids))!=len(ids):
        raise ValueError('history_run_ids_duplicated_or_invalid')
    matched=[r for r in runs if r.get('head_branch')==tag]
    if not matched:raise ValueError('tag_not_present_in_full_history')
    if any(r.get('head_sha')!=source_sha for r in matched):
        raise ValueError('tag_cross_head_replay')
    eligible=[r for r in matched if r.get('path')==WORKFLOW_PATH]
    if len(eligible)!=1:
        raise ValueError('tag_must_have_one_development_run')
    current=eligible[0]
    if str(current.get('id'))!=str(run_id) or current.get('run_attempt')!=1:
        raise ValueError('tag_not_first_unique_attempt')
    return {'run_id':current['id'],'repository_push_runs':len(runs),
            'tag_runs':matched,'date_window_counts':totals,
            'run_ids_sha256':hashlib.sha256(json.dumps(sorted(ids),separators=(',',':')).encode()).hexdigest()}

def github_json(*args):
    return json.loads(subprocess.check_output(['gh','api',*args],timeout=110))

def collect_history(repo):
    if not re.fullmatch(r'[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+',repo):
        raise ValueError('invalid_repository')
    metadata=github_json('repos/'+repo)
    created=date.fromisoformat(metadata['created_at'][:10]);today=datetime.now(timezone.utc).date()
    snapshot={'repository_created_date_utc':created.isoformat(),
              'checked_through_date_utc':today.isoformat(),'windows':[]}
    for start,end in windows_since(created,today):
        url=f'repos/{repo}/actions/runs?event=push&created={start}..{end}&per_page=100'
        pages=github_json('--paginate','--slurp',url)
        snapshot['windows'].append({'start':start,'end':end,'pages':pages})
    return snapshot

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--repo',required=True);p.add_argument('--tag',required=True)
    p.add_argument('--source-sha',required=True);p.add_argument('--run-id',required=True)
    p.add_argument('--output',type=Path,required=True)
    a=p.parse_args()
    snapshot=collect_history(a.repo)
    proof=single_first_push(snapshot,a.tag,a.source_sha,a.run_id)
    summary={'identity':'complete-month-windowed-repository-push-history-v1',
             'repo':a.repo,'source_sha':a.source_sha,'tag':a.tag,
             'repository_created_date_utc':snapshot['repository_created_date_utc'],
             'checked_through_date_utc':snapshot['checked_through_date_utc'],
             **proof}
    with a.output.open('x') as f:json.dump(summary,f,indent=2);f.write('\n')
    print(json.dumps({'complete_history_check':'PASS','date_windows':len(summary['date_window_counts']),
                      'total_push_runs':summary['repository_push_runs'],'tag_runs':len(summary['tag_runs'])}))

if __name__=='__main__':main()
