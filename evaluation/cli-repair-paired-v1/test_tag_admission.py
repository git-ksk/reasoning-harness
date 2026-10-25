"""No GitHub or external model calls: adversarial complete-history tests."""
from datetime import date
import unittest
from tag_admission import single_first_push,windows_since,WORKFLOW_PATH,TAG

SHA='1'*40
CURRENT={'id':123,'path':WORKFLOW_PATH,'event':'push','head_branch':TAG,'head_sha':SHA,'run_attempt':1,'created_at':'2026-10-25T12:00:00Z'}
OTHER={'id':124,'path':'.github/workflows/ci.yml','event':'push','head_branch':TAG,'head_sha':SHA,'run_attempt':1,'created_at':'2026-10-25T12:00:00Z'}
BASE={'id':125,'path':'.github/workflows/ci.yml','event':'push','head_branch':'main','head_sha':'3'*40,'run_attempt':1,'created_at':'2026-09-08T10:00:00Z'}

def pages(*runs):return [{'total_count':len(runs),'workflow_runs':list(runs)}]

def history(aug=(),sep=(BASE,),octo=(CURRENT,)):
    return {'repository_created_date_utc':'2026-08-29','checked_through_date_utc':'2026-10-26',
            'windows':[{'start':a,'end':b,'pages':p} for (a,b),p in zip(
                windows_since(date(2026,8,29),date(2026,10,26)),
                [pages(*aug),pages(*sep),pages(*octo)])]}

class TagAdmission(unittest.TestCase):
    def check(self,x):return single_first_push(x,TAG,SHA,123)
    def test_realistic_complete_unfiltered_repository_history(self):
        result=self.check(history(octo=(OTHER,CURRENT)))
        self.assertEqual(result['repository_push_runs'],3)
        self.assertEqual(result['date_window_counts'],[0,1,2])
        self.assertEqual(len(result['tag_runs']),2)

    def test_another_workflow_at_same_tag_is_harmless(self):
        self.assertEqual(self.check(history(octo=(OTHER,CURRENT)))['run_id'],123)

    def test_repeat_development_workflow_rejected(self):
        with self.assertRaisesRegex(ValueError,'tag_must_have_one_development_run'):
            self.check(history(octo=(CURRENT,dict(CURRENT,id=126))))

    def test_replay_from_an_older_month_is_rejected(self):
        replay=dict(OTHER,head_sha='2'*40,created_at='2026-09-09T08:00:00Z')
        with self.assertRaisesRegex(ValueError,'tag_cross_head_replay'):
            self.check(history(sep=(BASE,replay)))

    def test_replay_from_another_workflow_same_month_rejected(self):
        with self.assertRaisesRegex(ValueError,'tag_cross_head_replay'):
            self.check(history(octo=(dict(OTHER,head_sha='2'*40),CURRENT)))

    def test_wrong_current_attempt_or_run_rejected(self):
        for candidate,run_id in [(dict(CURRENT,run_attempt=2),123),(CURRENT,456)]:
            with self.subTest(run_id=run_id),self.assertRaisesRegex(ValueError,'tag_not_first_unique_attempt'):
                single_first_push(history(octo=(candidate,)),TAG,SHA,run_id)

    def test_incomplete_page_and_github_1000_cap_are_rejected(self):
        for changes in [{'total_count':2}, {'total_count':1000}, {'total_count':1001}, {'workflow_runs':None}]:
            data=history();data['windows'][-1]['pages'][0].update(changes)
            with self.subTest(changes=changes),self.assertRaises(ValueError):self.check(data)

    def test_missing_month_or_repeated_window_rejected(self):
        x=history();x['windows'].pop(1)
        with self.assertRaisesRegex(ValueError,'history_windows_incomplete'):self.check(x)
        x=history();x['windows'][1]['start']='2026-08-29'
        with self.assertRaisesRegex(ValueError,'history_window_gap_or_overlap'):self.check(x)

    def test_cross_window_duplicate_id_fails_closed(self):
        x=history(sep=(dict(BASE,id=123),))
        with self.assertRaisesRegex(ValueError,'history_run_ids_duplicated_or_invalid'):self.check(x)

    def test_date_leak_or_nonpush_event_fails_closed(self):
        for item in [dict(BASE,created_at='2026-10-01T00:00:00Z'),dict(BASE,event='workflow_dispatch')]:
            with self.subTest(item=item),self.assertRaises(ValueError):self.check(history(sep=(item,)))

    def test_wrong_identity_rejected(self):
        with self.assertRaisesRegex(ValueError,'tag_or_source_identity_changed'):
            single_first_push(history(),'wrong',SHA,123)
        with self.assertRaisesRegex(ValueError,'tag_or_source_identity_changed'):
            single_first_push(history(),TAG,'short',123)

    def test_month_partition_stays_contiguous_and_utc_date_inclusive(self):
        self.assertEqual(windows_since(date(2026,8,29),date(2026,10,26)),
            [('2026-08-29','2026-08-31'),('2026-09-01','2026-09-30'),('2026-10-01','2026-10-26')])
        self.assertEqual(windows_since(date(2026,12,31),date(2027,1,1)),
            [('2026-12-31','2026-12-31'),('2027-01-01','2027-01-01')])

    def test_live_workflow_pins_and_execution_identity_match_protocol(self):
        from pathlib import Path
        import json
        here=Path(__file__).resolve().parent;repo=here.parents[1]
        workflow=(repo/'.github/workflows/cli-repair-paired-development.yml').read_text()
        self.assertEqual(json.loads((here/'protocol.json').read_bytes())['execution_tag'],TAG)
        self.assertIn('      - '+TAG,workflow)
        self.assertIn("source=Path('evaluation/"+TAG+"')",workflow)
        self.assertIn("source=Path('evaluation/"+TAG+"')",(repo/'.github/workflows/cli-repair-paired-runtime.yml').read_text())
        self.assertIn('actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1',workflow)
        self.assertIn('actions/upload-artifact@cf430e030ddbb5b0abf93d22962f4752f3646cd9',workflow)
        self.assertNotIn('actions/checkout@v7',workflow)
        self.assertNotIn('actions/upload-artifact@v7',workflow)
