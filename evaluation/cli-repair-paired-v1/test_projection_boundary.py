"""Projection legality controls, with no accuracy questions or model calls."""
import copy
import unittest
from display_projection import source_transition, coverage_transition
from scorer import require


class ProjectionBoundary(unittest.TestCase):
    def setUp(self):
        self.hard = {'status': 'requires_verification', 'factual_claims': 1,
                     'covered_claims': 1, 'factual_claim_coverage': 1.0,
                     'uncovered_propositions': [{'key': 'value', 'value': '4'}]}
        self.context = {'citations': [], 'unresolved_inputs': [], 'input_reasons': {}, 'selected_claims': []}
        self.target = {'id': 'engineering', 'question': 'Engineering transport control', 'origin': 'model_proposed_untrusted'}
        self.view = {'ledger': {'plan_admitted': True, 'targets': [{'target': self.target, 'bindings': []}]},
                     'targets': [{'target': self.target, 'disposition': 'not_acquired', 'verified_proposition': None}],
                     'base_finalization': self.hard, 'plan_answer_complete': False,
                     'plan_evidence_complete': False, 'stop_reason': None}
        self.diagnostic = {**self.hard, 'text': 'Investigation coverage (admitted plan only; not proof that the whole question is answered):\n- target "engineering" / question "Engineering transport control" / selector null: "not_acquired"\nInvestigation stopped: not run in this turn. Unestablished targets have no asserted value.'}

    def test_source_preserves_withheld_result(self):
        source_transition(self.hard, self.context, copy.deepcopy(self.hard), require)

    def test_source_cannot_promote_withheld_result(self):
        changed = {**self.hard, 'status': 'qualified_partial_answer', 'text': 'value=4'}
        with self.assertRaisesRegex(ValueError, 'source_context_promoted_withheld_answer'):
            source_transition(self.hard, self.context, changed, require)

    def test_source_cannot_change_hard_coverage(self):
        for key, value in [('factual_claims', 2), ('covered_claims', 0), ('factual_claim_coverage', 0.0), ('uncovered_propositions', [])]:
            with self.subTest(key=key), self.assertRaisesRegex(ValueError, 'projection_changed_hard_coverage'):
                source_transition(self.hard, self.context, {**self.hard, key: value}, require)

    def test_coverage_allows_blocked_diagnostics_only(self):
        coverage_transition(self.view, self.diagnostic, require)

    def test_coverage_cannot_promote_withheld_result(self):
        with self.assertRaisesRegex(ValueError, 'coverage_invalid_display_transition'):
            coverage_transition(self.view, {**self.diagnostic, 'status': 'qualified_partial_answer'}, require)
        with self.assertRaisesRegex(ValueError, 'coverage_invalid_display_transition'):
            coverage_transition(self.view, {**self.diagnostic, 'text': 'value=4'}, require)

    def test_coverage_preserves_hard_counts_and_completeness(self):
        with self.assertRaisesRegex(ValueError, 'projection_changed_hard_coverage'):
            coverage_transition(self.view, {**self.diagnostic, 'covered_claims': 0}, require)
        with self.assertRaisesRegex(ValueError, 'coverage_answer_completeness'):
            coverage_transition({**self.view, 'plan_answer_complete': True}, self.diagnostic, require)


if __name__ == '__main__':
    unittest.main()
