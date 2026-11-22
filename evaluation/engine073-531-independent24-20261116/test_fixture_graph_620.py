"""Fixed invented 24-case/48-observation offline graph, never real inputs."""
import copy
import json
import tempfile
import unittest
from pathlib import Path

from auth620 import Registry, encode, decode, sha, formal_entrypoint
from first_claim_620 import FirstClaimLedger, inventory_slots, FIXED_CONDITIONS
from observation_620 import verify_observation
from blind_fixture_620 import BlindFixtureGraph, map_digest, citation_summary
from protected_plan_620 import ProtectedPlan, profile_digest, CAPTURE_FIELDS, FULL_COMMITS, QUALITY, PINS
from formal_scorer_candidate import ATOMS
from formal_preflight_620 import OfflineApprovalDAG, WitnessWindow, redacted_status, validate_billing_snapshot
from partial_adverse_620 import PartialAdverseReview, ADVERSE
from fixture_permissions_620 import fixture_billing
from test_auth620 import START, END
from fixture_permissions_620 import FixturePermissions
from formal_scorer_candidate import evaluate_synthetic
from local_store import initialize_local_store
from contracts import frozen_request_digest
from native_http import LIMITED_JOURNAL
from http_limits import FROZEN_HTTP_LIMITS
from native_admission import CATALOG, ADMISSION, DOMAIN, capture_display, verify_pre_advance
from review_auth import POLICY
from test_auth620 import fixture_registry, fixture_inventory, sign, h, NOW, EVAL
from test_formal_scorer_candidate import scenario


def fixed_binding(slot, authority):
    shard, cid, arm = slot
    model = FIXED_CONDITIONS["model"]
    question = "Synthetic question " + cid
    request = {"task": question}
    request_sha = sha(encode(request))
    job_id = cid + "-" + arm
    binding = {"contract":"reason-cli-normal-v3-native-http-binding-v1",
               "question":question,"provider":"mistral","model":model,
               "legacy_receipt_sha256":authority,"job":{
                   "job_id":job_id,"max_provider_attempts":12,"reserve_tokens":28096,
                   "http_limits":{**FROZEN_HTTP_LIMITS,"max_generations":12},
                   "frozen_request_binding_sha256":frozen_request_digest(
                       job_id=job_id,case_id=cid,arm=arm,question=question,request=request,
                       request_sha256=request_sha,provider="mistral",model=model,receipt_sha256=authority)}}
    return encode(binding)


def fixed_source():
    quote="Invented control fact."
    evidence={"id":"ev","source":"source","observation":quote,"facts":{"f":"present"},"metadata":{}}
    binding_ev={"evidence_sha256":sha(json.dumps(evidence,ensure_ascii=False,separators=(",",":")).encode()),
                "source_id":"source","source_version":"v1"}
    catalog={"contract":CATALOG,"sources":{"source":{"text":quote+" More invented context.",
             "source_version":"v1"}},"evidence_bindings":{"ev":binding_ev}}
    return quote,evidence,binding_ev,catalog


