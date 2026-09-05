#!/usr/bin/env python3
"""Deterministic identity-sufficiency qualification for evidence search.

The acquisition adapter may return a plausible entity and a fact, but the Harness
must decide whether the entity identity is sufficiently supported before the fact
is admissible. This module is deliberately model-free and network-free.
"""

from __future__ import annotations

from copy import deepcopy
from typing import Any, Mapping


IDENTITY_SUPPORTED = "rank1_cross_source_agreement"
IDENTITY_INSUFFICIENT = "identity_insufficient"


def qualify_identity(
    facts: Mapping[str, str], search_state: Mapping[str, Any]
) -> tuple[dict[str, str], dict[str, Any]]:
    """Fail closed when a resolved fact lacks strong entity-identity evidence.

    Candidate-set membership is treated as plausibility evidence only. For this
    first research candidate, admissible identity requires all of:

    * upstream state is ``fact_resolved``;
    * the Wikipedia top page is not a disambiguation page;
    * the resolved entity equals the Wikipedia top page's Wikibase item;
    * cross-source corroboration rank is exactly 1.

    Facts are suppressed when identity is insufficient, but bounded candidate
    provenance is preserved for an evidence-search planner to inspect.
    """

    qualified_facts = dict(facts)
    qualified_state = deepcopy(dict(search_state))

    if qualified_state.get("outcome_kind") != "fact_resolved":
        return qualified_facts, qualified_state

    reasons: list[str] = []
    resolved_entity = qualified_state.get("resolved_entity")
    corroboration_rank = qualified_state.get("corroboration_rank")
    wikipedia_candidates = qualified_state.get("wikipedia_candidates")

    wikipedia_top = None
    if isinstance(wikipedia_candidates, list) and wikipedia_candidates:
        candidate = wikipedia_candidates[0]
        if isinstance(candidate, dict):
            wikipedia_top = candidate

    if not isinstance(resolved_entity, str) or not resolved_entity:
        reasons.append("missing_resolved_entity")

    if wikipedia_top is None:
        reasons.append("missing_wikipedia_top_candidate")
    else:
        if bool(wikipedia_top.get("disambiguation")):
            reasons.append("wikipedia_top_is_disambiguation")
        wikipedia_top_entity = wikipedia_top.get("wikibase_item")
        if not isinstance(wikipedia_top_entity, str) or not wikipedia_top_entity:
            reasons.append("missing_wikipedia_top_entity")
        elif resolved_entity != wikipedia_top_entity:
            reasons.append("resolved_entity_differs_from_wikipedia_top")

    if corroboration_rank != 1:
        reasons.append("cross_source_agreement_not_rank1")

    if reasons:
        qualified_facts.clear()
        qualified_state["upstream_outcome_kind"] = "fact_resolved"
        qualified_state["outcome_kind"] = IDENTITY_INSUFFICIENT
        qualified_state["identity_supported"] = False
        qualified_state["identity_reasons"] = reasons
        qualified_state["suggested_action"] = "search_with_existing_context_or_stop"
        return qualified_facts, qualified_state

    qualified_state["identity_supported"] = True
    qualified_state["identity_reason"] = IDENTITY_SUPPORTED
    return qualified_facts, qualified_state
