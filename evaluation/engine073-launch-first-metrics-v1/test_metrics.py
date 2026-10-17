import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

spec = importlib.util.spec_from_file_location('metrics', Path(__file__).with_name('metrics.py'))
m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)

class MetricsTests(unittest.TestCase):
    def test_reject_all_does_not_look_precise(self):
        s=m.binary_metrics([(True,False)]*30+[(False,False)]*30)
        self.assertEqual(s['confusion'], dict(tp=0,fp=0,tn=30,fn=30))
        self.assertIsNone(s['precision']);self.assertEqual(s['recall'],0)
        self.assertEqual(s['f1_positive'],0);self.assertEqual(s['accuracy'],.5)
        self.assertEqual(s['specificity'],1);self.assertEqual(s['balanced_accuracy'],.5)
        self.assertEqual(s['predicted_positive_rate'],0);self.assertIsNone(s['mcc'])

    def test_formula_with_false_accept_and_false_withhold(self):
        s=m.binary_metrics([(True,True)]*3+[(True,False)]*2+[(False,True)]+[(False,False)]*4)
        self.assertEqual(s['precision'],.75);self.assertEqual(s['recall'],.6)
        self.assertAlmostEqual(s['f1_positive'],2/3)
        self.assertEqual(s['false_accept_rate'],.2);self.assertEqual(s['false_withhold_rate'],.4)
        self.assertEqual(s['balanced_accuracy'],.7)

    def test_perfect(self):
        s=m.binary_metrics([(True,True),(False,False)])
        for k in ('precision','recall','f1_positive','specificity','accuracy','balanced_accuracy','mcc'):
            self.assertEqual(s[k],1)

    def test_empty_or_negative_only_undefined(self):
        self.assertIsNone(m.binary_metrics([])['f1_positive'])
        s=m.binary_metrics([(False,False)])
        self.assertIsNone(s['recall']);self.assertIsNone(s['f1_positive'])
        self.assertIsNone(s['balanced_accuracy'])

    def test_missing_value_not_negative(self):
        for bad in (None,0,1,'false'):
            with self.assertRaises(ValueError):m.binary_metrics([(False,bad)])

    def make_archive(self,root):
        d=root/'evaluation/engine073-launch-independent-v22';d.mkdir(parents=True)
        (d/'first-summary.json').write_text('{}')
        f=d/'final-artifact-manifest.json'
        f.write_text(json.dumps({'versions':{'v22':{'first-summary.json':m.sha(d/'first-summary.json')}}}))
        return d,m.sha(f)

    def test_anchored_inventory_rejects_tamper_and_extra_file(self):
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp);d,anchor=self.make_archive(root)
            self.assertEqual(m.verified_versions(root,'v22',anchor)[1],2)
            (d/'extra').write_text('x')
            with self.assertRaisesRegex(ValueError,'Inventory'):m.verified_versions(root,'v22',anchor)
            (d/'extra').unlink();(d/'first-summary.json').write_text('{"modified":true}')
            with self.assertRaisesRegex(ValueError,'Artifact'):m.verified_versions(root,'v22',anchor)

    def test_manifest_swap_rejected(self):
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp);d,anchor=self.make_archive(root)
            (d/'final-artifact-manifest.json').write_text('{"versions":{}}')
            with self.assertRaisesRegex(ValueError,'anchor'):m.verified_versions(root,'v22',anchor)

    def test_operational_failure_is_unmeasured(self):
        with tempfile.TemporaryDirectory() as temp:
            d=Path(temp);s={'incomplete':1,'operational_errors':['stopped'],'observed_cases':0,'planned_cases':60}
            (d/'first-summary.json').write_text(json.dumps(s));(d/'first-results.json').write_text(json.dumps({'summary':s,'cases':[]}))
            self.assertEqual(m.corpus_metrics(d)['status'],'NOT_MEASURED')

    def make_corpus(self,d):
        expected=[True,True,False,False]
        observations=[(True,True),(False,False),(True,False),(False,False)]
        cases=[];rows=[]
        for i,(e,observed) in enumerate(zip(expected,observations)):
            c={'id':str(i),'polarity':'positive' if e else 'negative','expected':{axis:e for axis in m.AXES}}
            cases.append(c)
            rows.append({**c,'observed':dict(zip(m.AXES,observed)), 'checks':{axis:e==o for axis,o in zip(m.AXES,observed)}})
        s={'candidate_source_sha':'fixed-test-source','incomplete':0,'operational_errors':[],'planned_cases':4,'observed_cases':4,'passed':2,'failed':2}
        (d/'cases.json').write_text(json.dumps({'cases':cases}))
        (d/'first-summary.json').write_text(json.dumps(s));(d/'first-results.json').write_text(json.dumps({'summary':s,'cases':rows}))

    def test_axes_are_separate_not_joint_f1(self):
        with tempfile.TemporaryDirectory() as temp:
            d=Path(temp);self.make_corpus(d);s=m.corpus_metrics(d)
            self.assertEqual(s['axes']['v17_requested_relation']['f1_positive'],.5)
            self.assertAlmostEqual(s['axes']['v30_relevant']['f1_positive'],2/3)
            self.assertEqual(s['joint_first_passed'],2)

    def test_frozen_label_mismatch_not_rescored(self):
        with tempfile.TemporaryDirectory() as temp:
            d=Path(temp);self.make_corpus(d)
            rows=json.loads((d/'first-results.json').read_text());rows['cases'][0]['expected']['v30_relevant']=False
            (d/'first-results.json').write_text(json.dumps(rows))
            with self.assertRaisesRegex(ValueError,'label'):m.corpus_metrics(d)

if __name__ == '__main__':unittest.main(verbosity=2)
