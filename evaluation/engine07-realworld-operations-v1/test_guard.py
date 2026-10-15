import unittest
from guard import ProviderArmGuard, Outcome

class GuardTests(unittest.TestCase):
    def test_known_exhaustion_starts_no_request(self):
        guard=ProviderArmGuard(token_budget=100,known_quota_exhausted=True)
        def forbidden(): raise AssertionError('must not call the provider')
        decision,value=guard.invoke(20,forbidden)
        self.assertFalse(decision.allowed);self.assertIsNone(value);self.assertEqual(guard.started_invocations,0)
    def test_first_quota_suppresses_candidate_helper_and_remaining_cases(self):
        guard=ProviderArmGuard(token_budget=100);calls=[]
        def quota(): calls.append(1);return Outcome('quota',None,1), 'original failure'
        self.assertEqual(guard.invoke(20,quota)[1],'original failure')
        for _ in range(60):self.assertFalse(guard.invoke(20,quota)[0].allowed)
        self.assertEqual(len(calls),1);self.assertEqual(guard.latch,'typed_quota')
    def test_two_correlated_capacity_failures_latch(self):
        guard=ProviderArmGuard(token_budget=100)
        for failure in ('rate_limit','provider_unavailable'):
            guard.invoke(20,lambda:(Outcome(failure,1,1),None))
        self.assertFalse(guard.before_invocation(20).allowed)
    def test_success_resets_consecutive_capacity_counter(self):
        guard=ProviderArmGuard(token_budget=100)
        for failure in ('timeout',None,'timeout'):
            guard.invoke(20,lambda:(Outcome(failure,1,1),None))
        self.assertIsNone(guard.latch)
    def test_unknown_usage_stops_instead_of_assuming_zero(self):
        guard=ProviderArmGuard(token_budget=100)
        guard.invoke(20,lambda:(Outcome(None,None,1),None))
        self.assertEqual(guard.latch,'usage_unavailable');self.assertFalse(guard.before_invocation(20).allowed)
    def test_missing_attempts_fail_closed(self):
        guard=ProviderArmGuard(token_budget=100)
        guard.invoke(20,lambda:(Outcome(None,10,None),None))
        self.assertEqual(guard.latch,'attempt_accounting_unavailable')
    def test_reserve_stops_before_exceeding_self_budget(self):
        guard=ProviderArmGuard(token_budget=100)
        guard.invoke(70,lambda:(Outcome(None,70,1),None))
        self.assertFalse(guard.before_invocation(31).allowed);self.assertEqual(guard.started_invocations,1)
    def test_unexpected_actual_overspend_stops_following_requests(self):
        guard=ProviderArmGuard(token_budget=100)
        guard.invoke(20,lambda:(Outcome(None,101,1),None))
        self.assertEqual(guard.latch,'arm_token_budget_exhausted')
    def test_interrupted_callback_never_retries(self):
        guard=ProviderArmGuard(token_budget=100)
        def fail():raise RuntimeError('interrupted')
        with self.assertRaises(RuntimeError):guard.invoke(20,fail)
        self.assertFalse(guard.before_invocation(20).allowed)
    def test_provider_arms_are_independent(self):
        groq=ProviderArmGuard(token_budget=100,known_quota_exhausted=True);google=ProviderArmGuard(token_budget=100)
        self.assertFalse(groq.before_invocation(20).allowed);self.assertTrue(google.before_invocation(20).allowed)
    def test_no_parallel_invocations_within_an_arm(self):
        guard=ProviderArmGuard(token_budget=100);guard.before_invocation(20)
        with self.assertRaises(RuntimeError):guard.before_invocation(20)
    def test_invalid_budget_reserve_and_usage_never_create_capacity(self):
        for budget in (0,-1,True):
            with self.assertRaises(ValueError):ProviderArmGuard(token_budget=budget)
        guard=ProviderArmGuard(token_budget=100)
        with self.assertRaises(ValueError):guard.before_invocation(0)
        guard.invoke(20,lambda:(Outcome(None,-1,1),None));self.assertEqual(guard.latch,'invalid_usage')

if __name__=='__main__':unittest.main()
