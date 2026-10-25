"""Fail closed on an exclusive single-run development tag using repository-wide Actions history.

The workflow-specific REST endpoint can return 404 before the workflow is
registered on GitHub. Repository-wide tag-filtered run history exists and
preserves cross-source tag replay evidence (within GitHub retention).
"""

WORKFLOW_PATH='.github/workflows/cli-repair-paired-development.yml'

def single_first_push(pages,tag,source_sha,run_id):
    if not isinstance(pages,list) or not pages:
        raise ValueError('tag_history_unavailable')
    if not isinstance(tag,str) or tag!='cli-repair-paired-real24-development-v1':
        raise ValueError('tag_name_changed')
    if not isinstance(source_sha,str) or len(source_sha)!=40:
        raise ValueError('source_sha_invalid')
    runs=[]
    for page in pages:
        if not isinstance(page,dict) or not isinstance(page.get('workflow_runs'),list):
            raise ValueError('tag_history_unavailable')
        runs.extend(page['workflow_runs'])
    if not all(type(page.get('total_count')) is int and page['total_count']==len(runs) for page in pages):
        raise ValueError('tag_history_incomplete')
    if len({r.get('id') for r in runs if isinstance(r,dict)})!=len(runs):
        raise ValueError('tag_run_id_repeated_or_unavailable')
    # Even another workflow's earlier run at this tag pins its original source.
    if any(r.get('event')!='push' or r.get('head_branch')!=tag or r.get('head_sha')!=source_sha for r in runs):
        raise ValueError('tag_cross_head_replay_or_wrong_filter')
    matching=[r for r in runs if r.get('path')==WORKFLOW_PATH]
    if len(matching)!=1:
        raise ValueError('tag_must_have_one_development_run')
    current=matching[0]
    if str(current.get('id'))!=str(run_id) or current.get('run_attempt')!=1:
        raise ValueError('tag_not_first_unique_attempt')
    return current
