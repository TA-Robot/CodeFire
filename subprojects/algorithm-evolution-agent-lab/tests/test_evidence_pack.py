import unittest

from evoagent.evidence_pack import (
    EvidenceReadiness,
    ExperimentEvidencePackBuilder,
    ExperimentEvidenceSignal,
)


# cf-atom: TEST-experiment-evidence-pack-builder-blocks-incomplete-claims
class ExperimentEvidencePackBuilderTests(unittest.TestCase):
    def test_experiment_evidence_pack_builder_blocks_incomplete_claims(self):
        builder = ExperimentEvidencePackBuilder(minimum_improvement=0.01, minimum_replications=3)

        pack = builder.build(
            ExperimentEvidenceSignal(
                plan_id="plan-1",
                candidate_id="candidate-a",
                improvement=0.03,
                confidence=0.92,
                replication_count=2,
                stale_evidence_count=1,
                blocker_risk_count=1,
                missing_artifact_count=1,
                mitigation_notes=("rerun benchmark v2", "attach metrics artifact"),
            )
        )

        self.assertEqual(pack.readiness, EvidenceReadiness.BLOCKED)
        self.assertEqual(pack.blocking_reasons, ("missing_artifacts:1", "blocker_risks:1"))
        self.assertEqual(pack.review_notes, ("insufficient_replication", "stale_evidence:1"))
        self.assertEqual(pack.mitigation_checklist, ("rerun benchmark v2", "attach metrics artifact"))
        self.assertLess(pack.completeness_score, 1.0)

    def test_experiment_evidence_pack_builder_marks_ready_when_evidence_is_complete(self):
        builder = ExperimentEvidencePackBuilder(minimum_improvement=0.01, minimum_replications=2)

        pack = builder.build(
            ExperimentEvidenceSignal(
                plan_id="plan-2",
                candidate_id="candidate-b",
                improvement=0.04,
                confidence=0.95,
                replication_count=3,
            )
        )

        self.assertEqual(pack.readiness, EvidenceReadiness.READY)
        self.assertEqual(pack.blocking_reasons, ())
        self.assertEqual(pack.review_notes, ())
        self.assertGreaterEqual(pack.completeness_score, 0.99)

    def test_experiment_evidence_pack_builder_validates_counts(self):
        builder = ExperimentEvidencePackBuilder()

        with self.assertRaises(ValueError):
            builder.build(
                ExperimentEvidenceSignal(
                    plan_id="plan-3",
                    candidate_id="candidate-c",
                    improvement=0.1,
                    confidence=1.1,
                    replication_count=1,
                )
            )


if __name__ == "__main__":
    unittest.main()
