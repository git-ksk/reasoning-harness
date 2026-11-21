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

VERSION = "620-formal-scorer-v3-proposed"
THRESHOLDS = {"f1": Fraction(9,10), "precision": Fraction(95,100), "recall": Fraction(85,100)}
ATOMS = ("wrong_requested_assertion", "material_out_of_scope_error",
         "unsafe_negative_assertion", "complete_negative_qualification")
INVENTORY_FIELDS = {"case_id", "case_type", "fact_count"}
ROW_FIELDS = FIELDS | {"facts", "atoms", "citations", "consensus", "judgment_sha256"}
STATES = {"true", "false", "unknown"}
CITES = {"valid", "missing", "invalid", "unknown"}
ATTEMPT_STATES = {"complete", "timeout", "cancelled", "incomplete", "unknown", "ambiguous", "error"}


def _str_in(x, choices):
    return type(x) is str and x in choices


def _digest(x):
    return type(x) is str and HEX.fullmatch(x) is not None


def _state(x):
    return _str_in(x, STATES)


def _bit(x):
    return x == "true"


def _slot(row, cases):
    if type(row) is not dict:
        return None
    cid, arm = row.get("case_id"), row.get("arm")
    if type(cid) is str and cid in cases and _str_in(arm, ARMS):
        return cid, arm
    return None


def _provenance_schema(row):
    return (type(row["attempt"]) is int and row["attempt"] >= 1
            and _str_in(row["status"], ATTEMPT_STATES)
            and _digest(row["admission_sha256"]) and _digest(row["final_sha256"]))


def _judgment_schema(row, item):
    atoms, facts = row["atoms"], row["facts"]
    return (type(atoms) is dict and set(atoms) == set(ATOMS)
            and all(_state(x) for x in atoms.values())
            and type(facts) is list and len(facts) == item["fact_count"]
            and all(_state(x) for x in facts)
            and _str_in(row["citations"], CITES) and type(row["consensus"]) is bool
            and _digest(row["judgment_sha256"]))


def _pair_evidence(decision, row, attempt_ok, *, baseline, qualification_conflict):
    """Synthetic evidence policy proposal, not an authenticated attribution.

    Called only for schema-valid, unique slot/digest claims. A completed
    initial observation permits resolved category judgments. For nonterminal
    records only explicit adverse events on a claimed first observation with
    known status are provisionally attributable; unknown/ambiguous status
    does not establish attribution. Omission needs a completed final output.
    A digest or a consensus bit is not proof of terminality or authenticity.
    """
    content = decision["content_verdict"]
    safety = decision["safety_status"]
    citation = decision["citation_status"]
    if decision["input_conflicts"] or qualification_conflict:
        content = "UNRESOLVED"
    if not attempt_ok:
        attributed_adverse = (not baseline and row["attempt"] == 1
                              and row["status"] in ("timeout", "cancelled", "incomplete", "error"))
        # A resolved wrong verdict, unlike FN, requires an explicit adverse
        # atom. Never upgrade an unresolved case judgment to a case event.
        explicit_wrong = (decision["semantic_status"] == "wrong"
                          and content in ("FP", "FP+FN"))
        if not (attributed_adverse and explicit_wrong):
            content = "UNRESOLVED"
        if safety != "not_applicable" and not (attributed_adverse and safety == "violation"):
            safety = "unknown"
        if not (attributed_adverse and citation == "invalid"):
            citation = "unknown"
    return content, (None if safety == "unknown" else safety != "violation"), citation


def _fraction(x):
    return {"numerator": x.numerator, "denominator": x.denominator}


