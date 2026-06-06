import unittest

from evoagent.models import ExperimentPlan, Hypothesis
from evoagent.program import ResearchProgram
from evoagent.models import ExperimentGoal


# cf-atom: TEST-durable-research-identities-and-lineage
class DurableResearchIdentityTests(unittest.TestCase):
    def test_hypotheses_and_plans_have_stable_fallback_ids(self):
        first = Hypothesis("mutation", "change optimizer schedule", 0.03, 0.5)
        same = Hypothesis("mutation", "change optimizer schedule", 0.03, 0.5)
        explicit = Hypothesis("mutation", "change optimizer schedule", 0.03, 0.5, hypothesis_id="hyp_manual")
        plan = ExperimentPlan(first, "toy-optimizer", "adam", "loss", 0.3, command="python run.py")

        self.assertEqual(first.uid, same.uid)
        self.assertEqual(explicit.uid, "hyp_manual")
        self.assertTrue(first.uid.startswith("hyp_"))
        self.assertTrue(plan.uid.startswith("plan_"))

    def test_research_program_records_hypothesis_lineage(self):
        program = ResearchProgram(ExperimentGoal("Improve optimizer benchmark"))
        parent = Hypothesis("parent", "base idea", 0.02, 0.4)
        child = Hypothesis("child", "mutated idea", 0.04, 0.6)

        link = program.link_hypotheses(parent, child, "mutates")

        self.assertEqual(link.source_id, parent.uid)
        self.assertEqual(link.target_id, child.uid)
        self.assertEqual(link.relation, "mutates")
        self.assertEqual(program.hypothesis_links, [link])

    def test_research_program_rejects_unknown_lineage_relation(self):
        program = ResearchProgram(ExperimentGoal("Improve optimizer benchmark"))

        with self.assertRaises(ValueError):
            program.link_hypotheses(
                Hypothesis("parent", "base idea", 0.02, 0.4),
                Hypothesis("child", "mutated idea", 0.04, 0.6),
                "unclear",
            )


if __name__ == "__main__":
    unittest.main()
