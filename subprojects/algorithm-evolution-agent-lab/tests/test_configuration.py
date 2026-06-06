import unittest

from evoagent.configuration import ConfigurationCapture
from evoagent.models import ExperimentPlan, Hypothesis


# cf-atom: TEST-configuration-capture-records-reproducible-settings
class ConfigurationCaptureTests(unittest.TestCase):
    def test_configuration_capture_records_reproducible_settings(self):
        plan = make_plan()

        config = ConfigurationCapture().capture(
            plan,
            hyperparameters={"lr": 0.01, "depth": 3},
            seed=7,
            dataset_version="toy-v1",
            benchmark_split="validation",
            code_reference="CF-COMMIT-abc123",
            environment={"python": "3.11"},
        )

        self.assertEqual(config.plan_id, plan.uid)
        self.assertEqual(config.hyperparameters, (("depth", "3"), ("lr", "0.01")))
        self.assertEqual(config.environment, (("python", "3.11"),))
        self.assertTrue(config.fingerprint.startswith("cfg_"))
        self.assertEqual(ConfigurationCapture().validate(config), ())

    def test_configuration_capture_flags_missing_reproducibility_fields(self):
        config = ConfigurationCapture().capture(make_plan())

        self.assertEqual(
            {finding.kind for finding in ConfigurationCapture().validate(config)},
            {"missing_seed", "missing_dataset_version", "missing_benchmark_split", "missing_code_reference"},
        )


def make_plan() -> ExperimentPlan:
    return ExperimentPlan(
        Hypothesis("config", "capture exact settings", expected_gain=0.01, novelty=0.2),
        "toy-tabular",
        "baseline",
        "accuracy",
        0.1,
        command="run-config",
    )


if __name__ == "__main__":
    unittest.main()