def fixed_capture(registry, keys, slot, attempt, authority, parents, legacy_policy):
    shard, cid, arm = slot
    raw_binding=fixed_binding(slot,authority)
    binding=decode(raw_binding)
    model=binding["model"]; question=binding["question"]
    job_id=binding["job"]["job_id"]; request={"task":question}
    request_sha=sha(encode(request))
    limits=binding["job"]["http_limits"]
    req_bytes=len(encode(request))
    output_tokens=32
    corr = "fixture-" + attempt[:16]
    context = {"correlation_id":corr,"logical_attempt":1,"request_sha256":request_sha}
    usage = {"input_tokens":2,"output_tokens":3,"total_tokens":5}
    trace = {"contract":"reason-natural-diagnostic-trace-v1","provider":"mistral","model":model,
             "completion_status":"completed","events":[
                {"kind":"model_request","sequence":0,"correlation_id":corr,"attempt":1,
                 "provider":"mistral","model":model,"request":request,"request_sha256":request_sha,
                 "request_utf8_json_bytes":req_bytes,"requested_output_tokens":output_tokens},
                {"kind":"model_response","sequence":1,"correlation_id":corr,"attempt":1,
                 "provider":"mistral","model":model,
                 "response":{"provider_attempts":1,"usage":usage}}]}
    rid="http-"+corr+"-1"
    events=[{"kind":"binding","max_attempts":12,"token_reserve":28096,"limits":limits},
            {"kind":"preflight","request_id":rid,"context":context,
             "request_utf8_json_bytes":req_bytes,"input_reserve_tokens":req_bytes+512,
             "requested_output_tokens":output_tokens,"used_input_tokens":0,"used_output_tokens":0},
            {"kind":"request","request_id":rid,"context":context,"provider":"mistral","model":model,
             "wire_request_sha256":h("synthetic-wire-"+attempt),"start_elapsed_ns":0},
            {"kind":"response","request_id":rid,"http_status":200,
             "response_sha256":h("synthetic-response-"+attempt),"usage":usage,"failure_class":None},
            {"kind":"completed","request_id":rid}]
    chain=[]; prev="0"*64
    for i,event in enumerate(events):
        line=encode({"contract":LIMITED_JOURNAL,"sequence":i,"previous_sha256":prev,
                     "binding_sha256":sha(raw_binding),"event":event})
        chain.append(line+b"\n"); prev=sha(line)
    journal=b"".join(chain)
    quote,evidence,binding_ev,catalog=fixed_source()
    sources=encode(catalog)
    prefix="Negative fixture." if int(cid.rsplit("-",1)[1])>=16 else "Positive fixture."
    text=prefix+" "+json.dumps(quote)
    final=text.encode()
    terminal_state={"status":"grounded_answer","text":text,"uncovered_propositions":[],
                    "factual_claim_coverage":1.0}
    trace["events"].append({"kind":"finalization_state","sequence":2,"result":terminal_state})
    raw_trace=json.dumps(trace,ensure_ascii=False,sort_keys=True,separators=(",",":")).encode()
    cli={"schema_version":"reason-cli-output-v1","command":"ask","result":{
         "output_contract":"reason-natural-output-v6","task":question,
         "configuration":{"provider":"mistral","model":model},
         "exposed_text":{"policy_id":"harness-canonical-exposed-text-v1","renderer_text_exposed":False},
         "usage":{"run":{"tracked_from_session_start":True,"model_calls":1,"provider_attempts":1,**usage}},
         "finalization":terminal_state,
         "final_outcome":{"artifact":{"evidence":[evidence]}},"answer_context":{"citations":[{
             "evidence_id":"ev",**binding_ev,"quote":quote,"fact_start_byte":0,
             "fact_end_byte":len(prefix.encode()),"quote_start_byte":len(prefix.encode())+1,
             "quote_end_byte":len(final),"quote_display_encoding":"json_string","quote_sha256":sha(quote.encode())}]}}}
    cli["result"]["answer_context"]["citations"].append(dict(cli["result"]["answer_context"]["citations"][0]))
    stdout=json.dumps(cli,ensure_ascii=False,separators=(",",":")).encode()+b"\n"
    stderr=b"synthetic diagnostic\n"
    captured=encode(capture_display(cli,catalog))
    bundle=h("frozen invented native bundle")
    terminal=encode({"attempt_id":attempt,"slot":list(slot),"status":"complete",
        "binding_sha256":sha(raw_binding),"trace_sha256":sha(raw_trace),"journal_sha256":sha(journal),
        "stdout_sha256":sha(stdout),"stderr_sha256":sha(stderr),"sources_sha256":sha(sources),
        "capture_sha256":sha(captured),"final_length":len(final),"final_sha256":sha(final),
        "calls":1,"tokens":5,"native_bundle_sha256":bundle,"authority_receipt_sha256":authority})
    observation=sign(registry,keys,terminal,kind="terminal",role="capture",
                      artifact_id="terminal-"+attempt,parents=parents)
    expected={"contract":ADMISSION,"evaluation_id":EVAL,"job_id":job_id,
        "executor_id":"alias-executor","evaluator_id":"alias-safety_reviewer",
        "authority_receipt_sha256":authority,"native_bundle_sha256":bundle,
        "http_binding_sha256":sha(raw_binding),"packet_sha256":sha(captured),
        "observation_sha256":sha(observation),"sources_sha256":sha(sources),
        "stdout_sha256":sha(stdout),"stderr_sha256":sha(stderr)}
    segments={name:[{"start_byte":0,"end_byte":len(raw),"label":"nonfact","source_ids":[]}]
              for name,raw in (("stdout",stdout),("stderr",stderr))}
    p={**expected,"policy_sha256":sha(legacy_policy),"decision":"safe","full_sources_reviewed":True,
       "segments":segments,"notes":"Invented offline fixture only."}
    att=encode({"contract":ADMISSION,"algorithm":"Ed25519","key_id":"key-safety_reviewer",
               "payload":p,"signature_hex":keys["safety_reviewer"].sign(DOMAIN+encode(p)).signature.hex()})
    advance=verify_pre_advance(policy_bytes=legacy_policy,attestation=att,expected=expected,
                               stdout=stdout,stderr=stderr,sources=catalog["sources"])
    safety_raw=encode({"attempt_id":attempt,"terminal_receipt":sha(observation),
                      "legacy_attestation":sha(att),"legacy_authentication":sha(advance.authentication_bytes)})
    safety=sign(registry,keys,safety_raw,kind="admission",role="safety_reviewer",
                artifact_id="admission-"+attempt,parents=[sha(observation)])
    return dict(registry=registry,now=NOW,evaluation_id=EVAL,attempt_id=attempt,slot=slot,
         binding_raw=raw_binding,expected_binding_raw=raw_binding,trace_raw=raw_trace,journal_raw=journal,
         stdout=stdout,stderr=stderr,sources_raw=sources,source_pin=sha(sources),
         final_bytes=final,bundle_pin=bundle,authority_pin=authority,
         observation_envelope=observation,safety_envelope=safety,legacy_policy_raw=legacy_policy,
         legacy_policy_pin=sha(legacy_policy),legacy_attestation=att,executor_id="alias-executor",
         evaluator_id="alias-safety_reviewer",expected_parents=parents)


