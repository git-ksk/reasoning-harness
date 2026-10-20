"""Pure supplied structural fixture; no Engine, accuracy corpus, or scorer CLI."""
import copy, hashlib, importlib.util, json, os, sys
from pathlib import Path
HERE=Path(__file__).resolve().parent
INPUT=Path('/private/tmp/engine073-realworld-evaluator-review-snapshot-v2')
FIX=Path('/private/tmp/engine073-realworld-case-decision-delta-v5')
sys.dont_write_bytecode=True
sys.path.insert(0,str(INPUT))
import test_scorer

def load(name,path):
    spec=importlib.util.spec_from_file_location(name,path)
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
    return module

def save(name,value):
    data=(json.dumps(value,indent=2,ensure_ascii=False)+'\n').encode()
    with (HERE/name).open('xb') as f:f.write(data);f.flush();os.fsync(f.fileno())

versions={'original_v2':INPUT/'scorer.py','repaired_v5':FIX/'scorer.py'}
all_inputs={};all_rows={}
for identity,path in versions.items():
    module=load(identity,path);deps=module.frozen_dependencies(test_scorer.REPO)
    all_rows[identity]={}
    for label in ['positive','negative','ambiguous']:
        g,o,r,s=test_scorer.pair()
        g['label']=label
        if label!='positive':g['answerability']='none'
        r['claims'][0].update(grounding='unsupported',satisfied_groups=[])
        r['citation_edges'][0]['correct']=False
        row=module.validate_review(g,o,r,s,deps)
        all_inputs[label]={'gold':g,'observation':o,'review':r,'source_index':s}
        all_rows[identity][label]=row
        assert row['claim_metrics']['recall']==0
        assert row['complete_useful_answer'] is False
        assert 'unsupported_truth_or_display_promotion' in row['hard_violations']
        if label=='positive':assert (row['decision']=='accept')==(identity=='original_v2')
        else:assert row['decision']=='accept'
    g,o,r,s=test_scorer.pair()
    control=module.validate_review(g,o,r,s,deps)
    assert control['decision']=='accept' and control['complete_useful_answer'] is True
    all_rows[identity]['grounded_positive_control']=control
save('case-decision-structural-inputs.json',all_inputs)
save('case-decision-structural-first.json',all_rows)
summary={'scope':'pure supplied structural annotation fixture; no accuracy questions or cohort',
         'function_checks':8,'all_assertions_pass':True,
         'original_v2':{'unsupported_positive_decision':'accept','case_confusion_implication':'TP despite A1/S0/G1/H0; hard gate still rejects'},
         'repaired_v5':{'unsupported_positive_decision':'abstain','case_confusion_implication':'FN; negative/ambiguous unsafe accept still FP/hard'},
         'accuracy_Engine_actual_cohort_scorer_provider_model_network_Oracle_calls':0}
save('case-decision-structural-summary.json',summary)
print(json.dumps(summary,indent=2))
