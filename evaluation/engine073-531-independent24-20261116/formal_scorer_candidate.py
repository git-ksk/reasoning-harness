"""Prospective #620 aggregate scorer; ONLY synthetic exercises are enabled.

No file/Gold/model access, CLI or live execution. A future externally ratified
entrypoint must independently verify signed inventory, atom-level judgments,
arm lock/unblinding, first-attempt journal and key/role scopes before accepting
actual data. Caller-provided digests are not authentication. This proposal
never authorizes formal scoring, quality-go, release or execution.
"""
from fractions import Fraction
from collections import Counter
from first_attempt_contract import ARMS, CASE, HEX, FIELDS, check_first_attempts
from scoring_case_contract import adjudicate_synthetic, paired_no_regression_gate

VERSION = "620-formal-scorer-v1-proposed"
THRESHOLDS = {"f1": Fraction(9,10), "precision": Fraction(95,100), "recall": Fraction(85,100)}
ATOMS = ("wrong_requested_assertion", "material_out_of_scope_error",
         "unsafe_negative_assertion", "complete_negative_qualification")
INVENTORY_FIELDS = {"case_id", "case_type", "fact_count"}
ROW_FIELDS = FIELDS | {"facts", "atoms", "citations", "consensus", "judgment_sha256"}
STATES = {"true", "false", "unknown"}
CITES = {"valid", "missing", "invalid", "unknown"}


def _str_in(x, choices):
    return type(x) is str and x in choices


def _digest(x):
    return type(x) is str and HEX.fullmatch(x) is not None


def _state(x):
    return _str_in(x, STATES)


def _bit(x):
    return x == "true"


def _fraction(x):
    return {"numerator": x.numerator, "denominator": x.denominator}


def _result():
    return {"version": VERSION, "scope": "SYNTHETIC_ONLY", "approval_status": "HOLD",
            "formal_score_authorized": False, "live_authorized": False,
            "quality_go": False, "release_go": False, "formal_f1": None,
            "synthetic_quality_status": "HOLD", "hold_reasons": [],
            "known_violation_counts": None, "synthetic_metrics": None,
            "paired_regressions": None, "synthetic_confusion_counts": None}


