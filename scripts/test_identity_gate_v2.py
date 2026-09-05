#!/usr/bin/env python3
from __future__ import annotations

import copy
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from identity_gate_v2 import qualify_identity


def resolved_state(*, rank=1, resolved="Q100", wp_entity="Q100", disambiguation=False):
    return {
        "outcome_kind": "fact_resolved",
        "resolved_entity": resolved,
        "corroboration_mode": "original_query",
        "corroboration_rank": rank,
        "wikidata_candidate_ids": ["Q100", "Q200", "Q300"],
        "wikipedia_candidates": [
            {
                "title": "Example",
                "wikibase_item": wp_entity,
                "disambiguation": disambiguation,
            },
            {
                "title": "Example Other",
                "wikibase_item": "Q200",
                "disambiguation": False,
            },
        ],
        "property_id": "P17",
        "property_values": ["Q999"],
    }


class IdentityGateV2Tests(unittest.TestCase):
    def test_rank1_cross_source_agreement_keeps_fact(self):
        facts, state = qualify_identity({"example.country": "Q999"}, resolved_state())
        self.assertEqual(facts, {"example.country": "Q999"})
        self.assertTrue(state["identity_supported"])
        self.assertEqual(state["identity_reason"], "rank1_cross_source_agreement")
        self.assertEqual(state["outcome_kind"], "fact_resolved")

    def test_candidate_membership_at_rank2_is_not_identity_proof(self):
        facts, state = qualify_identity(
            {"example.country": "Q999"}, resolved_state(rank=2)
        )
        self.assertEqual(facts, {})
        self.assertEqual(state["outcome_kind"], "identity_insufficient")
        self.assertFalse(state["identity_supported"])
        self.assertIn("cross_source_agreement_not_rank1", state["identity_reasons"])
        self.assertEqual(state["upstream_outcome_kind"], "fact_resolved")

    def test_missing_corroboration_rank_fails_closed(self):
        state = resolved_state()
        state.pop("corroboration_rank")
        facts, qualified = qualify_identity({"example.country": "Q999"}, state)
        self.assertEqual(facts, {})
        self.assertEqual(qualified["outcome_kind"], "identity_insufficient")
        self.assertIn("cross_source_agreement_not_rank1", qualified["identity_reasons"])

    def test_resolved_entity_must_match_wikipedia_top(self):
        facts, state = qualify_identity(
            {"example.country": "Q999"},
            resolved_state(resolved="Q200", wp_entity="Q100", rank=1),
        )
        self.assertEqual(facts, {})
        self.assertIn(
            "resolved_entity_differs_from_wikipedia_top", state["identity_reasons"]
        )

    def test_disambiguation_page_cannot_support_identity(self):
        facts, state = qualify_identity(
            {"example.country": "Q999"}, resolved_state(disambiguation=True)
        )
        self.assertEqual(facts, {})
        self.assertIn("wikipedia_top_is_disambiguation", state["identity_reasons"])

    def test_missing_wikipedia_candidate_fails_closed(self):
        state = resolved_state()
        state["wikipedia_candidates"] = []
        facts, qualified = qualify_identity({"example.country": "Q999"}, state)
        self.assertEqual(facts, {})
        self.assertIn("missing_wikipedia_top_candidate", qualified["identity_reasons"])

    def test_non_resolved_state_passes_through_without_inventing_identity(self):
        source = {
            "outcome_kind": "ambiguous",
            "wikidata_candidate_ids": ["Q100", "Q200"],
            "wikipedia_candidates": [],
        }
        facts, state = qualify_identity({}, source)
        self.assertEqual(facts, {})
        self.assertEqual(state, source)

    def test_inputs_are_not_mutated(self):
        source_facts = {"example.country": "Q999"}
        source_state = resolved_state(rank=2)
        facts_before = copy.deepcopy(source_facts)
        state_before = copy.deepcopy(source_state)
        qualify_identity(source_facts, source_state)
        self.assertEqual(source_facts, facts_before)
        self.assertEqual(source_state, state_before)


if __name__ == "__main__":
    unittest.main()
