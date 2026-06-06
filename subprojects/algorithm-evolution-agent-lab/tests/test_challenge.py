import json
import tempfile
import unittest
from pathlib import Path

from evoagent.challenge import load_challenge_definition


# cf-atom: TEST-sota-challenge-loads-definition-file
class SotaChallengeDefinitionTests(unittest.TestCase):
    def test_sota_challenge_loads_definition_file(self):
        challenge = load_challenge_definition("challenges/toy_tabular_sample_efficiency.json")

        self.assertEqual(challenge.challenge_id, "toy-tabular-sample-efficiency")
        self.assertEqual(challenge.best_baseline.name, "regularized-polynomial")
        self.assertEqual(challenge.allowed_compute.max_trials, 20)
        self.assertIn("using validation labels during training", challenge.disallowed_shortcuts)

    def test_sota_challenge_requires_positive_target(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            path = Path(tmpdir) / "invalid.json"
            data = {
                "challenge_id": "invalid",
                "task": "task",
                "dataset": "dataset",
                "metric": "accuracy",
                "split": "validation",
                "target_improvement": 0.0,
                "allowed_compute": {"max_hours": 1.0, "max_trials": 1},
                "baselines": [{"name": "baseline", "metric_value": 0.7, "source": "local"}],
                "disallowed_shortcuts": ["shortcut"],
            }
            path.write_text(json.dumps(data), encoding="utf-8")

            with self.assertRaises(ValueError):
                load_challenge_definition(path)


if __name__ == "__main__":
    unittest.main()
