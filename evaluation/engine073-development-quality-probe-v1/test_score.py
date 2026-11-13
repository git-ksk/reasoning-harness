import copy
from pathlib import Path
import unittest
import score

ROOT = Path(__file__).resolve().parent

class DevelopmentSourceProbeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.cases = score.read_json(ROOT/"cases.json")
        cls.outputs = {a:score.read_jsonl(ROOT/(a+"-output.jsonl")) for a in score.ARMS}

    def test_case_inventory(self):
        cases = score.checked_cases(self.cases)
        self.assertEqual(len(cases),40)
        self.assertEqual(sum(c["expected_relevant"] for c in cases.values()),20)

    def test_observed_source_outputs_and_no_adoption(self):
        result=score.summarize(self.cases,self.outputs)
        self.assertEqual(result["model_api_calls"],0)
        self.assertIsNone(result["formal_f1"])
        self.assertFalse(result["quality_go"])
        self.assertFalse(result["adoption_go"])
        cand=result["arms"]["candidate"]["v30_relevant"]
        self.assertEqual((cand["tp"],cand["fp"],cand["tn"],cand["fn"]),(16,0,20,4))
        self.assertEqual(cand["mismatched_case_ids"],["p04","p11","p12","p15"])
        self.assertAlmostEqual(cand["f1_diagnostic"],8/9)
        self.assertEqual(result["arms"]["baseline"]["v30_relevant"]["tp"],0)
        self.assertEqual(result["arms"]["baseline"]["v30_relevant"]["fn"],20)
        self.assertEqual(result["arms"]["candidate"]["v17_requested_relation"],cand)

    def test_incomplete_duplicate_forged_outputs_refused(self):
        items=self.outputs["candidate"]
        for bad in [items[:-1],items+[items[0]],[items[0]]*40,
                    [{**r,"v30_relevant":1} for r in items],
                    [{**r,"formal_f1":1.0} for r in items]]:
            with self.assertRaises(ValueError):
                score.summarize(self.cases,{**self.outputs,"candidate":bad})

    def test_invalid_case_labels_refused(self):
        for mutate in [
            lambda c:c[0].update(expected_relevant="true"),
            lambda c:c[0].update(id=c[1]["id"]),
            lambda c:c.pop(),
            lambda c:c[0].update(signals=[]),
        ]:
            rows=copy.deepcopy(self.cases)
            mutate(rows)
            with self.assertRaises(ValueError):
                score.summarize(rows,self.outputs)

if __name__=="__main__":
    unittest.main()
