from pathlib import Path
import json,hashlib,subprocess,re
r=Path.cwd();host='ec6e55ff95b8364f9a5d09b8bbfe0ecc0c9d19d7';candidate='9082365ffecfafaa9ba0ac65ce56368ac77d4228';out=Path('/private/tmp/engine073-source27-static-path-report.json');assert not out.exists()
names=('derive_effective_evidence_local_qualification_v17','materialize_evidence_relevance_v30')
def scan(ref,prefix):
 paths=subprocess.check_output(['git','ls-tree','-r','--name-only',ref,prefix],text=True).splitlines();files={};hits=[]
 for rel in paths:
  if not rel.endswith('.rs'):continue
  data=subprocess.check_output(['git','show',ref+':'+rel]);files[rel]=hashlib.sha256(data).hexdigest()
  for n,line in enumerate(data.decode().splitlines(),1):
   if any(re.search(r'\b'+name+r'\b',line) for name in names):hits.append({'path':rel,'line':n,'text':line.strip(),'category':'module_export' if rel.endswith('/lib.rs') else 'evaluation_binary' if '/src/bin/' in rel else 'relevance_module' if rel.endswith('/evidence_relevance.rs') else 'other_source'})
 return {'ref':ref,'file_sha256':files,'hits':hits}
cli=scan(host,'crates/reasoning-harness-cli/src');core=scan(candidate,'crates/reasoning-harness-core/src')
ordinary_cli=[x for x in cli['hits'] if x['category']!='evaluation_binary'];outside_core=[x for x in core['hits'] if x['category'] not in ('module_export','relevance_module')]
record={'status':'STATIC_PREPARATION_ONLY','symbols':names,'common_published_cli_host':cli,'candidate_core':core,'non_evaluation_cli_source_symbol_matches':ordinary_cli,'core_matches_outside_relevance_module_or_exports':outside_core,'scope':'Literal Rust symbol-reference inventory only; public re-exports and private module tests may appear. Not complete call graph, macro/FFI/dynamic proof, runtime evaluation, all Engine feature inactivity, final-answer improvement or CLI acceptance. Prior same-host fixed Node runtime public-API counter0 and direct control are separate preserved evidence. Need explicit ordinary path of evaluated APIs plus actual runtime acquisition/admission/display trace before M0.','engine_scorer_provider_model_network_calls':0,'files_mutated':0}
assert not ordinary_cli and not outside_core
out.write_text(json.dumps(record,indent=2)+'\n');print('CLI files',len(cli['file_sha256']),'symbol matches',len(cli['hits']),'non-evaluation matches',len(ordinary_cli));print('core files',len(core['file_sha256']),'outside relevance/exports',len(outside_core))
