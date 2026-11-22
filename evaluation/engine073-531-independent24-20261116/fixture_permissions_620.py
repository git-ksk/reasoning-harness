"""Offline permission transitions for the fixed synthetic fixture only.

No real authorization is produced even when all fixture signatures verify.
Real same-account billing, time, ratification and independence remain HOLD.
"""
from pathlib import Path
import importlib.util

from auth620 import encode, decode, sha, require, formal_entrypoint
from first_claim_620 import FIXED_CONDITIONS

_BILLING_PATH = Path(__file__).resolve().parents[1] / "dev-lite" / "issue531_real24_api_billing_gate.py"


def fixture_billing():
    """Exercise the existing pure checker on invented values, without network."""
    spec = importlib.util.spec_from_file_location("rh620_billing_checker", _BILLING_PATH)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    result = module.check_account(
        {"payg_enabled": False},
        {"currency": "USD", "api_budget": {"payg_enabled": False, "currency": "USD",
                                         "initial_budget": "2.00", "usage_percentage": "0"}},
        {"data": [{"id": FIXED_CONDITIONS["model"]}]})
    # Do not export the checker's 'fresh authenticated' field as an actual proof.
    require(result["billing_gate"] == "PASS"
            and result["max_model_generations_after_admission"] == 576
            and result["total_token_upper_bound"] == 1348608, "fixture_billing_mismatch")
    return encode({"purpose": "synthetic-billing-fixture", "real_account_verified": False,
                   "caps": [576, 1348608]})


class FixturePermissions:
    """One-use synthetic signed state transitions, separate from actual permissions."""
    def __init__(self, ledger, registry, inventory_receipt, conditions_receipt, now):
        self.ledger, self.registry, self.now = ledger, registry, now
        self.parents = [inventory_receipt, conditions_receipt]
        self.pre = {}
        self.run = self.score = None
        self.billing = None
        self._run_material = None

    def expected_prospective(self, kind):
        require(kind in ("normative_ratification", "implementation_acceptance"),
                "prospective_kind")
        return encode({"operation": kind, "decision": "SYNTHETIC-STEP",
                       "conditions": FIXED_CONDITIONS,
                       "policy_unratified": True, "formal_implementation_unset": True})

    def prospective(self, kind, raw, envelope):
        require(self.run is None and kind not in self.pre, "prospective_order")
        require(raw == self.expected_prospective(kind), "prospective_binding")
        role = {"normative_ratification":"normative_ratifier",
                "implementation_acceptance":"implementation_acceptor"}[kind]
        receipt = self.registry.verify(envelope, raw, artifact_type=kind,
                         artifact_id=kind, role=role, parents=self.parents, now=self.now)
        self.ledger.append_fixture_stage(kind, encode({"payload":decode(raw),"envelope":decode(envelope)}))
        self.pre[kind] = receipt
        return receipt

    def attach_fixture_billing(self):
        require(self.run is None and self.billing is None, "billing_order")
        raw = fixture_billing()
        self.ledger.append_fixture_stage("fixture-billing", raw)
        self.billing = sha(raw)
        return self.billing

    def expected_run(self):
        require(self.run is None and set(self.pre) == {"normative_ratification","implementation_acceptance"}
                and self.billing is not None, "run_prerequisites")
        parents = [*self.parents, self.pre["normative_ratification"],
                   self.pre["implementation_acceptance"], self.billing]
        return encode({"operation":"run_permission","decision":"SYNTHETIC-STEP",
                       "nonce":"fixture-run-once","live_authorized":False,
                       "billing_fixture":self.billing}), parents

    def consume_run(self, raw, envelope):
        expected, parents = self.expected_run()
        require(raw == expected, "run_permission_binding")
        receipt = self.registry.verify(envelope,raw,artifact_type="run_permission",
                     artifact_id="fixture-run-once",role="run_approver",parents=parents,now=self.now)
        self.ledger.append_fixture_stage("fixture-run-once",encode({"payload":decode(raw),"envelope":decode(envelope)}))
        self.run = receipt
        self._run_material = (raw, envelope, parents)
        return receipt

    def reserve_slot(self, shard, cid, arm):
        """Reverify the exact synthetic run step before each reserved slot.

        Still no subprocess/API dispatch or real account authorization.
        """
        try:
            require(self.run is not None and self._run_material is not None,
                    "fixture_run_not_consumed")
            raw, envelope, parents = self._run_material
            require(self.ledger.read_fixture_stage("fixture-run-once") ==
                    encode({"payload":decode(raw),"envelope":decode(envelope)})
                    and self.ledger.read_fixture_stage("fixture-billing") == fixture_billing(),
                    "fixture_run_or_billing_changed")
            receipt = self.registry.verify(envelope,raw,artifact_type="run_permission",
                artifact_id="fixture-run-once",role="run_approver",parents=parents,now=self.now)
            require(receipt == self.run, "fixture_run_receipt_changed")
            return self.ledger.reserve(shard, cid, arm)
        except BaseException:
            self.ledger.halt()
            raise

    def expected_score(self, graph):
        require(self.run is not None and self.score is None, "score_order")
        # Projection rechecks durable closure; no pre-opening score fixture.
        inventory, rows = graph.projection()
        parents = [*self.parents,self.run,graph._closure,graph._lock_receipt,graph._opening_receipt]
        return encode({"operation":"score_permission","decision":"SYNTHETIC-STEP",
                       "nonce":"fixture-score-once","formal_score_authorized":False,
                       "projection_sha256":sha(encode({"inventory":inventory,"rows":rows}))}),parents

    def consume_score(self, graph, raw, envelope):
        expected,parents=self.expected_score(graph)
        require(raw == expected,"score_permission_binding")
        receipt=self.registry.verify(envelope,raw,artifact_type="score_permission",
                        artifact_id="fixture-score-once",role="score_approver",parents=parents,now=self.now)
        self.ledger.append_fixture_stage("fixture-score-once",encode({"payload":decode(raw),"envelope":decode(envelope)}))
        self.score=receipt
        return receipt