def decision_raw(bundle, values, targets=None):
    targets=bundle["targets"] if targets is None else targets
    rows=[]
    for target in targets:
        value=values[target]
        kind=("unknown" if value=="unknown" else "citation_entailment" if target.startswith("citation-")
              else "qualification" if target=="complete_negative_qualification" else
              "explicit_adverse" if not target.startswith("fact-") and value=="true" else
              "final_output_dependent_omission" if target.startswith("fact-") and value=="false" else "explicit_favorable")
        basis={"target":target,"value":value,"evidence_kind":kind,
               "evidence_token":bundle["evidence_token"],"display":bundle["display"],
               "source_texts":bundle["source_texts"],"judge_context":bundle["judge_context"],
               "citation_details":bundle["citation_details"]}
        rows.append({"target":target,"value":value,"evidence_kind":kind,
                     "evidence_token":bundle["evidence_token"],"basis_digest":sha(encode(basis))})
    return encode({"blind_id":bundle["blind_id"],"decisions":rows})


def fixed_setup(self,profile):
    dat,keys=fixture_registry(); raw_registry=encode(dat)
    registry=Registry(raw_registry,pinned_sha256=sha(raw_registry),evaluation_id=EVAL)
    def signed(raw,kind,role,aid,parents,**kw):
        return sign(registry,keys,raw,kind=kind,role=role,artifact_id=aid,parents=parents,**kw)
    def verified(raw,kind,role,aid,parents):
        env=signed(raw,kind,role,aid,parents)
        return registry.verify(env,raw,artifact_type=kind,artifact_id=aid,role=role,parents=parents,now=NOW)
    inventory=fixture_inventory()
    slots=inventory_slots(inventory)
    salt=h("public synthetic map salt")
    mapping=sorted([{"blind_id":"blind-"+h(salt+str(i))[:32],
                     "shard":s,"case_id":c,"arm":a} for i,(s,c,a) in enumerate(slots)],
                   key=lambda x:x["blind_id"])
    map_pin=map_digest(mapping,salt)
    inv_env=signed(inventory,"inventory","author","inventory",[])
    inv=registry.verify(inv_env,inventory,artifact_type="inventory",artifact_id="inventory",role="author",parents=[],now=NOW)
    authority_pin=h("frozen synthetic legacy authority")
    planned={slot:fixed_binding(slot,authority_pin) for slot in slots}
    planned_sources=sha(encode(fixed_source()[3]))
    legacy=encode({"contract":POLICY,"policy_id":"synthetic-local-only",
            "principals":[{"subject_id":"subject-executor","aliases":["alias-executor"],
                "role":"agent","conflict_groups":["execution"]},
                {"subject_id":"subject-safety_reviewer","aliases":["alias-safety_reviewer"],
                "role":"agent","conflict_groups":["review"]} ],
            "keys":[{"key_id":"key-safety_reviewer","subject_id":"subject-safety_reviewer",
                "public_key_hex":keys["safety_reviewer"].verify_key.encode().hex(),"revoked":False}]})
    identity_raw=encode({"contract":"invented-identity-fixture","independence_proven":False})
    isolation_raw=encode({"contract":"invented-isolation-fixture","isolation_proven":False})
    contexts=[{"case_id":c["case_id"],"question":"Synthetic question "+c["case_id"],
               "facts":[{"id":"fact-0","requirement":"Invented first fact"},
                        {"id":"fact-1","requirement":"Invented second fact"}],
               "citations":[{"id":"fact-0","requirement":"Support first invented fact"},
                            {"id":"fact-1","requirement":"Support second invented fact"}]}
              for c in decode(inventory)]
    contexts_raw=encode(contexts)
    plan_payload={k:h("invented plan "+k) for k in PINS}
    plan_payload.update(identity_evidence_digest=sha(identity_raw),isolation_evidence_digest=sha(isolation_raw))
    plan_payload.update(contract="620-protected-plan/1",evaluation_id=EVAL,
        registry_digest=registry.digest,store_digest=profile_digest(profile),inventory_receipt=inv,
        conditions=FIXED_CONDITIONS,full_commits=FULL_COMMITS,quality=QUALITY,
        legacy_annotation_receipts=[h("invented annotation "+str(i)) for i in range(3)],
        runtime_digests={k:h("invented runtime "+k) for k in ("baseline","candidate","cli","python","dependencies","scorer")},
        blind_mapping_commitment=map_pin,legacy_policy_pin=sha(legacy),
        billing_account_pin=h("invented billing account"),approval_start=START,approval_end=END,
        executor_id="alias-executor",evaluator_id="alias-safety_reviewer",
        slots=[dict(shard=slot[0],case_id=slot[1],arm=slot[2],binding_sha256=sha(planned[slot]),
                    source_pin=planned_sources,bundle_pin=h("frozen invented native bundle"),authority_pin=authority_pin)
               for slot in slots],
        targets=[dict(case_id=c["case_id"],atoms=list(ATOMS),facts=["fact-0","fact-1"],
                      required_citations=["fact-0","fact-1"],judge_context_digest=sha(encode(ctx)))
                              for c,ctx in zip(decode(inventory),contexts,strict=True)])
    conditions_raw=encode(plan_payload)
    cond_env=signed(conditions_raw,"execution_conditions","coordinator","conditions",[inv])
    plan=ProtectedPlan(profile=profile,pin=sha(conditions_raw),raw=conditions_raw,registry=registry,
          inventory_raw=inventory,inventory_envelope=inv_env,conditions_envelope=cond_env,
          legacy_policy_raw=legacy,judge_contexts_raw=contexts_raw,now=NOW)
    with self.assertRaises(ValueError):
        ProtectedPlan(profile=profile,pin=h("foreign plan"),raw=conditions_raw,registry=registry,
                  inventory_raw=inventory,inventory_envelope=inv_env,conditions_envelope=cond_env,
                  legacy_policy_raw=legacy,judge_contexts_raw=contexts_raw,now=NOW)
    changed=decode(conditions_raw); changed["conditions"]["calls_per_cli"]=13
    changed_raw=encode(changed)
    with self.assertRaises(ValueError):
        ProtectedPlan(profile=profile,pin=sha(changed_raw),raw=changed_raw,registry=registry,
                  inventory_raw=inventory,inventory_envelope=inv_env,
                  conditions_envelope=signed(changed_raw,"execution_conditions","coordinator","conditions",[inv]),
                  legacy_policy_raw=legacy,judge_contexts_raw=contexts_raw,now=NOW)
    cond=plan.conditions_receipt
    ledger=FirstClaimLedger(profile,dataset_sha256=plan.dataset_digest,inventory_raw=inventory,
           inventory_receipt=inv,conditions_receipt=cond,evaluation_id=EVAL,blind_map_sha256=map_pin,
        lane="formal-preflight-fixture")
    plan.bind_ledger(ledger)
    checkpoint_raw=encode({"contract":"620-witness/1","evaluation_id":EVAL,
        "registry_digest":registry.digest,"sequence":1,"previous":None,
        "observed_at":START,"expires_at":END})
    witness_env=signed(checkpoint_raw,"checkpoint","journal_witness","witness-1",[])
    clock=[NOW]
    witness=WitnessWindow(ledger=ledger,registry=registry,expected_pin=sha(witness_env),
        expected_sequence=1,expected_previous=None,raw=checkpoint_raw,envelope=witness_env,clock=lambda:clock[0])
    identity_env=signed(identity_raw,"identity_attestation","identity_authority","offline-identity",[inv])
    isolation_env=signed(isolation_raw,"isolation_attestation","isolation_witness","offline-isolation",[sha(identity_env),inv])
    flow=OfflineApprovalDAG(plan=plan,ledger=ledger,registry=registry,witness=witness,
        identity_raw=identity_raw,identity_envelope=identity_env,isolation_raw=isolation_raw,isolation_envelope=isolation_env)
    def consume(kind,role,graph=None):
        p,parents=flow.expected(kind,graph)
        env=signed(p,kind,role,decode(p)["nonce"],parents)
        return flow.consume(kind,p,env,graph)
    with self.assertRaises(ValueError): flow.expected("run_permission")
    for kind,role in (("normative_ratification","normative_ratifier"),
                      ("implementation_acceptance","implementation_acceptor")):
        consume(kind,role)
    with self.assertRaises(ValueError): flow.expected("run_permission")
    bill=encode(dict(contract="620-billing-snapshot/1",account_pin=plan.inputs()["billing_account_pin"],
        plan_digest=plan.digest,checked_at=START,expires_at=END,
        subscription={"payg_enabled":False},budget={"currency":"USD","api_budget":{"payg_enabled":False,
             "currency":"USD","initial_budget":"2.00","usage_percentage":"0"}},
        models={"data":[{"id":FIXED_CONDITIONS["model"]}]}))
    with self.assertRaises(ValueError):
        validate_billing_snapshot(encode({**decode(bill),"account_pin":h("another account")}),plan,NOW)
    flow.attach_billing_snapshot(bill,signed(bill,"billing_attestation","billing_witness","offline-billing-1",[cond,witness.digest]))
    p,parents=flow.expected("run_permission")
    with self.assertRaises(ValueError):
        flow.consume("run_permission",p,signed(p,"run_permission","score_approver",decode(p)["nonce"],parents))
    run=consume("run_permission","run_approver")
    with self.assertRaises(ValueError): flow.consume("run_permission",p,b"{}")
    with self.assertRaises(ValueError): flow.expected("normative_ratification")
    return dat,keys,registry,signed,inventory,slots,salt,mapping,inv,cond,authority_pin,planned,planned_sources,legacy,plan,ledger,witness,clock,flow,consume,run


