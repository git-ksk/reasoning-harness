"""Tag history checks are offline; do not call GitHub or providers."""
import copy
import unittest
from tag_admission import single_first_push, WORKFLOW_PATH

TAG='cli-repair-paired-real24-development-v1'
SHA='1'*40
CURRENT={'id':123,'path':WORKFLOW_PATH,'event':'push','head_branch':TAG,'head_sha':SHA,'run_attempt':1}
OTHER={'id':124,'path':'.github/workflows/ci.yml','event':'push','head_branch':TAG,'head_sha':SHA,'run_attempt':1}

def pages(*runs):
    return [{'total_count':len(runs),'workflow_runs':list(runs)}]

class TagAdmission(unittest.TestCase):
    def test_single_first_push_is_accepted(self):
        self.assertEqual(single_first_push(pages(CURRENT),TAG,SHA,123)['id'],123)

    def test_another_workflow_at_same_tag_is_harmless(self):
        self.assertEqual(single_first_push(pages(OTHER,CURRENT),TAG,SHA,123)['id'],123)

    def test_repeat_development_workflow_is_rejected(self):
        repeated=dict(CURRENT,id=125)
        with self.assertRaisesRegex(ValueError,'tag_must_have_one_development_run'):
            single_first_push(pages(CURRENT,repeated),TAG,SHA,123)

    def test_tag_replay_across_commits_is_rejected_even_by_other_workflow(self):
        old=dict(OTHER,head_sha='2'*40)
        with self.assertRaisesRegex(ValueError,'tag_cross_head_replay'):
            single_first_push(pages(old,CURRENT),TAG,SHA,123)

    def test_rerun_attempt_and_wrong_run_are_rejected(self):
        for mutated,expected in [(dict(CURRENT,run_attempt=2),123),(CURRENT,456)]:
            with self.subTest(mutated=mutated,expected=expected),self.assertRaisesRegex(ValueError,'tag_not_first_unique_attempt'):
                single_first_push(pages(mutated),TAG,SHA,expected)

    def test_incomplete_or_duplicate_history_fails_closed(self):
        scenarios=[([], 'tag_history_unavailable'),
                   ([{'total_count':2,'workflow_runs':[CURRENT]}], 'tag_history_incomplete'),
                   (pages(CURRENT,CURRENT),'tag_run_id_repeated'),
                   ([{'total_count':1,'workflow_runs':None}], 'tag_history_unavailable')]
        for history,error in scenarios:
            with self.subTest(error=error),self.assertRaisesRegex(ValueError,error):
                single_first_push(history,TAG,SHA,123)

    def test_missing_current_workflow_or_wrong_tag_filter_is_rejected(self):
        for history,error in [(pages(OTHER),'tag_must_have_one_development_run'),
                              (pages(dict(CURRENT,head_branch='other')),'tag_cross_head_replay')]:
            with self.subTest(error=error),self.assertRaisesRegex(ValueError,error):
                single_first_push(history,TAG,SHA,123)
        with self.assertRaisesRegex(ValueError,'tag_name_changed'):
            single_first_push(pages(CURRENT),'other',SHA,123)

if __name__=='__main__': unittest.main()
