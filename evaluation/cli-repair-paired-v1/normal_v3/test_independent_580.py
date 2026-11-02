"""Independent #580 counterexamples; fake/file backends, never provider/GCS."""
import json
import os
import signal
import time
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

import authority
from contracts import canonical, digest
import test_authority_580


class IndependentAuthorityReview(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        test_authority_580.AuthorityTests.setUpClass()

    @classmethod
    def tearDownClass(cls):
        test_authority_580.AuthorityTests.tearDownClass()

    def setUp(self):
        self.fixture = test_authority_580.AuthorityTests()

    def test_reissued_evaluation_id_same_plan_is_rejected(self):
        store, receipt = self.fixture.fixture("first-evaluation")
        second = dict(receipt, evaluation_id="reissued-evaluation")
        store.objects[f"{authority.PREFIX}/receipts/reissued-evaluation.json"] = canonical(second)
        service = authority.TrustedAuthority(store)
        service.start("first-evaluation", self.fixture.plan,
                      self.fixture.local_receipt, self.fixture.binding)
        with self.assertRaises((FileExistsError, authority.AuthorityFailure)):
            service.start("reissued-evaluation", self.fixture.plan,
                          self.fixture.local_receipt, self.fixture.binding)

    def test_reissued_alternate_local_ledger_and_budget_still_conflicts(self):
        # Plan hashes and first-request bindings can be re-minted by a bad
        # issuer after changing an old receipt's local ledger or budget.
        import copy
        store, first = self.fixture.fixture("original")
        authority.TrustedAuthority(store).start(
            "original", self.fixture.plan, self.fixture.local_receipt,
            self.fixture.binding)
        alt_plan = copy.deepcopy(self.fixture.plan)
        alt_local = self.fixture.local_receipt + b"\n"
        alt_plan["receipt_sha256"] = digest(alt_local)
        alt_plan["token_budget"] += 1000
        # A malicious second issuer can forge new frozen-request bindings;
        # the stable cohort must not treat that as a fresh budget entitlement.
        for jid in alt_plan["frozen_request_bindings"]:
            alt_plan["frozen_request_bindings"][jid] = "e" * 64
        alt_store, other_receipt = self.fixture.fixture(
            "ledger-relabel", plan=alt_plan, local_receipt=alt_local)
        for key, raw in alt_store.objects.items():
            store.objects[key] = raw
        with self.assertRaises(FileExistsError):
            authority.TrustedAuthority(store).start(
                "ledger-relabel", alt_plan, alt_local, self.fixture.binding)

    def test_reordered_scoring_input_rejected_even_with_all_48_hashes(self):
        store, _ = self.fixture.fixture("reordered-score")
        self.fixture.all_48(self.fixture.start(store, "reordered-score"))
        hashes = {j["id"]: digest(canonical({"id": j["id"]}))
                  for j in reversed(self.fixture.plan["jobs"])}
        with self.assertRaisesRegex(ValueError, "observation_order"):
            authority.TrustedAuthority(store).claim_score(
                "reordered-score", self.fixture.plan, self.fixture.local_receipt,
                self.fixture.binding, observation_index_sha256="1" * 64,
                review_manifest_sha256="2" * 64, observation_hashes=hashes)
        self.assertIsNone(store.read(authority._key("reordered-score", "first-score")))

    def test_noncanonical_run_and_completion_claims_are_rejected(self):
        for which in ("run", "completed"):
            with self.subTest(which=which):
                store, _ = self.fixture.fixture()
                self.fixture.all_48(self.fixture.start(store))
                key = authority._key("synthetic-run-580", which)
                store.objects[key] += b"\n"
                with self.assertRaisesRegex(ValueError, "noncanonical"):
                    authority.TrustedAuthority(store).claim_score(
                        "synthetic-run-580", self.fixture.plan,
                        self.fixture.local_receipt, self.fixture.binding,
                        observation_index_sha256="1" * 64,
                        review_manifest_sha256="2" * 64,
                        observation_hashes={j["id"]: digest(canonical({"id": j["id"]}))
                                            for j in self.fixture.plan["jobs"]})

    def test_cohort_claim_ambiguous_ack_burns_reissue_before_run(self):
        store, original = self.fixture.fixture("ambiguous-cohort")
        store.fail_after = "/cohorts/"
        with self.assertRaises(OSError):
            authority.TrustedAuthority(store).start(
                "ambiguous-cohort", self.fixture.plan,
                self.fixture.local_receipt, self.fixture.binding)
        self.assertFalse(any(k.endswith("/run.json") for k in store.objects))
        second = dict(original, evaluation_id="reissued-after-timeout")
        store.objects[f"{authority.PREFIX}/receipts/reissued-after-timeout.json"] = canonical(second)
        with self.assertRaises(FileExistsError):
            authority.TrustedAuthority(store).start(
                "reissued-after-timeout", self.fixture.plan,
                self.fixture.local_receipt, self.fixture.binding)

    def test_real_sigkill_leaves_detached_grandchild_without_supervisor(self):
        # Real OS safety counterexample, NOT scheduler descendant containment.
        # No network and explicit cleanup of the detached child in finally.
        with tempfile.TemporaryDirectory() as tmp:
            marker = Path(tmp) / "detached.pid"
            code = r'''
import os, subprocess, sys, time
p=subprocess.Popen([sys.executable,'-B','-c','import time;time.sleep(40)'],
                   start_new_session=True, stdin=subprocess.DEVNULL,
                   stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
with open(sys.argv[1],'w') as f:
 f.write(str(p.pid)); f.flush(); os.fsync(f.fileno())
time.sleep(40)
'''
            leader = subprocess.Popen([sys.executable, "-B", "-c", code, str(marker)],
                                      start_new_session=True,
                                      stdout=subprocess.DEVNULL,
                                      stderr=subprocess.DEVNULL)
            child_pid = None
            try:
                for _ in range(120):
                    if marker.exists() and marker.read_text().strip():
                        child_pid = int(marker.read_text().strip())
                        break
                    time.sleep(.025)
                self.assertIsNotNone(child_pid, "detached child did not start")
                os.kill(leader.pid, signal.SIGKILL)
                leader.wait(timeout=5)
                # A killed parent cannot revoke an independently created
                # process session or potential billable activity.
                os.kill(child_pid, 0)
            finally:
                if child_pid is not None:
                    try:
                        os.killpg(child_pid, signal.SIGKILL)
                    except ProcessLookupError:
                        pass
                if leader.poll() is None:
                    leader.kill()
                leader.wait(timeout=5)

    def test_file_backed_two_real_process_claim_race(self):
        """Two isolated OS processes, shared O_EXCL storage; NOT actual GCS/hosts."""
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            evaluation_id = "multiprocess-claim"
            store, receipt = self.fixture.fixture(evaluation_id)
            (root / "receipt.json").write_bytes(canonical(receipt))
            (root / "plan.json").write_bytes(canonical(self.fixture.plan))
            (root / "local-receipt.bin").write_bytes(self.fixture.local_receipt)
            # A flat path-mapped store keeps separate processes independent.
            code = r'''
import json, os, sys
from pathlib import Path
import authority
root=Path(sys.argv[1]); eid=sys.argv[2]
class FileStore:
 def read(self,key):
  path=root/key
  try: return path.read_bytes()
  except FileNotFoundError: return None
 def create(self,key,data):
  path=root/key; path.parent.mkdir(parents=True,exist_ok=True)
  fd=os.open(path,os.O_CREAT|os.O_EXCL|os.O_WRONLY,0o600)
  with os.fdopen(fd,'wb') as fp: fp.write(data); fp.flush(); os.fsync(fp.fileno())
  dirfd=os.open(path.parent,os.O_RDONLY)
  try: os.fsync(dirfd)
  finally: os.close(dirfd)
binding=authority.AuthorityBinding('a'*64,'b'*64,'c'*64,'d'*64,'trusted-executor','trusted-evaluator')
try:
 authority.TrustedAuthority(FileStore()).start(eid,json.loads((root/'plan.json').read_bytes()),(root/'local-receipt.bin').read_bytes(),binding)
 print('won')
except FileExistsError: print('lost')
'''
            rpath = root / authority.PREFIX / "receipts" / f"{evaluation_id}.json"
            rpath.parent.mkdir(parents=True)
            rpath.write_bytes(canonical(receipt))
            executable = str(Path(sys.executable))
            procs = [subprocess.Popen([executable, "-B", "-c", code, str(root), evaluation_id],
                                      stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                      cwd=str(Path(authority.__file__).parent)) for _ in range(2)]
            output = [p.communicate(timeout=20) for p in procs]
            self.assertEqual([p.returncode for p in procs], [0, 0], output)
            self.assertCountEqual([out.decode().strip() for out, _ in output], ["won", "lost"])
            self.assertTrue((root / authority._key(evaluation_id, "run")).exists())


    def test_file_backed_two_real_process_first_score_race(self):
        # Exercise the authoritative *scoring* read-verification and O_EXCL
        # with actual independent Python processes on one shared namespace.
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            eid = "multiprocess-score"
            store, _ = self.fixture.fixture(eid)
            session = self.fixture.start(store, eid)
            self.fixture.all_48(session)
            for key, data in store.objects.items():
                dst = root / key
                dst.parent.mkdir(parents=True, exist_ok=True)
                dst.write_bytes(data)
            (root / "plan.json").write_bytes(canonical(self.fixture.plan))
            (root / "local-receipt.bin").write_bytes(self.fixture.local_receipt)
            code = r'''
import json, os, sys
from pathlib import Path
import authority
from contracts import canonical,digest
root=Path(sys.argv[1]); eid=sys.argv[2]
class FileStore:
 def read(self,key):
  try: return (root/key).read_bytes()
  except FileNotFoundError: return None
 def create(self,key,data):
  path=root/key; path.parent.mkdir(parents=True,exist_ok=True)
  fd=os.open(path,os.O_CREAT|os.O_EXCL|os.O_WRONLY,0o600)
  with os.fdopen(fd,'wb') as fp: fp.write(data);fp.flush();os.fsync(fp.fileno())
  dirfd=os.open(path.parent,os.O_RDONLY)
  try: os.fsync(dirfd)
  finally: os.close(dirfd)
plan=json.loads((root/'plan.json').read_bytes())
binding=authority.AuthorityBinding('a'*64,'b'*64,'c'*64,'d'*64,'trusted-executor','trusted-evaluator')
try:
 authority.TrustedAuthority(FileStore()).claim_score(
  eid,plan,(root/'local-receipt.bin').read_bytes(),binding,
  observation_index_sha256='1'*64,review_manifest_sha256='2'*64,
  observation_hashes={j['id']:digest(canonical({'id':j['id']})) for j in plan['jobs']})
 print('won')
except FileExistsError: print('lost')
'''
            procs = [subprocess.Popen([sys.executable, "-B", "-c", code, str(root), eid],
                                      stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                      cwd=str(Path(authority.__file__).parent)) for _ in range(2)]
            output = [p.communicate(timeout=20) for p in procs]
            self.assertEqual([p.returncode for p in procs], [0, 0], output)
            self.assertCountEqual([out.decode().strip() for out, _ in output], ["won", "lost"])
            self.assertTrue((root / authority._key(eid, "first-score")).exists())


if __name__ == "__main__":
    unittest.main()
