import copy,unittest
import scorer,capture
from test_review_boundaries import normal_fixture
class TerminalChain(unittest.TestCase):
    def test_actual_two_stage_path(self):
        self.assertEqual(capture.bind_normal(*normal_fixture())['terminal_status'],'grounded_answer')
    def test_single_ungated_terminal_rejected(self):
        a=normal_fixture();t=a[1];t['events']=t['events'][:6];a[0]['result']['safety_observations']=[]
        with self.assertRaisesRegex(ValueError,'missing_renderer_or_safety_terminal'):scorer.check_terminal_order(t,a[0]['result'])
    def test_duplicate_safety_rejected(self):
        t=normal_fixture()[1];t['events'].append(copy.deepcopy(t['events'][-1]));t['events'][-1]['sequence']=8
        with self.assertRaisesRegex(ValueError,'duplicate_or_reordered'):scorer.check_terminal_order(t,normal_fixture()[0]['result'])
    def test_reversed_stages_rejected(self):
        t=normal_fixture()[1];t['events'][5]['stage']='answer_safety';t['events'][7]['stage']='renderer_candidate'
        with self.assertRaisesRegex(ValueError,'missing_renderer_or_safety'):scorer.check_terminal_order(t,normal_fixture()[0]['result'])
    def test_changed_final_display_rejected(self):
        a=normal_fixture();a[1]['events'][-1]['result']=copy.deepcopy(a[1]['events'][-1]['result']);a[1]['events'][-1]['result']['text']='swapped'
        with self.assertRaisesRegex(ValueError,'safety_final_result_mismatch'):capture.bind_normal(*a)
    def test_partial_trace_rejected(self):
        t=normal_fixture()[1];t['completion_status']='partial'
        with self.assertRaisesRegex(ValueError,'trace_not_completed'):scorer.check_terminal_order(t,normal_fixture()[0]['result'])
    def test_event_after_safety_rejected(self):
        t=normal_fixture()[1];t['events'].append({'sequence':8,'kind':'evidence_admission'})
        with self.assertRaisesRegex(ValueError,'events_after_final_safety'):scorer.check_terminal_order(t,normal_fixture()[0]['result'])
    def test_missing_safety_observation_rejected(self):
        a=normal_fixture();del a[1]['events'][6]
        for i,e in enumerate(a[1]['events']):e['sequence']=i
        with self.assertRaisesRegex(ValueError,'trace_output_safety_inventory'):capture.bind_normal(*a)
    def test_safety_round_swap_rejected(self):
        a=normal_fixture();a[1]['events'][6]['round']=2
        with self.assertRaisesRegex(ValueError,'safety_wrong_render_round'):capture.bind_normal(*a)
    def test_safety_target_swap_rejected(self):
        a=normal_fixture();a[1]['events'][6]['observation']['observation']['target']={'key':'other','value':'4'}
        with self.assertRaisesRegex(ValueError,'safety_target_inventory'):capture.bind_normal(*a)
    def test_safety_result_swap_rejected(self):
        a=normal_fixture();a[1]['events'][-1]['result']=copy.deepcopy(a[1]['events'][-1]['result']);a[1]['events'][-1]['result']['text']='swapped'
        with self.assertRaisesRegex(ValueError,'safety_final_result_mismatch'):capture.bind_normal(*a)
if __name__=='__main__':unittest.main()
