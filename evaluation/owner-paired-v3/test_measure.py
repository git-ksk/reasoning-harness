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
    return {"schema":"owner-paired-v3","questions":q,"sources":sources,"configs":configs}

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
class BudgetProtocol(unittest.TestCase):
    def failed(self, output=8192, message=None, cls="usage_budget_exceeded"):
        used={"tracked_from_session_start":True,"model_calls":8,"provider_attempts":9,
              "input_tokens":11000,"output_tokens":output,"total_tokens":11000+output}
        result={"status":"failed","failure":{"failure_class":cls,
                "message":message or "output-token budget exhausted (8192/8192)"},
                "partial_diagnostics":{"usage":{"run":used}}}
        return json.dumps({"schema_version":"reason-cli-output-v1","command":"ask","result":result}).encode()
    def test_exact_exhaustion_recorded_not_success(self):
        obs={"status":"incomplete"}
        m.terminal_observation(self.failed(),1,obs)
        self.assertEqual(obs["status"],"budget_exhausted")
        self.assertIsNone(obs["answer_status"])
        self.assertNotIn("final_sha256",obs)
    def test_unknown_failure_or_invalid_accounting_does_not_continue(self):
        for raw,rc in [(self.failed(8191),1),(self.failed(8193),1),
                       (self.failed(cls="provider_failure"),1),(self.failed(),None),
                       (self.failed(message="output-token budget exhausted (4096/4096)"),1)]:
            with self.subTest(rc=rc):
                obs={"status":"incomplete"}
                with self.assertRaises(ValueError):m.terminal_observation(raw,rc,obs)
                self.assertEqual(obs["status"],"incomplete")
    def test_success_requires_actual_final_display(self):
        raw=json.loads(self.failed())
        r=raw["result"];r["status"]="succeeded";r.pop("failure")
        r["finalization"]={"text":"Synthetic answer","status":"qualified_partial_answer"}
        obs={"status":"incomplete"}
        m.terminal_observation(json.dumps(raw).encode(),0,obs)
        self.assertEqual(obs["status"],"complete")
        self.assertEqual(obs["final_sha256"],m.sha(b"Synthetic answer"))
    def test_policy_identity_includes_actual_budget_and_separates_v2(self):
        p=fixture()
        old=m.sha(m.encode({"questions":sorted((x["question"],p["sources"][x["source_id"]]) for x in p["questions"]),
                            "core":m.ARMS,"cli":m.CLI,"model":m.MODEL}))
        self.assertNotEqual(m.cohort(p),old)
        self.assertEqual(m.POLICY["predecessor_run"],45101280387)
        b={"arms":{a:{"binary":"test"} for a in m.ARMS}}
        for arm in m.ARMS:
            args=m.argv(Path("/tmp/test"),b,p["questions"][0],arm,0)
            for f,v in [("--max-model-calls",12),("--max-output-tokens",8192),("--max-total-tokens",32192)]:
                self.assertEqual(args[args.index(f)+1],str(v))
    def test_billing_retains_no_overage_and_reports_expanded_caps(self):
        sub={"payg_enabled":False}
        budget={"currency":"USD","api_budget":{"currency":"USD","payg_enabled":False,"initial_budget":10,"usage_percentage":1}}
        models={"data":[{"id":m.MODEL}]}
        r=m.billing_for_policy(sub,budget,models)
        self.assertIs(r["api_payg_disabled"],True)
        self.assertEqual(r["total_token_upper_bound"],48*32192)
        self.assertEqual(r["output_token_upper_bound"],48*8192)
        with self.assertRaises(Exception):m.billing_for_policy({"payg_enabled":True},budget,models)

if __name__=="__main__":unittest.main()
