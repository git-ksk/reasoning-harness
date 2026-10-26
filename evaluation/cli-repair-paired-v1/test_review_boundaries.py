"""Guards for independently reproduced transport bugs; zero accuracy cases."""
import copy,json,tempfile,unittest
from pathlib import Path
import scorer,capture,corpus
from test_scorer import pair,REPO

def rebind(o,r):r['observation_sha256']=scorer.sha(json.dumps(o,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode())
def normal_fixture():
    provider='mistral';model='engineering-fixture';case={'id':'engineering-only','question':'Engineering transport control','lookup_key':'value','source_ids':['s'],'required_scope':{'product':'unit'}}
    source={'quote':'Value is four.','context_summary':'Structural fixture only.','document_id':'fixture','url':'https://example.invalid/fixture','locator':'unit','source_version':'sha256:'+'1'*64,'origin_group':'fixture','scope':case['required_scope'],'synthetic_mutation':False,'reference_facts':{'value':'4'},'retrieved_at_unix_seconds':100}
    scope={'product':{'kind':'values','values':['unit']}}
    record={'id':'snapshot:engineering-only:0','source':'s','observation':json.dumps({k:source[k] for k in ['quote','context_summary','document_id','url','locator','source_version','origin_group','scope','synthetic_mutation']},sort_keys=True),'facts':source['reference_facts'],'acquisition_metadata':{'scope':scope,'observed_at_unix_seconds':100,'retrieved_at_unix_seconds':100,'claimed_authority_class':'primary'}}
    prop={'key':'value','value':'4'};admitted={**{k:record[k] for k in ['id','source','observation','facts']},'metadata':{'scope':scope,'provenance_class':'primary','temporal':{'effective_from_unix_seconds':100,'effective_until_unix_seconds':200}}}
    artifact={'evidence':[admitted],'claims':[{'id':'c','state':'supported','proposition':prop,'evidence_ids':['snapshot:engineering-only:0']}],
              'verification_receipts':[{'id':'r','claim_id':'c','proposition':prop,'conclusion':'supported','evidence_ids':['snapshot:engineering-only:0'],'verifier':'qualified_structured_fact_equality'}]}
    runtime=json.loads((Path(__file__).parent/'protocol.json').read_text())['normal_bound']['safety_runtime']
    final={'status':'grounded_answer','text':'value = 4','uncovered_propositions':[]};outcome={'verdict':'accept','artifact':artifact};request={'task':'inert transport fixture','max_tokens':16}
    rendered={'text':final['text'],'factual_claims':[{'mode':'grounded','proposition':prop}]}
    response={'model':model,'text':json.dumps(rendered),'usage':{'input_tokens':12,'output_tokens':2,'total_tokens':14},'provider_attempts':1}
    events=[{'kind':'model_request','generation':1,'provider':provider,'model':model,'request':request},
            {'kind':'wire_request','generation':1,'provider':provider,'body':{'model':model}},
            {'kind':'wire_http_response','generation':1,'provider':provider,'status':200},
            {'kind':'model_response','generation':1,'response':response}]
    identity={'correlation_id':'q','attempt':1,'provider':provider,'model':model,'phase':'final_render','round':1}
    trace={'contract':'reason-natural-diagnostic-trace-v1','provider':provider,'model':model,'events':[
        {'sequence':0,'kind':'evidence_admission','target_id':'target','capability_id':'cap','admitted_evidence_ids':['snapshot:engineering-only:0']},
        {'sequence':1,'kind':'grounding_state','outcome':outcome},
        {**identity,'sequence':2,'kind':'model_request','request':request},
        {**identity,'sequence':3,'kind':'model_response','response':{'model':model,'usage':response['usage'],'provider_attempts':1,'bytes':len(response['text'].encode()),'structured':True,'value':rendered}},
        {'sequence':4,'kind':'final_render_state','phase':'final_render','round':1,'source':'model','candidate':{'text':final['text'],'factual_claims':[{'mode':'grounded','proposition':prop}]}},
        {'sequence':5,'kind':'finalization_state','phase':'finalization','round':1,'stage':'renderer_candidate','result':final},
        {'sequence':6,'kind':'answer_safety_state','round':1,'phase':'answer_safety','observation':{'render_round':1,'observation':{'runtime':runtime,'target':prop,'disposition':'preserve_baseline'}}},
        {'sequence':7,'kind':'finalization_state','phase':'finalization','round':1,'stage':'answer_safety','result':final}],
        'completion_status':'completed'}
    payload={'schema_version':'reason-cli-output-v1','command':'ask','result':{'output_contract':'reason-natural-output-v4','task':case['question'],'configuration':{'provider':provider,'model':model},'usage':{'run':{'model_calls':1}},'safety_runtime':runtime,'safety_observations':[trace['events'][6]['observation']],'finalization':final,'final_outcome':outcome,'investigation':{'telemetry':{'targets':[{'id':'target','question':case['question'],'expected_fact_key':'value','origin':'model_proposed_untrusted'}]}}}}
    policy={'sources':{'s':{'scope':scope,'authority_class':'primary','max_age_seconds':100}},'required_scope':scope,'authority_ranks':{'primary':20},'minimum_authority_class':'primary','evaluation_time_unix_seconds':150}
    config={'resolution':{'investigation':{'capabilities':[{'id':'cap','read_only':True,'supported_fact_keys':['value'],'admission':policy}]}}}
    journal=[{'corpus_sha256':'c'*64,'case':case['id'],'request':{'schema_version':'reason-investigation-external-resolver-request-v1','request':{'id':'investigation:target:cap','target':{'target_id':'target','question':case['question'],'expected_fact_key':'value'}}},'response':{'schema_version':'reason-external-resolver-response-v1','contribution':{'kind':'acquired_evidence','evidence':[record]}}}]
    data={'_capture_sha256':'c'*64,'_expected_case_id':case['id'],'cases':[case],'sources':{'s':source}}
    return [payload,trace,journal,config,data,events,provider,model]

class ReviewBoundaries(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.deps=scorer.frozen_dependencies(REPO)
    def test_valid_capture_transport(self):self.assertEqual(capture.bind_normal(*normal_fixture())['usage']['generations'],1)
    def repeated_snapshot_fixture(self):
        args=normal_fixture()
        target=copy.deepcopy(args[0]['result']['investigation']['telemetry']['targets'][0])
        target.update(id='target-two',question='Second target reads the same immutable snapshot')
        args[0]['result']['investigation']['telemetry']['targets'].append(target)
        second=copy.deepcopy(args[2][0]);second['request']['request'].update(id='investigation:target-two:cap',target={k:target[k] for k in ['question','expected_fact_key']})
        second['request']['request']['target']['target_id']=target['id'];args[2].append(second)
        args[1]['events'].insert(1,{'kind':'evidence_admission','target_id':target['id'],'capability_id':'cap','admitted_evidence_ids':[]})
        for n,event in enumerate(args[1]['events']):event['sequence']=n
        return args
    def test_exact_snapshot_reacquisition_keeps_one_admitted_record(self):
        result=capture.bind_normal(*self.repeated_snapshot_fixture())
        self.assertEqual(result['acquired_source_ids'],['s']);self.assertEqual(result['admitted_source_ids'],['s'])
        self.assertEqual(list(result['admission_routes']),['snapshot:engineering-only:0'])
    def test_reacquisition_cannot_hide_collision_or_duplicate_response(self):
        args=self.repeated_snapshot_fixture();args[2][1]['response']['contribution']['evidence'][0]['facts']={'value':'5'}
        with self.assertRaisesRegex(ValueError,'ambiguous_acquired_evidence'):capture.bind_normal(*args)
        args=self.repeated_snapshot_fixture();records=args[2][1]['response']['contribution']['evidence'];records.append(copy.deepcopy(records[0]))
        with self.assertRaisesRegex(ValueError,'ambiguous_acquired_evidence'):capture.bind_normal(*args)
    def test_reacquisition_preserves_request_target_and_single_admission(self):
        args=self.repeated_snapshot_fixture();args[2][1]['request']['request']['id']=args[2][0]['request']['request']['id']
        with self.assertRaisesRegex(ValueError,'duplicate_acquisition_request'):capture.bind_normal(*args)
        args=self.repeated_snapshot_fixture();args[2][1]['request']['request']['target']['target_id']='wrong-target'
        with self.assertRaisesRegex(ValueError,'acquisition_wrong_target'):capture.bind_normal(*args)
        args=self.repeated_snapshot_fixture();args[1]['events'][1]['admitted_evidence_ids']=['snapshot:engineering-only:0']
        with self.assertRaisesRegex(ValueError,'unacquired_admission'):capture.bind_normal(*args)
    def skipped_acquisition_fixture(self):
        args=self.repeated_snapshot_fixture();args[2].pop();args[1]['events'][1]['fact_keys']=[]
        return args
    def test_empty_skipped_action_adds_no_acquisition_credit(self):
        result=capture.bind_normal(*self.skipped_acquisition_fixture())
        self.assertEqual(result['acquired_source_ids'],['s']);self.assertEqual(result['admitted_source_ids'],['s'])
        self.assertEqual(list(result['admission_routes']),['snapshot:engineering-only:0'])
    def test_skipped_action_cannot_admit_or_name_foreign_route(self):
        for key,value in [('admitted_evidence_ids',['snapshot:engineering-only:0']),('fact_keys',['value']),('capability_id','unknown-capability')]:
            with self.subTest(key=key):
                args=self.skipped_acquisition_fixture();args[1]['events'][1][key]=value
                with self.assertRaisesRegex(ValueError,'admission_without_same_target_acquisition'):capture.bind_normal(*args)
        args=self.skipped_acquisition_fixture();args[1]['events'][1]['target_id']='unknown-target'
        with self.assertRaisesRegex(ValueError,'acquisition_target_not_planned'):capture.bind_normal(*args)
    def test_keyless_target_preserves_actual_selector_rules(self):
        for omitted in [False,True]:
            with self.subTest(omitted=omitted):
                args=normal_fixture();target=args[0]['result']['investigation']['telemetry']['targets'][0]
                if omitted:target.pop('expected_fact_key')
                else:target['expected_fact_key']=None
                self.assertEqual(capture.bind_normal(*args)['usage']['generations'],1)
                args[2][0]['request']['request']['target']['expected_fact_key']='other'
                with self.assertRaisesRegex(ValueError,'acquisition_wrong_fact_key'):capture.bind_normal(*args)
        for read_only,keys in [(True,['value','other']),(False,['value'])]:
            with self.subTest(read_only=read_only,keys=keys):
                args=normal_fixture();args[0]['result']['investigation']['telemetry']['targets'][0]['expected_fact_key']=None
                cap=args[3]['resolution']['investigation']['capabilities'][0];cap.update(read_only=read_only,supported_fact_keys=keys)
                args[2][0]['request']['request']['target']['expected_fact_key']=None
                self.assertEqual(capture.bind_normal(*args)['usage']['generations'],1)
                args[2][0]['request']['request']['target']['expected_fact_key']='value'
                with self.assertRaisesRegex(ValueError,'acquisition_wrong_fact_key'):capture.bind_normal(*args)
    def test_orphan_citation_cannot_disappear(self):
        g,o,r,s=pair();o['display_citations'].append({**o['display_citations'][0],'id':'orphan','artifact_id':'absent'});rebind(o,r)
        with self.assertRaisesRegex(ValueError,'orphan_citation_artifact'):scorer.validate_review(g,o,r,s,self.deps)
    def test_unadmitted_citation_cannot_earn_credit(self):
        g,o,r,s=pair();s['rejected']=s['s'];o['acquired_source_ids'].append('rejected');o['display_citations'][0]['source_id']='rejected';rebind(o,r)
        with self.assertRaisesRegex(ValueError,'misbound_correct_citation'):scorer.validate_review(g,o,r,s,self.deps)
    def test_helper_uses_supported_artifact_schema(self):
        g,o,r,s=pair();o['track']='helper_opt_in';o['terminal_status']='qualified';rebind(o,r)
        self.assertTrue(scorer.validate_review(g,o,r,s,self.deps)['complete_useful_answer'])
    def test_F1_and_missing_citation_gain_are_separate(self):
        g,o,r,s=pair();o['display_citations']=[];r['citation_edges']=[];rebind(o,r)
        row=scorer.validate_review(g,o,r,s,self.deps);self.assertEqual(row['decision'],'accept');self.assertFalse(row['complete_useful_answer'])
    def test_missing_actual_trace_generation_rejected(self):
        args=normal_fixture();args[1]['events']=[e for e in args[1]['events'] if e['kind'] not in ['model_request','model_response']]
        for n,e in enumerate(args[1]['events']):e['sequence']=n
        with self.assertRaisesRegex(ValueError,'renderer_without_actual_response'):capture.bind_normal(*args)
    def test_empty_supported_receipt_rejected(self):
        args=normal_fixture();args[0]['result']['final_outcome']['artifact']['verification_receipts'][0]['evidence_ids']=[]
        with self.assertRaisesRegex(ValueError,'empty_or_duplicate_receipt_evidence'):capture.bind_normal(*args)
    def test_nonstructured_receipt_rejected(self):
        args=normal_fixture();args[0]['result']['final_outcome']['artifact']['verification_receipts'][0]['verifier']='model-assertion'
        with self.assertRaisesRegex(ValueError,'unsupported_receipt_verifier'):capture.bind_normal(*args)
    def test_wrong_target_rejected(self):
        args=normal_fixture();args[2][0]['request']['request']['target']['target_id']='other'
        with self.assertRaisesRegex(ValueError,'acquisition_wrong_target'):capture.bind_normal(*args)
    def test_helper_baseline_incomplete_blocks_all_provider_tracks(self):
        g,o,r,s=pair();template=scorer.validate_review(g,o,r,s,self.deps);rows=[];planned=[]
        protocol=json.loads((Path(__file__).parent/'protocol.json').read_text())
        for provider in protocol['providers']:
            for track in protocol['tracks']:
                for arm in ['baseline','candidate']:
                    for n in range(24):
                        row=copy.deepcopy(template);rid=provider+'-'+track+'-'+arm+'-'+str(n)
                        row.update(id='transport-'+str(n),row_id=rid,provider=provider,track=track,arm=arm,expected='positive' if n<12 else 'negative',decision='accept' if n<12 else 'withhold')
                        row['complete_useful_answer']=n<12 and (arm=='candidate' or n<10)
                        if track=='helper_opt_in' and arm=='baseline':row['complete']=False;row['complete_useful_answer']=False
                        rows.append(row);planned.append({'id':row['id'],'row_id':rid,'provider':provider,'track':track,'arm':arm,'expected':row['expected'],'required_groups_G':1})
        report=scorer.summarize(rows,planned,protocol,self.deps)
        self.assertFalse(report['semantic_gate_pass'])
        for provider in protocol['providers']:self.assertIn(provider+'/helper_opt_in:baseline_incomplete',report['blockers'])
    def test_empty_M1_inventory_rejected(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);blob=b'{"files":{}}';(root/'manifest.json').write_bytes(blob)
            with self.assertRaisesRegex(ValueError,'M1_inventory_empty'):corpus.validate_manifest(root,{},scorer.sha(blob))
    def test_changed_gold_with_same_M1_rejected(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d)
            for name in ['corpus.json','gold.json','host-policy.json','preparation.json']:(root/name).write_text('{}')
            manifest=json.dumps({'files':{p.name:scorer.sha(p.read_bytes()) for p in root.iterdir()}}).encode();(root/'manifest.json').write_bytes(manifest)
            corpus.validate_manifest(root,{},scorer.sha(manifest));(root/'gold.json').write_text('{"changed":true}')
            with self.assertRaisesRegex(ValueError,'M1_file_changed'):corpus.validate_manifest(root,{},scorer.sha(manifest))
if __name__=='__main__':unittest.main()
