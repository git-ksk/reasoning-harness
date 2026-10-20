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
    def test_baseline_profile_legally_has_no_safety_observation(self):
        a=normal_fixture();a[0]['result']['safety_runtime']={'profile':'baseline'};a[0]['result']['safety_observations']=[]
        del a[1]['events'][6]
        for n,e in enumerate(a[1]['events']):e['sequence']=n
        self.assertEqual(scorer.check_terminal_order(a[1],a[0]['result'])['result']['status'],'grounded_answer')
        with self.assertRaisesRegex(ValueError,'unfrozen_safety_runtime'):capture.bind_normal(*a)
    def verification_path(self):
        a=normal_fixture();a[1]['events'][6]['observation']['observation']['disposition']='force_verification'
        final=copy.deepcopy(a[1]['events'][7]['result']);final.update(status='requires_verification',text=None,uncovered_propositions=[{'key':'value','value':'4'}])
        a[1]['events'][7]['result']=final;a[0]['result']['finalization']=final
        return a
    def test_unchanged_retry_tail_is_complete_withholding(self):
        a=self.verification_path();tail=copy.deepcopy(a[1]['events'][1]);tail.update(sequence=8,phase='finalization_retry_resolution',round=1);a[1]['events'].append(tail)
        self.assertEqual(capture.bind_normal(*a)['terminal_status'],'requires_verification')
    def test_changed_retry_tail_is_not_no_progress(self):
        a=self.verification_path();tail=copy.deepcopy(a[1]['events'][1]);tail.update(sequence=8,phase='finalization_retry_resolution',round=1);tail['outcome']['artifact']['claims'].append({'id':'new'})
        a[1]['events'].append(tail)
        with self.assertRaisesRegex(ValueError,'events_after_final_safety'):capture.bind_normal(*a)
    def test_fabricated_round_two_after_grounded_answer(self):
        a=normal_fixture();extra=copy.deepcopy(a[1]['events'][1:]);extra=[e for e in extra if e['kind'] not in ['model_request','model_response']]
        for e in extra:
            e['sequence']=len(a[1]['events']);e['round']=2
            if e['kind']=='answer_safety_state':e['observation']['render_round']=2
            a[1]['events'].append(e)
        a[0]['result']['safety_observations']=[e['observation'] for e in a[1]['events'] if e['kind']=='answer_safety_state']
        with self.assertRaisesRegex(ValueError,'retry_after_nonverification'):capture.bind_normal(*a)
    def test_orphan_renderer_record_rejected(self):
        a=normal_fixture();a[1]['events'].insert(5,copy.deepcopy(a[1]['events'][4]))
        for n,e in enumerate(a[1]['events']):e['sequence']=n
        with self.assertRaisesRegex(ValueError,'renderer_candidate_without_render'):capture.bind_normal(*a)
    def test_canonical_recovery_after_grounded_candidate_rejected(self):
        a=normal_fixture();render=copy.deepcopy(a[1]['events'][4]);render['source']='canonical_verified_target'
        state=copy.deepcopy(a[1]['events'][5]);state['stage']='canonical_verified_target'
        a[1]['events'][6:6]=[render,state]
        for n,e in enumerate(a[1]['events']):e['sequence']=n
        with self.assertRaisesRegex(ValueError,'canonical_recovery_precondition'):capture.bind_normal(*a)
if __name__=='__main__':unittest.main()
