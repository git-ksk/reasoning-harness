#!/usr/bin/env python3
"""Independent component review closure: record/copy/hash only; never rerun checks."""
from pathlib import Path
from datetime import datetime,timezone
import os,json,hashlib,subprocess
P=Path(__file__).resolve().parent
ROOT=Path('/Users/sawadakousuke/.codex/worktrees/engine073-retraction-safety/reasoning-harness')
sha=lambda f:hashlib.sha256(Path(f).read_bytes()).hexdigest()
now=lambda:datetime.now(timezone.utc).isoformat()
def dump(n,d):
 with (P/n).open('x') as f:json.dump(d,f,indent=2,ensure_ascii=False);f.write('\n');f.flush();os.fsync(f.fileno())
def text_new(n,s):
 with (P/n).open('x') as f:f.write(s);f.flush();os.fsync(f.fileno())
initial=json.loads((P/'initial-inventory.json').read_text())
head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip()
assert head==initial['HEAD']=='f6d99399a77932670adef048f1cce53917285d3c'
for n,fs in initial['component_files'].items():
 d=ROOT/'evaluation'/n
 assert {str(f.relative_to(d)) for f in d.rglob('*') if f.is_file()}==set(fs)
 assert all(sha(d/f)==h for f,h in fs.items())
 # Each original component's already-published index must also remain exact.
 index=json.loads((d/'artifact-sha256.json').read_text())
 assert all(sha(d/f)==h for f,h in index.items())
