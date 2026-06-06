import unittest
from unittest.mock import patch

from evoagent.models import ExperimentGoal
from evoagent.planner import CodexPlanner, StaticPlanner, parse_hypotheses_json


# cf-atom: TEST-static-planner-proposes-hypotheses
class StaticPlannerTests(unittest.TestCase):
    def test_static_planner_proposes_hypotheses(self):
        planner = StaticPlanner()

        hypotheses = planner.propose(ExperimentGoal("Improve tabular learning"), count=2)

        self.assertEqual(len(hypotheses), 2)
        self.assertTrue(all(h.title for h in hypotheses))


# cf-atom: TEST-codex-planner-parses-json-hypotheses
class CodexPlannerTests(unittest.TestCase):
    def test_parse_hypotheses_json_accepts_fenced_json(self):
        hypotheses = parse_hypotheses_json(
            '```json\n{"hypotheses":[{"title":"idea","rationale":"reason","expected_gain":0.1,"novelty":0.8}]}\n```'
        )

        self.assertEqual(hypotheses[0].title, "idea")
        self.assertEqual(hypotheses[0].expected_gain, 0.1)

    def test_codex_planner_invokes_codex_exec(self):
        completed = type(
            "Completed",
            (),
            {
                "returncode": 0,
                "stdout": '{"hypotheses":[{"title":"idea","rationale":"reason","expected_gain":0.2,"novelty":0.7}]}',
                "stderr": "",
            },
        )()

        with patch("subprocess.run", return_value=completed) as run:
            planner = CodexPlanner(command=("codex", "exec"), timeout_seconds=5)
            hypotheses = planner.propose(ExperimentGoal("Improve sample efficiency"), count=1)

        self.assertEqual(hypotheses[0].title, "idea")
        args = run.call_args.kwargs
        self.assertIn("read-only", args["args"] if "args" in args else run.call_args.args[0])
        self.assertIn("Improve sample efficiency", args["input"])


if __name__ == "__main__":
    unittest.main()