class FixtureGraphTests(unittest.TestCase):
    def test_fixed_synthetic_graph_and_denials(self):
        # No caller-provided fixture, paths, real IDs, evaluation questions or Gold.
        with tempfile.TemporaryDirectory() as tmp:
            profile=initialize_local_store(Path(tmp).resolve()/"synthetic-store")
            dat,keys,registry,signed,inventory,slots,salt,mapping,inv,cond,authority_pin,planned,planned_sources,legacy,plan,ledger,witness,clock,flow,consume,run=fixed_setup(self,profile)
            observations=[]; observation_inputs=[]
            for i,slot in enumerate(slots):
                attempt=flow.reserve_slot(slot)
                kw=fixed_capture(registry,keys,slot,attempt,authority_pin,[inv,cond,run],legacy)
                kw["expected_binding_raw"]=planned[slot]
                kw["source_pin"]=planned_sources
                if i==0:
                    for change in ({"final_bytes":b"wrong display"},{"attempt_id":h("foreign attempt")},
                                   {"journal_raw":kw["journal_raw"][:-1]},{"stderr":b"changed"},
                                   {"source_pin":h("wrong source")},{"observation_envelope":b"{}"},
                                   {"safety_envelope":b"{}"}):
                        with self.subTest(change=list(change)),self.assertRaises(ValueError):
                            verify_observation(**{**kw,**change})
                capture={k:kw[k] for k in CAPTURE_FIELDS}
                record=ledger.active_reservation()
                if i==0:
                    with self.assertRaises(ValueError):
                        plan.verify_capture(ledger=ledger,registry=registry,now=NOW,record=record,
                                  raw={**capture,"source_pin":planned_sources},run_receipt=run)
                    with self.assertRaises(ValueError):
                        plan.verify_capture(ledger=ledger,registry=registry,now=NOW,record=record,
                                  raw={**capture,"binding_raw":b"{}"},run_receipt=run)
                observation=flow.capture(capture)
                observations.append(observation); observation_inputs.append(capture)
            self.assertEqual(len({x["observation_sha256"] for x in observations}),48)
            self.assertEqual(len({x["final_sha256"] for x in observations}),2)
            graph=flow.build_graph(mapping,salt)
            with self.assertRaises(ValueError): graph.unblind(b"{}",b"{}")
            with self.assertRaises(ValueError): graph.lock_payload()
            with self.assertRaises(ValueError): flow.expected("score_permission",graph)
            first=graph.blind_ids()[0]; second=graph.blind_ids()[1]
            for bid in graph.blind_ids():
                bundle=graph.judge_bundle(bid)
                self.assertEqual(set(bundle),{"blind_id","display","source_texts","targets","evidence_token","judge_context","citation_details"})
                self.assertEqual(len([t for t in bundle["targets"] if t.startswith("citation-")]),4)
                negative=bundle["display"].startswith("Negative fixture.")
                values={t:("valid" if t.startswith("citation-") else
                          "true" if t.startswith("fact-") or (t=="complete_negative_qualification" and negative)
                          else "false") for t in bundle["targets"]}
                o=graph._by_id[bid]["observation"]
                parents=[o["observation_sha256"],o["admission_sha256"]]
                receipts=[]
                for role in ("judge_a","judge_b"):
                    choices=dict(values)
                    if bid==first and role=="judge_b": choices["fact-0"]="unknown"
                    p=decision_raw(bundle,choices)
                    env=signed(p,"atomic_judgment",role,bid+"-"+role,parents)
                    if bid==first and role=="judge_a":
                        foreign=graph.judge_bundle(second)
                        damaged=decode(p); damaged["decisions"][0]["evidence_token"]=foreign["evidence_token"]
                        with self.assertRaises(ValueError): graph.submit(bid,role,encode(damaged),env)
                        with self.assertRaises(ValueError): graph.submit(bid,role,decision_raw(bundle,choices,["fact-0"]),env)
                        bad_basis=decode(p); bad_basis["decisions"][0]["basis_digest"]=h("foreign basis")
                        bad_raw=encode(bad_basis)
                        with self.assertRaises(ValueError):
                            graph.submit(bid,role,bad_raw,signed(bad_raw,"atomic_judgment",role,bid+"-"+role,parents))
                        changed_choices={**choices,"fact-0":"false"}
                        with self.assertRaises(ValueError):
                            graph.submit(bid,role,decision_raw(bundle,changed_choices),env)
                    receipts.append(graph.submit(bid,role,p,env))
                if bid==first:
                    with self.assertRaises(ValueError): graph.lock_payload()
                    d=decision_raw(bundle,values,["fact-0"])
                    a=signed(d,"adjudication","adjudicator",bid+"-adjudication",receipts)
                    rp=encode({"blind_id":bid,"adjudication_receipt":sha(a),"decision":"synthetic-confirmed"})
                    rat=signed(rp,"adjudication_ratification","adjudication_ratifier",
                              bid+"-ratification",[*receipts,sha(a)])
                    with self.assertRaises(ValueError):
                        graph.adjudicate(bid,d,a,rp,signed(rp,"adjudication_ratification","adjudicator",
                            bid+"-ratification",[*receipts,sha(a)]))
                    graph.adjudicate(bid,d,a,rp,rat)
            p,parents=graph.lock_payload()
            lock=graph.lock(p,signed(p,"blind_lock","blind_lock_admin","blind-lock",parents))
            with self.assertRaises(ValueError): graph.submit(first,"judge_a",b"{}",b"{}")
            with self.assertRaises(ValueError): graph.lock_payload()
            with self.assertRaises(ValueError): flow.expected("score_permission",graph)
            opening={"mapping":mapping,"salt":salt,"lock_receipt":lock}
            for change in ({"salt":h("wrong")},{"mapping":mapping[:-1]}):
                p=encode({**opening,**change})
                with self.assertRaises(ValueError):
                    graph.unblind(p,signed(p,"unblind","unblinder","full-unblind",[lock]))
            p=encode(opening)
            graph.unblind(p,signed(p,"unblind","unblinder","full-unblind",[lock]))
            with self.assertRaises(ValueError): graph.unblind(p,b"{}")
            score_receipt=consume("score_permission","score_approver",graph)
            with self.assertRaises(ValueError): flow.consume("score_permission",b"{}",b"{}",graph)
            report=flow.inspect_score(graph)
            self.assertTrue(report["offline_graph_verified"])
            self.assertEqual(report["reason"],"external_gate_hold")
            self.assertFalse(report["formal_score_authorized"])
            self.assertIsNone(report["formal_f1"])
            self.assertFalse(flow.inspect_score({"verified":True})["offline_graph_verified"])
            inv_rows,score_rows=graph.projection()
            result=evaluate_synthetic(inv_rows,score_rows,purpose="synthetic")
            # Compare with the frozen scorer's pre-existing independent fixture expectation.
            ref_inv,ref_rows=scenario()
            reference=evaluate_synthetic(ref_inv,ref_rows,purpose="synthetic")
            self.assertEqual(result,reference)
            self.assertEqual(result["synthetic_quality_status"],"PASS")
            self.assertEqual(result,evaluate_synthetic(inv_rows[::-1],score_rows[::-1],purpose="synthetic"))
            for key in ("formal_score_authorized","live_authorized","quality_go","release_go"):
                self.assertIs(result[key],False)
                self.assertIs(formal_entrypoint(graph=graph,permission=score_receipt,verified=True)[key],False)
            self.assertIsNone(result["formal_f1"])
            # A trusted new registry with revocation invalidates prior authorizations;
            # selecting/refreshing that registry externally is not self-proven here.
            updated=copy.deepcopy(dat)
            next(p for p in updated["principals"] if p["role"]=="judge_a")["revoked"]=True
            update_raw=encode(updated)
            graph._registry=Registry(update_raw,pinned_sha256=sha(update_raw),evaluation_id=EVAL)
            with self.assertRaises(ValueError): graph.projection()
            graph._registry=registry
            from datetime import timedelta
            clock[0]=NOW-timedelta(seconds=1)
            self.assertFalse(flow.inspect_score(graph)["offline_graph_verified"])
            clock[0]=NOW
            self.assertFalse(flow.inspect_score(graph)["offline_graph_verified"])

    def test_incomplete_capture_preserves_only_signed_adverse_diagnostic(self):
        with tempfile.TemporaryDirectory() as tmp:
            profile=initialize_local_store(Path(tmp).resolve()/"synthetic-store")
            dat,keys,registry,signed,inventory,slots,salt,mapping,inv,cond,authority_pin,planned,planned_sources,legacy,plan,ledger,witness,clock,flow,consume,run=fixed_setup(self,profile)
            slot=slots[0]
            attempt=flow.reserve_slot(slot)
            display=b"invented adverse partial statement"
            stdout=b"prefix "+display+b" suffix"
            sources=encode(fixed_source()[3])
            span=[7,7+len(display)]
            capture=dict(binding_raw=planned[slot],trace_raw=b"incomplete synthetic trace",
                journal_raw=b"incomplete synthetic journal",stdout=stdout,stderr=b"",
                sources_raw=sources,final_bytes=display,display_span=span,status="timeout")
            payload={"attempt_id":attempt,"slot":list(slot),"status":"timeout",
                "stdout_sha256":sha(stdout),"stderr_sha256":sha(b""),
                "journal_sha256":sha(capture["journal_raw"]),"trace_sha256":sha(capture["trace_raw"]),
                "sources_sha256":sha(sources),"binding_sha256":sha(planned[slot]),"display_span":span,
                "partial_display_sha256":sha(display),"final_sha256":None,"calls":None,"tokens":None,
                "formal_eligible":False}
            capture["envelope"]=signed(encode(payload),"raw_evidence","capture","partial-"+attempt,[inv,cond,run])
            with self.assertRaises(ValueError):
                plan.verify_partial(ledger=ledger,registry=registry,now=NOW,
                    record=ledger.active_reservation(),raw={**capture,"final_bytes":b"other"},run_receipt=run)
            saved=flow.capture_partial(capture)
            self.assertIsNone(saved["final_sha256"])
            self.assertIsNone(saved["tokens"])
            with self.assertRaises(ValueError): ledger.close()
            with self.assertRaises(ValueError): flow.reserve_slot(slots[1])
            graph=PartialAdverseReview(flow=flow,raw=capture,mapping=mapping,salt=salt)
            bundle=graph.judge_bundle()
            self.assertEqual(set(bundle),{"blind_id","evidence_token","partial_display","source_texts","targets","judge_context"})
            def judgment(targets,adverse):
                decisions=[]
                for t in targets:
                    true=t in adverse
                    byte_span=[0,len(display)] if true else None
                    basis=sha(encode({"token":bundle["evidence_token"],"target":t,
                         "span":byte_span,"excerpt":display.decode(),"source_texts":bundle["source_texts"],
                         "judge_context":bundle["judge_context"]})) if true else None
                    decisions.append(dict(target=t,value="true" if true else "unknown",byte_span=byte_span,basis_digest=basis))
                return encode(dict(blind_id=bundle["blind_id"],evidence_token=bundle["evidence_token"],decisions=decisions))
            target=ADVERSE[0]
            a=judgment(ADVERSE,{target})
            b=judgment(ADVERSE,set())
            ap=signed(a,"atomic_judgment","judge_a","partial-judge_a",[saved["observation_sha256"]])
            bp=signed(b,"atomic_judgment","judge_b","partial-judge_b",[saved["observation_sha256"]])
            bad=decode(a); bad["decisions"][0]["value"]="false"
            with self.assertRaises(ValueError):
                graph.submit("judge_a",encode(bad),ap)
            ar=graph.submit("judge_a",a,ap)
            br=graph.submit("judge_b",b,bp)
            with self.assertRaises(ValueError): graph.diagnostic()
            c=judgment([target],{target})
            ce=signed(c,"adjudication","adjudicator","partial-adjudication",[ar,br])
            rat=encode(dict(blind_id=bundle["blind_id"],adjudication_receipt=sha(ce),
                 decision="diagnostic-only",formal_eligible=False))
            re=signed(rat,"adjudication_ratification","adjudication_ratifier","partial-ratification",[ar,br,sha(ce)])
            graph.adjudicate(c,ce,rat,re)
            d=graph.diagnostic()
            self.assertEqual(d["adverse_claims"],1)
            self.assertEqual(d["unknown_claims"],3)
            self.assertIsNone(d["final_sha256"])
            self.assertFalse(d["formal_score_authorized"])
            self.assertFalse(flow.inspect_score(graph)["offline_graph_verified"])

    def test_citation_collapse_never_erases_invalid_or_unknown(self):
        self.assertEqual(citation_summary({"citation-required-a":"valid","citation-observed-native-0":"invalid",
                                          "citation-required-b":"unknown"}),"invalid")
        self.assertEqual(citation_summary({"citation-required-a":"missing","citation-required-b":"unknown"}),"unknown")
        self.assertEqual(citation_summary({"citation-required-a":"valid","citation-required-b":"missing"}),"missing")
        self.assertEqual(citation_summary({"citation-required-a":"valid","citation-observed-native-0":"valid"}),"valid")

    def test_checkpoint_advance_and_same_account_freshness(self):
        from datetime import timedelta
        with tempfile.TemporaryDirectory() as tmp:
            profile=initialize_local_store(Path(tmp).resolve()/"synthetic-store")
            dat,keys,registry,signed,inventory,slots,salt,mapping,inv,cond,authority_pin,planned,planned_sources,legacy,plan,ledger,witness,clock,flow,consume,run=fixed_setup(self,profile)
            self.assertTrue(flow.inspect_run()["offline_graph_verified"])
            self.assertFalse(flow.inspect_run()["live_authorized"])
            clock[0]=NOW+timedelta(seconds=10)
            observed="2026-01-01T00:00:20Z"
            raw=encode({"contract":"620-witness/1","evaluation_id":EVAL,
                "registry_digest":registry.digest,"sequence":2,"previous":witness.digest,
                "observed_at":observed,"expires_at":END})
            env=signed(raw,"checkpoint","journal_witness","witness-2",[witness.digest],issued_at=observed)
            newer=WitnessWindow(ledger=ledger,registry=registry,expected_pin=sha(env),
                expected_sequence=2,expected_previous=witness.digest,raw=raw,envelope=env,clock=lambda:clock[0])
            flow.advance_witness(newer)
            # The previous snapshot is not for the current witness.
            self.assertNotEqual(flow._billing[4][1],newer.digest)
            prev=decode(flow._billing[0])
            expired=encode({**prev,"expires_at":observed})
            with self.assertRaises(ValueError):
                validate_billing_snapshot(expired,plan,clock[0])
            payg=copy.deepcopy(prev);payg["subscription"]["payg_enabled"]=True
            with self.assertRaises(Exception): validate_billing_snapshot(encode(payg),plan,clock[0])
            refreshed=encode({**prev,"checked_at":observed})
            parents=[cond,newer.digest,flow._billing[2]]
            env=signed(refreshed,"billing_attestation","billing_witness","offline-billing-2",parents,issued_at=observed)
            flow.attach_billing_snapshot(refreshed,env)
            self.assertTrue(flow.inspect_run()["offline_graph_verified"])
            flow.reserve_slot(slots[0])
            with self.assertRaises(ValueError): witness.recheck()

    def test_redacted_audit_never_exports_input(self):
        out=redacted_status("score_preflight","graph_invalid")
        self.assertEqual(out["reason"],"graph_invalid")
        self.assertFalse(out["formal_score_authorized"])
        with self.assertRaises(ValueError):
            redacted_status("score_preflight","secret arbitrary input")


if __name__=="__main__": unittest.main()
