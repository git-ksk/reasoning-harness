"""Fixed invented 24-case/48-observation offline graph, never real inputs."""
import copy
import json
import tempfile
import unittest
from pathlib import Path

from auth620 import Registry, encode, decode, sha, formal_entrypoint
from first_claim_620 import FirstClaimLedger, inventory_slots, FIXED_CONDITIONS
from observation_620 import verify_observation
from blind_fixture_620 import BlindFixtureGraph, map_digest
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
               "source_texts":bundle["source_texts"]}
        rows.append({"target":target,"value":value,"evidence_kind":kind,
                     "evidence_token":bundle["evidence_token"],"basis_digest":sha(encode(basis))})
    return encode({"blind_id":bundle["blind_id"],"decisions":rows})


class FixtureGraphTests(unittest.TestCase):
    def test_fixed_synthetic_graph_and_denials(self):
        # No caller-provided fixture, paths, real IDs, evaluation questions or Gold.
        with tempfile.TemporaryDirectory() as tmp:
            profile=initialize_local_store(Path(tmp).resolve()/"synthetic-store")
            dat,keys=fixture_registry(); raw_registry=encode(dat)
            registry=Registry(raw_registry,pinned_sha256=sha(raw_registry),evaluation_id=EVAL)
            def signed(raw,kind,role,aid,parents):
                return sign(registry,keys,raw,kind=kind,role=role,artifact_id=aid,parents=parents)
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
            inv=verified(inventory,"inventory","author","inventory",[])
            authority_pin=h("frozen synthetic legacy authority")
            planned={slot:fixed_binding(slot,authority_pin) for slot in slots}
            planned_sources=sha(encode(fixed_source()[3]))
            conditions_raw=encode({"conditions":FIXED_CONDITIONS,"map_commitment":map_pin,
                                    "inventory":inv,"authority":authority_pin,
                                    "source_pin":planned_sources,
                                    "first_bindings":[sha(planned[slot]) for slot in slots]})
            cond=verified(conditions_raw,"execution_conditions","coordinator","conditions",[inv])
            ledger=FirstClaimLedger(profile,dataset_sha256=sha(inventory),inventory_raw=inventory,
                   inventory_receipt=inv,conditions_receipt=cond,evaluation_id=EVAL,blind_map_sha256=map_pin)
            flow=FixturePermissions(ledger,registry,inv,cond,NOW)
            with self.assertRaises(ValueError): flow.expected_run()
            for kind,role in (("normative_ratification","normative_ratifier"),
                              ("implementation_acceptance","implementation_acceptor")):
                p=flow.expected_prospective(kind)
                flow.prospective(kind,p,signed(p,kind,role,kind,flow.parents))
            with self.assertRaises(ValueError): flow.expected_run()
            flow.attach_fixture_billing()
            p,parents=flow.expected_run()
            with self.assertRaises(ValueError):
                flow.consume_run(p,signed(p,"run_permission","score_approver","fixture-run-once",parents))
            run=flow.consume_run(p,signed(p,"run_permission","run_approver","fixture-run-once",parents))
            with self.assertRaises(ValueError): flow.consume_run(p,b"{}")
            with self.assertRaises(ValueError): flow.prospective("normative_ratification",b"{}",b"{}")
            legacy=encode({"contract":POLICY,"policy_id":"synthetic-local-only",
                    "principals":[{"subject_id":"subject-executor","aliases":["alias-executor"],
                        "role":"agent","conflict_groups":["execution"]},
                        {"subject_id":"subject-safety_reviewer","aliases":["alias-safety_reviewer"],
                        "role":"agent","conflict_groups":["review"]} ],
                    "keys":[{"key_id":"key-safety_reviewer","subject_id":"subject-safety_reviewer",
                        "public_key_hex":keys["safety_reviewer"].verify_key.encode().hex(),"revoked":False}]})
            observations=[]; observation_inputs=[]
            for i,slot in enumerate(slots):
                attempt=flow.reserve_slot(*slot)
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
                observation=verify_observation(**kw)
                ledger.record(attempt_id=attempt,status="complete",calls=observation["calls"],
                  tokens=observation["tokens"],admission_sha256=observation["admission_sha256"],
                  final_sha256=observation["final_sha256"],observation_sha256=observation["observation_sha256"])
                observations.append(observation); observation_inputs.append(kw)
            self.assertEqual(len({x["observation_sha256"] for x in observations}),48)
            self.assertEqual(len({x["final_sha256"] for x in observations}),2)
            closure=ledger.close()
            graph=BlindFixtureGraph(ledger=ledger,registry=registry,inventory_raw=inventory,
                inventory_receipt=inv,conditions_receipt=cond,closure_digest=closure,
                observation_inputs=observation_inputs,mapping=mapping,salt=salt,now=NOW)
            with self.assertRaises(ValueError): graph.unblind(b"{}",b"{}")
            with self.assertRaises(ValueError): graph.lock_payload()
            with self.assertRaises(ValueError): flow.expected_score(graph)
            first=graph.blind_ids()[0]; second=graph.blind_ids()[1]
            for bid in graph.blind_ids():
                bundle=graph.judge_bundle(bid)
                self.assertEqual(set(bundle),{"blind_id","display","source_texts","targets","evidence_token"})
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
            with self.assertRaises(ValueError): flow.expected_score(graph)
            opening={"mapping":mapping,"salt":salt,"lock_receipt":lock}
            for change in ({"salt":h("wrong")},{"mapping":mapping[:-1]}):
                p=encode({**opening,**change})
                with self.assertRaises(ValueError):
                    graph.unblind(p,signed(p,"unblind","unblinder","full-unblind",[lock]))
            p=encode(opening)
            graph.unblind(p,signed(p,"unblind","unblinder","full-unblind",[lock]))
            with self.assertRaises(ValueError): graph.unblind(p,b"{}")
            p,parents=flow.expected_score(graph)
            flow.consume_score(graph,p,signed(p,"score_permission","score_approver","fixture-score-once",parents))
            with self.assertRaises(ValueError): flow.consume_score(graph,p,b"{}")
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
                self.assertIs(formal_entrypoint(graph=graph,permission=flow.score,verified=True)[key],False)
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
            graph._now=NOW+timedelta(minutes=1)
            with self.assertRaises(ValueError): graph.projection()


if __name__=="__main__": unittest.main()