a=Path('/private/tmp/engine073-answer-metrics-independent-review-v1/v27-archive-verification.json')
with (P/'v27-archive-verification.opaque.json').open('xb') as f:f.write(a.read_bytes());f.flush();os.fsync(f.fileno())
anchor=ROOT/'evaluation/engine073-launch-independent-v27/final-artifact-manifest.json'
assert sha(anchor)=='abaae2385e6c17783cc7093c10e15424b6e4ce01080e8fcdea5eab3a9e5bb439'
old=json.loads(anchor.read_text())
assert len(old['files'])==927
assert all(sha(ROOT/f)==h for f,h in old['files'].items())
first=json.loads((P/'adversarial-observations.json').read_text())
assert first['existing_metrics_tests']['tests_run']==32 and first['existing_metrics_tests']['successful']
assert first['existing_trace_pure_tests']['tests_run']==4 and first['existing_trace_pure_tests']['successful']
findings=[
 {'id':'M0-F1','priority':'P2','status':'REPRODUCED_M0_BLOCKER','component':'engine073-answer-metrics-development-v1/display_binding.py','lines':[77,81,97,99],'problem':'The same claim occurrence ID can bind two distinct fact spans in answer and claim card. Set-based coverage collapses both physical occurrences into one review/arithmetic occurrence.','observed':'Two fact spans accepted; declared_byte_coverage COMPLETE; claim_occurrences1 and arithmetic displayed_claim_occurrences1. Constructed annotation gives F1=1; this is not an actual answer F1.','reproduction':'independent_checks.py display-occurrence-reused; display-occurrence-reuse.input.json','paired_control':'Distinct occurrence IDs for the same proposition report occurrences2 but deduplicate proposition A to1.','required_fix':'Give each physical displayed claim occurrence a unique occurrence ID and bind one occurrence to an explicit location/span-set; reject ID reuse across answer/card/caption. If a single occurrence can span fragments, freeze and validate that explicit fragment schema instead of implicit set membership.','boundary':'Semantic proposition identity and grounding remain external. This is a structural cardinality/contract defect, not a proof that the arithmetic formula or a real answer F1 is wrong.'},
 {'id':'M0-F2','priority':'P2','status':'REPRODUCED_M0_BLOCKER','component':'engine073-cli-trace-binding-development-v1/trace_binding.py','lines':[51,83,102,106],'problem':'Request/response matching is checked without requiring request completion before terminal finalization.','observed':'Ordered events request→admission→grounding→finalization→response still return STRUCTURAL_MATCH and model_calls1.','reproduction':'independent_checks.py model-response-after-terminal; late-response.input.json','paired_control':'Wrong model correlation and wrong acquisition target both fail in the same fully constructed graph.','required_fix':'Freeze the permitted event state machine. Match each request with exactly one valid response/failure and require all required calls completed before the terminal artifact/display. Reject late model responses/requests or record an explicitly separate non-scored phase; do not call such traces operationally complete.','boundary':'This reproduces acceptance of an inconsistent constructed trace. It does not assert that the preserved ordinary CLI traces were inconsistent or that a live request ran.'},
 {'id':'M0-F3','priority':'P2','status':'EXPRESSION_REPRODUCED_M0_BLOCKER','component':'engine073-active-source-host-development-v1/renderer_host.rs','lines':[19],'problem':'Option equality permits None==None when the source span is invalid and quote is missing or not a string.','observed':'A standard-library-only Rust program using the exact guard expression exits0 for invalid [0,99) on abc and quote=None.','reproduction':'renderer_option_guard.rs; renderer-guard-first.stdout; renderer-guard-invocations.json','required_fix':'Require a string quote and separately require valid nonempty ordered UTF-8 byte boundaries before comparing the exact source slice. Then validate source identity/version/time metadata against the frozen retained-source manifest before any Engine API call.','boundary':'Only the original guard expression was reproduced, without linking or executing Engine. Downstream Engine rejection is untested; no accepted final citation or source-attribution exploit is claimed. host.rs uses an earlier expect for its slice and is not the same Option bug.'},
 {'id':'M0-G1','priority':'P2','status':'REPRODUCED_FULL_M0_SCHEMA_GAP','component':'engine073-cli-trace-binding-development-v1/trace_binding.py','lines':[73,81],'problem':'A response={} dictionary counts as a completed model response. ModelRequest validation also checks only task/max_tokens.','observed':'STRUCTURAL_MATCH with one counted model call for the empty response, not a valid measured answer.','reproduction':'independent_checks.py empty-model-response; adversarial-observations.json','required_fix':'Use the actual recorded ModelRequest/ModelResponse/failure schema and strict numeric/enum validation. Bind request/model/provider/phase/attempt/output format; require finish/status/content and preserve complete versus unknown token/attempt/cost accounting. Missing records/usage cannot be zero or safe semantic withholding.','boundary':'The current README limits the adapter to structural identities and does not claim full usage/accounting validity. This is a required complete-M0 integration gate, not evidence that a historical response was empty.'}
]
remaining=[
 {'gate':'measurement_profile_and_active_path','required':'Freeze separate unchanged ordinary-CLI control, evaluated active relevance host, and isolated source-local renderer tracks. Pin both actual core builds and same host/settings/dependencies. Prove intended APIs run through the measured question/acquisition/final-answer path and retain a positive functional control. Existing node-control is withheld in both arms; manual renderer success cannot rescue it or show a candidate gain.'},
 {'gate':'gold_source_applicability_and_authority','required':'Before fresh authorship/observation, freeze retained raw/normalized sources, exact quotes/full context, URL/document/version hashes, capture/as-of dates, source scopes and authority. Precommit required groups, allowed equivalents, answerability and correction/conflict/time rules with gold author/reviewer roles. Curated reference facts and source-local qualification are not publisher truth or independent human approval.'},
 {'gate':'display_semantics_citation_and_utility','required':'Recorder must enumerate all actual answer/card/caption bytes independently of annotation. Bind occurrence/proposition/receipt/evidence/quote/source chains and fix F1. Then perform declared external segmentation, support, negation/strength/scope/time, contradiction/correction and citation review. Internal receipts are not displayed citations. Freeze usefulness/correct withholding/over-withholding measures and their populations separately from claim arithmetic and case-quality F1; do not invent thresholds in this review.'},
 {'gate':'strict_trace_and_first_once_scorer','required':'Fix F2/F3/G1 in a new namespace and test malformed/late/partial traces, actual raw schemas, output/source swaps and incomplete/abstention distinctions. Pin question/target/acquisition/admission/claim/final-display identities and model settings. Integrate one complete scorer with immutable M0-before-fresh-authoring, M1-before-first, exclusive invocation markers, raw preservation and fail-closed provenance. Missing/unreviewed/planned rows remain in operational and delivered-coverage denominators.'},
 {'gate':'bounded_runner_callbacks_and_budgets','required':'Integrate the actual host/callback/resolver/dependency closure with the reviewed durable shared guard. Freeze per-case/pair/lane model logical/raw/HTTP attempts, input/output/total tokens, elapsed time and cost/reserve limits. Record unknown usage/cost as unknown with conservative reserves, not zero. Carry quota latches, cancellation/process-group cleanup, no automatic failed-first retries and identical paired conditions into actual execution.'},
 {'gate':'independent_preflight_and_authorization','required':'Complete pure adversarial tests and no-network operational controls of the integrated stack, independently review/pin M0 and only then authorize unused real24 authorship. Preserve all prior raw/gold/scorers/first FAILs. Provider-specific execution approval/headroom/numeric recovery remains separate; Groq is not resumed by this review.'}
]
summary={'identity':'engine073-practical-component-independent-review-v1','reviewed_utc':now(),'candidate_checkout_HEAD':head,'result':'FAIL_FOR_COMPLETE_REAL_DOCUMENT_M0','current_components_contract':'Preparation only; arithmetic/saved structural binding/isolated host controls are not a complete semantic scorer, trace attestation, M0 or actual-answer F1. Their NOT_EVALUATED/null/no-gain limitations are correctly retained.','existing_constructed_tests':{'metrics32':'PASS','trace_inmemory4':'PASS','saved_corpus_dependent_trace_tests':'NOT_RUN: preserve independent source-author blind boundary; code reviewed only'},'findings':findings,'required_complete_M0_gates':remaining,'v27_archive_safe':True,'all928_original_references_unchanged':True,'all_three_components_inventories_and_original_indexes_unchanged':True,'Engine_or_actual_cohort_scorer_executions':0,'provider_model_network_Oracle_Groq_calls':0,'new_source_real24_or_gold_authorship':0,'Git_sharedtarget_existing_artifact_mutations_extra_agents':0,'standard_library_only_guard_reproduction':{'compile':1,'process':1,'Engine_linked':False},'parent_response':'Parent accepted the three boundaries and empty-response gap as M0 blockers; will fix in a new namespace. Existing reviewed component bytes remain immutable. No fresh authorship is authorized.'}
dump('review.json',summary)
text_new('README.md','''# Independent review of practical evaluation components

Result: **FAIL for complete real-document M0 readiness**. The current modules correctly remain preparation with `acceptance=NOT_EVALUATED` and actual-answer F1 unmeasured. No current or historical answer was rescored.

The 32 arithmetic/span constructed tests and four strictly in-memory trace tests pass. Saved-corpus-dependent trace tests were read but not run to preserve the independent source-author boundary. New adversarial fixtures reproduce:

1. One occurrence ID reused at two displayed fact spans is accepted and counted once. Fix location/occurrence cardinality before complete display safety review.
2. A model response that completes a request after finalization is accepted as `STRUCTURAL_MATCH`. Require the complete terminal event state machine.
3. The isolated renderer's guard accepts `None == None` for an invalid source span and absent/non-string quote. This is an expression-only std-Rust reproduction with no Engine linked; no forged final citation is claimed.
4. An empty model-response dict is counted as a complete pair. This demonstrates a missing full-M0 response/usage schema gate, beyond the current adapter's identity-only scope.

`review.json` records exact code locations, counterexamples, matched negative controls, limits and concrete remaining M0 gates. `independent_checks.py` and the input/raw logs preserve the first component observations; they do not author a new semantic holdout or new real-document gold. Pure constructed arithmetic F1 values must not be presented as actual-answer F1.

Existing output identities/byte spans and source-chain checks are useful preparation. They do not judge semantic support, source authenticity or applicability, citation correctness, usefulness, equal-condition paired gains, live budgets or complete callback/dependency containment. The active relevance Node control withholds in both arms; isolated manual renderer output does not rescue that result. No demonstrated quality gain or cross-cohort improvement is asserted.

All three original component inventories/indexes and all 928 v27/old/preparation references remain exact. The earlier post-copy v27 archive check passed for both copies' 99 readonly files; the managed source27-freeze is archive-safe. Root/Git/source/old scorer and artifacts are unchanged by this review.

Provider/model/network/Oracle/Groq, Engine, actual-cohort scorer, real24/source-case/gold authorship and additional agents: 0. Parent will repair in a new namespace and provide a separately frozen integrated stack for review. No fresh authorship is authorized by this result. The reviewer stops after readonly closure.
''')
dump('receipt.json',{'identity':summary['identity'],'closed_utc':now(),'result':summary['result'],'HEAD':head,'input_component_files':initial['component_files'],'all928_original_references_verified':True,'old_anchor':sha(anchor),'initial_inventory_sha256':sha(P/'initial-inventory.json'),'checks_script_sha256':sha(P/'independent_checks.py'),'first_checks_observations_sha256':sha(P/'adversarial-observations.json'),'review_sha256':sha(P/'review.json'),'no_component_checks_reexecuted_during_closure':True,'immutable_input_or_first_results_modified':False,'authorship_Engine_real_cohort_scorer_provider_model_network_Oracle_Groq_extra_agents':0,'next':'stop; future integrated M0 snapshot requires separate review and explicit authorship authorization'})
files={str(f.relative_to(P)):sha(f) for f in sorted(P.rglob('*')) if f.is_file()}
final={'identity':summary['identity'],'closed_utc':now(),'self_exclusion':'artifact-manifest.json excluded; its external SHA256 is final anchor','files':files,'file_count_self_excluding':len(files),'file_count_including_manifest':len(files)+1,'input_component_files':initial['component_files'],'input_HEAD':head,'old_corpus_anchor':sha(anchor),'old_corpus_references_including_manifest':928,'review_result':summary['result'],'all_old_and_input_bytes_unchanged':True,'Engine_actual_cohort_scorer_provider_model_network_Oracle_calls':0}
dump('artifact-manifest.json',final)
for f in P.rglob('*'):
 if f.is_file():
  with f.open('rb') as h:os.fsync(h.fileno())
  os.chmod(f,0o444)
fd=os.open(P,os.O_RDONLY)
try:os.fsync(fd)
finally:os.close(fd)
assert all(sha(P/n)==h for n,h in files.items())
assert all(not(f.stat().st_mode&0o222) for f in P.rglob('*') if f.is_file())
assert all(sha(ROOT/'evaluation'/n/f)==h for n,fs in initial['component_files'].items() for f,h in fs.items())
assert all(sha(ROOT/f)==h for f,h in old['files'].items())
print(json.dumps({'result':summary['result'],'final_anchor':sha(P/'artifact-manifest.json'),'new_review_files_including_manifest':len(files)+1,'old928_and_all3_components_unchanged':True,'HEAD':head,'existing_constructed_tests':'32+4 PASS','findings':'three reproduced boundaries + full-response schema M0 gap','new_authorship_Engine_real_cohort_scorer_provider_model_network_Oracle_Groq':0,'state':'CLOSED_READONLY_STOPPED'},indent=2))
