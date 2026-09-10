import importlib.util
import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location(
    'v30_acceptance', ROOT / 'scripts/natural_language_e2e_v30_acceptance.py'
)
M = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(M)


def report(
    role,
    provider,
    model,
    *,
    recall_hits=10,
    tool_hits=10,
    false_abs=0,
    stalls=2,
    triggers=1,
    eligible=1,
    mechanism_hits=None,
    terminal_ids=(),
    operational_failures=0,
    safety_field=None,
    precedence=0,
):
    commit = M.CONTROL if role == 'control' else M.CANDIDATE
    mechanism_hits = eligible if mechanism_hits is None else mechanism_hits
    terminal_ids = set(terminal_ids)
    cases = []
    for i in range(10):
        cid = f'i{i}'
        follow = i < 3
        target_recalled = i < recall_hits
        tool_selected = i < tool_hits
        trigger = follow and i < triggers
        continuation = follow and i < eligible
        mechanism = continuation and i < mechanism_hits
        case = {
            'id': cid,
            'kind': 'investigation',
            'expected': 'grounded',
            'target': {'key': f'k{i}', 'value': f'v{i}'},
            'coverage_contract_kind': 'no_result_followup_observational' if follow else None,
            'target_recalled': target_recalled,
            'tool_selection_success': tool_selected,
            'target_grounded': not (i < false_abs),
            'false_abstention': int(i < false_abs),
            'avoidable_followup_stall': int(follow and i < stalls),
            'trigger_exposed': trigger if follow else None,
            'continuation_eligible': continuation if follow else None,
            'mechanism_conformant': mechanism if continuation else None,
            'downstream_followup_useful': mechanism if continuation else None,
            'action_count': 1 if tool_selected else 0,
            'correctness_boundary_violations': 0,
        }
        if follow and trigger:
            case.update({
                'trigger_first_relevant_capability': f'cache-{i}',
                'trigger_first_relevant_status': 'no_result',
                'mechanism_followup_status': 'verification_progress' if mechanism else 'no_result',
            })
        if cid in terminal_ids:
            case['stop_reason'] = 'operational_terminal'
            case['generation_failure_observed'] = 1
            # Absence of future tool/trigger/finalization evidence is not a semantic negative.
            if not tool_selected:
                case['action_count'] = 0
            if follow and not trigger:
                case.pop('trigger_first_relevant_capability', None)
                case.pop('trigger_first_relevant_status', None)
        cases.append(case)
    for i in range(3):
        idx = 10 + i
        cases.append({
            'id': f's{i}',
            'kind': ['session_add', 'session_correct', 'session_resume_fork'][i],
            'target': {'key': f's{idx}', 'value': 'ok'},
            'target_grounded': not (idx < false_abs),
            'false_abstention': int(idx < false_abs),
            'correctness_boundary_violations': 0,
        })

    aggregate = {
        'total_cases': 13,
        'completed_cases': 13 - (0 if role == 'control' else operational_failures),
        'operational_failures': operational_failures,
        'investigation_cases': 10,
        'session_cases': 3,
        'target_recall': recall_hits / 10,
        'tool_selection_success_rate': tool_hits / 10,
        'false_abstentions': false_abs,
        'avoidable_followup_stalls': stalls,
        'trigger_exposed_cases': triggers,
        'continuation_eligible_cases': eligible,
        'continuation_ineligible_trigger_cases': max(0, triggers - eligible),
        'mechanism_conformance_rate': mechanism_hits / eligible if eligible else None,
        'harness_precedence_selections': precedence,
        'model_selected_action_calls': 4,
        'action_rejections': {},
        'provider_calls_observed': 10,
        'tool_calls': max(1, tool_hits),
        **{field: 0 for field in M.ZERO},
    }
    if safety_field:
        aggregate[safety_field] = 1
    return {
        'schema_version': M.REPORT_SCHEMA,
        'corpus_identity': M.CORPUS_ID,
        'evaluator_identity': M.EVALUATOR_ID,
        'scoring_identity': M.SCORING_ID,
        'coordinate_role': role,
        'product_coordinate': {'commit': commit},
        'provider': provider,
        'model': model,
        'seed': M.SEED,
        'max_tokens': M.MAX_TOKENS,
        'measurement_observability_passed': True,
        'operational_completeness_passed': operational_failures == 0,
        'report_gate_passed': operational_failures == 0 and not safety_field,
        'cases': cases,
        'aggregate': aggregate,
    }


