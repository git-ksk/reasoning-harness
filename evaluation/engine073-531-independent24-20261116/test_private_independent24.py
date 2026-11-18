"""Opt-in private Gold tamper tests; skipped in GitHub CI without Mac Gold."""
import copy,json,os,stat,sys,tempfile,unittest
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parent))
import validate_independent24 as val

@unittest.skipUnless(os.environ.get('REASON_616_PRIVATE_GOLD'), 'no private gold in public CI')
class PrivateGoldFailures(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.repo=val.HERE.parents[1]
        cls.path=Path(os.environ['REASON_616_PRIVATE_GOLD'])
        cls.gold,cls.raw=val.read_json(cls.path)
        cls.public,_=val.read_json(val.HERE/val.PUBLIC)
        cls.packet=val.PACKET
    def test_private_full_24_pass(self):
        x=val.verify(self.repo,self.path,self.packet)
        self.assertEqual(x['required_facts'],54)
        self.assertEqual(x['private_gold_cases'],24)
    def test_quote_tamper_fails_closed(self):
        g=copy.deepcopy(self.gold)
        g['cases'][0]['required_facts'][0]['evidence']['exact_quote']+='tampered'
        with self.assertRaisesRegex(ValueError,'quote_not_exact_or_modified'):
            val.verify_gold_data(g,self.public,self.packet)
    def test_fake_negative_label_fails_closed(self):
        g=copy.deepcopy(self.gold);g['cases'][0]['classification']='negative'
        with self.assertRaisesRegex(ValueError,'negative_label_or_reason_missing'):
            val.verify_gold_data(g,self.public,self.packet)
    def test_false_reviewer_signature_fails_closed(self):
        g=copy.deepcopy(self.gold);g['reviewer_signature']='forged'
        with self.assertRaisesRegex(ValueError,'private_gold_false_authority'):
            val.verify_gold_data(g,self.public,self.packet)
    def test_private_wrong_source_fails_closed(self):
        g=copy.deepcopy(self.gold);g['cases'][0]['source_sha256']='0'*64
        with self.assertRaisesRegex(ValueError,'gold_source_mismatch'):
            val.verify_gold_data(g,self.public,self.packet)
    def test_world_readable_gold_rejected(self):
        with tempfile.TemporaryDirectory(prefix='rh616-',dir='/tmp') as d:
            path=Path(d)/'gold.json';path.write_bytes(self.raw);path.chmod(0o644)
            with self.assertRaisesRegex(ValueError,'gold_permissions_not_0600'):
                val.private_file(path,self.repo)
    def test_valid_acl_but_modified_file_fails_sha(self):
        with tempfile.TemporaryDirectory(prefix='rh616-',dir='/tmp') as d:
            root=Path(d);root.chmod(0o700)
            path=root/'gold.json';path.write_bytes(self.raw+b' ');path.chmod(0o600)
            with self.assertRaisesRegex(ValueError,'private_gold_sha_or_canonical_mismatch'):
                val.verify(self.repo,path,self.packet)

if __name__=='__main__':unittest.main()
