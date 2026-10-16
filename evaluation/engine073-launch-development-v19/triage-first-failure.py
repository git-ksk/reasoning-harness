from pathlib import Path
import json,hashlib,subprocess
r=Path.cwd();ind=r/'evaluation/engine073-launch-independent-v19';out=r/'evaluation/engine073-launch-development-v19';out.mkdir(exist_ok=False)
s=json.loads((ind/'first-summary.json').read_text());results=json.loads((ind/'first-results.json').read_text())['cases']
assert s['outcome']=='FAIL' and s['passed']==49 and s['positive_v30_withheld']==11 and s['negative_v30_relevant_violations']==0
categories=[
 ('literal printed words vs event/source content',['li19-003','li19-009']),
 ('complete property assertion and its later reference',['li19-011','li19-035','li19-039','li19-049']),
 ('foreign event ownership, quote or owner apposition',['li19-027','li19-029','li19-057','li19-059']),
 ('several typed roles corrected by separate verbs',['li19-033']),
]
failed=[x for x in results if x['status']!='pass'];ids=[i for _,members in categories for i in members]
assert len(ids)==len(set(ids))==len(failed)==11 and set(ids)=={x['id'] for x in failed}
assert all(x['polarity']=='positive' and x['v30_disposition']=='ambiguous' for x in failed)
payload={'identity':'engine073-launch-development-v19-initial-failure-triage','source_sha':s['candidate_source_sha'],'first_summary_sha256':hashlib.sha256((ind/'first-summary.json').read_bytes()).hexdigest(),'first_results_sha256':hashlib.sha256((ind/'first-results.json').read_bytes()).hexdigest(),'first_outcome':s,'classification_kind':'post-observation input-surface composition only; not implementation root cause or independent acceptance','categories':[{'composition':n,'case_ids':members} for n,members in categories],'new_engine_executions':0,'new_provider_calls':0,'labels_or_first_results_changed':False,'source_changed':False}
(out/'initial-failure-triage.json').write_text(json.dumps(payload,indent=2)+'\n')
(out/'README.md').write_text('''# Successor19 — consumed first-failure triage only

The independently authored [v19 first observation](../engine073-launch-independent-v19/README.md) is FAIL49/60 at pinned source f3d9ad15ae2e4fc81dadaed87c4e55fa8bce1c7d. Negative30/30 withheld, positive19/30 retained. All11 failures are positive unresolved/Ambiguous, not operational errors. This bounded synthetic set does not establish real-source, ordinary-CLI/provider or release quality.

The parent read those first failed inputs after the exclusive first score and grouped their visible compositions: literal printed content2, complete-property references4, foreign-event owner/quote/apposition4, and several typed roles with separate correction verbs1. This is a triage index, not proof of an implementation cause. No new Engine execution, mutation to source, relabel, rescore or provider call occurred here.

The next repair must account for scopes and references across these compositions, preserve target/global/unknown-owner negatives and historical contracts, then pin a new source before genuinely new independent acceptance. New words alone and success on these consumed inputs cannot establish general-language quality. Preserve the first drafts, pre-observation scope correction, first observations and all prior failure records. The initial v19 author used agency negation in one pair; before complete freeze and any semantics, that pair was corrected to explicit event denial with original draft inputs/labels/static output retained. Expected polarities and the other58 were unchanged.

The prior [development18 bundle](../engine073-launch-development-v18/README.md) remains unchanged: known900 pass, full1541 pass/0fail/8ignored, strictClippy/fmt, original14, historical597 byte equality,20 fixtures,13 archive files. Those are development checks and do not override this new firstFAIL. Draft PR539 and #520 remain open. #531 fresh real24 is0/24, CLI adoption1/6 and Groq stays stopped. No release/merge/adoption is approved by this triage.
''')
(out/'triage-first-failure.py').write_bytes(Path('/private/tmp/engine073-triage-v19.py').read_bytes())
h={str(p.relative_to(out)):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(out.rglob('*')) if p.is_file()}
(out/'artifact-sha256.json').write_text(json.dumps({'source_sha':s['candidate_source_sha'],'files':h},indent=2)+'\n')
print('saved consumed v19 initial-failure surface triage; no new semantic execution')
