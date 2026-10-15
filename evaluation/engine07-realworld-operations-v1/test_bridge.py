import collections,json,pathlib,unittest,zipfile
from guard import ProviderArmGuard, Outcome, FailureClass
from bridge import outcome_from_normal,outcome_from_helper
ROOT=pathlib.Path(__file__).resolve().parents[2]

class BridgeTests(unittest.TestCase):
    def assert_stops(self, outcome, expected):
        guard=ProviderArmGuard(token_budget=100);guard.invoke(20,lambda:(outcome,None))
        self.assertEqual(guard.latch,expected);self.assertFalse(guard.before_invocation(20).allowed)
    def test_known_normal_failure_usage_does_not_mask_quota(self):
        payload={'result':{'status':'failed','failure':{'failure_class':'quota'},'partial_diagnostics':{'complete':False,'usage_is_lower_bound':False,'usage':{'run':{'total_tokens':5,'provider_attempts':1}}}}}
        outcome=outcome_from_normal(payload);self.assertEqual(outcome.tokens,5);self.assert_stops(outcome,'typed_quota')
    def test_known_helper_failure_usage_does_not_mask_quota(self):
        outcome=outcome_from_helper({'operational_failure':'provider_Quota','usage':{'total_tokens':5},'provider_attempts':1})
        self.assertEqual(outcome.failure_class,FailureClass.QUOTA);self.assert_stops(outcome,'typed_quota')
    def test_unbridged_failure_strings_fail_closed_even_with_known_usage(self):
        for raw in ('quota','Quota','provider_Quota','future_failure'):
            self.assert_stops(Outcome(raw,5,1),'unknown_failure_class')
    def test_unknown_envelope_classes_fail_closed(self):
        for raw in ('Quota','provider_Quota','future_failure'):
            self.assert_stops(outcome_from_normal({'result':{'status':'failed','failure':{'failure_class':raw}}}),'unknown_failure_class')
        self.assert_stops(outcome_from_helper({'operational_failure':'Quota','usage':{'total_tokens':5},'provider_attempts':1}),'unknown_failure_class')
    def test_missing_failure_class_and_malformed_frames_are_not_success(self):
        for payload in ({},[],{'result':{'status':'failed','failure':{}}}):
            self.assert_stops(outcome_from_normal(payload),'unknown_failure_class')
        for payload in ({},{'operational_failure':None,'equivalent':None},[]):
            self.assert_stops(outcome_from_helper(payload),'unknown_failure_class')
    def test_success_uses_measured_run_usage(self):
        outcome=outcome_from_normal({'result':{'output_contract':'reason-natural-output-v4','usage':{'run':{'total_tokens':5,'provider_attempts':2}}}})
        self.assertEqual(outcome,Outcome(None,5,2))
        self.assertEqual(outcome_from_helper({'operational_failure':None,'equivalent':False,'usage':{'total_tokens':6},'provider_attempts':1}),Outcome(None,6,1))
    def test_interrupted_lower_bound_usage_is_not_a_full_cost_total(self):
        payload={'result':{'status':'failed','failure':{'failure_class':'cancelled'},'partial_diagnostics':{'complete':False,'usage_is_lower_bound':True,'usage':{'run':{'total_tokens':5,'provider_attempts':1}}}}}
        outcome=outcome_from_normal(payload);self.assertIsNone(outcome.tokens);self.assertIsNone(outcome.provider_attempts);self.assert_stops(outcome,'usage_unavailable')
    def test_actual_first_groq_normal_and_helper_failures_map_strictly(self):
        archive=ROOT/'evaluation/engine07-realworld-v3-results/archives/groq.redacted.zip';normal=[];helper=[]
        with zipfile.ZipFile(archive) as z:
            for name in z.namelist():
                if '/normal-' in name and name.endswith('/stdout.json'):normal.append(outcome_from_normal(json.loads(z.read(name))))
                if name.endswith('/adviser/stdout.json'):helper.append(outcome_from_helper(json.loads(z.read(name))))
        self.assertEqual(collections.Counter(x.failure_class for x in normal),{FailureClass.QUOTA:47,FailureClass.USAGE_BUDGET_UNMEASURABLE:1})
        self.assertEqual(collections.Counter(x.failure_class for x in helper),{FailureClass.QUOTA:13,None:2})
        for outcome in normal+helper:
            if outcome.failure_class is FailureClass.QUOTA:self.assert_stops(outcome,'typed_quota')

if __name__=='__main__':unittest.main()
