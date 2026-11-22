"""Fixed, offline synthetic test entrypoint. No data, path or purpose arguments.

Run with the hash-pinned Python environment. This is a test entrypoint, not a
score API. The legacy arithmetic function is a pure reference, not an entrypoint.
"""
from pathlib import Path
import sys
import unittest
import io

def run_fixed_fixture():
    root=str(Path(__file__).resolve().parent)
    if root not in sys.path: sys.path.insert(0,root)
    from test_fixture_graph_620 import FixtureGraphTests
    suite=unittest.TestSuite([FixtureGraphTests("test_fixed_synthetic_graph_and_denials")])
    out=unittest.TextTestRunner(stream=io.StringIO()).run(suite)
    return {"fixture_status":"PASS" if out.wasSuccessful() else "HOLD",
            "formal_score_authorized":False,"live_authorized":False,
            "quality_go":False,"release_go":False,"formal_f1":None}

if __name__=="__main__":
    import json
    if len(sys.argv)!=1:
        raise SystemExit("fixed fixture accepts no arguments")
    result=run_fixed_fixture()
    print(json.dumps(result,sort_keys=True))
    raise SystemExit(0 if result["fixture_status"]=="PASS" else 1)