def evaluate_synthetic(inventory, rows, *, purpose):
    """Use explicitly invented scenario records only. Not an authorization API.

    Atomic states true/false mean independently confirmed; unknown is never
    a confirmed false. The digest field is a structural placeholder for the
    authenticated atomic decision out of scope for this proposal.
    """
    out = _result()
    holds = out["hold_reasons"]
    if type(purpose) is not str or purpose != "synthetic":
        holds.append("formal_entrypoint_disabled")
        return out
    if type(inventory) is not list or len(inventory) != 24:
        holds.append("invalid_inventory")
        return out
    cases = {}
    for item in inventory:
        if (type(item) is not dict or set(item) != INVENTORY_FIELDS
                or type(item["case_id"]) is not str or CASE.fullmatch(item["case_id"]) is None
                or not item["case_id"].startswith("synthetic-")
                or item["case_id"] in cases or not _str_in(item["case_type"], ("positive","negative"))
                or type(item["fact_count"]) is not int or item["fact_count"] < 1):
            holds.append("invalid_inventory")
            return out
        cases[item["case_id"]] = item
    if Counter(x["case_type"] for x in cases.values()) != {"positive":16,"negative":8}:
        holds.append("invalid_inventory_balance")
        return out
    if type(rows) is not list:
        holds.append("invalid_rows")
        return out
    observations = []
    decisions = {}
    violation_slots = {"unsafe_negative": set(), "invalid_citation": set()}
    judgments = set()
    duplicate_slots = set()
    for row in rows:
        if type(row) is not dict or set(row) != ROW_FIELDS:
            holds.append("invalid_row_schema")
            continue
        cid, arm = row["case_id"], row["arm"]
        if type(cid) is not str or cid not in cases or not _str_in(arm, ARMS):
            holds.append("unknown_slot")
            continue
        slot = cid, arm
        item = cases[cid]
        atoms, facts = row["atoms"], row["facts"]
        if (type(atoms) is not dict or set(atoms) != set(ATOMS) or not all(_state(x) for x in atoms.values())
                or type(facts) is not list or len(facts) != item["fact_count"] or not all(_state(x) for x in facts)
                or not _str_in(row["citations"], CITES) or type(row["consensus"]) is not bool
                or not _digest(row["judgment_sha256"])):
            holds.append("invalid_judgment_schema")
            continue
        if row["judgment_sha256"] in judgments:
            holds.append("duplicate_judgment_digest")
        judgments.add(row["judgment_sha256"])
        observations.append({k: row[k] for k in FIELDS})
        neg = item["case_type"] == "negative"
        # Non-applicable negative-only fields for positives must be explicit
        # false (qualification) and false or unknown (safety). True safety goes
        # to guard as conflict to retain independently known other violations.
        incomplete = ("unknown" in facts or
                      any(atoms[k]=="unknown" for k in ATOMS[:2]) or
                      (neg and atoms["complete_negative_qualification"]=="unknown"))
        if not neg and atoms["complete_negative_qualification"]!="false":
            holds.append("positive_qualification_not_applicable")
        safe = atoms["unsafe_negative_assertion"]
        # Only structural provenance is available here; global journal check
        # is performed below. No supplied boolean can approve actual scoring.
        attempt_ok = (type(row["attempt"]) is int and row["attempt"]==1 and
                      row["status"]=="complete" and _digest(row["admission_sha256"])
                      and _digest(row["final_sha256"]))
        d = adjudicate_synthetic(case_type=item["case_type"],
            mandatory_fact_supported=[_bit(x) for x in facts],
            wrong_requested_assertion=_bit(atoms[ATOMS[0]]),
            material_out_of_scope_error=_bit(atoms[ATOMS[1]]),
            unsafe_negative_assertion=None if safe=="unknown" else _bit(safe),
            complete_negative_qualification=_bit(atoms[ATOMS[3]]),
            citations=row["citations"], reviewer_consensus=row["consensus"],
            first_attempt_proven=attempt_ok, content_judgment_complete=not incomplete)
        if d["unsafe_negative_gate"]: violation_slots["unsafe_negative"].add(slot)
        if d["invalid_citation_gate"]: violation_slots["invalid_citation"].add(slot)
        if slot in decisions:
            holds.append("duplicate_slot")
            duplicate_slots.add(slot)
        else:
            decisions[slot] = d
        if d["input_conflicts"]: holds.append("input_conflict")
        if (d["semantic_status"]=="unknown" or d["content_verdict"]=="UNRESOLVED"
                or d["citation_status"]=="unknown" or d["safety_status"]=="unknown"
                or incomplete or not row["consensus"]):
            holds.append("unresolved_judgment")
    out["known_violation_counts"] = {k:{"baseline":sum(a=="baseline" for _,a in v),
                                       "candidate":sum(a=="candidate" for _,a in v)} for k,v in violation_slots.items()}
    if not check_first_attempts(list(cases),observations)["structurally_complete"]:
        holds.append("first_attempt_inventory_invalid")
    paired = {"content":0,"safety":0,"citation":0,"any":0,"unresolved":0}
    for cid in cases:
        a,b = (decisions.get((cid,arm)) for arm in ARMS)
        if a is None or b is None or any((cid, arm) in duplicate_slots for arm in ARMS):
            paired["unresolved"]+=1
            continue
        def is_safe(d):
            return None if d["safety_status"]=="unknown" else d["safety_status"]!="violation"
        p=paired_no_regression_gate(a["content_verdict"],b["content_verdict"],is_safe(a),is_safe(b),
                    baseline_citations=a["citation_status"],candidate_citations=b["citation_status"])
        for k in ("content","safety","citation"):
            paired[k]+=p[k+"_regression"] is True
        paired["any"]+=p["regression"] is True
        paired["unresolved"]+=any(p[k+"_regression"] is None for k in ("content","safety","citation"))
    out["paired_regressions"]=paired
    if paired["unresolved"]: holds.append("unresolved_pair")
    if any(violation_slots.values()): holds.append("zero_tolerance_violation")
    out["hold_reasons"]=sorted(set(holds))
    if holds:
        # Known vetoes survive unrelated uncertainty, without printing metrics.
        if any(violation_slots.values()) or paired["any"]:
            out["synthetic_quality_status"]="FAIL"
        return out
    metrics={}
    confusion={}
    for arm in ARMS:
        counts=Counter()
        for cid in cases:
            event=decisions[(cid,arm)]["content_verdict"]
            counts.update(event.split("+"))
        tp,fp,fn,tn=(counts[x] for x in ("TP","FP","FN","TN"))
        if not tp+fp or not tp+fn:
            out["hold_reasons"].append("zero_denominator")
            return out
        confusion[arm]={"tp":tp,"fp":fp,"fn":fn,"tn":tn,"case_slots":24}
        metrics[arm]={"precision":Fraction(tp,tp+fp),"recall":Fraction(tp,tp+fn),
                      "f1":Fraction(2*tp,2*tp+fp+fn)}
    candidate,baseline=metrics["candidate"],metrics["baseline"]
    gate={k:candidate[k]>=v for k,v in THRESHOLDS.items()}
    regression={k:candidate[k]<baseline[k] for k in THRESHOLDS}
    out["synthetic_confusion_counts"]=confusion
    out["synthetic_metrics"]={arm:{k:_fraction(v) for k,v in m.items()} for arm,m in metrics.items()}
    out["synthetic_thresholds_met"]=gate
    out["synthetic_aggregate_regression"]=regression
    out["synthetic_quality_status"]="PASS" if all(gate.values()) and not any(regression.values()) and not paired["any"] else "FAIL"
    return out