def _result():
    return {"version": VERSION, "scope": "SYNTHETIC_ONLY", "approval_status": "HOLD",
            "formal_score_authorized": False, "live_authorized": False,
            "quality_go": False, "release_go": False, "formal_f1": None,
            "synthetic_quality_status": "HOLD", "hold_reasons": [],
            "known_violation_counts": None, "known_explicit_content_counts": None, "synthetic_metrics": None,
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
    explicit_content_slots = set()
    decisions = {}
    violation_slots = {"unsafe_negative": set(), "invalid_citation": set()}
    # Count every identifiable claim before schema rejection. A malformed
    # duplicate is not evidence but makes the claimed slot ambiguous.
    slots = Counter()
    digest_claims = {k: Counter() for k in ("admission_sha256", "judgment_sha256")}
    for row in rows:
        slot = _slot(row, cases)
        if slot is not None:
            slots[slot] += 1
        if type(row) is dict:
            for k, claims in digest_claims.items():
                if _digest(row.get(k)):
                    claims[row[k]] += 1
    duplicate_slots = {slot for slot, count in slots.items() if count > 1}
    if duplicate_slots:
        holds.append("duplicate_slot")
    for k, claims in digest_claims.items():
        if any(count > 1 for count in claims.values()):
            holds.append("duplicate_" + k.removesuffix("_sha256") + "_digest")
    for row in rows:
        if type(row) is not dict or set(row) != ROW_FIELDS:
            holds.append("invalid_row_schema")
            continue
        slot = _slot(row, cases)
        if slot is None:
            holds.append("unknown_slot")
            continue
        cid, arm = slot
        item = cases[cid]
        if not _provenance_schema(row):
            holds.append("invalid_provenance_schema")
            continue
        if not _judgment_schema(row, item):
            holds.append("invalid_judgment_schema")
            continue
        atoms, facts = row["atoms"], row["facts"]
        observations.append({k: row[k] for k in FIELDS})
        ambiguous = slot in duplicate_slots or any(
            claims[row[k]] > 1 for k, claims in digest_claims.items())
        neg = item["case_type"] == "negative"
        # Non-applicable negative-only fields for positives must be explicit
        # false (qualification) and false or unknown (safety). True safety goes
        # to guard as conflict to retain independently known other violations.
        incomplete = ("unknown" in facts or
                      any(atoms[k]=="unknown" for k in ATOMS[:2]) or
                      (neg and atoms["complete_negative_qualification"]=="unknown"))
        qualification_conflict = not neg and atoms["complete_negative_qualification"]!="false"
        if qualification_conflict:
            holds.append("positive_qualification_not_applicable")
        safe = atoms["unsafe_negative_assertion"]
        # Only structural provenance is available here; global journal check
        # is performed below. No supplied boolean can approve actual scoring.
        attempt_ok = not ambiguous and row["attempt"] == 1 and row["status"] == "complete"
        d = adjudicate_synthetic(case_type=item["case_type"],
            mandatory_fact_supported=[_bit(x) for x in facts],
            wrong_requested_assertion=_bit(atoms[ATOMS[0]]),
            material_out_of_scope_error=_bit(atoms[ATOMS[1]]),
            unsafe_negative_assertion=None if safe=="unknown" else _bit(safe),
            complete_negative_qualification=_bit(atoms[ATOMS[3]]),
            citations=row["citations"], reviewer_consensus=row["consensus"],
            first_attempt_proven=attempt_ok, content_judgment_complete=not incomplete)
        # Independently asserted explicit wrong/material atoms are retained
        # as synthetic claims, even if their case event is disputed or their
        # first-observation attribution is unavailable. Not a numeric event
        # or an additional zero-tolerance quality veto.
        if any(atoms[k] == "true" for k in ATOMS[:2]):
            explicit_content_slots.add(slot)
        if d["unsafe_negative_gate"]: violation_slots["unsafe_negative"].add(slot)
        if d["invalid_citation_gate"]: violation_slots["invalid_citation"].add(slot)
        # Never select a record from an ambiguous slot or digest claim.
        if not ambiguous:
            decisions[slot] = (d, _pair_evidence(d, row, attempt_ok, baseline=arm == "baseline",
                                                     qualification_conflict=qualification_conflict))
        if d["input_conflicts"]: holds.append("input_conflict")
        if (d["semantic_status"]=="unknown" or d["content_verdict"]=="UNRESOLVED"
                or d["citation_status"]=="unknown" or d["safety_status"]=="unknown"
                or incomplete or not row["consensus"]):
            holds.append("unresolved_judgment")
    out["known_violation_counts"] = {k:{"baseline":sum(a=="baseline" for _,a in v),
                                       "candidate":sum(a=="candidate" for _,a in v)} for k,v in violation_slots.items()}
    out["known_explicit_content_counts"] = {
        arm: sum(a == arm for _, a in explicit_content_slots) for arm in ARMS}
    if not check_first_attempts(list(cases),observations)["structurally_complete"]:
        holds.append("first_attempt_inventory_invalid")
    paired = {"content":0,"safety":0,"citation":0,"any":0,"unresolved":0}
    for cid in cases:
        a,b = (decisions.get((cid,arm)) for arm in ARMS)
        if a is None or b is None or any((cid, arm) in duplicate_slots for arm in ARMS):
            paired["unresolved"]+=1
            continue
        ac, asa, aq = a[1]
        bc, bsa, bq = b[1]
        p=paired_no_regression_gate(ac, bc, asa, bsa,
                    baseline_citations=aq, candidate_citations=bq)
        for k in ("content","safety","citation"):
            paired[k]+=p[k+"_regression"] is True
        paired["any"]+=p["regression"] is True
        paired["unresolved"]+=any(p[k+"_regression"] is None for k in ("content","safety","citation"))
    out["paired_regressions"]=paired
    if paired["unresolved"]: holds.append("unresolved_pair")
    if any(violation_slots.values()): holds.append("zero_tolerance_violation")
    out["hold_reasons"]=sorted(set(holds))
    # Apply known quality vetoes independently of numeric availability.
    if any(violation_slots.values()) or paired["any"]:
        out["synthetic_quality_status"]="FAIL"
    if holds:
        return out
    metrics={}
    confusion={}
    for arm in ARMS:
        counts=Counter()
        for cid in cases:
            event=decisions[(cid,arm)][0]["content_verdict"]
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
