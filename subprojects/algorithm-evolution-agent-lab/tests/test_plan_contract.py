import unittest

from evoagent.models import ExperimentPlan, Hypothesis
from evoagent.plan_contract import ExperimentPlanContractBuilder


# cf-atom: TEST-experiment-plan-contract-validates-required-schema
class ExperimentPlanContractBuilderTests(unittest.TestCase):
    def test_experiment_plan_contract_validates_required_schema(self):
        hypothesis = Hypothesis("schema", "check full plan fields", expected_gain=0.02, novelty=0.3)
        plan = ExperimentPlan(
            hypothesis,
            "toy-tabular",
            "baseline",
            "accuracy",
            0.1,
            command="python experiment.py",
            artifact_paths=("metrics.json", "config.json"),
        )

        builder = ExperimentPlanContractBuilder()
        contract = builder.build(plan, analysis_criteria=("compare against baseline", "record caveats"))

        self.assertEqual(contract.plan_id, plan.uid)
        self.assertEqual(contract.hypothesis_id, hypothesis.uid)
        self.assertIn("0.02", contract.expected_outcome)
        self.assertEqual(builder.validate(contract), ())

    def test_experiment_plan_contract_flags_missing_command_artifacts_and_analysis(self):
        hypothesis = Hypothesis("schema", "missing fields", expected_gain=0.02, novelty=0.3)
        plan = ExperimentPlan(hypothesis, "toy-tabular", "baseline", "accuracy", 0.1)

        findings = ExperimentPlanContractBuilder().validate(
            ExperimentPlanContractBuilder().build(plan, analysis_criteria=())
        )

        self.assertEqual(
            {finding.kind for finding in findings},
            {"missing_runner_command", "missing_artifact_contract", "missing_analysis_criteria"},
        )


if __name__ == "__main__":
    unittest.main()
