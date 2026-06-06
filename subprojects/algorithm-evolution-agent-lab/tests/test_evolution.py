import unittest

from evoagent.evolution import AblationAxis, AblationGenerator, CandidateEvolutionOperators, MutationSpec
from evoagent.models import Hypothesis


# cf-atom: TEST-candidate-mutation-preserves-lineage
class CandidateEvolutionOperatorTests(unittest.TestCase):
    def test_candidate_mutation_preserves_lineage(self):
        parent = Hypothesis("feature gate", "remove noisy feature groups", 0.05, 0.4)
        operator = CandidateEvolutionOperators()

        child = operator.mutate(
            parent,
            MutationSpec(
                name="seeded ablation",
                rationale="test the mechanism with one feature family removed",
                expected_gain_delta=0.02,
                novelty_delta=0.1,
            ),
        )
        link = operator.lineage(parent, child, "mutates")

        self.assertIn("seeded ablation", child.title)
        self.assertGreater(child.expected_gain, parent.expected_gain)
        self.assertEqual(link.source_id, parent.uid)
        self.assertEqual(link.target_id, child.uid)
        self.assertEqual(link.relation, "mutates")

    # cf-atom: TEST-candidate-crossover-combines-parent-mechanisms
    def test_candidate_crossover_combines_parent_mechanisms(self):
        left = Hypothesis("regularization schedule", "late regularization", 0.06, 0.3)
        right = Hypothesis("interaction sparsifier", "sparse feature crosses", 0.04, 0.7)
        operator = CandidateEvolutionOperators()

        child = operator.crossover(left, right, title="sparse late-regularized interactions")
        left_link = operator.lineage(left, child, "combines")
        right_link = operator.lineage(right, child, "combines")

        self.assertEqual(child.title, "sparse late-regularized interactions")
        self.assertAlmostEqual(child.expected_gain, 0.05)
        self.assertGreater(child.novelty, right.novelty)
        self.assertEqual({left_link.source_id, right_link.source_id}, {left.uid, right.uid})

    # cf-atom: TEST-ablation-generator-creates-controlled-followups
    def test_ablation_generator_creates_controlled_followups(self):
        parent = Hypothesis("sparse ensemble", "combine filters with ensemble pruning", 0.08, 0.65)
        axes = (
            AblationAxis("without filters", "feature filters", "measure whether filters carry the gain"),
            AblationAxis("without pruning", "ensemble pruning", "measure whether pruning carries the gain"),
        )

        ablations = AblationGenerator().generate(parent, axes)
        links = AblationGenerator().lineage_links(parent, ablations)

        self.assertEqual(len(ablations), 2)
        self.assertTrue(all("ablate" in hypothesis.title for hypothesis in ablations))
        self.assertTrue(all(hypothesis.expected_gain < parent.expected_gain for hypothesis in ablations))
        self.assertEqual({link.relation for link in links}, {"ablates"})


if __name__ == "__main__":
    unittest.main()
