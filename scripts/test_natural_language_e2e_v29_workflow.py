from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
PAIRED = ROOT / '.github/workflows/natural-language-e2e-v29-paired-live.yml'
CROSS = ROOT / '.github/workflows/natural-language-e2e-v29-cross-model-live.yml'


class V29WorkflowPolicyTests(unittest.TestCase):
    def test_mistral_pair_uses_preserving_orchestrator(self):
        text = PAIRED.read_text(encoding='utf-8')
        self.assertIn('Run paired Mistral canonical observation with evidence preservation', text)
        self.assertIn('python3 scripts/paired_canonical_observation.py', text)
        self.assertIn('--control-required /tmp/v29-mistral-control.json', text)
        self.assertIn('--candidate-required /tmp/v29-mistral-candidate.json', text)
        self.assertIn('--acceptance-required /tmp/v29-mistral-pair-acceptance.json', text)
        self.assertIn('--output /tmp/v29-mistral-paired-orchestration.json', text)
        self.assertNotIn('Run first paired Mistral control observation', text)
        self.assertNotIn('Run first paired Mistral candidate observation', text)

    def test_google_pair_uses_preserving_orchestrator(self):
        text = CROSS.read_text(encoding='utf-8')
        self.assertIn('Run paired Google canonical observation with evidence preservation', text)
        self.assertIn('python3 scripts/paired_canonical_observation.py', text)
        self.assertIn('--control-required "/tmp/v29-${SLUG}-control.json"', text)
        self.assertIn('--candidate-required "/tmp/v29-${SLUG}-candidate.json"', text)
        self.assertIn('--acceptance-required "/tmp/v29-${SLUG}-pair-acceptance.json"', text)
        self.assertIn('--output "/tmp/v29-${SLUG}-paired-orchestration.json"', text)
        self.assertNotIn('Run paired Google control and candidate observations', text)

    def test_pre_live_coordinate_and_version_guards_are_explicit(self):
        for path in (PAIRED, CROSS):
            text = path.read_text(encoding='utf-8')
            self.assertIn('64f6669b872094577a77bcb4137a161aadb66f6a', text)
            self.assertIn('29a9e4be6273dbffeda324e15517dc64930ad315', text)
            self.assertIn('ccce3e56b3093746450db05a07ac2dacc473fa1b', text)
            self.assertIn("grep -q '^version = \"0.4.1\"$' Cargo.toml", text)
            self.assertIn('git diff --exit-code "$CANDIDATE_COMMIT" -- Cargo.toml Cargo.lock crates', text)

    def test_cross_model_parallelism_policy_remains_serial(self):
        text = CROSS.read_text(encoding='utf-8')
        self.assertIn('fail-fast: false', text)
        self.assertIn('max-parallel: 1', text)
        self.assertIn('scripts/validate_cross_model_concurrency_policy.py', text)


if __name__ == '__main__':
    unittest.main()
