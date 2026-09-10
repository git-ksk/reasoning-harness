from pathlib import Path
import subprocess
import sys
import unittest

ROOT = Path(__file__).resolve().parents[1]
PAIRED = ROOT / '.github/workflows/natural-language-e2e-v31-paired-live.yml'
CROSS = ROOT / '.github/workflows/natural-language-e2e-v31-cross-model-live.yml'


class V31WorkflowPolicyTests(unittest.TestCase):
    def test_mistral_pair_uses_preserving_orchestrator(self):
        text = PAIRED.read_text(encoding='utf-8')
        self.assertIn('Run paired Mistral canonical observation with evidence preservation', text)
        self.assertIn('natural-language-e2e-v31-r2-paired-live', text)
        self.assertIn('natural-language-e2e-v31-freeze-r2', text)
        self.assertIn('python3 scripts/paired_canonical_observation.py', text)
        self.assertIn('--control-required /tmp/v31-mistral-control.json', text)
        self.assertIn('--candidate-required /tmp/v31-mistral-candidate.json', text)
        self.assertIn('--acceptance-required /tmp/v31-mistral-pair-acceptance.json', text)
        self.assertIn('--output /tmp/v31-mistral-paired-orchestration.json', text)
        self.assertIn('--allow-control-nonzero-with-evidence', text)
        self.assertIn('--heartbeat-seconds 45', text)
        self.assertNotIn('Run first paired Mistral control observation', text)
        self.assertNotIn('Run first paired Mistral candidate observation', text)

    def test_google_pair_uses_preserving_orchestrator(self):
        text = CROSS.read_text(encoding='utf-8')
        self.assertIn('Run paired Google canonical observation with evidence preservation', text)
        self.assertIn('natural-language-e2e-v31-r2-cross-model-live', text)
        self.assertIn('natural-language-e2e-v31-freeze-r2', text)
        self.assertIn('python3 scripts/paired_canonical_observation.py', text)
        self.assertIn('--control-required "/tmp/v31-${SLUG}-control.json"', text)
        self.assertIn('--candidate-required "/tmp/v31-${SLUG}-candidate.json"', text)
        self.assertIn('--acceptance-required "/tmp/v31-${SLUG}-pair-acceptance.json"', text)
        self.assertIn('--output "/tmp/v31-${SLUG}-paired-orchestration.json"', text)
        self.assertIn('--allow-control-nonzero-with-evidence', text)
        self.assertIn('--heartbeat-seconds 45', text)
        self.assertNotIn('Run paired Google control and candidate observations', text)

    def test_pre_live_coordinate_and_version_guards_are_explicit(self):
        for path in (PAIRED, CROSS):
            text = path.read_text(encoding='utf-8')
            self.assertIn('6bde9227d56ba237cb305777adf452461a0df864', text)
            self.assertIn('29a9e4be6273dbffeda324e15517dc64930ad315', text)
            self.assertIn('ccce3e56b3093746450db05a07ac2dacc473fa1b', text)
            self.assertIn('a91e16c017efcc14bdd698afab8c588af7e6cbac', text)
            self.assertIn('cf0cada8f4cf666f75b8dfb6c012a6ca63fb43a3', text)
            self.assertIn("grep -q '^version = \"0.4.1\"$' Cargo.toml", text)
            self.assertIn('git diff --exit-code "$CANDIDATE_COMMIT" -- Cargo.toml Cargo.lock crates', text)

    def test_paired_helper_exposes_v2_cli_contract(self):
        helper = ROOT / 'scripts/paired_canonical_observation.py'
        text = helper.read_text(encoding='utf-8')
        self.assertIn('CONTRACT_ID = "paired-canonical-observation-v2"', text)
        cp = subprocess.run(
            [sys.executable, str(helper), '--help'],
            cwd=ROOT,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            check=False,
        )
        self.assertEqual(cp.returncode, 0, cp.stderr)
        self.assertIn('--allow-control-nonzero-with-evidence', cp.stdout)
        self.assertIn('--heartbeat-seconds', cp.stdout)

    def test_google_models_use_two_model_specific_quota_lanes(self):
        text = CROSS.read_text(encoding='utf-8')
        self.assertIn('fail-fast: false', text)
        self.assertIn('max-parallel: 2', text)
        self.assertIn('scripts/validate_cross_model_concurrency_policy.py', text)


if __name__ == '__main__':
    unittest.main()
