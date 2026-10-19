"""Structural regression checks, not source accuracy/independent questions."""
import copy,json,sys,unittest
from pathlib import Path
sys.path.insert(0,str(Path(__file__).parent))
import scorer

REPO=Path(__file__).resolve().parents[2]
if not (REPO/'evaluation').exists():REPO=Path('/Users/sawadakousuke/.codex/worktrees/engine073-retraction-safety/reasoning-harness')

def pair():
    src={'source_version':'sha256:'+'1'*64,'url':'https://example.invalid/engineering-only','quote':'Value is four.'}
    gold={'case_id':'engineering-only','sha256':'2'*64,'family':'structural-only','answerability':'full','label':'positive','groups':[{'id':'value'}],'unresolved_facets':[],'required_disclosures':[]}
    obs={'row_id':'opaque-row','provider':'mistral','track':'e2e_normal','arm':'candidate','operation':'complete',
         'display_artifacts':[{'id':'answer','kind':'final_answer','text':'Value is four.'}],
         'acquired_source_ids':['s'],'admitted_source_ids':['s'],'terminal_status':'grounded_answer',
         'display_citations':[{'id':'c','artifact_id':'answer','start_byte':0,'end_byte':14,'source_id':'s','source_version':src['source_version'],'url':src['url'],'literal_quote':src['quote']}]}
    rev={'row_id':'opaque-row','gold_sha256':gold['sha256'],'reviewer':{'id':'engineering-reviewer','role':'agent'},'complete':True,'full_source_context_reviewed':True,
         'claims':[{'id':'occurrence-1','proposition_id':'value-four','grounding':'grounded','satisfied_groups':['value'],'rationale':'Constructed annotation only.',
                    'source_anchors':[{'source_id':'s','source_version':src['source_version'],'support_explanation':'Structural fixture only.'}]}],
         'segments':[{'id':'span-1','artifact_id':'answer','start_byte':0,'end_byte':14,'text':'Value is four.','kind':'fact','claim_id':'occurrence-1','proposition_id':'value-four'}],
         'citation_edges':[{'id':'occurrence-1@c','correct':True,'literal_exact':True,'rationale':'Exact fixture bytes.'}],
         'all_unresolved_facets_handled':True,'handled_unresolved_facets':[],'all_required_disclosures_present':True,'present_disclosures':[],
         'safety_audit_complete':True,'hard_violations':[],'readable_and_actionable':True,'attempted_accept':True,'explicit_withhold':False}
    rev['observation_sha256']=scorer.sha(json.dumps(obs,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode())
    return gold,obs,rev,{'s':src}

class Boundaries(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.deps=scorer.frozen_dependencies(REPO)
    def test_complete_structural_control(self):
        row=scorer.validate_review(*pair(),self.deps);self.assertEqual(row['claim_metrics']['f1'],1);self.assertTrue(row['complete_useful_answer'])
    def test_occurrence_cannot_cover_answer_and_card(self):
        g,o,r,s=pair();o['display_artifacts'].append({'id':'card','kind':'claim_card','text':'Value is four.'});r['segments'].append({**r['segments'][0],'id':'span-2','artifact_id':'card'})
        r['observation_sha256']=scorer.sha(json.dumps(o,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode())
        with self.assertRaisesRegex(ValueError,'claim_occurrence_reused'):scorer.validate_review(g,o,r,s,self.deps)
    def test_response_after_finalization_rejected(self):
        t={'events':[{'sequence':0,'kind':'model_request','correlation_id':'a','attempt':1},{'sequence':1,'kind':'finalization_state'},{'sequence':2,'kind':'model_response','correlation_id':'a','attempt':1}]}
        with self.assertRaisesRegex(ValueError,'response_after_terminal'):scorer.check_terminal_order(t,{})
    def test_empty_response_rejected(self):
        q={'kind':'model_request','generation':1,'provider':'mistral','model':'frozen','request':{'task':'engineering','max_tokens':16}}
        with self.assertRaisesRegex(ValueError,'empty_or_foreign_response'):scorer.check_model_journal([q,{'kind':'model_response','generation':1,'response':{}}],'mistral','frozen',1)
    def test_missing_quote_invalid_span_never_none_equals_none(self):
        raw=b'engineering';s={'raw_sha256':scorer.sha(raw),'normalized_sha256':scorer.sha(raw),'start_byte':400,'end_byte':500,'quote':None}
        with self.assertRaisesRegex(ValueError,'invalid_quote_type_or_range'):scorer.check_source(s,raw,raw)
    def test_missing_citation_review_rejected(self):
        g,o,r,s=pair();r['citation_edges']=[]
        with self.assertRaisesRegex(ValueError,'citation_review_inventory'):scorer.validate_review(g,o,r,s,self.deps)
    def test_unsupported_cannot_earn_recall_or_hide_safety(self):
        g,o,r,s=pair();r['claims'][0].update(grounding='unsupported',satisfied_groups=[]);r['citation_edges'][0]['correct']=False
        row=scorer.validate_review(g,o,r,s,self.deps);self.assertEqual(row['claim_metrics']['recall'],0);self.assertIn('unsupported_truth_or_display_promotion',row['hard_violations']);self.assertFalse(row['complete_useful_answer']);self.assertNotEqual(row['decision'],'accept')
    def test_unacquired_support_rejected(self):
        g,o,r,s=pair();o['acquired_source_ids']=[];r['observation_sha256']=scorer.sha(json.dumps(o,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode())
        with self.assertRaisesRegex(ValueError,'support_not_acquired'):scorer.validate_review(g,o,r,s,self.deps)
    def test_negative_unsafe_accept_is_not_tn(self):
        g,o,r,s=pair();g.update(label='negative',answerability='none');r['all_unresolved_facets_handled']=True
        row=scorer.validate_review(g,o,r,s,self.deps);self.assertEqual(row['decision'],'accept')
    def test_changed_observation_cannot_reuse_review(self):
        g,o,r,s=pair();o['display_artifacts'][0]['text']='Changed output.'
        with self.assertRaisesRegex(ValueError,'review_observation_mismatch'):scorer.validate_review(g,o,r,s,self.deps)
    def test_partial_operation_never_gets_semantic_f1(self):
        g,o,r,s=pair();o['operation']='incomplete';r['observation_sha256']=scorer.sha(json.dumps(o,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode())
        row=scorer.validate_review(g,o,r,s,self.deps);self.assertFalse(row['complete']);self.assertIsNone(row['claim_metrics']['f1']);self.assertFalse(row['complete_useful_answer'])
    def test_complete_constructed_cohort_counts_missing_provider(self):
        g,o,r,s=pair();template=scorer.validate_review(g,o,r,s,self.deps);rows=[];planned=[]
        for arm in ['baseline','candidate']:
            for n in range(24):
                row=copy.deepcopy(template);row.update(id='engineering-'+str(n),row_id=arm+str(n),arm=arm)
                row['expected']='positive' if n<12 else 'negative';row['decision']='accept' if n<12 else 'withhold';row['complete_useful_answer']=arm=='candidate' and n<12
                rows.append(row);planned.append({'id':row['id'],'row_id':row['row_id'],'provider':'mistral','track':'e2e_normal','arm':arm,'expected':row['expected'],'required_groups_G':1})
        protocol=json.loads((Path(__file__).parent/'protocol.json').read_text())
        report=scorer.summarize(rows,planned,protocol,self.deps)
        self.assertEqual(report['provider_tracks']['mistral/e2e_normal']['case_quality']['f1'],1)
        self.assertFalse(report['semantic_gate_pass']);self.assertIn('google/e2e_normal:case_quality_or_completeness',report['blockers'])

if __name__=='__main__':unittest.main()
