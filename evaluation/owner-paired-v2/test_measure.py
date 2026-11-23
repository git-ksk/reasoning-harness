import base64
import copy
import gzip
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import measure as m

def fixture():
    q, sources, configs = [], {}, {}
    for i in range(12):
        k=f"doc{i}"
        h=m.sha(k.encode()); sources[k]=h
        url="https://raw.githubusercontent.com/example/repo/"+"a"*40+"/doc.md"
        ob={"context_summary":"Invented unit-test source","document_id":k,"locator":"doc.md:1-1","origin_group":"example","quote":"Invented source sentence.","scope":{},"source_version":"sha256:"+h,"synthetic_mutation":False,"url":url}
        ev={"id":k+"-e","source":k+"-s","observation":json.dumps(ob),"facts":{"example.key":"invented"},"acquisition_metadata":{}}
        cap={"id":"source","kind":"captured_evidence","read_only":True,"supported_fact_keys":["example.key"],"evidence":[ev],"admission":{}}
        r={"evidence_id":ev["id"],"source_id":ev["source"],"proposition":{"key":"example.key","value":"invented"},"url":url,"source_version":"sha256:"+h,"evidence_sha256":"a"*64}
        configs[k]={"schema_version":"reason-config-v1","resolution":{"investigation":{"capabilities":[cap],"max_actions":4,"max_no_progress_rounds":2,"max_rounds":3,"max_targets":3,"planner_max_tokens":512}},"answer_context":{"schema_version":"reason-answer-context-config-v1","sources":[r],"unresolved_inputs":[]}}
        q.extend({"case_id":f"q{i}-{j}","source_id":k,"question":f"Invented standalone question number {i}-{j}?"} for j in range(2))
    return {"schema":"owner-paired-v2","questions":q,"sources":sources,"configs":configs}

def secret(p): return base64.b64encode(gzip.compress(m.encode(p),mtime=0)).decode()
class Controls(unittest.TestCase):
    def test_tag_roundtrip(self):
        p=fixture()
        self.assertEqual(m.parse_secret(secret(p),m.tag(p)),p)
    def test_config_drift_does_not_create_new_cohort(self):
        p=fixture(); c=copy.deepcopy(p); c["configs"]["doc0"]["answer_context"]["sources"][0]["evidence_sha256"]="b"*64
        self.assertEqual(m.cohort(p),m.cohort(c))
        self.assertNotEqual(m.tag(p),m.tag(c))
        with self.assertRaises(ValueError):m.parse_secret(secret(c),m.tag(p))
    def test_alias_order_does_not_create_new_cohort(self):
        p=fixture(); c=copy.deepcopy(p); c["questions"].reverse(); c["questions"][0]["case_id"]="renamed"
        self.assertEqual(m.cohort(p),m.cohort(c))
    def test_no_gold_labels_extra_fields_or_nonlocal_capability(self):
        for mutate in (lambda p:p.update(gold={}),lambda p:p["questions"][0].update(label="positive"),lambda p:p["configs"]["doc0"]["resolution"]["investigation"]["capabilities"][0].update(kind="http")):
            p=fixture();mutate(p)
            with self.assertRaises(ValueError):m.validate(p)
    def test_duplicate_case_and_source_drift(self):
        p=fixture();p["questions"][1]["case_id"]=p["questions"][0]["case_id"]
        with self.assertRaises(ValueError):m.validate(p)
        p=fixture();p["sources"]["doc0"]="b"*64
        with self.assertRaises(ValueError):m.validate(p)
    def test_stage_exclusive(self):
        p=fixture()
        with tempfile.TemporaryDirectory() as t:
            r=Path(t)/"input"
            with patch.dict(os.environ,{"OWNER_EVAL_INPUT":secret(p)}):m.stage(r,m.tag(p))
            self.assertEqual(m.load(r/"payload.json"),p)
            self.assertFalse((r/"gold.json").exists())
            with patch.dict(os.environ,{"OWNER_EVAL_INPUT":secret(p)}):
                with self.assertRaises(FileExistsError):m.stage(r,m.tag(p))
    def test_measure_rejects_ordinary_local_or_retry(self):
        with patch.dict(os.environ,{"GITHUB_ACTIONS":"false","GITHUB_RUN_ATTEMPT":"1"},clear=True):
            with self.assertRaisesRegex(ValueError,"one_shot_context"):m.measure(Path("/nonexistent"),Path("/nonexistent"),0,"x")
        with patch.dict(os.environ,{"GITHUB_ACTIONS":"true","GITHUB_RUN_ATTEMPT":"2","GITHUB_REF":"refs/tags/x"},clear=True):
            with self.assertRaisesRegex(ValueError,"one_shot_context"):m.measure(Path("/nonexistent"),Path("/nonexistent"),0,"x")
    def test_preflight_denies_stage_config_mutation(self):
        p=fixture()
        with tempfile.TemporaryDirectory() as t:
            r=Path(t)/"input"
            with patch.dict(os.environ,{"OWNER_EVAL_INPUT":secret(p)}):m.stage(r,m.tag(p))
            (r/"configs/doc0.json").write_text("{}")
            with patch.object(m,"build_check",return_value={}):
                with self.assertRaisesRegex(ValueError,"staged_config_drift"):m.preflight(r,Path("unused"))
if __name__=="__main__":unittest.main()
