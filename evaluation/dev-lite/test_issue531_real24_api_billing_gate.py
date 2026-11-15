import unittest
import issue531_real24_api_billing_gate as gate

GOOD_SUB={'payg_enabled':False,'plan_name':'Free'}
GOOD_BUD={'currency':'USD','api_budget':{'currency':'USD','initial_budget':10,'usage_percentage':0.5,'payg_enabled':False},'vibe_budget':{'currency':'USD','payg_enabled':True}}
GOOD_MODELS={'data':[{'id':'ministral-8b-2512'}]}

class AccountGateTests(unittest.TestCase):
    def test_good_case(self):
        x=gate.check_account(GOOD_SUB,GOOD_BUD,GOOD_MODELS)
        self.assertEqual(x['billing_gate'],'PASS')
        self.assertIs(x['api_payg_disabled'],True)
    def test_missing_or_enabled_payg(self):
        for payg in [None,True,'false',0]:
            with self.subTest(payg=payg):
                with self.assertRaises(gate.BillingGateError):
                    gate.check_account({**GOOD_SUB,'payg_enabled':payg},GOOD_BUD,GOOD_MODELS)
    def test_api_pool_enabled(self):
        with self.assertRaises(gate.BillingGateError):
            gate.check_account(GOOD_SUB,{**GOOD_BUD,'api_budget':{**GOOD_BUD['api_budget'],'payg_enabled':True}},GOOD_MODELS)
    def test_absent_api_budget(self):
        with self.assertRaises(gate.BillingGateError):
            gate.check_account(GOOD_SUB,{'currency':'USD'},GOOD_MODELS)
    def test_depleted_allowance(self):
        with self.assertRaises(gate.BillingGateError):
            gate.check_account(GOOD_SUB,{**GOOD_BUD,'api_budget':{**GOOD_BUD['api_budget'],'usage_percentage':99}},GOOD_MODELS)
    def test_invalid_percent(self):
        for value in [-1,101,float('nan'),None]:
            with self.subTest(value=value):
                with self.assertRaises(gate.BillingGateError):
                    gate.check_account(GOOD_SUB,{**GOOD_BUD,'api_budget':{**GOOD_BUD['api_budget'],'usage_percentage':value}},GOOD_MODELS)
    def test_mismatched_currency(self):
        with self.assertRaises(gate.BillingGateError):
            gate.check_account(GOOD_SUB,{**GOOD_BUD,'currency':'JPY'},GOOD_MODELS)
    def test_model_unavailable(self):
        with self.assertRaises(gate.BillingGateError):
            gate.check_account(GOOD_SUB,GOOD_BUD,{'data':[]})
    def test_only_allowlisted_get_endpoints(self):
        with self.assertRaises(gate.BillingGateError):
            gate.get_json(None,'fake','/v1/chat/completions')

if __name__=='__main__':unittest.main()