class AcceptanceTests(unittest.TestCase):
    def test_clean_strict_utility_improvement_passes(self):
        control = report('control', 'mistral', 'ministral-8b-latest', stalls=2, triggers=1)
        candidate = report('candidate', 'mistral', 'ministral-8b-latest', stalls=1, triggers=2)
        row = M.compare_pair(control, candidate)
        self.assertEqual(row['classification'], M.PASS)
        self.assertTrue(row['passed'])

    def test_flat_non_ceiling_is_definite_fail_for_strict_requirement(self):
        control = report('control', 'mistral', 'ministral-8b-latest', stalls=2, triggers=1)
        candidate = report('candidate', 'mistral', 'ministral-8b-latest', stalls=2, triggers=1)
        row = M.compare_pair(control, candidate)
        self.assertEqual(row['classification'], M.FAIL)
        self.assertIn('strict utility improvement is impossible', row['reasons'][0])

    def test_tradeoff_with_definite_trigger_regression_fails(self):
        control = report('control', 'mistral', 'ministral-8b-latest', stalls=2, triggers=1)
        candidate = report('candidate', 'mistral', 'ministral-8b-latest', stalls=1, triggers=0)
        row = M.compare_pair(control, candidate)
        self.assertEqual(row['classification'], M.FAIL)

    def test_exact_control_ceiling_allows_exact_preservation(self):
        control = report('control', 'google', 'gemma-4-31b-it', stalls=0, triggers=3, eligible=3)
        candidate = report('candidate', 'google', 'gemma-4-31b-it', stalls=0, triggers=3, eligible=3)
        self.assertEqual(M.compare_pair(control, candidate)['classification'], M.PASS)
        worse = report('candidate', 'google', 'gemma-4-31b-it', stalls=0, triggers=2, eligible=2)
        self.assertEqual(M.compare_pair(control, worse)['classification'], M.FAIL)

    def test_incomplete_control_can_pass_only_against_adverse_bounds(self):
        control = report(
            'control', 'google', 'gemini-3.5-flash-lite',
            recall_hits=10, tool_hits=2, false_abs=2, stalls=2, triggers=0, eligible=0,
            terminal_ids={'i2', 'i7'}, operational_failures=2,
        )
        candidate = report(
            'candidate', 'google', 'gemini-3.5-flash-lite',
            recall_hits=10, tool_hits=10, false_abs=0, stalls=0, triggers=3, eligible=3,
        )
        row = M.compare_pair(control, candidate)
        self.assertEqual(row['classification'], M.PASS, row)
        self.assertEqual(row['control_bounds']['target_recall']['lower_bound'], 1.0)
        self.assertGreater(row['control_bounds']['tool_selection_success_rate']['upper_bound'], 0.2)

    def test_incomplete_control_yields_inconclusive_when_non_regression_not_proven(self):
        control = report(
            'control', 'google', 'gemini-3.5-flash-lite',
            false_abs=7, stalls=2, triggers=0, eligible=0,
            terminal_ids={'i1', 'i2'}, operational_failures=2,
        )
        candidate = report(
            'candidate', 'google', 'gemini-3.5-flash-lite',
            false_abs=7, stalls=0, triggers=3, eligible=3,
        )
        row = M.compare_pair(control, candidate)
        self.assertEqual(row['classification'], M.INCONCLUSIVE)
        self.assertFalse(row['passed'])

    def test_candidate_operational_failure_is_hard_fail(self):
        control = report('control', 'mistral', 'ministral-8b-latest', stalls=2, triggers=1)
        candidate = report(
            'candidate', 'mistral', 'ministral-8b-latest', stalls=1, triggers=2,
            operational_failures=1,
        )
        row = M.compare_pair(control, candidate)
        self.assertEqual(row['classification'], M.FAIL)
        self.assertIn('candidate: candidate row not operationally complete', row['reasons'])

    def test_candidate_mechanism_nonconformance_remains_hard_fail(self):
        control = report('control', 'mistral', 'ministral-8b-latest', stalls=2, triggers=1)
        candidate = report(
            'candidate', 'mistral', 'ministral-8b-latest',
            stalls=1, triggers=2, eligible=2, mechanism_hits=1,
        )
        row = M.compare_pair(control, candidate)
        self.assertEqual(row['classification'], M.FAIL)
        self.assertIn(
            'candidate: #249 candidate mechanism conformance must be 1.0 when legally executable',
            row['reasons'],
        )

    def test_observed_control_safety_violation_is_hard_fail(self):
        control = report(
            'control', 'mistral', 'ministral-8b-latest', stalls=2, triggers=1,
            safety_field='unsupported_structured_claims',
        )
        candidate = report('candidate', 'mistral', 'ministral-8b-latest', stalls=1, triggers=2)
        row = M.compare_pair(control, candidate)
        self.assertEqual(row['classification'], M.FAIL)
        self.assertIn('control: unsupported_structured_claims must be zero', row['reasons'])

    def test_diagnostic_counts_do_not_enter_v13_gate(self):
        control = report('control', 'mistral', 'ministral-8b-latest', stalls=2, triggers=1)
        candidate = report('candidate', 'mistral', 'ministral-8b-latest', stalls=1, triggers=1)
        candidate['aggregate'].update({
            'action_rejection_diagnostic_records': 99,
            'precedence_skip_reasons': {'same_key_sibling': 99},
            'action_rejection_count': 99,
        })
        row = M.compare_pair(control, candidate)
        self.assertEqual(row['classification'], M.PASS)

    def test_no_cross_model_averaging_inconclusive_row_blocks_release(self):
        rows = [{'classification': M.PASS}, {'classification': M.INCONCLUSIVE}, {'classification': M.PASS}]
        self.assertEqual(M._overall(rows), M.INCONCLUSIVE)

    def test_groq_candidate_generic_path_still_requires_clean_candidate(self):
        clean = report('candidate', 'groq', 'openai/gpt-oss-120b', stalls=1, triggers=1)
        self.assertEqual(M.groq_row(clean)['classification'], M.PASS)
        bad = report(
            'candidate', 'groq', 'openai/gpt-oss-120b', stalls=1, triggers=1,
            operational_failures=1,
        )
        self.assertEqual(M.groq_row(bad)['classification'], M.FAIL)


if __name__ == '__main__':
    unittest.main()
