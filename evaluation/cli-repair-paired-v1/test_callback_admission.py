"""Pre-HTTP reservation/receipt controls; no model or accuracy cases."""
import copy
import json
from pathlib import Path
from types import SimpleNamespace
import tempfile
import unittest
from unittest.mock import patch
from preparation_guard import require_callback_admission, validate_reserved_callback
from development import checked_binaries, require_operational_review_identity, require_operational_verdict, OPERATIONAL_REVIEWER, require_bound_readiness


class CallbackAdmission(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.receipt = self.root / 'receipt.json'
        self.digest = 'a' * 64
        self.args = SimpleNamespace(provider='mistral', model='ministral-8b-2512', corpus=self.root / 'corpus.json',
                                    host_policy=self.root / 'host-policy.json', bin_dir=Path('/eval/bin'),
                                    case='engineering', arm='baseline', track='e2e_normal')
        self.argv = ['/python', '/callback', '--provider', self.args.provider]
        self.plan = {'purpose': 'CONSUMED_REAL24_ENGINEERING_DIAGNOSTIC_NOT_ACCEPTANCE', 'provider': self.args.provider,
                     'model': self.args.model, 'development_receipt_file': str(self.receipt),
                     'artifacts': {str(self.receipt): self.digest}, 'corpus_root': str(self.root),
                     'jobs': [{'id': 'engineering-baseline', 'argv': self.argv, 'contract': 'normal',
                               'budget': {'aggregate_input_tokens': 24000, 'aggregate_output_tokens': 4096}}]}
        self.state = {'provider': self.args.provider, 'model': self.args.model, 'rows': [],
                      'guard': {'inflight': True, 'latch': None}, 'current_reserve': 28096, 'charged_reservations': 28096}

    def test_live_requires_receipt_before_runtime_or_inputs(self):
        with patch('development.checked_receipt') as checked:
            with self.assertRaisesRegex(ValueError, 'callback_requires_frozen_receipt'):
                require_callback_admission(self.args, {})
            checked.assert_not_called()

    def test_wrong_receipt_digest_rejected_before_loading(self):
        self.receipt.write_text('{}')
        env = {'CLI_REPAIR_DEVELOPMENT_RECEIPT': str(self.receipt), 'CLI_REPAIR_DEVELOPMENT_RECEIPT_SHA256': '0' * 64}
        with patch('development.checked_receipt') as checked:
            with self.assertRaisesRegex(ValueError, 'callback_receipt_sha_mismatch'):
                require_callback_admission(self.args, env)
            checked.assert_not_called()

    def test_exact_current_durable_scope_passes_without_HTTP(self):
        validate_reserved_callback(self.args, self.plan, self.state, self.root / 'engineering-baseline', self.argv, self.receipt, self.digest)

    def test_changed_or_unreserved_scope_cannot_execute(self):
        mutations = [
            ('old_M0_purpose', lambda p, s: p.update(purpose='fresh_acceptance')),
            ('model', lambda p, s: p.update(model='other')),
            ('provider', lambda p, s: p.update(provider='google')),
            ('receipt', lambda p, s: p['artifacts'].update({str(self.receipt): 'b' * 64})),
            ('not_inflight', lambda p, s: s['guard'].update(inflight=False)),
            ('latched', lambda p, s: s['guard'].update(latch='quota')),
            ('wrong_reserve', lambda p, s: s.update(current_reserve=1)),
            ('not_charged', lambda p, s: s.update(charged_reservations=0)),
            ('finished', lambda p, s: s.update(rows=[{'complete': True}])),
            ('argv', lambda p, s: p['jobs'][0].update(argv=['other'])),
            ('job', lambda p, s: p['jobs'][0].update(id='other'))]
        for name, mutation in mutations:
            with self.subTest(name=name):
                plan, state = copy.deepcopy(self.plan), copy.deepcopy(self.state)
                mutation(plan, state)
                with self.assertRaises(ValueError):
                    validate_reserved_callback(self.args, plan, state, self.root / 'engineering-baseline', self.argv, self.receipt, self.digest)

    def test_both_complete_binary_sets_are_required(self):
        import hashlib
        names=['reason','adviser','source-host','audit-probe','registry']
        identity={'host_commit':'73bbd18173b4dfcd6664a26241de0385f2eb2e3e','arms':{}}
        for arm,pin in [('baseline','4abee90f0501f0d8fc09b453a0b45c942fbed69e'),('candidate','ab3c6322921c8c738ec35bd018345917b5afa538')]:
            binaries={}
            for name in names:
                value=(name+'-'+arm).encode();(self.root/(name+'-'+arm)).write_bytes(value)
                binaries[name]=hashlib.sha256(value).hexdigest()
            identity['arms'][arm]={'engine_commit':pin,'cli_package_version':'0.5.4','binaries':binaries}
        checked_binaries(identity,self.root)
        for count in [9,0]:
            changed=copy.deepcopy(identity)
            if count==9:changed['arms']['baseline']['binaries'].pop('reason')
            else:
                for arm in changed['arms'].values():arm['binaries']={}
            with self.subTest(count=count),self.assertRaisesRegex(ValueError,'development_binary_inventory_incomplete'):
                checked_binaries(changed,self.root)


    def test_operational_review_agent_identity_is_not_semantic_reviewer(self):
        protocol=json.loads((Path(__file__).parent/'protocol.json').read_bytes())
        receipt={'source_commit':'1'*40,'runtime_proof_sha256':'2'*64}
        review={'result':'PASS','reviewer':OPERATIONAL_REVIEWER,'reviewer_kind':'agent_nonhuman',
                'scope':'cli-repair-paired-development-runtime',
                'formal_verdict_issued':True,'verdict_stage':'FINAL_INDEPENDENT_OPERATIONAL_REVIEW',
                'synthetic_offline_integration_input_only':False,'not_formal_operational_verdict':False,**receipt}
        require_operational_review_identity(protocol,review,receipt)
        for reviewer in ['/root/paired_evaluation_independent','owner','']:
            with self.subTest(reviewer=reviewer):
                altered=copy.deepcopy(review);altered['reviewer']=reviewer
                with self.assertRaisesRegex(ValueError,'development_full_operational_review_missing'):
                    require_operational_review_identity(protocol,altered,receipt)

    def test_bounded_result_and_forged_policy_not_admissible(self):
        protocol=json.loads((Path(__file__).parent/'protocol.json').read_bytes())
        receipt={'source_commit':'1'*40,'runtime_proof_sha256':'2'*64}
        review={'result':'PASS','reviewer':OPERATIONAL_REVIEWER,'reviewer_kind':'agent_nonhuman',
                'scope':'cli-repair-paired-development-runtime',
                'formal_verdict_issued':True,'verdict_stage':'FINAL_INDEPENDENT_OPERATIONAL_REVIEW',
                'synthetic_offline_integration_input_only':False,'not_formal_operational_verdict':False,**receipt}
        for result in ['PASS_BOUNDED_SOURCE_AND_OFFLINE_RUNTIME_ONLY','PENDING','FAIL']:
            altered=copy.deepcopy(review);altered['result']=result
            with self.subTest(result=result),self.assertRaisesRegex(ValueError,'development_full_operational_review_missing'):
                require_operational_review_identity(protocol,altered,receipt)
        protocol['operational_review']['reviewer_id']='/root/paired_evaluation_independent'
        with self.assertRaisesRegex(ValueError,'development_operational_reviewer_policy_changed'):
            require_operational_review_identity(protocol,review,receipt)

    def test_operational_verdict_binds_exact_reviewed_image_source_and_proof(self):
        protocol=json.loads((Path(__file__).parent/'protocol.json').read_bytes())
        receipt={'source_commit':'1'*40,'runtime_proof_sha256':'2'*64,
                 'paired_host_identity_sha256':'3'*64,'runtime_image_id':'sha256:'+'4'*64,
                 'runtime_image_archive_sha256':'5'*64}
        review={'result':'PASS','reviewer':OPERATIONAL_REVIEWER,'reviewer_kind':'agent_nonhuman',
                'scope':'cli-repair-paired-development-runtime',
                'formal_verdict_issued':True,'verdict_stage':'FINAL_INDEPENDENT_OPERATIONAL_REVIEW',
                'synthetic_offline_integration_input_only':False,'not_formal_operational_verdict':False,
                'source_commit':receipt['source_commit'],'runtime_proof_sha256':receipt['runtime_proof_sha256']}
        verdict={**review,**receipt}
        require_operational_verdict(protocol,review,verdict,receipt)
        for name in ['source_commit','runtime_proof_sha256','paired_host_identity_sha256',
                     'runtime_image_id','runtime_image_archive_sha256']:
            changed=copy.deepcopy(verdict);changed[name]='bad'
            with self.subTest(field=name),self.assertRaisesRegex(ValueError,'review_not_for_runtime|operational_review_not_for_source_or_proof'):
                require_operational_verdict(protocol,review,changed,receipt)



    def test_provisional_actual_verdict_rejected_even_when_receipt_claims_final(self):
        protocol=json.loads((Path(__file__).parent/'protocol.json').read_bytes())
        receipt={'source_commit':'1'*40,'runtime_proof_sha256':'2'*64,
                 'paired_host_identity_sha256':'3'*64,'runtime_image_id':'sha256:'+'4'*64,
                 'runtime_image_archive_sha256':'5'*64}
        review={**receipt,'result':'PASS','reviewer':OPERATIONAL_REVIEWER,'reviewer_kind':'agent_nonhuman',
                'scope':'cli-repair-paired-development-runtime','formal_verdict_issued':True,
                'verdict_stage':'FINAL_INDEPENDENT_OPERATIONAL_REVIEW',
                'synthetic_offline_integration_input_only':False,'not_formal_operational_verdict':False}
        for changed_field,changed_value in [
                ('formal_verdict_issued',False),('verdict_stage','CANDIDATE'),
                ('synthetic_offline_integration_input_only',True),('not_formal_operational_verdict',True)]:
            verdict=copy.deepcopy(review);verdict[changed_field]=changed_value
            with self.subTest(changed_field=changed_field),self.assertRaisesRegex(ValueError,'development_rejects_provisional_or_synthetic_review'):
                require_operational_verdict(protocol,review,verdict,receipt)
        require_operational_verdict(protocol,review,review,receipt)

    def test_provisional_pass_cannot_admit_metadata_or_real_callbacks(self):
        from provider_readiness import verify
        import hashlib
        protocol=json.loads((Path(__file__).parent/'protocol.json').read_bytes())
        runtime_dir=self.root/'repo';runtime_dir.mkdir()
        (runtime_dir/'immutable.py').write_text('sealed-runtime')
        proof_path=self.root/'runtime-proof.json'
        inventory={'immutable.py':hashlib.sha256((runtime_dir/'immutable.py').read_bytes()).hexdigest()}
        proof={'source_commit':'1'*40,'paired_host_identity_sha256':'2'*64,
               'runtime_image_id':'sha256:'+'3'*64,'runtime_image_archive_sha256':'4'*64,
               'runtime_files':inventory}
        proof_path.write_text(json.dumps(proof))
        reviewed_digest=hashlib.sha256(json.dumps(inventory,sort_keys=True,separators=(',',':')).encode()).hexdigest()
        base={'result':'PASS','reviewer':OPERATIONAL_REVIEWER,'reviewer_kind':'agent_nonhuman',
              'scope':'cli-repair-paired-development-runtime','source_commit':proof['source_commit'],
              'runtime_proof_sha256':hashlib.sha256(proof_path.read_bytes()).hexdigest(),
              'paired_host_identity_sha256':proof['paired_host_identity_sha256'],
              'runtime_image_id':proof['runtime_image_id'],
              'runtime_image_archive_sha256':proof['runtime_image_archive_sha256'],
              'reviewed_runtime_files_sha256':reviewed_digest}
        import provider_readiness
        for update in [
                {'synthetic_offline_integration_input_only':True,'not_formal_operational_verdict':True},
                {'formal_verdict_issued':False,'verdict_stage':'FINAL_INDEPENDENT_OPERATIONAL_REVIEW',
                 'synthetic_offline_integration_input_only':False,'not_formal_operational_verdict':False},
                {'formal_verdict_issued':True,'verdict_stage':'DRAFT',
                 'synthetic_offline_integration_input_only':False,'not_formal_operational_verdict':False},
                {'formal_verdict_issued':True,'verdict_stage':'FINAL_INDEPENDENT_OPERATIONAL_REVIEW',
                 'synthetic_offline_integration_input_only':True,'not_formal_operational_verdict':False},
                {'formal_verdict_issued':True,'verdict_stage':'FINAL_INDEPENDENT_OPERATIONAL_REVIEW',
                 'synthetic_offline_integration_input_only':False,'not_formal_operational_verdict':True}]:
            provisional=base | update
            review_path=self.root/'review.json';review_path.write_text(json.dumps(provisional))
            with self.subTest(update=update),patch.object(provider_readiness,'REPO',runtime_dir):
                with self.assertRaisesRegex(ValueError,'development_rejects_provisional_or_synthetic_review'):
                    verify(review_path,proof_path)
                with self.assertRaisesRegex(ValueError,'development_rejects_provisional_or_synthetic_review'):
                    require_operational_review_identity(protocol,provisional,base)
        final=base|{'formal_verdict_issued':True,'verdict_stage':'FINAL_INDEPENDENT_OPERATIONAL_REVIEW',
                    'synthetic_offline_integration_input_only':False,'not_formal_operational_verdict':False}
        review_path.write_text(json.dumps(final))
        with patch.object(provider_readiness,'REPO',runtime_dir):
            verified_protocol, verified_proof=verify(review_path,proof_path)
        self.assertEqual(verified_proof,proof)
        self.assertEqual(verified_protocol['operational_review'],protocol['operational_review'])


    def _build_dual_metadata(self):
        import hashlib,time
        protocol=json.loads((Path(__file__).parent/'protocol.json').read_bytes())
        source='1'*40;proof='2'*64;review='3'*64
        folder=self.root/'provider-readiness';folder.mkdir(exist_ok=True)
        receipt={'source_commit':source,'runtime_proof_sha256':proof,'required_provider_readiness':{}}
        now=int(time.time())
        for provider in ['mistral','google']:
            record={'provider':provider,'model':protocol['providers'][provider]['model'],
                    'source_commit':source,'runtime_proof_sha256':proof,
                    'operational_review_sha256':review,
                    'checked_at_unix_seconds':now-10,'valid_until_unix_seconds':now-10+86400,
                    'model_generations':0,'maximum_metadata_requests':1,
                    'metadata_requests_attempted':1,'authentication_and_model_available':True,
                    'quota_headroom':'UNKNOWN','full_cohort_completion_guaranteed':False,
                    'http_status':200,'response_rate_limits':{},'error_class':None}
            path=folder/(provider+'.json');path.write_text(json.dumps(record))
            receipt['required_provider_readiness'][provider]={
                'new_live_run_allowed':True,'metadata_report_sha256':hashlib.sha256(path.read_bytes()).hexdigest(),
                'valid_until_unix_seconds':record['valid_until_unix_seconds']}
        return receipt,protocol,review,folder

    def test_two_valid_metadata_records_required_before_either_provider(self):
        receipt,protocol,review,folder=self._build_dual_metadata()
        require_bound_readiness(self.receipt,receipt,protocol,review)
        # A single provider's otherwise valid result cannot start a partial cohort.
        (folder/'google.json').unlink()
        with self.assertRaisesRegex(ValueError,'provider_metadata_evidence_inventory_changed'):
            require_bound_readiness(self.receipt,receipt,protocol,review)

    def test_metadata_receipt_hash_and_semantics_are_not_self_attested(self):
        import hashlib
        for kind in ['unbound_sha','wrong_model','wrong_source','wrong_review','unauthorized_http',
                     'unauthenticated','missing_get','extra_generation','expired','future_created',
                     'unbounded_quota_assertion','missing_provider_admission']:
            # Separate temp fixture state per mutation, no model or key required.
            with tempfile.TemporaryDirectory() as d:
                path=Path(d)/'receipt.json';folder=Path(d)/'provider-readiness';folder.mkdir()
                receipt,protocol,review,original=self._build_dual_metadata()
                for f in original.iterdir(): (folder/f.name).write_bytes(f.read_bytes())
                target=folder/'mistral.json';record=json.loads(target.read_text())
                if kind=='unbound_sha':receipt['required_provider_readiness']['mistral']['metadata_report_sha256']='0'*64
                elif kind=='wrong_model':record['model']='different-model'
                elif kind=='wrong_source':record['source_commit']='0'*40
                elif kind=='wrong_review':record['operational_review_sha256']='0'*64
                elif kind=='unauthorized_http':record['http_status']=401
                elif kind=='unauthenticated':record['authentication_and_model_available']=False
                elif kind=='missing_get':record['metadata_requests_attempted']=0
                elif kind=='extra_generation':record['model_generations']=1
                elif kind=='expired':record['valid_until_unix_seconds']=0
                elif kind=='future_created':record['checked_at_unix_seconds']+=200
                elif kind=='unbounded_quota_assertion':record['full_cohort_completion_guaranteed']=True
                elif kind=='missing_provider_admission':receipt['required_provider_readiness'].pop('google')
                if kind not in ['unbound_sha','missing_provider_admission']:
                    target.write_text(json.dumps(record))
                    receipt['required_provider_readiness']['mistral']['metadata_report_sha256']=hashlib.sha256(target.read_bytes()).hexdigest()
                with self.subTest(kind=kind),self.assertRaises(ValueError):
                    require_bound_readiness(path,receipt,protocol,review)


if __name__ == '__main__':
    unittest.main()
